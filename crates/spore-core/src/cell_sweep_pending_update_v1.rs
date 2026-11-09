//! Checked pending-only BBB880 endpoint update. No sorted-branch crossing,
//! live actor binding, allocation, persistence or callback is modeled.
use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum X87Admission { Default027f }
#[derive(Debug,PartialEq,Eq)]
pub enum Error {NotPending,InvalidIndex,DuplicateLink,UnsupportedFloat}
/// The caller admits disjoint snapshots and x87 control 027f. Each link must
/// point into the supplied axis array at distinct positions; the caller admits
/// that these entries belong to the selected slot. These extra checks are not
/// source checks. Endpoint flags and slot words are treated as opaque.
/// Normal and zero binary32 words are exact x87 load/store values under that
/// admission; no interval or actor identity is inferred. Errors do not mutate.
pub fn update(slot:&Slot,axes:&mut [Vec<Endpoint>;2],min:[u32;2],max:[u32;2],_x87:X87Admission)->Result<(),Error>{
 let links=slot.endpoint_links;
 if links.iter().any(|x|x&0x8000==0){return Err(Error::NotPending)}
 for axis in 0..2{
  let lo=usize::from(links[axis]&0x7fff);let hi=usize::from(links[axis+2]&0x7fff);
  if lo==hi{return Err(Error::DuplicateLink)}
  axes[axis].get(lo).ok_or(Error::InvalidIndex)?;axes[axis].get(hi).ok_or(Error::InvalidIndex)?;
  for word in [min[axis],max[axis]]{let x=f32::from_bits(word);if !(x.is_normal()||x==0.){return Err(Error::UnsupportedFloat)}}
 }
 // Source store order: min X, max X, min Y, max Y.
 for axis in 0..2{axes[axis][usize::from(links[axis]&0x7fff)].value_bits=min[axis];axes[axis][usize::from(links[axis+2]&0x7fff)].value_bits=max[axis]}
 Ok(())
}
#[cfg(test)] mod tests{
 use super::*;
 fn setup()->(Slot,[Vec<Endpoint>;2]){let slot=Slot{actor:4,tag:9,endpoint_links:[0x8001,0x8000,0x8000,0x8001]};let e=|opening|Endpoint{opening,slot:0,value_bits:1f32.to_bits()};(slot,[vec![e(1),e(0)],vec![e(0),e(1)]])}
 #[test] fn copies_selected_words_and_preserves_other_fields(){let (slot,mut a)=setup();let old=slot;update(&slot,&mut a,[0x80000000,1f32.to_bits()],[3f32.to_bits(),4f32.to_bits()],X87Admission::Default027f).unwrap();assert_eq!(a[0][1].value_bits,0x80000000);assert_eq!(a[0][0].value_bits,3f32.to_bits());assert_eq!(a[1][0].value_bits,1f32.to_bits());assert_eq!(slot,old);}
 #[test] fn rejects_invalid_inputs_without_writes(){let (mut s,mut a)=setup();let before=a.clone();assert_eq!(update(&s,&mut a,[0,0],[3f32.to_bits(),1],X87Admission::Default027f),Err(Error::UnsupportedFloat));assert_eq!(a,before);s.endpoint_links[0]=0;assert_eq!(update(&s,&mut a,[0,0],[0,0],X87Admission::Default027f),Err(Error::NotPending));assert_eq!(a,before);}
}
