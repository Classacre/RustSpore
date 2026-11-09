//! Bounded E59A90 helper path; lookups and callback results supplied by caller.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Actor { pub field_248:u32, pub field_fc:u32 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Route { Returned { al:u8, flag_after:u8, callback_called:bool }, AlternatePending }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, MissingAuxSnapshot, MissingCallbackResult }
/// Caller admits source entry, stable disjoint state, and the actor lookup result.
/// Auxiliary snapshot is fresh after the optional pool lookup; flag and callback
/// presence are fresh after any lookups before their original reads. A callback result
/// is supplied only if called, and its record lifetime must survive the callback.
/// The nonzero auxiliary route stops before E50480; no live lookup or callback runs.
pub fn helper(actor:Option<Actor>,flag:u8,callback_present:bool,aux_24:Option<u32>,callback_al:Option<u8>,admitted:bool)->Result<Route,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let Some(a)=actor else{return Ok(Route::Returned{al:1,flag_after:flag,callback_called:false})};
 if a.field_248!=0 && a.field_fc!=0 && aux_24.ok_or(Error::MissingAuxSnapshot)?!=0{return Ok(Route::AlternatePending)}
 if !callback_present || flag==1{return Ok(Route::Returned{al:1,flag_after:flag,callback_called:false})}
 let al=callback_al.ok_or(Error::MissingCallbackResult)?;
 Ok(Route::Returned{al,flag_after:1,callback_called:true})
}
#[cfg(test)] mod tests{use super::*;#[test]fn branch_and_callback(){let a=Some(Actor{field_248:1,field_fc:1});assert_eq!(helper(a,0,true,None,Some(0),true),Err(Error::MissingAuxSnapshot));assert_eq!(helper(a,0,true,Some(1),None,true),Ok(Route::AlternatePending));assert_eq!(helper(a,0,true,Some(0),Some(0),true),Ok(Route::Returned{al:0,flag_after:1,callback_called:true}));assert_eq!(helper(a,1,true,Some(0),None,true),Ok(Route::Returned{al:1,flag_after:1,callback_called:false}));}}
