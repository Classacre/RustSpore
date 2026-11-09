//! Bounded 693267 restore-wrapper handoff after an admitted candidate lookup.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum RefId{Input,Candidate,OldInput,OldCandidate}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{Retain(RefId),Release(RefId),Reject{input:RefId,candidate:RefId}}
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,UnsupportedControl}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub input_slot:Option<RefId>,pub candidate_slot:Option<RefId>,pub actions:Vec<Action>}
/// Candidate and Input denote distinct, stable non-null objects. Validation and
/// the post-lookup slots are caller supplied. Callbacks return normally without
/// reentrancy or mutation; the local SEH marker is synthetic and real SEH,
/// object lifetime, lookup, serialization and root adoption are not modeled.
pub fn project(old_input:Option<RefId>,old_candidate:Option<RefId>,valid:bool,cw:u16,mx:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(cw,0x027f|0x037f)||mx!=0x1f80{return Err(Error::UnsupportedControl)}
 let(mut input_slot,mut candidate_slot)=(old_input,old_candidate);let mut actions=Vec::new();
 if valid {
  if input_slot!=Some(RefId::Input){actions.push(Action::Retain(RefId::Input));input_slot=Some(RefId::Input);if let Some(x)=old_input{actions.push(Action::Release(x));}}
  if candidate_slot!=Some(RefId::Candidate){actions.push(Action::Retain(RefId::Candidate));candidate_slot=Some(RefId::Candidate);if let Some(x)=old_candidate{actions.push(Action::Release(x));}}
 } else {actions.push(Action::Reject{input:RefId::Input,candidate:RefId::Candidate});}
 actions.push(Action::Release(RefId::Candidate));Ok(Outcome{input_slot,candidate_slot,actions})
}
#[cfg(test)]mod tests{use super::*;#[test]fn preserves_set_and_cleanup_order(){let x=project(Some(RefId::OldInput),Some(RefId::OldCandidate),true,0x27f,0x1f80,true).unwrap();assert_eq!(x.actions,vec![Action::Retain(RefId::Input),Action::Release(RefId::OldInput),Action::Retain(RefId::Candidate),Action::Release(RefId::OldCandidate),Action::Release(RefId::Candidate)]);let y=project(None,None,false,0x37f,0x1f80,true).unwrap();assert_eq!(y.actions,vec![Action::Reject{input:RefId::Input,candidate:RefId::Candidate},Action::Release(RefId::Candidate)]);}}
