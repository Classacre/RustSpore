//! Bounded original 8D8010 close helper and 8D81CD caller.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingPostCallback}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:u8,pub field_14:u32,pub field_26c:u32,pub backing_calls:u32}
/// The caller supplies fresh +264 at helper entry and stable stream storage.
/// For +264 zero, `after_callback` contains fresh (+14,+26C) after the
/// normally returning nonreentrant synthetic backing +18 callback. For nonzero,
/// `before` is the helper-entry (+14,+26C) snapshot; the helper clears +26C.
/// Preserved BL is supplied and returned; backing AL is ignored. A real close,
/// stream ownership, callback side effects beyond supplied snapshots and lifetime
/// are not established.
pub fn project(bl:u8,field_264:u32,before:(u32,u32),after_callback:Option<(u32,u32)>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let (f14,f26,calls)=if field_264==0{let (a,b)=after_callback.ok_or(Error::MissingPostCallback)?;(a,b,1)}else{(before.0,0,0)};
 Ok(Outcome{returned:bl,field_14:if bl==0{0}else{f14},field_26c:f26,backing_calls:calls})
}
#[cfg(test)]mod tests{use super::*;#[test]fn paths(){assert_eq!(project(0,0,(1,2),Some((3,4)),true).unwrap(),Outcome{returned:0,field_14:0,field_26c:4,backing_calls:1});assert_eq!(project(1,1,(1,2),None,true).unwrap().field_26c,0);assert_eq!(project(0,0,(0,0),None,true),Err(Error::MissingPostCallback));}}
