//! Controlled BBB880 sorted-branch composition. Pair effects are disabled,
//! records are synthetic, and no persistent source ownership is inferred.
use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
use crate::cell_sweep_sorted_crossing_v1::{self as crossing,Direction,Side};
#[derive(Debug,PartialEq,Eq)] pub enum Error{InvalidSlot,InvalidBounds,UnsupportedFloat,Crossing(crossing::Error)}
#[derive(Debug,PartialEq,Eq)] pub struct Updated{pub slots:Vec<Slot>,pub axes:[Vec<Endpoint>;2]}
/// Values must be finite normal/zero with strict intervals, source snapshots
/// disjoint and stable, ranks coherent, and all intermediate endpoint values
/// unique. Pair callbacks/effects are disabled as at BBB880's helper calls.
/// Extra checks are fixture safeguards; source itself does not enforce them.
/// Inputs are immutable so a rejected intermediate returns no partial writes.
pub fn update(slots:&[Slot],axes:&[Vec<Endpoint>;2],selected:usize,min:[u32;2],max:[u32;2])->Result<Updated,Error>{
 if selected>=slots.len(){return Err(Error::InvalidSlot)}
 for a in 0..2{for bits in [min[a],max[a]]{let f=f32::from_bits(bits);if !(f.is_normal()||f==0.){return Err(Error::UnsupportedFloat)}}if f32::from_bits(min[a])>=f32::from_bits(max[a]){return Err(Error::InvalidBounds)}}
 let mut result=Updated{slots:slots.to_vec(),axes:axes.clone()};
 for axis in 0..2 {
  // Validate both chosen links, all ranks, values and sortedness even when no
  // source helper is needed; leftward no-op must remain sorted.
  for side in [Side::Minimum,Side::Maximum]{crossing::reorder(&result.slots,axis,&result.axes[axis],selected,side,Direction::Left).map_err(Error::Crossing)?;}
  for (side,direction,wanted) in [(Side::Minimum,Direction::Left,min[axis]),(Side::Maximum,Direction::Right,max[axis]),(Side::Minimum,Direction::Right,min[axis]),(Side::Maximum,Direction::Left,max[axis])] {
   let link=axis+if side==Side::Minimum{0}else{2};let rank=usize::from(result.slots[selected].endpoint_links[link]);let old=f32::from_bits(result.axes[axis][rank].value_bits);let new=f32::from_bits(wanted);
   let condition=match direction{Direction::Left=>new<old,Direction::Right=>new>old};
   if condition {result.axes[axis][rank].value_bits=wanted;let r=crossing::reorder(&result.slots,axis,&result.axes[axis],selected,side,direction).map_err(Error::Crossing)?;result.slots=r.slots;result.axes[axis]=r.endpoints;}
  }
 }
 Ok(result)
}
#[cfg(test)] mod tests{
 use super::*;fn e(flag:u16,slot:u16,v:f32)->Endpoint{Endpoint{opening:flag,slot,value_bits:v.to_bits()}}
 fn fixture()->(Vec<Slot>,[Vec<Endpoint>;2]){(vec![Slot{actor:1,tag:0,endpoint_links:[0,0,3,3]},Slot{actor:2,tag:0,endpoint_links:[1,1,2,2]}],[vec![e(0,0,0.),e(0,1,2.),e(1,1,3.),e(1,0,5.)],vec![e(0,0,0.),e(0,1,2.),e(1,1,3.),e(1,0,5.)]])}
 #[test] fn shrinking_crosses_in_source_order(){let (s,a)=fixture();let out=update(&s,&a,0,[2.5f32.to_bits(),1f32.to_bits()],[2.75f32.to_bits(),4f32.to_bits()]).unwrap();assert_eq!(out.slots[0].endpoint_links,[1,0,2,3]);assert_eq!(out.slots[1].endpoint_links,[0,1,3,2]);assert_eq!(a[0][0].value_bits,0f32.to_bits());}
 #[test] fn rejects_intermediate_tie_without_mutation(){let (s,a)=fixture();let before=a.clone();assert_eq!(update(&s,&a,0,[2f32.to_bits(),0],[4f32.to_bits(),5f32.to_bits()]),Err(Error::Crossing(crossing::Error::Tie)));assert_eq!(a,before);}
}
