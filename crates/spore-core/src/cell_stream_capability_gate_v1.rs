//! Bounded 8D80BA stream capability gate with continuations stopped.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub enum Branch{BackingMissing,CapabilityMatched,CapabilityRejected}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub branch:Branch,pub field_14:u32,pub capability_calls:u32}
/// Caller admits the preceding enabled open-prefix state, stable disjoint
/// stream/backing storage and normally returning nonreentrant capability call.
/// A null backing and successful capability test stop before their respective
/// continuations; no file open, later return, or live owner is inferred.
pub fn project(first:u32,capabilities:Option<u32>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let needed=if first&2==0{1}else{3};
 match capabilities{None=>Ok(Outcome{branch:Branch::BackingMissing,field_14:needed,capability_calls:0}),Some(caps) if caps&needed==needed=>Ok(Outcome{branch:Branch::CapabilityMatched,field_14:needed,capability_calls:1}),Some(_)=>Ok(Outcome{branch:Branch::CapabilityRejected,field_14:0,capability_calls:1})}
}
#[cfg(test)]mod tests{use super::*;#[test]fn masks_and_null(){assert_eq!(project(2,Some(1),true).unwrap().branch,Branch::CapabilityRejected);assert_eq!(project(2,Some(3),true).unwrap().field_14,3);assert_eq!(project(1,None,true).unwrap().branch,Branch::BackingMissing);}}
