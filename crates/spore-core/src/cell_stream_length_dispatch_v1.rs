//! Bounded 8D8160 length dispatch with downstream operations pending.
#[derive(Debug, PartialEq, Eq)]
pub enum Error { Unadmitted }
#[derive(Debug, PartialEq, Eq)]
pub enum Branch { PresentPending, ZeroReadPending, ZeroWritePending }
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome { pub branch: Branch, pub field_14: u32, pub length_calls: u32 }
/// Admits a stable nonnull synthetic backing, stream +264 equal to zero, and a
/// normally returning nonreentrant backing +1C callback. `length` is its fresh
/// result, and `field_14` is the fresh value after that callback returns. All downstream operations are pending; no actual open or file access
/// and no live identity or lifetime are established.
pub fn project(field_14: u32, length: u32, admitted: bool) -> Result<Outcome, Error> {
    if !admitted { return Err(Error::Unadmitted); }
    let branch = if length != 0 { Branch::PresentPending } else if field_14 & 2 == 0 { Branch::ZeroReadPending } else { Branch::ZeroWritePending };
    Ok(Outcome { branch, field_14, length_calls: 1 })
}
#[cfg(test)] mod tests { use super::*; #[test] fn dispatch() {
    assert_eq!(project(1, 0, true).unwrap().branch, Branch::ZeroReadPending);
    assert_eq!(project(3, 0, true).unwrap().branch, Branch::ZeroWritePending);
    assert_eq!(project(3, 5, true).unwrap().branch, Branch::PresentPending);
    assert_eq!(project(1, 0, false), Err(Error::Unadmitted));
} }

