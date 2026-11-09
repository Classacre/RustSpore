//! Controlled append-only two-frame inspection. Stable old snapshots and all
//! new actor records are caller admissions, not source lifecycle or identities.
use crate::cell_fresh_sweep_registration_v1::{FreshCollection,SortedFresh,Endpoint};
use crate::cell_source_sweep_pairs_v1::{self as sweep,SweepPairs};
use crate::cell_sweep_old_fresh_merge_v1 as merge;
#[derive(Debug,PartialEq,Eq)]
pub enum Error {First(crate::cell_fresh_sweep_registration_v1::Error),TooManySlots,DuplicateActor,Merge(merge::Error),Pairs(sweep::SweepError)}
#[derive(Debug,PartialEq,Eq)]
pub struct TwoFrame {pub first:SortedFresh,pub second:SortedFresh,pub pairs:SweepPairs}
/// Frame one is an owned fresh snapshot. Frame two explicitly appends the
/// separate supplied snapshot, with a checked slot-index translation. No
/// mutation/update, removal, source actor binding or callback is implied.
/// The final pair diagnostic independently sorts the supplied bounds again.
pub fn inspect(first:&FreshCollection,added:&FreshCollection,capacity:usize)->Result<TwoFrame,Error>{
 let a=first.sorted_fresh().map_err(Error::First)?;
 let n=a.slots.len();let total=n.checked_add(added.slots().len()).filter(|&n|n<=sweep::MAX_SLOTS).ok_or(Error::TooManySlots)?;
 let mut ids=std::collections::HashSet::new();if first.slots().iter().chain(added.slots()).any(|s|!ids.insert(s.actor)){return Err(Error::DuplicateActor)}
 let mut slots=a.slots.clone();slots.extend_from_slice(added.slots());
 let mut axes:[Vec<Endpoint>;2]=std::array::from_fn(|_|Vec::new());
 for axis in 0..2 {
  let fresh:Vec<_>=added.endpoints(axis).expect("axis in range").iter().map(|e|Endpoint{slot:(usize::from(e.slot)+n) as u16,..*e}).collect();
  let result=merge::merge(axis,&slots,&a.axes[axis],&fresh).map_err(Error::Merge)?;
  slots=result.slots;axes[axis]=result.endpoints;
 }
 let mut input=first.sweep_slots();input.extend(added.sweep_slots());debug_assert_eq!(input.len(),total);
 let pairs=sweep::rebuild(&input,capacity).map_err(Error::Pairs)?;
 Ok(TwoFrame{first:a,second:SortedFresh{slots,axes},pairs})
}
/// Caller-supplied raw pre-sort values for one newly appended synthetic slot.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PendingBounds {pub slot:usize,pub min:[u32;2],pub max:[u32;2]}
#[derive(Debug,PartialEq,Eq)]
pub enum PendingError {InvalidSlot,DuplicateSlot,Update(crate::cell_sweep_pending_update_v1::Error),Registration(crate::cell_fresh_sweep_registration_v1::Error),Inspect(Error)}
/// Separately admitted path: pending snapshot records must still belong to the
/// caller-supplied slots, and x87 027f is explicit. Updates are applied to owned
/// temporary arrays, then bounds are checked by fresh registration. Thus any
/// error leaves both caller snapshots unchanged. Sorted endpoint updates,
/// persistence and source actor ownership remain outside this fixture.
pub fn inspect_with_pending_updates(first:&FreshCollection,added:&FreshCollection,updates:&[PendingBounds],capacity:usize,x87:crate::cell_sweep_pending_update_v1::X87Admission)->Result<TwoFrame,PendingError>{
 let mut axes=[added.endpoints(0).expect("axis").to_vec(),added.endpoints(1).expect("axis").to_vec()];
 let mut seen=std::collections::HashSet::new();
 for change in updates {
  let slot=added.slots().get(change.slot).ok_or(PendingError::InvalidSlot)?;
  if !seen.insert(change.slot){return Err(PendingError::DuplicateSlot)}
  crate::cell_sweep_pending_update_v1::update(slot,&mut axes,change.min,change.max,x87).map_err(PendingError::Update)?;
 }
 let mut reconstructed=FreshCollection::new(added.slots().len()).map_err(PendingError::Registration)?;
 for slot in added.slots(){
  let mut min=[0;2];let mut max=[0;2];
  for axis in 0..2 {
   let lo=usize::from(slot.endpoint_links[axis]&0x7fff);let hi=usize::from(slot.endpoint_links[axis+2]&0x7fff);
   let a=axes[axis].get(lo).ok_or(PendingError::InvalidSlot)?;let b=axes[axis].get(hi).ok_or(PendingError::InvalidSlot)?;
   min[axis]=a.value_bits;max[axis]=b.value_bits;
  }
  reconstructed.append(slot.actor,slot.tag,min,max).map_err(PendingError::Registration)?;
 }
 inspect(first,&reconstructed,capacity).map_err(PendingError::Inspect)
}
#[cfg(test)] mod tests{
 use super::*;
 fn append(c:&mut FreshCollection,id:u32,min:[f32;2],max:[f32;2]){c.append(id,0,min.map(f32::to_bits),max.map(f32::to_bits)).unwrap();}
 #[test] fn retains_first_then_merges_new_slot_and_pairs(){let mut a=FreshCollection::new(1).unwrap();let mut b=FreshCollection::new(1).unwrap();append(&mut a,10,[0.,0.],[5.,6.]);append(&mut b,20,[1.,2.],[4.,3.]);let out=inspect(&a,&b,2).unwrap();assert_eq!(out.first.slots[0].endpoint_links,[0,0,1,1]);assert_eq!(out.second.slots[0].endpoint_links,[0,0,3,3]);assert_eq!(out.second.slots[1].endpoint_links,[1,1,2,2]);assert_eq!(out.pairs.stored_order(),&[(10,20)]);assert_eq!(out.second.axes[0][1].slot,1);assert_eq!(a.slots()[0].endpoint_links,[0x8000,0x8000,0x8001,0x8001]);}
 #[test] fn pending_update_changes_second_frame_before_merge(){let mut a=FreshCollection::new(1).unwrap();let mut b=FreshCollection::new(1).unwrap();append(&mut a,10,[0.,0.],[5.,6.]);append(&mut b,20,[8.,9.],[10.,11.]);let change=PendingBounds{slot:0,min:[1f32.to_bits(),2f32.to_bits()],max:[4f32.to_bits(),3f32.to_bits()]};let before=b.endpoints(0).unwrap().to_vec();let out=inspect_with_pending_updates(&a,&b,&[change],2,crate::cell_sweep_pending_update_v1::X87Admission::Default027f).unwrap();assert_eq!(out.pairs.stored_order(),&[(10,20)]);assert_eq!(out.second.axes[0][1].value_bits,1f32.to_bits());assert_eq!(b.endpoints(0).unwrap(),before);}
 #[test] fn pending_update_rejections_leave_snapshots_unchanged(){let mut a=FreshCollection::new(1).unwrap();let mut b=FreshCollection::new(1).unwrap();append(&mut a,10,[0.,0.],[5.,6.]);append(&mut b,20,[8.,9.],[10.,11.]);let before=b.endpoints(0).unwrap().to_vec();let mut change=PendingBounds{slot:0,min:[1f32.to_bits(),2f32.to_bits()],max:[4f32.to_bits(),3f32.to_bits()]};let policy=crate::cell_sweep_pending_update_v1::X87Admission::Default027f;assert_eq!(inspect_with_pending_updates(&a,&b,&[change,change],2,policy),Err(PendingError::DuplicateSlot));change.max[0]=0;assert_eq!(inspect_with_pending_updates(&a,&b,&[change],2,policy),Err(PendingError::Registration(crate::cell_fresh_sweep_registration_v1::Error::InvalidBounds)));assert_eq!(b.endpoints(0).unwrap(),before);}
 #[test] fn rejects_duplicate_ids_and_cross_frame_ties(){let mut a=FreshCollection::new(1).unwrap();let mut b=FreshCollection::new(1).unwrap();append(&mut a,10,[0.,0.],[5.,6.]);append(&mut b,10,[1.,2.],[4.,3.]);assert_eq!(inspect(&a,&b,2),Err(Error::DuplicateActor));let mut c=FreshCollection::new(1).unwrap();append(&mut c,20,[0.,2.],[4.,3.]);assert_eq!(inspect(&a,&c,2),Err(Error::Merge(merge::Error::Tie)));}
}
