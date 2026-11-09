//! Checked 50C600 shelf accumulation with explicitly supplied group token.
//! The caller owns preallocated storage and supplies width and vertical bounds;
//! no sorting, shelf selection, actual allocator or live ownership is inferred.
//! Finite normal-or-zero inputs/intermediates and MXCSR 1F80 are admitted.
//! Checked failure is transactional; native appends before arithmetic.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Capacity, NonFiniteOrSubnormal, UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Shelf {pub width:u32,pub height:u32,pub groups:Vec<u32>}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Append a caller-owned token and update binary32 shelf width and height.
pub fn append(shelf:&mut Shelf,group:u32,vertical:[u32;2],width:u32,policy:Policy)->Result<(),Error>{
 if shelf.groups.len()==shelf.groups.capacity() || shelf.groups.len()>=u32::MAX as usize{return Err(Error::Capacity)}
 let lo=admitted(f32::from_bits(vertical[0]))?;let hi=admitted(f32::from_bits(vertical[1]))?;let width=admitted(f32::from_bits(width))?;
 let prior_w=admitted(f32::from_bits(shelf.width))?;let prior_h=admitted(f32::from_bits(shelf.height))?;
 let (new_w,new_h)=native_sse::scoped(policy,||{
  let new_w=admitted(prior_w+width)?;let height=admitted(hi-lo)?;
  Ok::<_,Error>((new_w.to_bits(),if prior_h>height{prior_h.to_bits()}else{height.to_bits()}))
 })?;
 shelf.groups.push(group);shelf.width=new_w;shelf.height=new_h;Ok(())
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn max_and_signed_zero_ties(){
  let mut s=Shelf{width:0,height:0,groups:Vec::with_capacity(2)};
  append(&mut s,7,[0,0x80000000],1f32.to_bits(),Policy::Mxcsr1f80).unwrap();assert_eq!((s.width,s.height,s.groups.clone()),(1f32.to_bits(),0x80000000,vec![7]));
  append(&mut s,8,[0,2f32.to_bits()],1f32.to_bits(),Policy::Mxcsr1f80).unwrap();assert_eq!((s.width,s.height,s.groups),(2f32.to_bits(),2f32.to_bits(),vec![7,8]));
 }
 #[test] fn rejects_without_mutation(){
  let mut s=Shelf{width:0,height:0,groups:Vec::new()};let prior=s.clone();assert_eq!(append(&mut s,0,[0,0],0,Policy::Mxcsr1f80),Err(Error::Capacity));assert_eq!(s,prior);
  let mut s=Shelf{width:0x7f7fffff,height:0,groups:Vec::with_capacity(1)};let prior=s.clone();assert_eq!(append(&mut s,0,[0,0],0x7f7fffff,Policy::Mxcsr1f80),Err(Error::NonFiniteOrSubnormal));assert_eq!(s,prior);
 }
}
