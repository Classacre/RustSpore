//! Candidate full fresh aid construction and bounded pending-event reset.
//! Private clock fields derive from9A33C0/9A2FA0, not supplied counter snapshots.
//! Source global serial remains explicit. No recycled references, resource
//! binding, clock advance, event dispatch, live actor or scheduling admission.
use std::fmt;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum SourceGlobalSerial{Unestablished,ExplicitSnapshot(u32)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum EventResetBoundary{Unestablished,FreshOnly,BoundedPending{count:u32,capacity:u32}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum FreshAidError{UnestablishedGlobalSerial,UnestablishedEventBoundary,EventCapacityOutOfRange,EventCountExceedsCapacity}
impl fmt::Display for FreshAidError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source fresh aid clock fields: {self:?}")}}
impl std::error::Error for FreshAidError{}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct FreshAidClockState{serial:u32,next_serial:u32,event_capacity:u32}
impl FreshAidClockState{
 pub fn cycle_epoch(self)->u32{1}
 pub fn crossing_counter(self)->u32{0}
 pub fn active(self)->bool{false}
 pub fn constructor_loop_byte(self)->u8{0}
 pub fn stored_time_bits(self)->u64{0}
 pub fn duration_bits(self)->u64{0}
 pub fn source_serial(self)->u32{self.serial}
 pub fn next_global_serial(self)->u32{self.next_serial}
 pub fn pending_events(self)->u32{0}
 pub fn event_capacity(self)->u32{self.event_capacity}
 /// Defined scalar clock/serial/vector-size projection; no pointer bytes.
 pub fn to_le_bytes(self)->[u8;56]{let mut b=[0;56];b[..4].copy_from_slice(&u32::MAX.to_le_bytes());b[4..8].copy_from_slice(&self.serial.to_le_bytes());b[36..40].copy_from_slice(&self.cycle_epoch().to_le_bytes());b[48..52].copy_from_slice(&self.event_capacity.to_le_bytes());b[52..56].copy_from_slice(&self.next_serial.to_le_bytes());b}
}
/// Full fresh constructor preclears resource/owner/binding/wrapper pointers;
/// release, wrapper-copy and allocator branches are source-unreachable. A bounded
/// eight-byte-record vector may then be cleared through full9A1690/7D0230:
/// oldend is the erased suffix end, so no record copy or destructor executes.
pub fn initialize_fresh_aid(serial:SourceGlobalSerial,events:EventResetBoundary)->Result<FreshAidClockState,FreshAidError>{
 let serial=match serial{SourceGlobalSerial::Unestablished=>return Err(FreshAidError::UnestablishedGlobalSerial),SourceGlobalSerial::ExplicitSnapshot(v)=>v};
 let capacity=match events{EventResetBoundary::Unestablished=>return Err(FreshAidError::UnestablishedEventBoundary),EventResetBoundary::FreshOnly=>0,EventResetBoundary::BoundedPending{count,capacity}=>{if capacity>16{return Err(FreshAidError::EventCapacityOutOfRange);}if count>capacity{return Err(FreshAidError::EventCountExceedsCapacity);}capacity}};
 Ok(FreshAidClockState{serial,next_serial:serial.wrapping_add(1),event_capacity:capacity})
}
