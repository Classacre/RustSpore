//! Bounded 9315B0 argument translation, before CreateFileW.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub enum Branch{AlreadyActive,NoAccess,CreatePending}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub branch:Branch,pub access:u32,pub share:u32,pub disposition:u32,pub flags:u32}
/// The handle, flags and stable stream/path storage are supplied snapshots.
/// The translated values are pending CreateFileW arguments, not an OS call.
/// Later success/error, path validity, file state and ownership are unverified.
pub fn project(handle:u32,access:u32,mode:u32,share:u32,flags:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let empty=|branch|Outcome{branch,access:0,share:0,disposition:0,flags:0};
 if handle!=u32::MAX{return Ok(empty(Branch::AlreadyActive))}if access==0{return Ok(empty(Branch::NoAccess))}
 let desired=(if access&1!=0{0x8000_0000}else{0})|(if access&2!=0{0x4000_0000}else{0});
 let sharing=(if share&1!=0{1}else{0})|(if share&2!=0{3}else{0});
 let mapped=if mode==6{if access&2!=0{4}else{3}}else{mode};
 let disposition=if (1..=5).contains(&mapped){mapped}else{3};
 let attributes=if flags&1!=0{0x0800_0000}else if flags&2!=0{0x1000_0000}else{0};
 Ok(Outcome{branch:Branch::CreatePending,access:desired,share:sharing,disposition,flags:attributes})
}
#[cfg(test)]mod tests{use super::*;#[test]fn mapping(){assert_eq!(project(0,1,6,0,0,true).unwrap().branch,Branch::AlreadyActive);assert_eq!(project(u32::MAX,0,6,0,0,true).unwrap().branch,Branch::NoAccess);assert_eq!(project(u32::MAX,3,6,3,3,true).unwrap(),Outcome{branch:Branch::CreatePending,access:0xc000_0000,share:3,disposition:4,flags:0x0800_0000});}}
