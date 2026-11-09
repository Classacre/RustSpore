//! Checked E7812A..E7814D handoff to synthetic BBAC30 removal.
//! Actor and manager state are fresh caller admissions; no source actor,
//! registration ownership, lifecycle or later teardown is established.
use crate::cell_sweep_remove_v1::{self as removal,Storage};

#[derive(Debug,PartialEq,Eq)]
pub enum Error { UnstableActor, UnadmittedManager, MissingManager, Removal(removal::Error) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Pending { Actor248Read { removal_called:bool } }

/// The sentinel path does not inspect or require a manager. For other values,
/// the caller admits fresh disjoint manager/slot ownership and coherent free
/// links; the checked removal adds bounds and mutation-free rejection.
pub fn apply(actor_364:u32,actor_stable:bool,manager_owned_stable:bool,manager:Option<&mut Storage>)->Result<Pending,Error>{
 if !actor_stable{return Err(Error::UnstableActor)}
 if actor_364==u32::MAX{return Ok(Pending::Actor248Read{removal_called:false})}
 if !manager_owned_stable{return Err(Error::UnadmittedManager)}
 let manager=manager.ok_or(Error::MissingManager)?;
 removal::remove(manager,actor_364).map_err(Error::Removal)?;
 Ok(Pending::Actor248Read{removal_called:true})
}
#[cfg(test)] mod tests {
 use super::*;
 use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
 fn fixture()->Storage{let e=||Endpoint{opening:0,slot:7,value_bits:42};Storage{slots:vec![Slot{actor:9,tag:8,endpoint_links:[0,0x8000,0x8001,1]},Slot{actor:u32::MAX,tag:u32::MAX,endpoint_links:[6;4]}],old:[vec![e(),e()],vec![e(),e()]],pending:[vec![e(),e()],vec![e(),e()]],live:1,high:2,free_head:1}}
 #[test] fn sentinel_needs_no_manager_and_valid_slot_unlinks(){assert_eq!(apply(u32::MAX,true,false,None),Ok(Pending::Actor248Read{removal_called:false}));let mut s=fixture();assert_eq!(apply(0,true,true,Some(&mut s)),Ok(Pending::Actor248Read{removal_called:true}));assert_eq!((s.live,s.free_head,s.slots[0].tag),(0,0,u32::MAX));}
 #[test] fn rejects_bad_admissions_without_mutation(){let mut s=fixture();let before=s.clone();assert_eq!(apply(0,false,true,Some(&mut s)),Err(Error::UnstableActor));assert_eq!(apply(0,true,false,Some(&mut s)),Err(Error::UnadmittedManager));assert_eq!(apply(0,true,true,None),Err(Error::MissingManager));assert_eq!(apply(9,true,true,Some(&mut s)),Err(Error::Removal(removal::Error::InvalidSlot)));assert_eq!(s,before);}
}
