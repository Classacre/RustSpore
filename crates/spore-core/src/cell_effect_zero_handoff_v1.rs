//! Checked E5E100 zero-weight callback-order projection.
//! Tokens and callback outcomes are caller-supplied fixture admissions. No
//! callback, destruction, allocator, live pool or post-callback state is run.

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum FirstEffect { Unchanged, Cleared, Replaced(u64) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { UnstableOrAliased, InvalidToken, UnexpectedFirstEffect, MissingFirstEffect }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Projection {
 /// The first virtual call, with literal argument 1, was admitted by caller.
 pub first_object:Option<u64>,
 /// When present, the slot was cleared before this pending second virtual call.
 pub pending_second_object:Option<u64>,
}
/// `effect_after_first` describes a caller-scripted outcome, not an executed
/// virtual call. The second callback can itself mutate the slot; no state after
/// that pending call is claimed. Stable, disjoint object/vtable lifetimes are
/// caller admissions and nonzero tokens are fixture identities, not pointers.
pub fn project(initial:Option<u64>,effect_after_first:Option<FirstEffect>,stable_disjoint:bool)->Result<Projection,Error>{
 if !stable_disjoint{return Err(Error::UnstableOrAliased)}
 match initial {
  None=>{if effect_after_first.is_some(){return Err(Error::UnexpectedFirstEffect)}Ok(Projection{first_object:None,pending_second_object:None})},
  Some(0)=>Err(Error::InvalidToken),
  Some(first)=>{
   let second=match effect_after_first.ok_or(Error::MissingFirstEffect)?{FirstEffect::Unchanged=>Some(first),FirstEffect::Cleared=>None,FirstEffect::Replaced(0)=>return Err(Error::InvalidToken),FirstEffect::Replaced(other)=>Some(other)};
   Ok(Projection{first_object:Some(first),pending_second_object:second})
  }
 }
}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn reread_distinguishes_retained_null_and_replaced(){assert_eq!(project(None,None,true).unwrap(),Projection{first_object:None,pending_second_object:None});for (effect,second) in [(FirstEffect::Unchanged,Some(1)),(FirstEffect::Cleared,None),(FirstEffect::Replaced(2),Some(2))]{assert_eq!(project(Some(1),Some(effect),true).unwrap(),Projection{first_object:Some(1),pending_second_object:second});}}
 #[test] fn admissions_reject_inconsistent_scripts(){assert_eq!(project(None,Some(FirstEffect::Cleared),true),Err(Error::UnexpectedFirstEffect));assert_eq!(project(Some(1),None,true),Err(Error::MissingFirstEffect));assert_eq!(project(Some(0),None,true),Err(Error::InvalidToken));assert_eq!(project(Some(1),Some(FirstEffect::Replaced(0)),true),Err(Error::InvalidToken));assert_eq!(project(None,None,false),Err(Error::UnstableOrAliased));}
}
