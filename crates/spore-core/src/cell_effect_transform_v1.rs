//! Bounded E5E29A transform arithmetic and basis copy; effect dispatch excluded.
use crate::native_sse::{self,Policy,SseError};
#[derive(Debug,PartialEq)]pub enum Error{Unadmitted,NonFinite,Sse(SseError)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
/// Supplied corners and basis are finite, stable, disjoint snapshots; the
/// original helper copies basis words. Global vector words read earlier in the
/// source slice are overwritten by midpoint fields before the cut. No effect
/// callback, live pool, object identity or runtime placement is inferred.
pub fn project(corners:[u32;6],basis:[u32;9],policy:Policy,admitted:bool)->Result<[u32;14],Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !corners.iter().chain(basis.iter()).all(|&x|f32::from_bits(x).is_finite()){return Err(Error::NonFinite)}
 native_sse::scoped(policy,||{use std::hint::black_box as b;let c=corners.map(f32::from_bits);let x=b(b(c[3])+b(c[0]))*b(0.5);let y=b(b(c[4])+b(c[1]))*b(0.5);let z=b(b(c[5])+b(c[2]))*b(0.5);let span=b(c[3])-b(c[0]);if ![x,y,z,span].iter().all(|x|x.is_finite()){return Err(Error::NonFinite)}let mut out=[0u32;14];out[..5].copy_from_slice(&[0x0002_0004,x.to_bits(),y.to_bits(),z.to_bits(),span.to_bits()]);out[5..].copy_from_slice(&basis);Ok(out)})
}
#[cfg(test)]mod tests{use super::*;#[test]fn midpoint_and_bounds(){let one=1f32.to_bits();let basis=[one,0,0,0,one,0,0,0,one];let c=[0,0,0,4f32.to_bits(),6f32.to_bits(),0];let x=project(c,basis,Policy::Mxcsr1f80,true).unwrap();assert_eq!(&x[..5],&[0x20004,2f32.to_bits(),3f32.to_bits(),0,4f32.to_bits()]);assert_eq!(project(c,basis,Policy::Mxcsr1f80,false),Err(Error::Unadmitted));}}
