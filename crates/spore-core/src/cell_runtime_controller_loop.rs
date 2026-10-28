//! Candidate fullA00310 +9FFE30 setter, with two disjoint reached slots/aids.
//! Explicit controller snapshots remain separate from live actor/resource routing,
//! controller construction, fresh counters and event scheduling.
use std::fmt;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ControllerSlot{pub epoch:u32,pub linked_handle:u32,pub has_aid:bool,pub loop_request:u8}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ControllerLoopSnapshot{pub primary_index:u8,pub secondary_index:u8,pub primary:ControllerSlot,pub secondary:ControllerSlot,pub aid_loop_bytes:[u8;2]}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum PropertyLoopMode{Repeat,Other}
impl PropertyLoopMode{/// A0C120 property+3C equal1 chooses1; allother values choose0.
 pub fn from_source_word(raw:u32)->Self{if raw==1{Self::Repeat}else{Self::Other}}
 fn byte(self)->u8{u8::from(self==Self::Repeat)}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum ControllerLoopError{PrimaryIndexOutOfRange,SecondaryIndexOutOfRange,AliasedSlots,PrimaryHandleIndexMismatch,LinkedHandleIndexMismatch}
impl fmt::Display for ControllerLoopError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"bounded source controller loop setter: {self:?}")}}
impl std::error::Error for ControllerLoopError{}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ControllerLoopResult{pub snapshot:ControllerLoopSnapshot,pub source_success:bool,pub resolved_primary:bool,pub resolved_secondary:bool}
impl ControllerLoopResult{pub fn to_le_bytes(self)->[u8;28]{let mut b=[0;28];for(i,v)in[u32::from(self.source_success),u32::from(self.snapshot.primary.loop_request),u32::from(self.snapshot.secondary.loop_request),u32::from(self.snapshot.aid_loop_bytes[0]),u32::from(self.snapshot.aid_loop_bytes[1]),u32::from(self.resolved_primary),u32::from(self.resolved_secondary)].iter().enumerate(){b[4*i..4*i+4].copy_from_slice(&v.to_le_bytes());}b}}
/// Full resolver has noindexbounds. This bounded protocol provides exactly two
/// distinct slot objects and two disjoint aid objects. Only accessed handles are
/// checked; an invalid linked handle after primary epoch mismatch remains unread.
/// A linked fallback aid is secondary and is never mutated by this setter.
pub fn apply_property_loop(mut snapshot:ControllerLoopSnapshot,handle:u32,mode:PropertyLoopMode)->Result<ControllerLoopResult,ControllerLoopError>{
 if snapshot.primary_index>=16{return Err(ControllerLoopError::PrimaryIndexOutOfRange);}
 if snapshot.secondary_index>=16{return Err(ControllerLoopError::SecondaryIndexOutOfRange);}
 if snapshot.primary_index==snapshot.secondary_index{return Err(ControllerLoopError::AliasedSlots);}
 let(mut primary,mut secondary)=(false,false);
 if handle!=0{
  if handle&255!=u32::from(snapshot.primary_index)+1{return Err(ControllerLoopError::PrimaryHandleIndexMismatch);}
  if snapshot.primary.epoch==handle>>8{
   primary=true;let link=snapshot.primary.linked_handle;
   if link!=0{if link&255!=u32::from(snapshot.secondary_index)+1{return Err(ControllerLoopError::LinkedHandleIndexMismatch);}secondary=snapshot.secondary.epoch==link>>8;}
  }
 }
 if primary{snapshot.primary.loop_request=mode.byte();if snapshot.primary.has_aid{snapshot.aid_loop_bytes[0]=mode.byte();}}
 Ok(ControllerLoopResult{snapshot,source_success:primary,resolved_primary:primary,resolved_secondary:secondary})
}
