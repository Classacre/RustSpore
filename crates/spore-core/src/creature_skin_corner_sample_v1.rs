//! Bounded 53F540 corner-cache hit/miss composition.
use crate::{creature_skin_corner_lookup_v1 as lookup, creature_skin_lattice_world_v1 as lattice, native_sse::Policy};

#[derive(Debug, PartialEq, Eq)]
pub enum Error { InvalidBuffers, InvalidWorld(lattice::Error) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Hit, Miss }
/// Arrays represent nonaliasing preallocated native storage. Callback must complete
/// successfully; no allocation, failure/exception, live dispatch or runtime identity is modeled.
/// `records[n]` contains six native words, including the preserved +08 word.
/// Invalid input is rejected before mutation, outside the native behavior contract.
pub fn sample<F: FnOnce([f32; 3], &mut [u32; 2])>(
 key:u32, keys:&mut [u32], values:&mut [[u32;2]], records:&mut [[u32;6]],
 status:&mut u32, count:&mut u32, out:&mut [u32;2], aligned:[f32;3], step:f32,
 policy:Policy, field:F,
)->Result<Outcome,Error> {
 if keys.len()!=lookup::SLOT_COUNT || values.len()!=lookup::SLOT_COUNT {return Err(Error::InvalidBuffers);}
 let index=lookup::lookup(keys,key).ok_or(Error::InvalidBuffers)?;
 if index < 0x8000 && keys[index as usize]==key { *out=values[index as usize]; return Ok(Outcome::Hit); }
 let n=*count as usize;
 if n>=records.len() {return Err(Error::InvalidBuffers);}
 let point=lattice::world(key,aligned,step,policy,lattice::Admission).map_err(Error::InvalidWorld)?;
 *status |= index & 0xffff8000;
 records[n][0]=key;
 records[n][3..6].copy_from_slice(&point.map(f32::to_bits));
 records[n][1]=u32::MAX;
 field(point,out);
 out[1]=*count;
 let slot=(index & 0x7fff) as usize;
 keys[slot]=key;
 values[slot]=*out;
 *count=count.wrapping_add(1);
 Ok(Outcome::Miss)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn miss_then_hit_preserves_record_and_skips_callback() {
  let mut keys=vec![u32::MAX;lookup::SLOT_COUNT];let mut vals=vec![[0;2];lookup::SLOT_COUNT];let mut records=[[77;6];1];let(mut status,mut count,mut out)=(3,0,[9,10]);let mut calls=0;
  assert_eq!(sample(0x010203,&mut keys,&mut vals,&mut records,&mut status,&mut count,&mut out,[0.;3],0.5,Policy::Mxcsr1f80,|p,o|{calls+=1;assert_eq!(p,[0.5,1.,1.5]);*o=[42,99]}),Ok(Outcome::Miss));
  assert_eq!(out,[42,0]);assert_eq!(records[0],[0x010203,u32::MAX,77,0.5f32.to_bits(),1f32.to_bits(),1.5f32.to_bits()]);
  assert_eq!(sample(0x010203,&mut keys,&mut vals,&mut records,&mut status,&mut count,&mut out,[0.;3],0.5,Policy::Mxcsr1f80,|_,_|panic!("hit called field")),Ok(Outcome::Hit));assert_eq!((count,calls,status),(1,1,3));
 }
 #[test] fn full_table_overwrites_last_and_sets_status() {
  let mut keys=vec![0;lookup::SLOT_COUNT];let mut vals=vec![[0;2];lookup::SLOT_COUNT];let mut records=[[8;6];1];let(mut status,mut count,mut out)=(2,0,[0;2]);
  assert_eq!(sample(0x010203,&mut keys,&mut vals,&mut records,&mut status,&mut count,&mut out,[0.;3],1.,Policy::Mxcsr1f80,|_,o|o[0]=17),Ok(Outcome::Miss));
  assert_eq!(status,0xffff8002);assert_eq!(keys[0x7fff],0x010203);assert_eq!(vals[0x7fff],[17,0]);assert_eq!(records[0][2],8);
 }
 #[test] fn missing_capacity_does_not_mutate() {
  let mut keys=vec![u32::MAX;lookup::SLOT_COUNT];let mut vals=vec![[0;2];lookup::SLOT_COUNT];let(mut status,mut count,mut out)=(0,0,[7,8]);
  assert_eq!(sample(1,&mut keys,&mut vals,&mut [],&mut status,&mut count,&mut out,[0.;3],1.,Policy::Mxcsr1f80,|_,_|panic!()),Err(Error::InvalidBuffers));assert_eq!((status,count,out),(0,0,[7,8]));
 }
}
