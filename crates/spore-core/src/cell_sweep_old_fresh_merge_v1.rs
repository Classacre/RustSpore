//! One-axis checked BBA7C0 old/fresh merge model. Inputs are caller-owned,
//! disjoint snapshots; this does not produce or validate persistent lifecycle.
use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
use std::collections::HashSet;
const LIMIT:usize=4096;
#[derive(Debug,PartialEq,Eq)]
pub enum Error {TooManyEndpoints,InvalidAxis,InvalidWord,InvalidFloat,UnsortedOld,Tie,DuplicateLink,StaleOldRank}
#[derive(Debug,PartialEq,Eq)]
pub struct Merged {pub endpoints:Vec<Endpoint>,pub slots:Vec<Slot>}
/// Safety admissions are stricter than source: finite normal/zero keys, no
/// ties, disjoint immutable snapshots, <=4096 total entries, valid slot/flag,
/// distinct live links and preexisting old ranks that match their positions.
/// Tombstone marker FFFE is skipped; its value must still be a valid sort key.
pub fn merge(axis:usize,slots:&[Slot],old:&[Endpoint],fresh:&[Endpoint])->Result<Merged,Error>{
 if axis>1{return Err(Error::InvalidAxis)}
 if old.len().checked_add(fresh.len()).filter(|&n|n<=LIMIT).is_none(){return Err(Error::TooManyEndpoints)}
 let mut keys=HashSet::new();let mut links=HashSet::new();let mut previous=None;
 for (is_old,entries) in [(true,old),(false,fresh)]{
  for (i,e) in entries.iter().enumerate(){
   let x=f32::from_bits(e.value_bits);if !(x.is_normal()||x==0.){return Err(Error::InvalidFloat)}
   let key=if x==0. {0}else{e.value_bits};if !keys.insert(key){return Err(Error::Tie)}
   if e.opening!=0xfffe {
    if e.opening>1||e.slot>=0x8000||usize::from(e.slot)>=slots.len(){return Err(Error::InvalidWord)}
    let link=axis+if e.opening==0{0}else{2};
    if !links.insert((e.slot,link)){return Err(Error::DuplicateLink)}
    if is_old&&slots[e.slot as usize].endpoint_links[link]!=i as u16{return Err(Error::StaleOldRank)}
    if is_old {if previous.is_some_and(|p:f32|p>=x){return Err(Error::UnsortedOld)}previous=Some(x)}
   }
  }
 }
 let mut entries:Vec<_>=old.iter().chain(fresh).filter(|e|e.opening!=0xfffe).copied().collect();
 entries.sort_unstable_by(|a,b|if crate::cell_sweep_endpoint_compare_v1::compare(a.value_bits,b.value_bits).expect("admitted values")<0{std::cmp::Ordering::Less}else{std::cmp::Ordering::Greater});
 let mut out=slots.to_vec();for (rank,e) in entries.iter().enumerate(){out[e.slot as usize].endpoint_links[axis+if e.opening==0{0}else{2}]=rank as u16}
 Ok(Merged{endpoints:entries,slots:out})
}
#[cfg(test)] mod tests{
 use super::*;
 fn e(flag:u16,slot:u16,v:f32)->Endpoint{Endpoint{opening:flag,slot,value_bits:v.to_bits()}}
 #[test] fn merges_interleaved_and_compacts_tombstones(){let slots=vec![Slot{actor:1,tag:0,endpoint_links:[0,77,2,88]},Slot{actor:2,tag:0,endpoint_links:[0x8000,66,0x8001,99]}];let old=[e(0,0,1.),e(0xfffe,0,2.),e(1,0,5.)];let fresh=[e(1,1,4.),e(0,1,3.)];let out=merge(0,&slots,&old,&fresh).unwrap();assert_eq!(out.endpoints,[e(0,0,1.),e(0,1,3.),e(1,1,4.),e(1,0,5.)]);assert_eq!(out.slots[0].endpoint_links,[0,77,3,88]);assert_eq!(out.slots[1].endpoint_links,[1,66,2,99]);assert_eq!(slots[0].endpoint_links,[0,77,2,88]);}
 #[test] fn rejects_stale_ranks_ties_and_duplicate_links(){let slots=[Slot{actor:1,tag:0,endpoint_links:[1,0,2,0]}];assert_eq!(merge(0,&slots,&[e(0,0,1.)],&[]),Err(Error::StaleOldRank));let slots=[Slot{actor:1,tag:0,endpoint_links:[0,0,1,0]}];assert_eq!(merge(0,&slots,&[e(0,0,1.)],&[e(1,0,1.)]),Err(Error::Tie));assert_eq!(merge(0,&slots,&[e(0,0,1.)],&[e(0,0,2.)]),Err(Error::DuplicateLink));}
}
