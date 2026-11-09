//! Bounded 627DE0 effect request routing with supplied callback snapshots.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Callback { pub al: u8, pub slot_after: u32 }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Call { pub id: u32, pub slot_before: u32, pub slot_after: u32 }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Outcome { pub primary: Call, pub fallback: Option<Call>, pub slot_after: u32 }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Error { Unadmitted, MissingFallback }
/// A valid non-null world, stable disjoint slot and returning scripted callbacks
/// are admitted. Callback results are supplied snapshots; no provider, allocation,
/// real asset identity, lifetime or following placement is modeled.
pub fn project(id:u32, slot_before:u32, primary:Callback, fallback:Option<Callback>, admitted:bool)->Result<Outcome,Error>{
 if !admitted {return Err(Error::Unadmitted)}
 let first=Call{id,slot_before,slot_after:primary.slot_after};
 if primary.al==1 {return Ok(Outcome{primary:first,fallback:None,slot_after:primary.slot_after})}
 let second=fallback.ok_or(Error::MissingFallback)?;
 Ok(Outcome{primary:first,fallback:Some(Call{id:0xe18d6423,slot_before:primary.slot_after,slot_after:second.slot_after}),slot_after:second.slot_after})
}
#[cfg(test)] mod tests {use super::*;#[test]fn only_one_suppresses_fallback(){let p=Callback{al:2,slot_after:7};let f=Callback{al:1,slot_after:9};let x=project(3,4,p,Some(f),true).unwrap();assert_eq!(x.fallback.unwrap().slot_before,7);assert_eq!(x.slot_after,9);assert_eq!(project(3,4,p,None,true),Err(Error::MissingFallback));assert!(project(3,4,Callback{al:1,slot_after:0},None,true).unwrap().fallback.is_none());}}
