//! Checked 50AB3C..50AB9A seed selection with supplied state and group id.
//! This does not generate state, allocate groups, or establish runtime identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { CandidateOutOfRange, TooManyFaces }
/// Select the candidate if its state is at most `group`; otherwise return the
/// first state at most `group`, or `None` if there is no eligible face.
pub fn select(states: &[u32], candidate: u32, group: u32) -> Result<Option<u32>, Error> {
 if states.len() > u32::MAX as usize { return Err(Error::TooManyFaces); }
 let i = candidate as usize;
 if i >= states.len() { return Err(Error::CandidateOutOfRange); }
 if states[i] <= group { return Ok(Some(candidate)); }
 Ok(states.iter().position(|&value| value <= group).map(|index| index as u32))
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn selects_candidate_before_first_other_eligible() {
  assert_eq!(select(&[0, 4, 8], 1, 4), Ok(Some(1)));
  assert_eq!(select(&[0, 4, 8], 2, 4), Ok(Some(0)));
 }
 #[test] fn unsigned_boundaries_and_absence() {
  assert_eq!(select(&[u32::MAX, 0x80000000, 0x7fffffff], 0, 0x7fffffff), Ok(Some(2)));
  assert_eq!(select(&[5, u32::MAX], 1, 4), Ok(None));
  assert_eq!(select(&[5], 1, 4), Err(Error::CandidateOutOfRange));
  assert_eq!(select(&[], 0, 0), Err(Error::CandidateOutOfRange));
 }
}
