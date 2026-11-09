//! Checked projection of B71CC0 and the first E780D0 write.
//!
//! Slot words are caller-supplied readable, non-null, stable and disjoint
//! snapshots, not native pointers or pool ownership. Equality with the source
//! pool +10 bound is explicitly admitted because native uses signed JG rather
//! than the checked pool's stricter bound. The fresh B71D00 lookup and all
//! teardown remain pending.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { UnstableOrAliased, NegativeBound, EqualBoundNotAdmitted, UnreadableSlot }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary { InvalidActorReturn, FreshLookupPending { actor_word:u32 } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection { pub player_word:u32, pub boundary:Boundary }

/// `slot_words[index]` is the first word at the source-computed slot address;
/// address calculation and a non-null readable slot are caller admissions.
/// No input memory is changed; the returned player word is the modeled write.
pub fn project(actor_word:u32, player_word:u32, signed_bound:i32, slot_words:&[u32],
               stable_disjoint:bool, equal_bound_readable:bool)->Result<Projection,Error> {
    if !stable_disjoint {return Err(Error::UnstableOrAliased)}
    if signed_bound<0 {return Err(Error::NegativeBound)}
    let invalid=||Projection{player_word,boundary:Boundary::InvalidActorReturn};
    if actor_word==0 {return Ok(invalid())}
    let index=(actor_word & 0xffff) as usize;
    let bound=signed_bound as usize;
    if index>bound {return Ok(invalid())}
    if index==bound && !equal_bound_readable {return Err(Error::EqualBoundNotAdmitted)}
    let stored=*slot_words.get(index).ok_or(Error::UnreadableSlot)?;
    if stored!=actor_word {return Ok(invalid())}
    Ok(Projection{player_word:if actor_word==player_word {0}else{player_word},boundary:Boundary::FreshLookupPending{actor_word}})
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn valid_match_clear_nonmatch_retain_and_invalid_return(){
  let slots=[0x10000,0x20001];
  assert_eq!(project(0x10000,0x10000,1,&slots,true,true).unwrap(),Projection{player_word:0,boundary:Boundary::FreshLookupPending{actor_word:0x10000}});
  assert_eq!(project(0x10000,0x20001,1,&slots,true,true).unwrap().player_word,0x20001);
  for id in [0,0x30000,0x30002] {assert_eq!(project(id,0x12345,1,&slots,true,true).unwrap(),Projection{player_word:0x12345,boundary:Boundary::InvalidActorReturn});}
 }
 #[test] fn equality_and_readability_are_checked_before_slot_access(){
  let slots=[0x10000,0x20001];
  assert_eq!(project(0x20001,0x20001,1,&slots,true,false),Err(Error::EqualBoundNotAdmitted));
  assert_eq!(project(0x20001,0x20001,1,&slots[..1],true,true),Err(Error::UnreadableSlot));
  assert_eq!(project(0x20001,0x20001,1,&slots,true,true).unwrap().player_word,0);
  assert_eq!(project(0x10000,0,1,&slots,false,true),Err(Error::UnstableOrAliased));
  assert_eq!(project(0x10000,0,-1,&slots,true,true),Err(Error::NegativeBound));
 }
}
