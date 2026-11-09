//! Bounded 69E499 registry miss joined to 91FBE0 factory validation.
use crate::{cell_provider_registry_factory_v1 as registry,cell_provider_factory_validate_v1 as factory};
#[derive(Clone,Debug,PartialEq,Eq)]pub enum Error{Registry(registry::Error),Factory(factory::Error)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Route{Hit,Miss{candidate:Option<u32>,validator_result:Option<Option<u32>>}}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub slot:Option<u32>,pub registry_actions:Vec<registry::Action>,pub factory_actions:Vec<factory::Action>}
/// The lookup result and post-lookup slot are fresh supplied snapshots. A hit
/// stops pending. On miss, a scripted factory+24 supplies a candidate, whose
/// validator return may be a different object. Source stores that final return.
/// Stable callback/object lifetimes, factory identity and normal return are
/// admitted; no real lookup, reference counts, insertion or live identity follows.
pub fn project(route:Route,post_lookup_slot:Option<u32>,key:u32,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 match route{
  Route::Hit=>{let r=registry::project(true,post_lookup_slot,key,None,x87,mxcsr,admitted).map_err(Error::Registry)?;Ok(Outcome{slot:r.slot,registry_actions:r.actions,factory_actions:vec![]})},
  Route::Miss{candidate,validator_result}=>{let f=factory::project(key,0x0179c807,0,0,candidate,validator_result,x87,mxcsr,admitted).map_err(Error::Factory)?;let r=registry::project(false,post_lookup_slot,key,Some(f.result),x87,mxcsr,admitted).map_err(Error::Registry)?;Ok(Outcome{slot:r.slot,registry_actions:r.actions,factory_actions:f.actions})}
 }
}
#[cfg(test)]mod tests{use super::*;use factory::Action;#[test]fn distinct_result_and_failure_release(){let x=project(Route::Miss{candidate:Some(3),validator_result:Some(Some(1))},Some(2),7,0x27f,0x1f80,true).unwrap();assert_eq!(x.slot,Some(1));let y=project(Route::Miss{candidate:Some(3),validator_result:Some(None)},Some(2),7,0x37f,0x1f80,true).unwrap();assert_eq!(y.slot,None);assert_eq!(y.factory_actions[2..],[Action::ReleaseFirst(3),Action::ReleaseSecond(3)]);}}
