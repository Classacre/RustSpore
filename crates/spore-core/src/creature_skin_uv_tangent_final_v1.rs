//! Conditional 509508..509685 tangent projection, normalization and cross.
//! The caller admits stable same-object normal indexing and a CRT-compatible
//! binary32 square-root spill. Finite normal-or-zero values and MXCSR 1F80 are
//! required. No source allocation, runtime identity or material is established.
use crate::{creature_skin_normal_v1::SqrtSpill,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{NonFiniteOrSubnormal,InvalidLength,Sqrt,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
fn admitted(v:f32)->Result<f32,Error>{if v.is_finite()&&!v.is_subnormal(){Ok(v)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Returns the normalized projected candidate and normal cross tangent.
pub fn project<S:SqrtSpill>(candidate:[u32;3],normal:[u32;3],policy:Policy,sqrt:&mut S)->Result<[[u32;3];2],Error>{
 let t=candidate.map(f32::from_bits);let n=normal.map(f32::from_bits);for x in t.into_iter().chain(n){admitted(x)?;}
 native_sse::scoped(policy,||{
  let zz=admitted(n[2]*t[2])?;let yy=admitted(n[1]*t[1])?;let xx=admitted(n[0]*t[0])?;let dot=admitted(admitted(xx+yy)?+zz)?;
  let mut p=[0f32;3];for i in 0..3{p[i]=admitted(t[i]-admitted(n[i]*dot)?)?;}
  let x=admitted(p[0]*p[0])?;let y=admitted(p[1]*p[1])?;let z=admitted(p[2]*p[2])?;let len2=admitted(admitted(x+y)?+z)?;if len2<=0.0{return Err(Error::InvalidLength)}
  let len=admitted(sqrt.sqrt_spill(len2).ok_or(Error::Sqrt)?)?;if len<=0.0{return Err(Error::Sqrt)}let inv=admitted(1f32/len)?;let mut u=[0f32;3];for i in 0..3{u[i]=admitted(p[i]*inv)?;}
  let bx=admitted(admitted(n[1]*u[2])?-admitted(n[2]*u[1])?)?;let by=admitted(admitted(n[2]*u[0])?-admitted(n[0]*u[2])?)?;let bz=admitted(admitted(n[0]*u[1])?-admitted(n[1]*u[0])?)?;
  Ok([u.map(f32::to_bits),[bx.to_bits(),by.to_bits(),bz.to_bits()]])
 })
}
#[cfg(test)]mod tests{use super::*;struct Math;impl SqrtSpill for Math{fn sqrt_spill(&mut self,x:f32)->Option<f32>{Some(x.sqrt())}}
 #[test]fn projects_and_crosses(){let b=|x:f32|x.to_bits();let q=project([b(2.),0,b(1.)],[0,0,b(1.)],Policy::Mxcsr1f80,&mut Math).unwrap();assert_eq!(q,[[b(1.),0,0],[0,b(1.),0]]);}
 #[test]fn rejects_zero_and_bad_sqrt(){let b=|x:f32|x.to_bits();let p=Policy::Mxcsr1f80;assert_eq!(project([0,0,b(1.)],[0,0,b(1.)],p,&mut Math),Err(Error::InvalidLength));struct Bad;impl SqrtSpill for Bad{fn sqrt_spill(&mut self,_:f32)->Option<f32>{None}}assert_eq!(project([b(1.),0,0],[0,0,b(1.)],p,&mut Bad),Err(Error::Sqrt));}
}
