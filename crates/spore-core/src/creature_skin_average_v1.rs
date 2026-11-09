//! Bounded 53F951..53F983 position averaging, before correction/normal work.
use crate::{native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
#[derive(Debug,PartialEq)]pub struct Output{pub reciprocal:f32,pub position:[f32;3]}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFinite,X87(X87Error),UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
impl From<X87Error> for Error{fn from(e:X87Error)->Self{Self::X87(e)}}
/// The source FILDs the zero-extended unsigned count as a signed qword, divides
/// one by it in x87, spills the reciprocal to binary32, then multiplies each
/// position component by that spill using SSE. Valid nonzero count, finite
/// arithmetic, masked exceptions, x87 stack and supplied FP controls are
/// admissions; no status flags, correction, normals or final mesh are modeled.
pub fn average(position:[f32;3],count:u32,sse:Policy,x87:Context)->Result<Output,Error>{
 if count==0||position.iter().any(|v|!v.is_finite()){return Err(Error::InvalidInput)}
 let n=Extended::from_f64(count as f64)?;let one=Extended::from_i32(1);let reciprocal=x87.store_f32(x87.div(one,n)?)?;
 if !reciprocal.is_finite(){return Err(Error::NonFinite)}
 native_sse::scoped(sse,||{let mut result=[0.;3];for i in 0..3{result[i]=position[i]*reciprocal;if !result[i].is_finite(){return Err(Error::NonFinite)}}Ok(Output{reciprocal,position:result})})
}
#[cfg(test)]mod tests{use super::*;use crate::native_x87::{Precision,Rounding};fn ctx()->Context{Context{precision:Precision::Bits53,rounding:Rounding::NearestEven}}
 #[test]fn thirds_and_sign_zero(){let o=average([3.,-6.,-0.],3,Policy::Mxcsr1f80,ctx()).unwrap();assert_eq!(o.reciprocal.to_bits(),(1f32/3.).to_bits());assert_eq!(o.position[0],1.);assert_eq!(o.position[1],-2.);assert_eq!(o.position[2].to_bits(),0x80000000);}
 #[test]fn unsigned_count_above_i32_range(){let o=average([1.,2.,4.],u32::MAX,Policy::Mxcsr1f80,ctx()).unwrap();assert_eq!(o.reciprocal.to_bits(),0x2f800000);}
 #[test]fn zero_and_nonfinite_rejected(){assert_eq!(average([0.;3],0,Policy::Mxcsr1f80,ctx()),Err(Error::InvalidInput));assert_eq!(average([f32::NAN,0.,0.],1,Policy::Mxcsr1f80,ctx()),Err(Error::InvalidInput));}
}
