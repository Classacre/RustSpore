//! Bounded integer-filter projection using explicitly supplied data.
//!
//! This does not discover or assert runtime registration, actor identity,
//! source-side global adjustment, or randomized final selection.
use std::collections::BTreeSet;

pub const MAX_CANDIDATES: usize = 65_536;
pub const MAX_ENTRIES: usize = 65_536;
pub const MAX_POSTINGS: usize = 1_000_000;
pub const MAX_CONDITIONS: usize = 64;
pub const TYPE_FIELD: u32 = 0x02dd90af;
pub const TYPE_VALUE: i32 = 0x3d97a8e4;
pub const RANGE_ZERO: u32 = 0xdddd0000;
pub const RANGE_ONE: u32 = 0xdddd0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid { CandidatesLimit, EntriesLimit, PostingsLimit, ConditionsLimit }

/// A source-oriented value and its associated candidate ordinals; input order
/// is not assumed. Repeated values, fields and ordinals are permitted.
pub struct Entry<'a> {
    pub field: u32,
    pub value: i32,
    pub postings: &'a [u32],
}

/// A signed half-open range. This projection treats equal or reversed bounds
/// as empty; the original iterator behavior for reversed bounds is unverified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Condition { pub field: u32, pub start: i32, pub end: i32 }

/// Build a range from an already-adjusted type-1 row pair. This omits the
/// condition when both bounds are negative; it does not perform global-state
/// adjustments made by the native caller before the observed function.
pub fn adjusted_pair(field: u32, lower: i32, upper: i32) -> Option<Condition> {
    if lower < 0 && upper < 0 { return None; }
    let start = lower.max(0);
    let end = if upper == -1 { 10 } else { upper.min(10) } + 1;
    Some(Condition { field, start, end })
}

pub fn type_condition() -> Condition {
    Condition { field: TYPE_FIELD, start: TYPE_VALUE, end: TYPE_VALUE + 1 }
}

/// Intersect the input candidate set with the union of matching postings for
/// each condition. Returns unique ascending ordinals, not native output order.
pub fn filter(candidates: &[u32], entries: &[Entry<'_>], conditions: &[Condition]) -> Result<Vec<u32>, Invalid> {
    if candidates.len() > MAX_CANDIDATES { return Err(Invalid::CandidatesLimit); }
    if entries.len() > MAX_ENTRIES { return Err(Invalid::EntriesLimit); }
    if conditions.len() > MAX_CONDITIONS { return Err(Invalid::ConditionsLimit); }
    let mut total = 0usize;
    for entry in entries {
        total = total.checked_add(entry.postings.len()).ok_or(Invalid::PostingsLimit)?;
        if total > MAX_POSTINGS { return Err(Invalid::PostingsLimit); }
    }
    let mut result: BTreeSet<u32> = candidates.iter().copied().collect();
    for condition in conditions {
        let mut matching = BTreeSet::new();
        for entry in entries {
            if entry.field == condition.field && entry.value >= condition.start && entry.value < condition.end {
                matching.extend(entry.postings.iter().copied());
            }
        }
        result.retain(|ordinal| matching.contains(ordinal));
        if result.is_empty() { break; }
    }
    Ok(result.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_half_open_unsorted_union_and_intersection() {
        let entries = [
            Entry { field: 1, value: 3, postings: &[3, 2, 2] },
            Entry { field: 1, value: -2, postings: &[1, 2] },
            Entry { field: 1, value: 1, postings: &[3, 1] },
            Entry { field: 1, value: 1, postings: &[4] },
            Entry { field: 2, value: 8, postings: &[3, 2] },
        ];
        let conditions = [Condition { field: 1, start: -2, end: 3 }, Condition { field: 2, start: 8, end: 9 }];
        assert_eq!(filter(&[4, 3, 2, 1, 3], &entries, &conditions), Ok(vec![2, 3]));
    }

    #[test]
    fn empty_missing_and_signed_extremes() {
        let entries = [Entry { field: 1, value: i32::MIN, postings: &[1] }, Entry { field: 1, value: i32::MAX, postings: &[2] }];
        assert_eq!(filter(&[1, 2], &entries, &[Condition { field: 1, start: i32::MIN, end: i32::MAX }]), Ok(vec![1]));
        assert_eq!(filter(&[1], &entries, &[Condition { field: 1, start: 0, end: 0 }]), Ok(vec![]));
        assert_eq!(filter(&[1], &entries, &[Condition { field: 1, start: 4, end: -1 }]), Ok(vec![]));
        assert_eq!(filter(&[1], &entries, &[Condition { field: 99, start: -1, end: 1 }]), Ok(vec![]));
        assert_eq!(filter(&[], &entries, &[]), Ok(vec![]));
        assert_eq!(filter(&[2, 1, 2], &entries, &[]), Ok(vec![1, 2]));
    }

    #[test]
    fn adjusted_ranges_and_type_condition() {
        assert_eq!(adjusted_pair(RANGE_ONE, -1, -1), None);
        assert_eq!(adjusted_pair(RANGE_ONE, 2, -1), Some(Condition { field: RANGE_ONE, start: 2, end: 11 }));
        assert_eq!(adjusted_pair(RANGE_ZERO, -1, 50), Some(Condition { field: RANGE_ZERO, start: 0, end: 11 }));
        assert_eq!(adjusted_pair(RANGE_ZERO, 4, -2), Some(Condition { field: RANGE_ZERO, start: 4, end: -1 }));
        assert_eq!(type_condition(), Condition { field: TYPE_FIELD, start: TYPE_VALUE, end: TYPE_VALUE + 1 });
    }

    #[test]
    fn rejects_input_limits_before_filtering() {
        assert_eq!(filter(&vec![0; MAX_CANDIDATES + 1], &[], &[]), Err(Invalid::CandidatesLimit));
        assert_eq!(filter(&[], &[], &vec![type_condition(); MAX_CONDITIONS + 1]), Err(Invalid::ConditionsLimit));
        let entries: Vec<_> = (0..=MAX_ENTRIES).map(|_| Entry { field: 0, value: 0, postings: &[] }).collect();
        assert_eq!(filter(&[], &entries, &[]), Err(Invalid::EntriesLimit));
        let large = vec![0; MAX_POSTINGS + 1];
        assert_eq!(filter(&[], &[Entry { field: 0, value: 0, postings: &large }], &[]), Err(Invalid::PostingsLimit));
    }
}
