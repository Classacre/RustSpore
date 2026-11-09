//! Bounded 509119..5092D8 UV determinant and branch projection.
//! Caller supplies stable UV/corner arrays and a non-sentinel metadata corner.
//! Source signed neighbor tables and threshold are fixed to the pinned PE words.
//! Finite normal-or-zero operands/intermediates and MXCSR 1F80 are admitted;
//! allocation, metadata sentinel handling, tangents and live identity are not modeled.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{InvalidCorner,InvalidUv,NonFiniteOrSubnormal,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Output{pub determinant:u32,pub fallback:bool}
fn admitted(v:f32)->Result<f32,Error>{if v.is_finite()&&!v.is_subnormal(){Ok(v)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Compute the oriented determinant for one explicitly selected corner.
pub fn project(uvs:&[[u32;2]],corners:&[u32],corner:usize,policy:Policy)->Result<Output,Error>{
 let j=match corner%3{0|1=>corner.checked_add(1),_=>corner.checked_sub(2)}.ok_or(Error::InvalidCorner)?;
 let k=match corner%3{0=>corner.checked_add(2),_=>corner.checked_sub(1)}.ok_or(Error::InvalidCorner)?;
 let get=|i:usize|->Result<[f32;2],Error>{let index=*corners.get(i).ok_or(Error::InvalidCorner)? as usize;let words=*uvs.get(index).ok_or(Error::InvalidUv)?;let uv=words.map(f32::from_bits);admitted(uv[0])?;admitted(uv[1])?;Ok(uv)};
 let a=get(corner)?;let b=get(j)?;let c=get(k)?;
 native_sse::scoped(policy,||{
  let x=admitted(b[0]-a[0])?;let y=admitted(b[1]-a[1])?;let z=admitted(c[0]-a[0])?;let w=admitted(c[1]-a[1])?;
  let p=admitted(x*w)?;let q=admitted(z*y)?;let determinant=admitted(p-q)?;
  Ok(Output{determinant:determinant.to_bits(),fallback:determinant<=f32::from_bits(0x33d6bf95)})
 })
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn threshold_negative_and_positive(){let t=0x33d6bf95;let mut uv=[[0,0],[1f32.to_bits(),0],[0,t]];let p=Policy::Mxcsr1f80;let corners=[0,1,2];let x=project(&uv,&corners,0,p).unwrap();assert_eq!(x,Output{determinant:t,fallback:true});uv[2][1]=(-1f32).to_bits();assert!(project(&uv,&corners,0,p).unwrap().fallback);uv[2][1]=1f32.to_bits();assert!(!project(&uv,&corners,0,p).unwrap().fallback);for i in 1..3{assert_eq!(project(&uv,&corners,i,p).unwrap(),Output{determinant:1f32.to_bits(),fallback:false});}}
 #[test]fn rejects_out_of_bounds_and_nonfinite(){let p=Policy::Mxcsr1f80;assert_eq!(project(&[[0,0]],&[0],0,p),Err(Error::InvalidCorner));assert_eq!(project(&[[0,0]],&[0,1,0],0,p),Err(Error::InvalidUv));assert_eq!(project(&[[0,0],[f32::NAN.to_bits(),0]],&[0,1,0],0,p),Err(Error::NonFiniteOrSubnormal));}
}
