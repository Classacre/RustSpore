//! E7D043 pending ending-call gate only; caller supplies fresh snapshots.
//! Prior progress-accounting, growth calls, resource effects and E6E920 are excluded.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action { NoEndingCall, PendingEndingCall }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Outcome { pub action:Action, pub resolve_globals:bool, pub fresh_second_read:bool }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, MissingFirst, MissingSecond }
/// The caller admits reaching the native tail, successful reference construction,
/// resolution and cleanup when reached, and stable lifetime of each supplied
/// snapshot; the source re-resolves the globals for its conditional second read.
/// A pending call is not an ending, event dispatch or stage transition.
pub fn gate(progress:i32,first_dc:Option<u32>,second_dc:Option<u32>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if progress!=1000{return Ok(Outcome{action:Action::NoEndingCall,resolve_globals:false,fresh_second_read:false})}
 let first=first_dc.ok_or(Error::MissingFirst)?;
 if first==1{return Ok(Outcome{action:Action::PendingEndingCall,resolve_globals:true,fresh_second_read:false})}
 let second=second_dc.ok_or(Error::MissingSecond)?;
 Ok(Outcome{action:if second==2{Action::PendingEndingCall}else{Action::NoEndingCall},resolve_globals:true,fresh_second_read:true})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn pending_gate_and_reloads(){assert_eq!(gate(999,None,None,true).unwrap().action,Action::NoEndingCall);assert_eq!(gate(1000,Some(1),None,true).unwrap().action,Action::PendingEndingCall);assert_eq!(gate(1000,Some(2),Some(1),true).unwrap().action,Action::NoEndingCall);assert_eq!(gate(1000,Some(0),Some(2),true).unwrap().action,Action::PendingEndingCall);assert_eq!(gate(1000,Some(2),None,true),Err(Error::MissingSecond));assert_eq!(gate(1000,None,None,true),Err(Error::MissingFirst));assert_eq!(gate(1000,None,None,false),Err(Error::Unadmitted));}
}
