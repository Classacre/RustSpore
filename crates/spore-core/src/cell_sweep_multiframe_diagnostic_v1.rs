//! Opt-in synthetic multiframe composition. Stable snapshots, slot ownership
//! and the absence of source lifecycle changes are caller admissions.
use crate::cell_fresh_sweep_registration_v1::{FreshCollection,SortedFresh,Endpoint};
use crate::{cell_sweep_sorted_update_v1 as sorted,cell_sweep_pending_update_v1 as pending,cell_sweep_old_fresh_merge_v1 as merge,cell_source_sweep_pairs_v1 as sweep};
use crate::cell_sweep_two_frame_diagnostic_v1::PendingBounds;
#[derive(Debug,PartialEq,Eq)] pub enum Error{First(crate::cell_fresh_sweep_registration_v1::Error),InvalidSlot,DuplicateSlot,DuplicateActor,TooManySlots,Pending(pending::Error),Sorted(sorted::Error),Merge(merge::Error),Pairs(sweep::SweepError)}
#[derive(Debug,PartialEq,Eq)] pub struct Diagnostic{pub first:SortedFresh,pub updated_old:SortedFresh,pub combined:SortedFresh,pub pairs:sweep::SweepPairs}
/// Update one selected old slot and zero or more newly appended pending slots,
/// then merge two axes. Both inputs remain immutable on every outcome. The
/// pair diagnostic independently re-sorts reconstructed combined bounds.
/// Unique finite normal/zero endpoints, strict valid intervals, coherent ranks,
/// nonaliasing, stable slot ownership, x87 027f and no lifecycle/pair effects
/// are fixture admissions or checked restrictions, not native validation.
pub fn inspect(first:&FreshCollection,added:&FreshCollection,old_slot:usize,old_min:[u32;2],old_max:[u32;2],changes:&[PendingBounds],capacity:usize,x87:pending::X87Admission)->Result<Diagnostic,Error>{
 let before=first.sorted_fresh().map_err(Error::First)?;let n=before.slots.len();
 n.checked_add(added.slots().len()).filter(|&n|n<=sweep::MAX_SLOTS).ok_or(Error::TooManySlots)?;
 let mut ids=std::collections::HashSet::new();if first.slots().iter().chain(added.slots()).any(|s|!ids.insert(s.actor)){return Err(Error::DuplicateActor)}
 let moved=sorted::update(&before.slots,&before.axes,old_slot,old_min,old_max).map_err(Error::Sorted)?;
 let updated_old=SortedFresh{slots:moved.slots,axes:moved.axes};
 let mut fresh=[added.endpoints(0).expect("axis").to_vec(),added.endpoints(1).expect("axis").to_vec()];let mut seen=std::collections::HashSet::new();
 for change in changes {let slot=added.slots().get(change.slot).ok_or(Error::InvalidSlot)?;if !seen.insert(change.slot){return Err(Error::DuplicateSlot)}pending::update(slot,&mut fresh,change.min,change.max,x87).map_err(Error::Pending)?;}
 let mut slots=updated_old.slots.clone();slots.extend_from_slice(added.slots());let mut axes:[Vec<Endpoint>;2]=std::array::from_fn(|_|Vec::new());
 for axis in 0..2 {let new:Vec<_>=fresh[axis].iter().map(|e|Endpoint{slot:(usize::from(e.slot)+n) as u16,..*e}).collect();let r=merge::merge(axis,&slots,&updated_old.axes[axis],&new).map_err(Error::Merge)?;slots=r.slots;axes[axis]=r.endpoints;}
 let mut input=Vec::with_capacity(slots.len());
 for slot in &slots{let mut min=[0.;2];let mut max=[0.;2];for axis in 0..2 {let lo=usize::from(slot.endpoint_links[axis]);let hi=usize::from(slot.endpoint_links[axis+2]);min[axis]=f32::from_bits(axes[axis].get(lo).ok_or(Error::InvalidSlot)?.value_bits);max[axis]=f32::from_bits(axes[axis].get(hi).ok_or(Error::InvalidSlot)?.value_bits)}input.push(sweep::SweepSlot{id:slot.actor as i32,tag:slot.tag,min,max});}
 let pairs=sweep::rebuild(&input,capacity).map_err(Error::Pairs)?;
 Ok(Diagnostic{first:before,updated_old,combined:SortedFresh{slots,axes},pairs})
}
#[cfg(test)]mod tests{
 use super::*;fn add(c:&mut FreshCollection,id:u32,min:[f32;2],max:[f32;2]){c.append(id,0,min.map(f32::to_bits),max.map(f32::to_bits)).unwrap();}
 #[test]fn sorted_and_pending_updates_reach_shared_merge(){let mut a=FreshCollection::new(2).unwrap();let mut b=FreshCollection::new(1).unwrap();add(&mut a,10,[0.,0.],[5.,6.]);add(&mut a,20,[2.,2.],[3.,3.]);add(&mut b,30,[8.,8.],[9.,9.]);let ch=PendingBounds{slot:0,min:[2.25f32.to_bits(),1f32.to_bits()],max:[2.8f32.to_bits(),4f32.to_bits()]};let before=b.endpoints(0).unwrap().to_vec();let out=inspect(&a,&b,0,[1f32.to_bits(),0f32.to_bits()],[4f32.to_bits(),6f32.to_bits()],&[ch],4,pending::X87Admission::Default027f).unwrap();assert_eq!(out.first.axes[0][0].value_bits,0f32.to_bits());assert_eq!(out.updated_old.axes[0][0].value_bits,1f32.to_bits());assert_eq!(out.combined.slots.len(),3);assert_eq!(out.pairs.stored_order().len(),3);assert_eq!(b.endpoints(0).unwrap(),before);}
 #[test]fn failures_are_atomic(){let mut a=FreshCollection::new(1).unwrap();let mut b=FreshCollection::new(1).unwrap();add(&mut a,10,[0.,0.],[5.,6.]);add(&mut b,20,[8.,8.],[9.,9.]);let before=b.endpoints(0).unwrap().to_vec();let ch=PendingBounds{slot:0,min:[1f32.to_bits(),1f32.to_bits()],max:[2f32.to_bits(),2f32.to_bits()]};assert_eq!(inspect(&a,&b,0,[0,0],[5f32.to_bits(),6f32.to_bits()],&[ch,ch],2,pending::X87Admission::Default027f),Err(Error::DuplicateSlot));assert_eq!(b.endpoints(0).unwrap(),before);}
}
