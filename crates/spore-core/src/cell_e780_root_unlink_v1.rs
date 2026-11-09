//! Checked E780D0 fresh-lookup to E869F0 strict-root projection.
//! Tokens represent explicitly supplied synthetic identities, not addresses or
//! proof of ownership. Other branches, manager lifecycle and actor teardown
//! remain outside this model.

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Snapshot {
 pub actor_word:u32, pub fresh_slot_word:u32, pub actor_token:u64,
 pub container_token:u64, pub manager_token:u64, pub node_token:u64,
 pub container_root:u64, pub manager_count:u32, pub old_free_head:u64,
 pub stable_disjoint_owned:bool,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { UnstableOrUnowned, InvalidIdentity, RootMismatch, EmptyManager }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Projection {
 pub container_root:u64, pub manager_count:u32, pub node_next:u64,
 pub manager_free_head:u64, pub pending_actor_364_read:u64,
}
/// The caller admits a freshly readable slot and same-generation actor,
/// nonaliasing stable manager/container/node ownership and the old head word.
/// Positive count and matching slot identity are added safety checks; source
/// B71D00 itself performs no validity check and native count can wrap.
pub fn project(x:Snapshot)->Result<Projection,Error>{
 if !x.stable_disjoint_owned{return Err(Error::UnstableOrUnowned)}
 if x.actor_word==0||x.actor_word!=x.fresh_slot_word||x.actor_token==0||x.container_token==0||x.manager_token==0||x.node_token==0{return Err(Error::InvalidIdentity)}
 if x.container_root!=x.node_token{return Err(Error::RootMismatch)}
 if x.manager_count==0{return Err(Error::EmptyManager)}
 Ok(Projection{container_root:0,manager_count:x.manager_count-1,node_next:x.old_free_head,manager_free_head:x.node_token,pending_actor_364_read:x.actor_token})
}

#[cfg(test)] mod tests {
 use super::*;
 fn sample()->Snapshot{Snapshot{actor_word:0x10000,fresh_slot_word:0x10000,actor_token:11,container_token:22,manager_token:33,node_token:44,container_root:44,manager_count:7,old_free_head:55,stable_disjoint_owned:true}}
 #[test] fn root_unlink_projects_ordered_writes_and_pending_actor(){assert_eq!(project(sample()).unwrap(),Projection{container_root:0,manager_count:6,node_next:55,manager_free_head:44,pending_actor_364_read:11});let mut x=sample();x.old_free_head=0;x.manager_count=1;let out=project(x).unwrap();assert_eq!((out.manager_count,out.node_next),(0,0));}
 #[test] fn rejects_stale_unowned_nonroot_and_empty(){let mut x=sample();x.fresh_slot_word=1;assert_eq!(project(x),Err(Error::InvalidIdentity));x=sample();x.stable_disjoint_owned=false;assert_eq!(project(x),Err(Error::UnstableOrUnowned));x=sample();x.container_root=99;assert_eq!(project(x),Err(Error::RootMismatch));x=sample();x.manager_count=0;assert_eq!(project(x),Err(Error::EmptyManager));}
}
