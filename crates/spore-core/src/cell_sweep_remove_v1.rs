//! Checked BBAC30-shaped synthetic removal; no source actor or lifecycle binding.
use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Storage {pub slots:Vec<Slot>,pub old:[Vec<Endpoint>;2],pub pending:[Vec<Endpoint>;2],pub live:u32,pub high:u32,pub free_head:u32}
#[derive(Debug,PartialEq,Eq)]
pub enum Error {InvalidSlot,NoLiveSlots,InvalidHead,InvalidLink,DuplicateLink}
/// Caller admits that the target is live, endpoint links belong to it, free
/// chain is coherent and all arrays are disjoint. Additional bounds, nonzero
/// live count and distinct selected entries are checked before any mutation;
/// these are fixture rules and are not checks performed by the source.
pub fn remove(s:&mut Storage,index:u32)->Result<(),Error>{
 if s.live==0{return Err(Error::NoLiveSlots)}
 if index>=s.high||index as usize>=s.slots.len(){return Err(Error::InvalidSlot)}
 if s.free_head!=u32::MAX&&(s.free_head>=s.high||s.free_head as usize>=s.slots.len()||s.free_head==index){return Err(Error::InvalidHead)}
 let links=s.slots[index as usize].endpoint_links;let mut seen=std::collections::HashSet::new();
 for i in 0..4{let axis=i%2;let pending=links[i]&0x8000!=0;let idx=usize::from(links[i]&0x7fff);let array=if pending{&s.pending[axis]}else{&s.old[axis]};if idx>=array.len(){return Err(Error::InvalidLink)}if !seen.insert((axis,pending,idx)){return Err(Error::DuplicateLink)}}
 // Preserve the native max-X, min-X, max-Y, min-Y write order.
 for i in [2,0,3,1]{let axis=i%2;let idx=usize::from(links[i]&0x7fff);let array=if links[i]&0x8000!=0{&mut s.pending[axis]}else{&mut s.old[axis]};array[idx].opening=0xfffe;}
 let slot=&mut s.slots[index as usize];slot.actor=s.free_head;slot.tag=u32::MAX;s.live-=1;s.free_head=index;Ok(())
}
#[cfg(test)] mod tests{
 use super::*;
 fn fixture()->Storage{let e=||Endpoint{opening:0,slot:7,value_bits:42};Storage{slots:vec![Slot{actor:9,tag:8,endpoint_links:[0,0x8000,0x8001,1]},Slot{actor:u32::MAX,tag:u32::MAX,endpoint_links:[6;4]}],old:[vec![e(),e()],vec![e(),e()]],pending:[vec![e(),e()],vec![e(),e()]],live:1,high:2,free_head:1}}
 #[test] fn mixed_removal_tombstones_and_prepends_free_chain(){let mut s=fixture();let old=s.clone();remove(&mut s,0).unwrap();assert_eq!((s.live,s.high,s.free_head),(0,2,0));assert_eq!((s.slots[0].actor,s.slots[0].tag),(1,u32::MAX));assert_eq!(s.slots[0].endpoint_links,old.slots[0].endpoint_links);for e in [&s.old[0][0],&s.pending[0][1],&s.pending[1][0],&s.old[1][1]]{assert_eq!((e.opening,e.slot,e.value_bits),(0xfffe,7,42));}}
 #[test] fn invalid_links_reject_without_mutation(){let mut s=fixture();s.slots[0].endpoint_links[0]=0x7fff;let before=s.clone();assert_eq!(remove(&mut s,0),Err(Error::InvalidLink));assert_eq!(s,before);}
}
