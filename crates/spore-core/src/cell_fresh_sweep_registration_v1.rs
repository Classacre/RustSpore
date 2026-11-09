//! Controlled fresh BBAB40 append fixture. It does not model source allocation,
//! removals, free-list reuse, concurrent mutation or persistent sweep history.
use crate::cell_source_sweep_pairs_v1::{SweepSlot,MAX_SLOTS};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Endpoint {pub opening:u16,pub slot:u16,pub value_bits:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Slot {pub actor:u32,pub tag:u32,pub endpoint_links:[u16;4]}
#[derive(Debug,PartialEq,Eq)]
pub enum Error {InvalidCapacity,Full,InvalidBounds,EndpointTie}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SortedFresh {pub slots:Vec<Slot>,pub axes:[Vec<Endpoint>;2]}
/// Inspection-only fresh append, sort and pair composition. `suppressed` in
/// `pairs` is a Rust diagnostic, not a source manager field.
#[derive(Debug,PartialEq,Eq)]
pub struct FreshConsumer {pub sorted:SortedFresh,pub pairs:crate::cell_source_sweep_pairs_v1::SweepPairs}
#[derive(Debug,PartialEq,Eq)]
pub enum ConsumerError {Sort(Error),Sweep(crate::cell_source_sweep_pairs_v1::SweepError)}
#[derive(Debug)]
pub struct FreshCollection {capacity:usize,slots:Vec<Slot>,axes:[Vec<Endpoint>;2]}
impl FreshCollection {
 pub fn new(capacity:usize)->Result<Self,Error>{if capacity>MAX_SLOTS{return Err(Error::InvalidCapacity)}Ok(Self{capacity,slots:Vec::new(),axes:[Vec::new(),Vec::new()]})}
 /// Explicit finite-normal/zero disjoint snapshot and capacity preflight are
 /// extra fixture policies. Original source copies floats through x87 and does
 /// not locally perform these checks. No mutation occurs on checked error.
 pub fn append(&mut self,actor:u32,tag:u32,min:[u32;2],max:[u32;2])->Result<u32,Error>{
  if self.slots.len()>=self.capacity{return Err(Error::Full)}
  for i in 0..2 {let a=f32::from_bits(min[i]);let b=f32::from_bits(max[i]);if !((a.is_normal()||a==0.)&&(b.is_normal()||b==0.)&&a<b){return Err(Error::InvalidBounds)}}
  let slot=self.slots.len() as u16;let endpoint=self.axes[0].len() as u16;
  let lo=0x8000|endpoint;let hi=0x8000|(endpoint+1);
  self.slots.push(Slot{actor,tag,endpoint_links:[lo,lo,hi,hi]});
  for axis in 0..2 {self.axes[axis].push(Endpoint{opening:0,slot,value_bits:min[axis]});self.axes[axis].push(Endpoint{opening:1,slot,value_bits:max[axis]});}
  Ok(slot as u32)
 }
 /// Fresh-only interpretation of the two source endpoint sorts/merges.
 /// Unique finite endpoints and disjoint immutable input are required. This
 /// does not model qsort ties, old entries, tombstones or persistent history.
 pub fn sorted_fresh(&self)->Result<SortedFresh,Error>{
  let mut out=SortedFresh{slots:self.slots.clone(),axes:self.axes.clone()};
  for axis in 0..2 {
   let mut seen=std::collections::HashSet::with_capacity(out.axes[axis].len());
   for e in &out.axes[axis]{let v=f32::from_bits(e.value_bits);let key=if v==0.0{0}else{e.value_bits};if !seen.insert(key){return Err(Error::EndpointTie)}}
   out.axes[axis].sort_unstable_by(|a,b|if crate::cell_sweep_endpoint_compare_v1::compare(a.value_bits,b.value_bits).expect("append admitted endpoints")<0{std::cmp::Ordering::Less}else{std::cmp::Ordering::Greater});
   for (rank,e) in out.axes[axis].iter().enumerate(){out.slots[e.slot as usize].endpoint_links[axis+if e.opening==0{0}else{2}]=rank as u16;}
  }
  Ok(out)
 }
 /// Checked fresh-only composition; no prior endpoints, allocation, live
 /// registration or actor identity is admitted. The two sorts use the same
 /// finite, unique input; the sweep retains its own checked admission limits.
 pub fn consume_fresh(&self,pair_capacity:usize)->Result<FreshConsumer,ConsumerError>{
  let sorted=self.sorted_fresh().map_err(ConsumerError::Sort)?;
  let pairs=crate::cell_source_sweep_pairs_v1::rebuild(&self.sweep_slots(),pair_capacity).map_err(ConsumerError::Sweep)?;
  Ok(FreshConsumer{sorted,pairs})
 }
 pub fn slots(&self)->&[Slot]{&self.slots}
 pub fn endpoints(&self,axis:usize)->Option<&[Endpoint]>{self.axes.get(axis).map(Vec::as_slice)}
 /// Current append snapshot only. The caller must still validate endpoint ties
 /// and sweep restrictions; there is no source persistent-state claim.
 pub fn sweep_slots(&self)->Vec<SweepSlot>{self.slots.iter().enumerate().map(|(i,s)|SweepSlot{id:s.actor as i32,tag:s.tag,min:[f32::from_bits(self.axes[0][2*i].value_bits),f32::from_bits(self.axes[1][2*i].value_bits)],max:[f32::from_bits(self.axes[0][2*i+1].value_bits),f32::from_bits(self.axes[1][2*i+1].value_bits)]}).collect()}
}
#[cfg(test)] mod tests{
 use super::*;
 #[test] fn appends_source_shaped_endpoints_and_checked_capacity(){let mut c=FreshCollection::new(2).unwrap();assert_eq!(c.append(0x80010000,9,[0f32.to_bits(),(-2f32).to_bits()],[1f32.to_bits(),2f32.to_bits()]).unwrap(),0);assert_eq!(c.append(0x20001,0,[1f32.to_bits(),3f32.to_bits()],[2f32.to_bits(),4f32.to_bits()]).unwrap(),1);assert_eq!(c.slots()[1].endpoint_links,[0x8002,0x8002,0x8003,0x8003]);assert_eq!(c.endpoints(0).unwrap()[2],Endpoint{opening:0,slot:1,value_bits:1f32.to_bits()});assert_eq!(c.sweep_slots()[0].id,0x80010000u32 as i32);assert_eq!(c.append(3,0,[0,0],[1f32.to_bits();2]),Err(Error::Full));assert_eq!(c.slots().len(),2);}
 #[test] fn fresh_sort_rewrites_slot_ranks_and_rejects_ties(){let mut c=FreshCollection::new(2).unwrap();c.append(10,0,[0f32.to_bits(),0f32.to_bits()],[3f32.to_bits(),3f32.to_bits()]).unwrap();c.append(20,0,[1f32.to_bits(),1f32.to_bits()],[2f32.to_bits(),2f32.to_bits()]).unwrap();let sorted=c.sorted_fresh().unwrap();assert_eq!(sorted.slots[0].endpoint_links,[0,0,3,3]);assert_eq!(sorted.slots[1].endpoint_links,[1,1,2,2]);assert_eq!(c.slots()[0].endpoint_links,[0x8000,0x8000,0x8001,0x8001]);let mut t=FreshCollection::new(2).unwrap();t.append(1,0,[0,0],[1f32.to_bits();2]).unwrap();t.append(2,0,[0,2f32.to_bits()],[3f32.to_bits(),4f32.to_bits()]).unwrap();assert_eq!(t.sorted_fresh(),Err(Error::EndpointTie));}
 #[test] fn fresh_consumer_preserves_sorted_ranks_and_capacity(){let mut c=FreshCollection::new(2).unwrap();c.append(10,0,[0f32.to_bits(),0f32.to_bits()],[3f32.to_bits(),3f32.to_bits()]).unwrap();c.append(20,0,[1f32.to_bits(),1f32.to_bits()],[2f32.to_bits(),2f32.to_bits()]).unwrap();let out=c.consume_fresh(0).unwrap();assert_eq!(out.sorted.slots[0].endpoint_links,[0,0,3,3]);assert_eq!(out.sorted.slots[1].endpoint_links,[1,1,2,2]);assert!(out.pairs.stored_order().is_empty());assert_eq!(out.pairs.suppressed,1);let out=c.consume_fresh(1).unwrap();assert_eq!(out.pairs.stored_order(),&[(10,20)]);assert_eq!(c.slots()[0].endpoint_links,[0x8000,0x8000,0x8001,0x8001]);}
 #[test] fn bad_input_does_not_commit(){let mut c=FreshCollection::new(1).unwrap();assert_eq!(c.append(1,0,[0,0],[0,1f32.to_bits()]),Err(Error::InvalidBounds));assert!(c.slots().is_empty());assert_eq!(c.append(1,0,[1,0],[1f32.to_bits();2]),Err(Error::InvalidBounds));assert!(c.endpoints(0).unwrap().is_empty());}
}
