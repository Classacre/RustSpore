//! Bounded 8D818F failure continuation; downstream operations pending.
#[derive(Debug,PartialEq,Eq)] pub enum Error { Unadmitted, MissingCallbackResult }
#[derive(Debug,PartialEq,Eq)] pub enum Branch { ClosePending, SuccessPending }
#[derive(Debug,PartialEq,Eq)] pub struct Outcome { pub branch:Branch, pub calls_other:u32, pub calls_6c:u32 }
/// +1D and +14 are fresh snapshots after the scripted +2D0/+10 callback;
/// BL is admitted zero from the preceding failed +64 call. A reachable +6C
/// supplies fresh AL. Stable disjoint stream/backing state and normally returning
/// nonreentrant callbacks are admitted; close, +90, files and live owner are pending.
pub fn project(flag_after:u8, field_14_after:u32, result_6c:Option<u8>, admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if flag_after==0 || field_14_after&2==0{return Ok(Outcome{branch:Branch::ClosePending,calls_other:1,calls_6c:0})}
 let al=result_6c.ok_or(Error::MissingCallbackResult)?;
 Ok(Outcome{branch:if al==0{Branch::ClosePending}else{Branch::SuccessPending},calls_other:1,calls_6c:1})
}
#[cfg(test)]mod tests{use super::*;#[test]fn gates(){assert_eq!(project(0,2,None,true).unwrap().branch,Branch::ClosePending);assert_eq!(project(1,2,Some(2),true).unwrap().branch,Branch::SuccessPending);assert_eq!(project(1,2,None,true),Err(Error::MissingCallbackResult));}}
