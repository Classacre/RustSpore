//! Conditional 509119..509507 pre-orthogonalization tangent candidate.
//! Stable UVs, corner and position-index buffers and same-object provenance are
//! caller admissions. Finite normal-or-zero operands/intermediates under 1F80;
//! fallback, normal projection, normalization, material and live state excluded.
use crate::{creature_skin_uv_determinant_v1::{self as det},native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{InvalidCorner,InvalidUv,InvalidPosition,InvalidDeterminant,NonFiniteOrSubnormal,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
impl From<det::Error> for Error{fn from(e:det::Error)->Self{match e{det::Error::InvalidCorner=>Self::InvalidCorner,det::Error::InvalidUv=>Self::InvalidUv,det::Error::NonFiniteOrSubnormal=>Self::NonFiniteOrSubnormal,det::Error::UnsupportedArchitecture=>Self::UnsupportedArchitecture}}}
fn admitted(v:f32)->Result<f32,Error>{if v.is_finite()&&!v.is_subnormal(){Ok(v)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Returns a three-word candidate, before projection against a normal.
pub fn project(uvs:&[[u32;2]],corners:&[u32],positions:&[[u32;3]],position_indices:&[u32],corner:usize,policy:Policy)->Result<[u32;3],Error>{
 let d=det::project(uvs,corners,corner,policy)?;if d.fallback{return Err(Error::InvalidDeterminant)}
 let j=match corner%3{0|1=>corner.checked_add(1),_=>corner.checked_sub(2)}.ok_or(Error::InvalidCorner)?;
 let k=match corner%3{0=>corner.checked_add(2),_=>corner.checked_sub(1)}.ok_or(Error::InvalidCorner)?;
 let uv=|i:usize|->Result<[f32;2],Error>{let n=*corners.get(i).ok_or(Error::InvalidCorner)? as usize;let v=uvs.get(n).ok_or(Error::InvalidUv)?.map(f32::from_bits);for x in v{admitted(x)?;}Ok(v)};
 let pos=|i:usize|->Result<[f32;3],Error>{let n=*position_indices.get(i).ok_or(Error::InvalidCorner)? as usize;let v=positions.get(n).ok_or(Error::InvalidPosition)?.map(f32::from_bits);for x in v{admitted(x)?;}Ok(v)};
 let a=uv(corner)?;let b=uv(j)?;let c=uv(k)?;let p=pos(corner)?;let q=pos(j)?;let r=pos(k)?;
 native_sse::scoped(policy,||{
  let dy1=admitted(b[1]-a[1])?;let dy2=admitted(c[1]-a[1])?;let inv=admitted(1f32/f32::from_bits(d.determinant))?;
  let s1=admitted(dy2*inv)?;let s2=admitted(f32::from_bits(dy1.to_bits()^0x80000000)*inv)?;
  let mut x=[0u32;3];for axis in 0..3{let e1=admitted(q[axis]-p[axis])?;let e2=admitted(r[axis]-p[axis])?;let a=admitted(s1*e1)?;let b=admitted(s2*e2)?;x[axis]=admitted(a+b)?.to_bits();}Ok(x)
 })
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn basis_and_three_lanes(){let u=[[0,0],[1f32.to_bits(),0],[0,1f32.to_bits()]];let c=[0,1,2];let p=[[0,0,0],[2f32.to_bits(),0,0],[0,3f32.to_bits(),0]];for i in 0..3{assert_eq!(project(&u,&c,&p,&c,i,Policy::Mxcsr1f80).unwrap(),[2f32.to_bits(),0,0]);}}
 #[test]fn rejects_fallback_and_bad_position(){let p=Policy::Mxcsr1f80;let u=[[0,0],[1f32.to_bits(),0],[0,0]];let c=[0,1,2];assert_eq!(project(&u,&c,&[[0,0,0];3],&c,0,p),Err(Error::InvalidDeterminant));let u=[[0,0],[1f32.to_bits(),0],[0,1f32.to_bits()]];assert_eq!(project(&u,&c,&[[0,0,0]],&c,0,p),Err(Error::InvalidPosition));}
}
