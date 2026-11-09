//! Conditional transition-button event selection and player gate composition.
use crate::cell_transition_player_gate_v1::{self, Outcome as PlayerOutcome, PlayerFlags};
pub const EVENT_TYPE: u32 = 0x287259f6;
pub const BUTTON_ID: u32 = 0x06455c5e;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerSnapshot { pub root_5158_after_helper: u32, pub flags_after_lookup: Option<PlayerFlags> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Unhandled, TransitionButton(PlayerOutcome) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Unadmitted, UnsupportedSameTypeId, MissingPlayerSnapshot }
/// Caller admits the event pointer, listener delivery and lifetimes. Other IDs of
/// this event type may invoke other handlers and are intentionally unsupported.
/// The matching action is composed only with caller-supplied post-helper and
/// post-lookup snapshots; the pending service call is not a stage transition.
pub fn dispatch(event_type:u32, id:u32, player:Option<PlayerSnapshot>, admitted:bool)->Result<Outcome,Error> {
 if !admitted { return Err(Error::Unadmitted); }
 if event_type!=EVENT_TYPE { return Ok(Outcome::Unhandled); }
 if id!=BUTTON_ID {
  if id==BUTTON_ID-1 || id==BUTTON_ID+1 { return Ok(Outcome::Unhandled); }
  return Err(Error::UnsupportedSameTypeId);
 }
 let p=player.ok_or(Error::MissingPlayerSnapshot)?;
 Ok(Outcome::TransitionButton(cell_transition_player_gate_v1::gate(p.root_5158_after_helper,p.flags_after_lookup,true).expect("admitted")))
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn admits_only_bounded_events_and_snapshots() {
  assert_eq!(dispatch(0,BUTTON_ID,None,true),Ok(Outcome::Unhandled));
  assert_eq!(dispatch(EVENT_TYPE,BUTTON_ID-1,None,true),Ok(Outcome::Unhandled));
  assert_eq!(dispatch(EVENT_TYPE,BUTTON_ID+1,None,true),Ok(Outcome::Unhandled));
  assert_eq!(dispatch(EVENT_TYPE,0,None,true),Err(Error::UnsupportedSameTypeId));
  assert_eq!(dispatch(EVENT_TYPE,BUTTON_ID,None,true),Err(Error::MissingPlayerSnapshot));
  assert_eq!(dispatch(EVENT_TYPE,BUTTON_ID,None,false),Err(Error::Unadmitted));
  let p=PlayerSnapshot{root_5158_after_helper:9,flags_after_lookup:Some(PlayerFlags{byte_112:0,byte_113:0})};
  assert!(matches!(dispatch(EVENT_TYPE,BUTTON_ID,Some(p),true),Ok(Outcome::TransitionButton(x)) if x.pending_after_call && x.root_5158==5));
 }
}
