//! Ordered edge-pair lookup used by 53FBA0 (50EB00).
use crate::creature_skin_corner_lookup_v1::{corner_hash,SLOT_COUNT};
/// Native first-word empty marker is high byte FF; ordered words are not canonicalized.
pub fn edge_hash(first:u32,second:u32)->u32 {corner_hash(first).wrapping_mul(0x01000193)^second}
/// Each entry holds two key words. Returns FFFFFFFF on source full-table exhaustion;
/// None means the supplied table does not meet the fixed-capacity admission.
pub fn lookup(keys:&[[u32;2]],first:u32,second:u32)->Option<u32>{
 if keys.len()!=SLOT_COUNT{return None;}
 let h=edge_hash(first,second);let mut i=(h&0x7fff) as usize;
 let matches=|entry:[u32;2]|entry==[first,second];
 if keys[i][0]>>24==0xff||matches(keys[i]){return Some(i as u32);}
 let stride=((h.rotate_left(16)|1)&0x7fff) as usize;let mut count=1u32;
 loop {i=(i+stride)&0x7fff;count+=1;if matches(keys[i])||keys[i][0]>>24==0xff{return Some(i as u32);}if count>0x8000{return Some(u32::MAX);}}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn ordered_pair_and_full_second_word(){assert_ne!(edge_hash(0x123456,1),edge_hash(0x123456,0x1000001));assert_ne!(edge_hash(1,2),edge_hash(2,1));}
 #[test]fn hit_collision_and_empty(){let mut t=vec![[u32::MAX;2];SLOT_COUNT];let(a,b)=(0x123456,0x765432);let i=lookup(&t,a,b).unwrap() as usize;t[i]=[a,b];assert_eq!(lookup(&t,a,b),Some(i as u32));t[i]=[a,b^1];let j=lookup(&t,a,b).unwrap() as usize;assert_ne!(i,j);t[j]=[a,b];assert_eq!(lookup(&t,a,b),Some(j as u32));}
 #[test]fn full_and_bad_size(){assert_eq!(lookup(&[],1,2),None);assert_eq!(lookup(&vec![[0,0];SLOT_COUNT],1,2),Some(u32::MAX));}
}
