//! General-slope 4FD3C0 slice with explicit caller-supplied CRT spill results.
//! This does not bind a CRT or establish native behavior for an adapter.
use crate::{creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error,Precision,Rounding}};
#[derive(Clone,Copy,Debug)] pub struct MathAdmission;
#[derive(Clone,Copy,Debug,PartialEq)] pub struct Sample {pub lane: Lane, pub radius: f32}
/// Each return is the finite binary32 result after the original CRT return is
/// stored to memory. Inputs must be interpreted under the admitted CRT/x87
/// semantics; arbitrary Rust math adapters are only experiments.
pub trait CrtMath {fn pow_spill(&mut self,base:f64,exponent:f64)->Result<f32,()>;fn log_spill(&mut self,value:f32)->Result<f32,()>;}
#[derive(Debug,PartialEq,Eq)] pub enum Error {InvalidInput,UnsupportedBranch,NonFinite,Math,ExcessWork,AllocationFailed,X87(X87Error),UnsupportedArchitecture}
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
impl From<X87Error> for Error {fn from(e:X87Error)->Self{Self::X87(e)}}
fn ext(x:f32)->Result<Extended,Error>{Ok(Extended::from_f32(x)?)}
fn call_pow<M:CrtMath>(m:&mut M,a:f64,b:f64)->Result<f32,Error>{let x=m.pow_spill(a,b).map_err(|_|Error::Math)?;if !x.is_finite(){return Err(Error::NonFinite)}Ok(x)}
fn call_log<M:CrtMath>(m:&mut M,a:f32)->Result<f32,Error>{let x=m.log_spill(a).map_err(|_|Error::Math)?;if !x.is_finite(){return Err(Error::NonFinite)}Ok(x)}
/// Equal unit endpoint coefficients; strictly positive threshold, spacing and
/// radii. Only nonzero general-slope branch is modeled, with an explicitly
/// admitted, operation-accurate CRT provider and x87 027F. Work and finite
/// checks are stricter than native. Result contains interior lanes only.
pub fn sample<M:CrtMath>(start:[f32;3],end:[f32;3],radius_a:f32,radius_b:f32,spacing:f32,threshold:f32,key:u32,policy:Policy,x87:Context,math:&mut M,_:MathAdmission)->Result<Vec<Sample>,Error>{
 if x87.precision!=Precision::Bits53 || x87.rounding!=Rounding::NearestEven || start.into_iter().chain(end).chain([radius_a,radius_b,spacing,threshold]).any(|x|!x.is_finite()) || radius_a<=0. || radius_b<=0. || spacing<=0. || threshold<=0. || threshold>=1. {return Err(Error::InvalidInput);}
 native_sse::scoped(policy,||{
  let mut d=[0.;3];for i in 0..3{d[i]=end[i]-start[i];}
  let xx=d[0]*d[0];let yy=d[1]*d[1];let zz=d[2]*d[2];let sum=(xx+yy)+zz;
  if !sum.is_finite() || sum<=0. {return Err(Error::InvalidInput);}
  let length=x87.store_f32(x87.sqrt(ext(sum)?)?)?;if length<=0. || !length.is_finite(){return Err(Error::NonFinite);}
  let inv=1.0/length;let direction=d.map(|v|v*inv);
  let effective=|r:f32,math:&mut M|->Result<f32,Error>{let p=call_pow(math,threshold as f64,0.25)?;let factor=x87.store_f32(x87.sqrt(x87.sub(ext(1.)?,ext(p)?)?)?)?;let value=x87.add(x87.mul(ext(factor)?,ext(r)?)?,ext(f32::from_bits(0x358637bd))?)?;Ok(x87.store_f32(value)?)};
  let a=effective(radius_a,math)?;let b=effective(radius_b,math)?;
  let slope=(a-b)/length;if !slope.is_finite() || (slope.abs() as f64)<=1e-6{return Err(Error::UnsupportedBranch);}
  let q=(1.0-spacing*slope)/(1.0+spacing*slope);
  let base=1.0-(length*slope)/a;
  if !q.is_finite()||!base.is_finite()||q<=0.||base<=0.||q==1. {return Err(Error::InvalidInput);}
  let log_base=call_log(math,base)?;let log_q=call_log(math,q)?;
  let ratio=x87.store_f32(x87.div(ext(log_base)?,ext(log_q)?)?)?;
  if !ratio.is_finite() || ratio<=0. {return Err(Error::NonFinite);}let n=ratio.ceil();if n>4096. {return Err(Error::ExcessWork);}let count=n as usize;
  let root=call_pow(math,base as f64,1.0/(n as f64))?;
  let adjusted=(1.0-root)/((1.0+root)*slope);
  let q2=(1.0-adjusted*slope)/(1.0+adjusted*slope);
  if !adjusted.is_finite()||!q2.is_finite()||q2<=0. {return Err(Error::NonFinite);}
  let mut out=Vec::new();out.try_reserve_exact(count.saturating_sub(1)).map_err(|_|Error::AllocationFailed)?;
  for i in 1..count {
   let power=call_pow(math,q2 as f64,i as f64)?;let r_eff=power*a;
   let p=call_pow(math,threshold as f64,0.25)?;let factor=x87.store_f32(x87.sqrt(x87.sub(ext(1.)?,ext(p)?)?)?)?;
   let radius=x87.store_f32(x87.div(ext(r_eff)?,ext(factor)?)?)?;
   let distance=((1.0-power)*a)/slope;let mut center=[0.;3];for axis in 0..3{center[axis]=start[axis]+direction[axis]*distance;}
   let cutoff=radius*radius;let scale=1.0/cutoff;
   if [power,r_eff,radius,distance,cutoff,scale].into_iter().chain(center).any(|x|!x.is_finite()) || radius<=0. {return Err(Error::NonFinite);}
   out.push(Sample{lane:Lane{center,cutoff,scale,coefficient:1.,key},radius});
  }
  Ok(out)
 })
}
#[cfg(test)] mod tests {
 use super::*;
 struct Host;impl CrtMath for Host{fn pow_spill(&mut self,b:f64,e:f64)->Result<f32,()>{Ok(b.powf(e) as f32)} fn log_spill(&mut self,v:f32)->Result<f32,()>{Ok((v as f64).ln() as f32)}}
 fn ctx()->Context{Context{precision:Precision::Bits53,rounding:Rounding::NearestEven}}
 #[test]fn rejects_wrong_domain_and_branch(){assert_eq!(sample([0.;3],[1.,0.,0.],1.,1.,0.3,0.01,1,Policy::Mxcsr1f80,ctx(),&mut Host,MathAdmission),Err(Error::UnsupportedBranch));assert_eq!(sample([0.;3],[1.,0.,0.],1.,2.,0.3,0.,1,Policy::Mxcsr1f80,ctx(),&mut Host,MathAdmission),Err(Error::InvalidInput));}
 #[test]fn honors_adapter_failures(){struct Bad;impl CrtMath for Bad{fn pow_spill(&mut self,_:f64,_:f64)->Result<f32,()>{Err(())}fn log_spill(&mut self,_:f32)->Result<f32,()>{Err(())}}assert_eq!(sample([0.;3],[1.,0.,0.],1.,2.,0.3,0.01,1,Policy::Mxcsr1f80,ctx(),&mut Bad,MathAdmission),Err(Error::Math));}
}

