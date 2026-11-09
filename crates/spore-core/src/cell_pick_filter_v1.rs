//! Supplied-input projection of Cell `E6B560`'s candidate predicate.
//! No actor pool, cache provider, definition resolver, name lookup, selection
//! ordering or live player binding is implied. The separate `E620D0` path is
//! deliberately outside this function. Errors are Rust admission boundaries.
use crate::{cell_definition::CellDefinition,cell_movement_scaling::progress_scale_index};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct CacheBytes {pub flag_1250:u8,pub flag_1251:u8}
#[derive(Clone,Copy,Debug)]
pub struct Input<'a>{
 pub player_handle:u32, pub candidate_id:u32,
 /// Word +358 from the explicitly admitted resolved player actor.
 pub player_size:Option<u32>,
 /// Word +358 from the explicitly admitted candidate actor.
 pub candidate_size:u32,
 /// Explicit result of the player E67A50 cache lookup.
 pub player_cache:Option<CacheBytes>,
 /// Only read after the source size relation accepts 0 or 1.
 pub candidate_definition:Option<CellDefinition<'a>>,
 /// Read only when a reached +358 equals FFFFFFFF; admitted domain 0..1000.
 pub progress:Option<i32>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {MissingPlayer,MissingCache,MissingProgress,UnsupportedProgress,MissingDefinition}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ResultTrace {pub accepted:bool,pub player_resolved:bool,pub cache_read:bool,pub progress_reads:u8,pub relation:Option<i32>,pub definition_read:bool}

/// Models the reached native branch under supplied, stable, nonaliasing inputs.
/// Cache and player references are read before the size relation, while the
/// candidate definition is only reached when the relation is 0 or 1.
pub fn evaluate(input:Input<'_>)->Result<ResultTrace,Error>{
 let mut trace=ResultTrace{accepted:false,player_resolved:false,cache_read:false,progress_reads:0,relation:None,definition_read:false};
 if input.player_handle==0||input.candidate_id==input.player_handle{return Ok(trace)}
 let player=input.player_size.ok_or(Error::MissingPlayer)?;trace.player_resolved=true;
 let cache=input.player_cache.ok_or(Error::MissingCache)?;trace.cache_read=true;
 let mut read_size=|word:u32|->Result<i32,Error>{
  if word!=u32::MAX{return Ok(word as i32)}
  trace.progress_reads+=1;let progress=input.progress.ok_or(Error::MissingProgress)?;
  progress_scale_index(progress).map(|x|x as i32).map_err(|_|Error::UnsupportedProgress)
 };
 let a=read_size(player)?;let b=read_size(input.candidate_size)?;
 let relation=b.wrapping_sub(a).wrapping_add(2).clamp(0,4);trace.relation=Some(relation);
 if relation>1{return Ok(trace)}
 let definition=input.candidate_definition.ok_or(Error::MissingDefinition)?;trace.definition_read=true;
 let kind=definition.cell_type();
 trace.accepted=(cache.flag_1251!=0&&kind==3)||(cache.flag_1250!=0&&kind==2);
 Ok(trace)
}

#[cfg(test)] mod tests{
 use super::*;use crate::cell_definition::{self,DEFINITION_SIZE};
 fn input<'a>(definition:Option<CellDefinition<'a>>)->Input<'a>{Input{player_handle:1,candidate_id:2,player_size:Some(5),candidate_size:4,player_cache:Some(CacheBytes{flag_1250:1,flag_1251:0}),candidate_definition:definition,progress:None}}
 #[test] fn bypass_and_reached_dependencies(){
  let mut i=input(None);i.player_handle=0;i.player_size=None;i.player_cache=None;assert!(!evaluate(i).unwrap().player_resolved);
  i.player_handle=2;assert!(!evaluate(i).unwrap().cache_read);
  i.player_handle=1;assert_eq!(evaluate(i),Err(Error::MissingPlayer));
  i.player_size=Some(5);assert_eq!(evaluate(i),Err(Error::MissingCache));
  i.player_cache=Some(CacheBytes{flag_1250:0,flag_1251:0});i.candidate_size=5;
  assert_eq!(evaluate(i).unwrap().relation,Some(2));
  i.candidate_size=4;assert_eq!(evaluate(i),Err(Error::MissingDefinition));
 }
 #[test] fn source_type_flags_and_raw_nonzero(){
  let mut bytes=[0u8;DEFINITION_SIZE];bytes[0xb4..0xb8].copy_from_slice(&2u32.to_le_bytes());let d=cell_definition::parse(&bytes).unwrap();
  let mut i=input(Some(d));i.player_cache=Some(CacheBytes{flag_1250:0x80,flag_1251:0});assert!(evaluate(i).unwrap().accepted);
  i.player_cache=Some(CacheBytes{flag_1250:0,flag_1251:1});assert!(!evaluate(i).unwrap().accepted);
 }
 #[test] fn sentinel_and_wrapping_relation(){
  let mut i=input(None);i.player_size=Some(u32::MAX);i.candidate_size=u32::MAX;i.progress=Some(1000);
  let t=evaluate(i).unwrap();assert_eq!(t.progress_reads,2);assert_eq!(t.relation,Some(2));
  i.progress=Some(1001);assert_eq!(evaluate(i),Err(Error::UnsupportedProgress));
  i.player_size=Some(0x80000000);i.candidate_size=0x7fffffff;i.progress=None;
  assert_eq!(evaluate(i),Err(Error::MissingDefinition)); // wrapping (-1)+2 = 1
 }
}
