//! Finite 53ED50 packed lattice-coordinate to world-coordinate transform.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug)] pub struct Admission;
#[derive(Debug,PartialEq,Eq)] pub enum Error {InvalidInput,NonFinite,UnsupportedArchitecture}
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// The caller supplies the surface worker's aligned +08 origin and accepted
/// +14 step; no grid/queue, source identity or isosurface point is inferred.
/// Source consumes packed bits 16..23, 8..15, 0..7 as x,y,z respectively,
/// multiplies each by step, then adds origin, under admitted finite operations.
pub fn world(packed:u32,aligned:[f32;3],step:f32,policy:Policy,_:Admission)->Result<[f32;3],Error>{
 if aligned.into_iter().chain([step]).any(|x|!x.is_finite())||step<=0. {return Err(Error::InvalidInput);}
 native_sse::scoped(policy,||{let coord=[(packed>>16)&255,(packed>>8)&255,packed&255];let mut out=[0.;3];for i in 0..3 {let product=step*(coord[i] as f32);out[i]=aligned[i]+product;if !product.is_finite()||!out[i].is_finite(){return Err(Error::NonFinite);}}Ok(out)})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn packed_bytes_and_ignored_high_byte(){let x=world(0xff030201,[-1.,2.,3.],0.5,Policy::Mxcsr1f80,Admission).unwrap();assert_eq!(x,[0.5,3.,3.5]);assert_eq!(x,world(0x00030201,[-1.,2.,3.],0.5,Policy::Mxcsr1f80,Admission).unwrap());}
 #[test]fn rejects_invalid_and_overflow(){assert_eq!(world(0,[0.;3],0.,Policy::Mxcsr1f80,Admission),Err(Error::InvalidInput));assert_eq!(world(0x00ff0000,[0.;3],f32::MAX,Policy::Mxcsr1f80,Admission),Err(Error::NonFinite));}
}
