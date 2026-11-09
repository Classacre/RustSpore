//! Admitted first-hit database choice for a Cell source-resource fixture.
//!
//! `8DE010` inserts entries in descending signed priority, after earlier
//! entries of equal priority. Under the `8DF1A0` no-type-mapping and no-given-
//! database branch, `8DD930` walks that order and selects the first database
//! whose virtual +34 returns success for the exact requested key.
//!
//! The caller supplies successful registration order, signed priorities and
//! frozen query results. This model does not read packages, call databases,
//! resolve type aliases, model callbacks/cache/fallback, or establish which
//! products were mounted. Concurrent changes, failed registration, callbacks
//! that mutate the list, and side effects are excluded. It is not a runtime
//! package winner or player identity.

/// One successfully registered database, in actual registration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatabaseAdmission {
    pub signed_priority: i32,
    /// Caller-admitted result of this database's exact-key query.
    pub exact_key_available: bool,
}

/// Index in `registrations` of the first successful query under the admitted
/// source order. Equal priorities retain registration order. `None` indicates
/// no admitted match; it is not a model of the native fallback route.
pub fn select_first_exact(registrations: &[DatabaseAdmission]) -> Option<usize> {
    let mut selected: Option<usize> = None;
    for (index, entry) in registrations.iter().enumerate() {
        if !entry.exact_key_available {
            continue;
        }
        if selected.is_none_or(|prior| entry.signed_priority > registrations[prior].signed_priority)
        {
            selected = Some(index);
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(priority: i32, present: bool) -> DatabaseAdmission {
        DatabaseAdmission {
            signed_priority: priority,
            exact_key_available: present,
        }
    }
    #[test]
    fn higher_signed_priority_wins_and_missing_entries_do_not_mask_hits() {
        let rows = [
            entry(101, false),
            entry(0, true),
            entry(2, true),
            entry(-3, true),
        ];
        assert_eq!(select_first_exact(&rows), Some(2));
    }
    #[test]
    fn equal_priorities_follow_registration_order_and_extreme_values_are_signed() {
        let rows = [
            entry(i32::MIN, true),
            entry(i32::MAX, true),
            entry(i32::MAX, true),
        ];
        assert_eq!(select_first_exact(&rows), Some(1));
        assert_eq!(
            select_first_exact(&[entry(-1, true), entry(0, true)]),
            Some(1)
        );
    }
    #[test]
    fn no_matching_database_is_not_a_fallback_selection() {
        assert_eq!(select_first_exact(&[]), None);
        assert_eq!(
            select_first_exact(&[entry(2, false), entry(0, false)]),
            None
        );
    }
    #[test]
    fn source_order_matches_explicit_stable_insertion_for_all_small_cases() {
        let choices = [
            entry(-1, false),
            entry(-1, true),
            entry(0, false),
            entry(0, true),
            entry(1, false),
            entry(1, true),
        ];
        for a in choices {
            for b in choices {
                for c in choices {
                    let rows = [a, b, c];
                    let mut order: Vec<usize> = Vec::new();
                    for (i, row) in rows.iter().enumerate() {
                        let at = order
                            .iter()
                            .position(|&j| rows[j].signed_priority < row.signed_priority)
                            .unwrap_or(order.len());
                        order.insert(at, i);
                    }
                    let expected = order.into_iter().find(|&i| rows[i].exact_key_available);
                    assert_eq!(select_first_exact(&rows), expected);
                }
            }
        }
    }
}
