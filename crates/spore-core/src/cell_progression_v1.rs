//! Bounded integer accounting from retail Cell procedure E7CE40 (SDK x86).
//!
//! This is an unintegrated, versioned reconstruction. It does not resolve food,
//! choose the source flag, or perform the calls made by the native procedure.
//! The caller supplies the state *after* the earlier conditional E79750 call;
//! that call may have side effects which are not modeled here.
//!
//! Evidence: `target/source-cell/native-cell-progression-add-sdk.asm`,
//! instructions E7CE91 through E7CF13. Offsets are intentionally retained
//! because the semantic names and full lifecycle of these counters are unknown.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressStateV1 {
    pub progress_1c: i32,
    pub counter_20: i32,
    pub counter_24: i32,
    pub counter_28: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressAccountingV1 {
    pub state: ProgressStateV1,
    /// Whether the following native growth/tutorial/transition route is entered.
    /// The effects and completion conditions of that route are not modeled.
    pub changed_route: bool,
}

/// Apply the integer accounting portion of E7CE40.
///
/// `amount` is the raw signed stack word, not the float suggested by the
/// decompiler. `source_byte` uses the native zero/nonzero test. Arithmetic
/// wraps like x86 32-bit integer instructions; the cap uses a signed compare.
/// The counter names identify save-state offsets, not established meanings.
pub fn account_progress_v1(
    mut state: ProgressStateV1,
    amount: i32,
    source_byte: u8,
) -> ProgressAccountingV1 {
    let before = state.progress_1c;
    state.progress_1c = before.wrapping_add(amount).min(1000);
    let overflow = before.wrapping_sub(state.progress_1c).wrapping_add(amount);
    if overflow > 0 {
        let counter = if source_byte != 0 {
            &mut state.counter_24
        } else {
            &mut state.counter_28
        };
        *counter = counter.wrapping_add(overflow);
    }
    let changed_route = before != state.progress_1c;
    if changed_route && source_byte != 0 {
        state.counter_20 = state.counter_20.wrapping_add(state.progress_1c.wrapping_sub(before));
    }
    ProgressAccountingV1 { state, changed_route }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(progress: i32) -> ProgressStateV1 {
        ProgressStateV1 { progress_1c: progress, counter_20: 7, counter_24: 11, counter_28: 13 }
    }

    #[test]
    fn source_nonzero_records_actual_increase_and_overflow() {
        let result = account_progress_v1(state(990), 25, 2);
        assert_eq!(result.state, ProgressStateV1 {
            progress_1c: 1000, counter_20: 17, counter_24: 26, counter_28: 13,
        });
        assert!(result.changed_route);
    }

    #[test]
    fn zero_source_records_overflow_even_when_progress_does_not_change() {
        let result = account_progress_v1(state(1000), 4, 0);
        assert_eq!(result.state, ProgressStateV1 {
            progress_1c: 1000, counter_20: 7, counter_24: 11, counter_28: 17,
        });
        assert!(!result.changed_route);
    }

    #[test]
    fn negative_and_overflowing_inputs_preserve_signed_x86_behavior() {
        let result = account_progress_v1(state(5), -8, 1);
        assert_eq!(result.state.progress_1c, -3);
        assert_eq!(result.state.counter_20, -1);
        assert_eq!(result.state.counter_24, 11);
        let wrapped = account_progress_v1(state(i32::MAX), 1, 0);
        assert_eq!(wrapped.state.progress_1c, i32::MIN);
        assert_eq!(wrapped.state.counter_28, 13);
    }
}
