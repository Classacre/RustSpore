//! Bounded 8D8040 stream-open prefix with file/external work stopped.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub enum Branch{Active,Deferred,Disabled,OpenPending,ExternalPending}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub branch:Branch,pub field_18:u32}
/// Stable disjoint stream storage and unchanged inputs are caller admissions.
/// Nonzero active returns before other branches. An external object is stopped
/// before its virtual call; otherwise a nonzero third byte stores first at +18,
/// and a zero third byte stops at the initialized-byte/open boundary. No actual
/// file open, external call, later source return, or live owner is inferred.
pub fn project(active:u32,external:u32,third:u8,initialized:u8,first:u32,initial_18:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let branch=if active!=0{Branch::Active}else if external!=0{Branch::ExternalPending}else if third!=0{Branch::Deferred}else if initialized==0{Branch::Disabled}else{Branch::OpenPending};
 let field_18=if branch==Branch::Deferred{first}else{initial_18};Ok(Outcome{branch,field_18})
}
#[cfg(test)]mod tests{use super::*;#[test]fn priority_and_deferred(){assert_eq!(project(1,1,1,1,7,9,true).unwrap().branch,Branch::Active);assert_eq!(project(0,1,1,1,7,9,true).unwrap().branch,Branch::ExternalPending);assert_eq!(project(0,0,2,0,7,9,true).unwrap().field_18,7);assert_eq!(project(0,0,0,1,7,9,true).unwrap().branch,Branch::OpenPending);}}
