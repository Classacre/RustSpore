//! Bounded 8D8171/+64 dispatch with close and failure continuations stopped.
#[derive(Debug, PartialEq, Eq)] pub enum Error { Unadmitted, MissingCallbackResult }
#[derive(Debug, PartialEq, Eq)] pub enum Branch { ClosePending, FailurePending, Returned(u8) }
#[derive(Debug, PartialEq, Eq)] pub struct Outcome { pub branch: Branch, pub calls_6c: u32, pub calls_64: u32 }
/// `start_at_6c` selects the admitted preceding zero-length write branch;
/// otherwise the preceding nonzero-length branch enters directly at +64.
/// Callback results are fresh AL snapshots. The caller admits a stable stream
/// and vtable, normally returning nonreentrant callbacks and an intact source
/// frame. Close and +2D0 effects remain pending; no live file or owner follows.
pub fn project(start_at_6c: bool, result_6c: Option<u8>, result_64: Option<u8>, admitted: bool) -> Result<Outcome, Error> {
 if !admitted { return Err(Error::Unadmitted); }
 let calls_6c = u32::from(start_at_6c);
 if start_at_6c && result_6c.ok_or(Error::MissingCallbackResult)? == 0 {
  return Ok(Outcome { branch: Branch::ClosePending, calls_6c, calls_64: 0 });
 }
 let al = result_64.ok_or(Error::MissingCallbackResult)?;
 Ok(Outcome { branch: if al == 0 { Branch::FailurePending } else { Branch::Returned(al) }, calls_6c, calls_64: 1 })
}
#[cfg(test)] mod tests { use super::*; #[test] fn paths() {
 assert_eq!(project(true,Some(0),None,true).unwrap().branch,Branch::ClosePending);
 assert_eq!(project(true,Some(1),Some(0),true).unwrap().branch,Branch::FailurePending);
 assert_eq!(project(false,None,Some(2),true).unwrap().branch,Branch::Returned(2));
 assert_eq!(project(true,None,None,true),Err(Error::MissingCallbackResult));
} }
