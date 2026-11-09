//! Checked 50B419..50B53E bounds expansion, area accumulation and 50C570 rotation.
//! Caller supplies all bounds, flags, margin and incoming area. Finite normal or
//! zero values are admitted for inputs and intermediate results, with explicit
//! MXCSR 1F80. Rotation's x87 copies are exact for this admitted domain. This
//! does not perform atlas packing or establish actual groups or live identity.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { NonFiniteOrSubnormal, UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Output {pub bounds:[u32;4],pub flags:u32,pub area:u32}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Preserve source binary32 operation and mutation order; returns owned output.
pub fn project(bounds:[u32;4],flags:u32,area:u32,margin:u32,policy:Policy)->Result<Output,Error>{
 let [lo,hi,lv,hv]=bounds.map(f32::from_bits);let area=f32::from_bits(area);let margin=f32::from_bits(margin);
 for x in [lo,hi,lv,hv,area,margin]{admitted(x)?;}
 native_sse::scoped(policy,||{
  let hi=admitted(hi+margin)?;let lo=admitted(lo-margin)?;let hv=admitted(hv+margin)?;let lv=admitted(lv-margin)?;
  let width=admitted(hi-lo)?;let height=admitted(hv-lv)?;let product=admitted(width*height)?;let area=admitted(product+area)?;
  let (bounds,flags)=if width>height {([lv.to_bits(),hv.to_bits(),lo.to_bits(),hi.to_bits()],flags^1)} else {([lo.to_bits(),hi.to_bits(),lv.to_bits(),hv.to_bits()],flags)};
  Ok(Output{bounds,flags,area:area.to_bits()})
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
 use super::*;
 #[test] fn rotate_no_rotate_and_tie(){
  let bits=|a:[f32;4]|a.map(f32::to_bits);let z=0f32.to_bits();
  let a=project(bits([0.,2.,0.,1.]),2,z,z,Policy::Mxcsr1f80).unwrap();assert_eq!((a.bounds,a.flags,a.area),(bits([0.,1.,0.,2.]),3,2f32.to_bits()));
  let b=project(bits([0.,1.,0.,2.]),2,z,z,Policy::Mxcsr1f80).unwrap();assert_eq!((b.bounds,b.flags,b.area),(bits([0.,1.,0.,2.]),2,2f32.to_bits()));
  let c=project(bits([0.,1.,0.,1.]),3,z,z,Policy::Mxcsr1f80).unwrap();assert_eq!((c.flags,c.area),(3,1f32.to_bits()));
 }
 #[test] fn rejects_nonfinite_subnormal_and_overflow(){
  let p=Policy::Mxcsr1f80;assert_eq!(project([0,0,0,0],0,0,1,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(project([0,0x7f800000,0,0],0,0,0,p),Err(Error::NonFiniteOrSubnormal));
  assert_eq!(project([0,0x7f7fffff,0,0x7f7fffff],0,0,0,p),Err(Error::NonFiniteOrSubnormal));
 }
}
