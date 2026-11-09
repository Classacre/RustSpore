//! Checked E59080 projection with two explicitly admitted pool snapshots.
//! This does not establish live pool ownership, callbacks or actor teardown.
use crate::cell_e780_player_clear_v1::{self as validate,Boundary as PoolBoundary};

#[derive(Clone,Copy,Debug)]
pub struct Pool<'a>{pub signed_bound:i32,pub slot_words:&'a [u32],pub equal_bound_readable:bool}
#[derive(Clone,Copy,Debug)]
pub struct Snapshot<'a>{
 pub record_handle:u32,pub record_pool:Pool<'a>,pub record_slot_word:u32,
 pub record_actor_handle:u32,pub actor_pool:Option<Pool<'a>>,
 pub actor_36c:u32,pub actor_370:u32,pub record_free_head:u32,pub record_live:u32,
 pub stable_disjoint_owned:bool,
}
#[derive(Debug,PartialEq,Eq)]
pub enum Error {UnstableOrUnowned,Pool(validate::Error),MissingActorPool,RecordMismatch,EmptyRecordPool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Projection {pub record_slot_word:u32,pub record_free_head:u32,pub record_live:u32,pub actor_36c:u32,pub actor_370:u32,pub released:bool,pub actor_cleared:bool}

fn valid(id:u32,pool:Pool<'_>)->Result<bool,Error>{
 let p=validate::project(id,0,pool.signed_bound,pool.slot_words,true,pool.equal_bound_readable).map_err(Error::Pool)?;
 Ok(matches!(p.boundary,PoolBoundary::FreshLookupPending{..}))
}
/// The first invalid record returns without needing a second pool. For a valid
/// record, its +0C actor handle and readable actor pool are fresh admissions.
/// Checked identity, positive count and equal-bound rules add safety beyond
/// native B71CC0/B71D50. Inputs are not modified; returned words are projected.
pub fn project(x:Snapshot<'_>)->Result<Projection,Error>{
 if !x.stable_disjoint_owned{return Err(Error::UnstableOrUnowned)}
 let mut out=Projection{record_slot_word:x.record_slot_word,record_free_head:x.record_free_head,record_live:x.record_live,actor_36c:x.actor_36c,actor_370:x.actor_370,released:false,actor_cleared:false};
 if !valid(x.record_handle,x.record_pool)?{return Ok(out)}
 if x.record_slot_word!=x.record_handle{return Err(Error::RecordMismatch)}
 if x.record_live==0{return Err(Error::EmptyRecordPool)}
 let actor_pool=x.actor_pool.ok_or(Error::MissingActorPool)?;
 if valid(x.record_actor_handle,actor_pool)?{out.actor_36c=0;out.actor_370=0;out.actor_cleared=true;}
 out.record_slot_word=x.record_free_head;out.record_free_head=x.record_handle&0xffff;out.record_live-=1;out.released=true;
 Ok(out)
}

#[cfg(test)] mod tests{
 use super::*;
 fn sample<'a>(records:&'a [u32],actors:&'a [u32])->Snapshot<'a>{let pool=|slot_words|Pool{signed_bound:0,slot_words,equal_bound_readable:true};Snapshot{record_handle:0x10000,record_pool:pool(records),record_slot_word:0x10000,record_actor_handle:0x10000,actor_pool:Some(pool(actors)),actor_36c:1,actor_370:2,record_free_head:u32::MAX,record_live:1,stable_disjoint_owned:true}}
 #[test] fn first_invalid_second_invalid_and_valid_have_distinct_effects(){let records=[0x10000];let actors=[0x10000];let mut x=sample(&records,&actors);x.record_handle=0x20000;x.actor_pool=None;let p=project(x).unwrap();assert!(!p.released);assert_eq!((p.actor_36c,p.actor_370),(1,2));x=sample(&records,&actors);x.record_actor_handle=0x30000;let p=project(x).unwrap();assert!(p.released);assert!(!p.actor_cleared);assert_eq!((p.record_slot_word,p.record_free_head,p.record_live),(u32::MAX,0,0));let p=project(sample(&records,&actors)).unwrap();assert!(p.released&&p.actor_cleared);assert_eq!((p.actor_36c,p.actor_370),(0,0));}
 #[test] fn added_admissions_reject_unreadable_or_unstable(){let records=[0x10000];let actors=[0x10000];let mut x=sample(&records,&actors);x.record_pool.equal_bound_readable=false;assert_eq!(project(x),Err(Error::Pool(validate::Error::EqualBoundNotAdmitted)));x=sample(&records,&actors);x.actor_pool=None;assert_eq!(project(x),Err(Error::MissingActorPool));x=sample(&records,&actors);x.record_live=0;assert_eq!(project(x),Err(Error::EmptyRecordPool));x=sample(&records,&actors);x.stable_disjoint_owned=false;assert_eq!(project(x),Err(Error::UnstableOrUnowned));}
}
