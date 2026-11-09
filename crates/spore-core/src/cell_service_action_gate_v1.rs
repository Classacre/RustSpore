//! Conditional 7E3868 service-token gate; changed/equal continuations pending.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Branch{Equal,Changed}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Outcome{pub branch:Branch,pub current:u32,pub requested:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unstable,InvalidServiceToken}
/// Caller admits the candidate service implementation, stable +88 snapshot,
/// valid synthetic entry frame and object lifetime. `service_token` is a fixture
/// identity, not a native pointer. Neither continuation or SEH setup is run;
/// Changed is not evidence of callback execution or completed transition.
pub fn gate(service_token:u64,current:u32,requested:u32,stable:bool)->Result<Outcome,Error>{if !stable{return Err(Error::Unstable)}if service_token==0{return Err(Error::InvalidServiceToken)}Ok(Outcome{branch:if current==requested{Branch::Equal}else{Branch::Changed},current,requested})}
#[cfg(test)]mod tests{use super::*;#[test]fn equal_changed_and_admissions(){assert_eq!(gate(1,u32::MAX,u32::MAX,true).unwrap().branch,Branch::Equal);assert_eq!(gate(1,1,u32::MAX,true).unwrap().branch,Branch::Changed);assert_eq!(gate(0,1,1,true),Err(Error::InvalidServiceToken));assert_eq!(gate(1,1,1,false),Err(Error::Unstable));}}
