//! Checked 50C198..50C2B2 per-UV rotation, translation and scale.
//! Inputs, group bounds/flags, offsets and scale are supplied explicitly.
//! Finite normal-or-zero inputs and intermediates, MXCSR 1F80 and exact x87
//! copies in this domain are admitted. Actual atlas placement, group ownership,
//! texture binding, allocator behavior and live identity are not inferred.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { NonFiniteOrSubnormal, UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Transform one caller-supplied UV pair in source arithmetic order.
pub fn transform(uv:[u32;2],u_bounds:[u32;2],flags:u32,offset:[u32;2],scale:u32,policy:Policy)->Result<[u32;2],Error>{
 let mut u=admitted(f32::from_bits(uv[0]))?;let mut v=admitted(f32::from_bits(uv[1]))?;
 let lo=admitted(f32::from_bits(u_bounds[0]))?;let hi=admitted(f32::from_bits(u_bounds[1]))?;
 let du=admitted(f32::from_bits(offset[0]))?;let dv=admitted(f32::from_bits(offset[1]))?;let scale=admitted(f32::from_bits(scale))?;
 native_sse::scoped(policy,||{
  let sum=admitted(hi+lo)?;
  if flags&1!=0 {let old_u=u;u=admitted(sum-v)?;v=old_u;}
  let u=admitted(u+du)?;let v=admitted(v+dv)?;
  let u=admitted(u*scale)?;let v=admitted(v*scale)?;Ok([u.to_bits(),v.to_bits()])
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn rotate_translate_scale_and_flag(){
  let b=|v:f32|v.to_bits();let p=Policy::Mxcsr1f80;
  assert_eq!(transform([b(1.),b(2.)],[0,b(4.)],1,[b(0.5),b(-0.25)],b(2.),p),Ok([b(5.),b(1.5)]));
  assert_eq!(transform([b(1.),b(2.)],[0,b(4.)],2,[b(0.5),b(-0.25)],b(2.),p),Ok([b(3.),b(3.5)]));
 }
 #[test] fn rejects_nonfinite_subnormal_and_overflow(){
  let p=Policy::Mxcsr1f80;assert_eq!(transform([1,0],[0,0],0,[0,0],0,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(transform([0,0],[0,0],0,[0,0],0x7f800000,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(transform([0,0],[0x7f7fffff,0x7f7fffff],0,[0,0],0,p),Err(Error::NonFiniteOrSubnormal));
 }
}
