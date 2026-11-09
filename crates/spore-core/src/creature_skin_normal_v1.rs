//! Bounded direct 436D60/421BD0 normalization of an admitted usable gradient.
use crate::native_sse::{self,Policy,SseError};
/// Caller supplies the binary32 spill produced by the selected sqrt routine.
pub trait SqrtSpill{fn sqrt_spill(&mut self,input:f32)->Option<f32>;}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFinite,Sqrt,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// Models SSE length sum, explicit x87/CRT sqrt spill, SSE reciprocal/multiply,
/// then sign-bit XOR. The caller admits a usable gradient, finite arithmetic,
/// successful sqrt with native-compatible FP controls and stable callbacks.
/// No correction branch, allocator, sticky flags, full mesh or identity modeled.
pub fn normalize<S:SqrtSpill>(gradient:[f32;3],policy:Policy,sqrt:&mut S)->Result<[f32;3],Error>{
 if gradient.iter().any(|v|!v.is_finite()){return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{let xx=gradient[0]*gradient[0];let yy=gradient[1]*gradient[1];let zz=gradient[2]*gradient[2];let mag=(xx+yy)+zz;if [xx,yy,zz,mag].into_iter().any(|v|!v.is_finite()){return Err(Error::NonFinite)}if mag<=0.{return Err(Error::InvalidInput)}let root=sqrt.sqrt_spill(mag).ok_or(Error::Sqrt)?;if !root.is_finite()||root<=0.{return Err(Error::Sqrt)}let inv=1f32/root;if !inv.is_finite(){return Err(Error::NonFinite)}let mut out=[0.;3];for i in 0..3{let v=gradient[i]*inv;if !v.is_finite(){return Err(Error::NonFinite)}out[i]=f32::from_bits(v.to_bits()^0x80000000);}Ok(out)})
}
#[cfg(test)]mod tests{use super::*;struct Math;impl SqrtSpill for Math{fn sqrt_spill(&mut self,x:f32)->Option<f32>{Some(x.sqrt())}}
 #[test]fn unit_and_signed_zero(){let n=normalize([3.,4.,0.],Policy::Mxcsr1f80,&mut Math).unwrap();assert_eq!(n[0],-0.6);assert_eq!(n[1],-0.8);assert_eq!(n[2].to_bits(),0x80000000);}
 #[test]fn zero_and_invalid_provider_are_rejected(){struct Bad;impl SqrtSpill for Bad{fn sqrt_spill(&mut self,_:f32)->Option<f32>{None}}assert_eq!(normalize([0.;3],Policy::Mxcsr1f80,&mut Math),Err(Error::InvalidInput));assert_eq!(normalize([1.,0.,0.],Policy::Mxcsr1f80,&mut Bad),Err(Error::Sqrt));}
}
