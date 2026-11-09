//! Checked 50BF38..50BFFC per-group atlas offset projection.
//! Group bounds, shelf height/shift, row coordinates and row index are explicit.
//! Finite normal-or-zero values and intermediates under MXCSR 1F80 are admitted.
//! Source row pairing, atlas placement/scale, texture and live identity are not
//! inferred. Owned output excludes native partial mutation on rejected inputs.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{NonFiniteOrSubnormal,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Output{pub width:u32,pub height:u32,pub offset:[u32;2]}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Compute unscaled offsets for one supplied group and shelf.
pub fn project(bounds:[u32;4],shelf_height:u32,shelf_shift:u32,row_x:u32,row_y:u32,row_index:u32,policy:Policy)->Result<Output,Error>{
 let [lo,hi,lv,hv]=bounds.map(f32::from_bits);let shelf_height=f32::from_bits(shelf_height);let shelf_shift=f32::from_bits(shelf_shift);let row_x=f32::from_bits(row_x);let row_y=f32::from_bits(row_y);
 for x in [lo,hi,lv,hv,shelf_height,shelf_shift,row_x,row_y]{admitted(x)?;}
 native_sse::scoped(policy,||{
  let width=admitted(hi-lo)?;let height=admitted(hv-lv)?;let du=admitted(row_x-lo)?;let mut dv=admitted(row_y-lv)?;
  if row_index&1!=0{let gap=admitted(shelf_height-height)?;let shifted=admitted(gap+shelf_shift)?;dv=admitted(shifted+dv)?;}
  Ok(Output{width:width.to_bits(),height:height.to_bits(),offset:[du.to_bits(),dv.to_bits()]})
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn even_and_odd_offsets(){
  let b=|x:f32|x.to_bits();let p=Policy::Mxcsr1f80;let bounds=[0,b(1.),0,b(0.5)];
  let a=project(bounds,b(1.),b(0.25),b(0.5),b(0.75),0,p).unwrap();assert_eq!((a.width,a.height,a.offset),(b(1.),b(0.5),[b(0.5),b(0.75)]));
  let a=project(bounds,b(1.),b(0.25),b(0.5),b(0.75),1,p).unwrap();assert_eq!(a.offset,[b(0.5),b(1.5)]);
 }
 #[test] fn rejects_nonfinite_subnormal_and_overflow(){
  let p=Policy::Mxcsr1f80;assert_eq!(project([1,0,0,0],0,0,0,0,0,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(project([0,0,0,0],0,0,0,0x7f800000,0,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(project([0xff7fffff,0x7f7fffff,0,0],0,0,0,0,0,p),Err(Error::NonFiniteOrSubnormal));
 }
}
