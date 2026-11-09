//! Bounded 69E499 inner lookup and registry factory output slice.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingFactory,UnsupportedControl}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{InnerLookup,HitPending,Factory{key:u32,type_id:u32},Store(Option<u32>)}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub actions:Vec<Action>,pub slot:Option<u32>}
/// `inner_hit` is the fresh supplied result of the inner lookup. `slot_after_lookup`
/// is observed after that callback; on miss `factory_result` is the fresh return
/// from the scripted factory callback, and source stores it even if that callback
/// changed the slot. A hit stops before the separately modeled hit path. Caller
/// admits valid stable frame/owner/factory lifetimes and callbacks returning; no
/// map semantics, cache insertion, factory effects or live identity are inferred.
pub fn project(inner_hit:bool,slot_after_lookup:Option<u32>,key:u32,factory_result:Option<Option<u32>>,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted||slot_after_lookup==Some(0)||factory_result==Some(Some(0)){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut actions=vec![Action::InnerLookup];
 if inner_hit{actions.push(Action::HitPending);return Ok(Outcome{actions,slot:slot_after_lookup})}
 let result=factory_result.ok_or(Error::MissingFactory)?;actions.push(Action::Factory{key,type_id:0x0179c807});actions.push(Action::Store(result));Ok(Outcome{actions,slot:result})
}
#[cfg(test)]mod tests{use super::*;#[test]fn hit_stops_and_miss_stores_fresh_return(){let h=project(true,Some(7),12,None,0x27f,0x1f80,true).unwrap();assert_eq!(h.actions,vec![Action::InnerLookup,Action::HitPending]);assert_eq!(h.slot,Some(7));let m=project(false,Some(9),12,Some(None),0x37f,0x1f80,true).unwrap();assert_eq!(m.actions,vec![Action::InnerLookup,Action::Factory{key:12,type_id:0x0179c807},Action::Store(None)]);assert_eq!(m.slot,None);assert_eq!(project(false,None,0,None,0x27f,0x1f80,true),Err(Error::MissingFactory));}}
