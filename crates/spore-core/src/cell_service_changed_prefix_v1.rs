//! Controlled 7E3934 changed-path prefix; callbacks and map are scripted.
use crate::cell_service_action_gate_v1::Branch;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum MapResult{Missing,Found(u32)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Projection{pub old_for_first:u32,pub old_for_pair:u32,pub requested:u32,pub service_8c_before_pair:u32,pub pending_found_callback:Option<u32>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unstable,WrongBranch,InconsistentEntry}
/// The first scripted callback's post-return +88 value is explicitly supplied,
/// because original source re-reads it before the pair callback. The map result
/// is a caller admission; no actual service callbacks or map lookup occur.
/// Stable/disjoint service, iterator and node lifetimes and normal scripted
/// returns are required. The before-pair +8C write is not a post-callback snapshot.
/// Requiring old_before != requested is an extra fixture restriction: an earlier
/// branch-to-entry callback could have changed +88. Stops before later effects.
pub fn project(branch:Branch,old_before:u32,requested:u32,old_after_first:u32,map:MapResult,stable:bool)->Result<Projection,Error>{
 if !stable{return Err(Error::Unstable)}if branch!=Branch::Changed{return Err(Error::WrongBranch)}if old_before==requested{return Err(Error::InconsistentEntry)}
 Ok(Projection{old_for_first:old_before,old_for_pair:old_after_first,requested,service_8c_before_pair:u32::MAX,pending_found_callback:match map{MapResult::Missing=>None,MapResult::Found(index)=>Some(index)}})
}
#[cfg(test)]mod tests{use super::*;#[test]fn fresh_old_and_map_branch(){let p=project(Branch::Changed,1,2,3,MapResult::Found(9),true).unwrap();assert_eq!((p.old_for_first,p.old_for_pair,p.requested,p.service_8c_before_pair,p.pending_found_callback),(1,3,2,u32::MAX,Some(9)));assert_eq!(project(Branch::Changed,1,2,1,MapResult::Missing,true).unwrap().pending_found_callback,None);assert_eq!(project(Branch::Equal,1,2,1,MapResult::Missing,true),Err(Error::WrongBranch));assert_eq!(project(Branch::Changed,1,1,1,MapResult::Missing,true),Err(Error::InconsistentEntry));assert_eq!(project(Branch::Changed,1,2,1,MapResult::Missing,false),Err(Error::Unstable));}}

