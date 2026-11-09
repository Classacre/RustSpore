//! Checked E6E920 prefix with explicit synthetic lookup and lifetime admissions.
//! Stops before effect or eligible-player continuations and executes no callback.
use crate::cell_ending_gate_v1::Action;
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Actor { pub token:u64, pub byte_112:u8, pub byte_113:u8, pub byte_188:u8 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Pending { Effect, EligiblePlayer(u64), None }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Projection { pub lookup_player:bool, pub pending:Pending }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unstable, NoPendingEndingCall, MissingLookup, UnusedLookup, InvalidToken }
/// Outer `Option` distinguishes an unprovided lookup from an admitted null
/// result. Stable/disjoint reference, globals, pool slot and actor lifetimes are
/// caller admissions; the token is a synthetic identity, not a native pointer.
/// Helpers are admitted to return normally and their side effects are outside
/// this projection. Source tests bytes as zero/nonzero, not as booleans.
pub fn project(action:Action,global_e0:u32,flag:u8,lookup:Option<Option<Actor>>,stable_disjoint:bool)->Result<Projection,Error>{
 if !stable_disjoint{return Err(Error::Unstable)}
 if action!=Action::PendingEndingCall{return Err(Error::NoPendingEndingCall)}
 if global_e0==0 && flag==0 {if lookup.is_some(){return Err(Error::UnusedLookup)}return Ok(Projection{lookup_player:false,pending:Pending::Effect})}
 let actor=lookup.ok_or(Error::MissingLookup)?;
 let pending=match actor{None=>Pending::None,Some(a)=>{if a.token==0{return Err(Error::InvalidToken)}if a.byte_112!=0||a.byte_113!=0||a.byte_188!=0{Pending::None}else{Pending::EligiblePlayer(a.token)}}};
 Ok(Projection{lookup_player:true,pending})
}
#[cfg(test)] mod tests{use super::*;
 fn actor()->Actor{Actor{token:7,byte_112:0,byte_113:0,byte_188:0}}
 #[test]fn gates_effect_and_each_actor_byte(){let action=Action::PendingEndingCall;assert_eq!(project(action,0,0,None,true).unwrap().pending,Pending::Effect);assert_eq!(project(action,0,1,Some(None),true).unwrap().pending,Pending::None);assert_eq!(project(action,1,0,Some(Some(actor())),true).unwrap().pending,Pending::EligiblePlayer(7));for a in [Actor{byte_112:255,..actor()},Actor{byte_113:2,..actor()},Actor{byte_188:1,..actor()}]{assert_eq!(project(action,1,0,Some(Some(a)),true).unwrap().pending,Pending::None);}}
 #[test]fn rejects_unadmitted_or_inconsistent_snapshots(){let action=Action::PendingEndingCall;assert_eq!(project(action,1,0,None,true),Err(Error::MissingLookup));assert_eq!(project(action,0,0,Some(None),true),Err(Error::UnusedLookup));assert_eq!(project(action,1,0,Some(Some(Actor{token:0,..actor()})),true),Err(Error::InvalidToken));assert_eq!(project(action,1,0,Some(None),false),Err(Error::Unstable));assert_eq!(project(Action::NoEndingCall,1,0,Some(None),true),Err(Error::NoPendingEndingCall));}
}
