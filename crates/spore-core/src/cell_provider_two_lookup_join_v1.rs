//! Controlled 69E46F two-lookup, hit callback and miss factory composition.
use crate::{cell_provider_cache_hit_v1 as hit,cell_provider_registry_factory_v1 as registry};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unadmitted,UnsupportedControl,Registry(registry::Error),Hit(hit::Error)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Inner{Hit{candidate:u32,post_callback_slot:Option<u32>},Miss{factory_result:Option<u32>}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Outer{Miss{post_lookup_slot:Option<u32>},Found{post_inner_lookup_slot:Option<u32>,inner:Inner}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Route{OuterMiss,InnerHit,Factory}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub route:Route,pub slot:Option<u32>,pub lookup_calls:u8,pub registry_actions:Vec<registry::Action>,pub hit_actions:Option<[hit::Action;2]>}
/// Source clears the slot before the first lookup. The supplied snapshots are
/// fresh after each scripted lookup/callback; neither lookup is assumed inert.
/// On outer miss we stop at the source failure branch. A hit calls the candidate
/// and takes a fresh post-callback slot. A miss stores the fresh factory return.
/// Stable disjoint objects/frames and callbacks returning are admitted, not
/// evidence about actual maps, refcounts, factory, insertion or live identity.
pub fn project(outer:Outer,key:u32,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 match outer{
  Outer::Miss{post_lookup_slot}=>{if post_lookup_slot==Some(0){return Err(Error::Unadmitted)}Ok(Outcome{route:Route::OuterMiss,slot:post_lookup_slot,lookup_calls:1,registry_actions:vec![],hit_actions:None})},
  Outer::Found{post_inner_lookup_slot,inner}=>{
   let (is_hit,factory)=match inner{Inner::Hit{..}=>(true,None),Inner::Miss{factory_result}=>(false,Some(factory_result))};
   let r=registry::project(is_hit,post_inner_lookup_slot,key,factory,x87,mxcsr,true).map_err(Error::Registry)?;
   match inner{Inner::Miss{..}=>Ok(Outcome{route:Route::Factory,slot:r.slot,lookup_calls:2,registry_actions:r.actions,hit_actions:None}),Inner::Hit{candidate,post_callback_slot}=>{let h=hit::project(candidate,post_callback_slot,x87,mxcsr,true).map_err(Error::Hit)?;Ok(Outcome{route:Route::InnerHit,slot:h.post_callback_slot,lookup_calls:2,registry_actions:r.actions,hit_actions:Some(h.actions)})}}
  }
 }
}
#[cfg(test)]mod tests{use super::*;#[test]fn fresh_results_and_branch_boundaries(){let h=project(Outer::Found{post_inner_lookup_slot:Some(2),inner:Inner::Hit{candidate:3,post_callback_slot:Some(4)}},5,0x27f,0x1f80,true).unwrap();assert_eq!((h.route,h.slot),(Route::InnerHit,Some(4)));let m=project(Outer::Found{post_inner_lookup_slot:Some(2),inner:Inner::Miss{factory_result:None}},5,0x37f,0x1f80,true).unwrap();assert_eq!((m.route,m.slot),(Route::Factory,None));assert_eq!(project(Outer::Miss{post_lookup_slot:Some(0)},0,0x27f,0x1f80,true),Err(Error::Unadmitted));}}
