//! Restricted zero-threshold, equal-radius 4FD3C0 segment branch.
//! All other inputs and complete native grid construction are outside scope.
use crate::{creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error,Precision,Rounding}};
#[derive(Clone,Copy,Debug)]
pub struct MathAdmission;
#[derive(Debug,PartialEq,Eq)]
pub enum Error { InvalidInput, NonFinite, ExcessWork, AllocationFailed, X87(X87Error), UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
impl From<X87Error> for Error {fn from(e:X87Error)->Self{Self::X87(e)}}
/// Assumes source CRT pow(+0, .25) = +0 and sqrt(1) = 1, successful
/// append/storage, and a matching x87 context. This admits query +0C == +0,
/// both endpoint coefficients 1, exactly equal positive endpoint radii, and x87 027F;
/// only the source's low-slope branch is in scope. No endpoint lanes or prior
/// bounds are returned. Checked work/finite restrictions are not native.
pub fn sample(start:[f32;3],end:[f32;3],radius:f32,spacing:f32,key:u32,policy:Policy,x87:Context,_:MathAdmission)->Result<Vec<Lane>,Error>{
 if x87.precision != Precision::Bits53 || x87.rounding != Rounding::NearestEven {return Err(Error::InvalidInput);}
 if start.into_iter().chain(end).chain([radius,spacing]).any(|x|!x.is_finite()) || radius<=0. || spacing<=0. {return Err(Error::InvalidInput);}
 native_sse::scoped(policy,||{
  let mut d=[0.;3];for i in 0..3 {d[i]=end[i]-start[i];}
  let xx=d[0]*d[0];let yy=d[1]*d[1];let zz=d[2]*d[2];let sum=(xx+yy)+zz;
  if !sum.is_finite() || sum<=0. {return Err(Error::InvalidInput);}
  let length=x87.store_f32(x87.sqrt(Extended::from_f32(sum)?)?)?;
  if length<=0. || !length.is_finite() {return Err(Error::NonFinite);}
  let inv=1.0/length;let direction=d.map(|v|v*inv);
  let effective=x87.store_f32(x87.add(Extended::from_f32(radius)?,Extended::from_f32(f32::from_bits(0x358637bd))?)?)?;
  let denominator=(2.0*spacing)*effective;
  let ratio=length/denominator;
  if [inv,effective,denominator,ratio].into_iter().chain(direction).any(|v|!v.is_finite()) || denominator<=0. || ratio<=0. {return Err(Error::NonFinite);}
  let steps=ratio.ceil();if steps>4096. {return Err(Error::ExcessWork);} let count=steps as usize;
  let adjusted=length/((2.0*effective)*steps);
  let cutoff=effective*effective;let scale=1.0/cutoff;
  if [adjusted,cutoff,scale].into_iter().any(|v|!v.is_finite()) {return Err(Error::NonFinite);}
  let mut out=Vec::new();out.try_reserve_exact(count.saturating_sub(1)).map_err(|_|Error::AllocationFailed)?;
  for i in 1..count {
   let dist=((2.0*adjusted)*effective)*(i as f32);
   let mut center=[0.;3];for axis in 0..3 {center[axis]=start[axis]+direction[axis]*dist;}
   if !dist.is_finite() || center.into_iter().any(|v|!v.is_finite()) {return Err(Error::NonFinite);}
   out.push(Lane{center,cutoff,scale,coefficient:1.,key});
  }
  Ok(out)
 })
}
#[cfg(test)] mod tests {
 use super::*;
 fn c()->Context{Context{precision:Precision::Bits53,rounding:Rounding::NearestEven}}
 #[test] fn produces_ordered_interior_only(){let out=sample([0.;3],[1.,0.,0.],1.,0.3,42,Policy::Mxcsr1f80,c(),MathAdmission).unwrap();assert_eq!(out.len(),1);assert_eq!(out[0].center,[0.5,0.,0.]);assert_eq!(out[0].key,42);}
 #[test] fn rejects_degenerate_and_bounds_work(){assert_eq!(sample([0.;3],[0.;3],1.,0.3,0,Policy::Mxcsr1f80,c(),MathAdmission),Err(Error::InvalidInput));assert_eq!(sample([0.;3],[100.,0.,0.],0.01,0.001,0,Policy::Mxcsr1f80,c(),MathAdmission),Err(Error::ExcessWork));}
}
