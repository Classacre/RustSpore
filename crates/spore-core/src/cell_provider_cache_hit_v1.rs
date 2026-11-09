//! Bounded 69E570 cache-hit slot store and callback boundary.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedControl }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Action { Store(u32), Callback(u32) }
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Outcome { pub actions:[Action;2], pub post_callback_slot:Option<u32> }
/// Candidate is the nonzero node+14 value from an already admitted successful
/// non-sentinel inner lookup. The callback must return normally and the caller
/// supplies a fresh post-callback slot snapshot, since it may have changed.
/// Node/object/slot validity and lifetimes are caller admissions; no cache lookup,
/// reference count, callback effects or live provider identity is inferred.
pub fn project(candidate:u32,post_callback_slot:Option<u32>,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted||candidate==0||post_callback_slot==Some(0){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 Ok(Outcome{actions:[Action::Store(candidate),Action::Callback(candidate)],post_callback_slot})
}
#[cfg(test)]mod tests{use super::*;#[test]fn callback_result_is_fresh(){let x=project(1,Some(2),0x27f,0x1f80,true).unwrap();assert_eq!(x.actions,[Action::Store(1),Action::Callback(1)]);assert_eq!(x.post_callback_slot,Some(2));assert_eq!(project(0,None,0x27f,0x1f80,true),Err(Error::Unadmitted));}}
