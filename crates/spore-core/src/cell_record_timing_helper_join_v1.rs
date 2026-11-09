//! Explicit join of the bounded E7E6F0 timing and E59A90 helper projections.
use crate::cell_record_timing_prefix_v1::{self,Error as TimingError};
use crate::cell_record_update_helper_v1::{self,Actor,Error as HelperError,Route};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Timing(TimingError), Helper(HelperError) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Outcome {
 AlternatePending { remaining_before_helper:u32 },
 Returned { remaining_before_helper:u32, flag_after_helper:u8, callback_called:bool, pending_event_path:bool, pending_other_path:bool },
}
/// Caller supplies active FP context and post-lookup helper snapshots at their
/// original reads. The helper may mutate the record after the exposed subtraction.
/// AlternatePending stops before E50480; neither downstream path runs or implies
/// live scheduler delivery, actor identity, or callback behavior.
pub fn join(old:u32,delta:u32,mxcsr:u32,actor:Option<Actor>,flag:u8,callback_present:bool,aux_24:Option<u32>,callback_al:Option<u8>,admitted:bool)->Result<Outcome,Error>{
 let timing=cell_record_timing_prefix_v1::step(old,delta,0,mxcsr,admitted).map_err(Error::Timing)?;
 match cell_record_update_helper_v1::helper(actor,flag,callback_present,aux_24,callback_al,admitted).map_err(Error::Helper)? {
  Route::AlternatePending=>Ok(Outcome::AlternatePending{remaining_before_helper:timing.remaining_before_helper}),
  Route::Returned{al,flag_after,callback_called}=>Ok(Outcome::Returned{remaining_before_helper:timing.remaining_before_helper,flag_after_helper:flag_after,callback_called,pending_event_path:al==0,pending_other_path:al!=0}),
 }
}
#[cfg(test)]mod tests{use super::*;#[test]fn source_order_and_pending_alternate(){let actor=Some(Actor{field_248:1,field_fc:1});assert!(matches!(join(0x3f800000,0x3e800000,0x1f80,actor,0,true,Some(1),None,true),Ok(Outcome::AlternatePending{remaining_before_helper:0x3f400000})));assert!(matches!(join(0x3f800000,0x3e800000,0x1f80,actor,0,true,Some(0),Some(0),true),Ok(Outcome::Returned{flag_after_helper:1,pending_event_path:true,..})));}}
