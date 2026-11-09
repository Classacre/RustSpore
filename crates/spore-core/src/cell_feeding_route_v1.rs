//! Bounded composition of the E7D0A0 feeding and E7CE40 accounting slices.
//!
//! Static evidence: `target/source-cell/native-cell-eating-sdk.asm`, E7D0D2
//! through E7D2BA, and `native-cell-progression-add-sdk.asm`, E7CE40 through
//! E7CF13. This is not a complete feeding operation and is not integrated into
//! gameplay. In particular, effects and calls between these slices may mutate
//! state. Inputs below must be captured at the stated boundaries.

use crate::cell_definition::{eating_numerics, CellDefinition, CellEatingNumerics};
use crate::cell_progression_v1::{account_progress_v1, ProgressAccountingV1, ProgressStateV1};

/// The resolved E4CAA0 result, or its explicit null/special-ID route. A missing
/// resource must not be silently treated as this null route by the caller.
#[derive(Debug, Clone, Copy)]
pub enum ConsumedV1<'a> {
    Definition(CellDefinition<'a>),
    NativeNullOrSpecial,
}

/// Result at E7D1D8 or the earlier zero-food return. This slice assumes the
/// player reference existed and bomb/nova calls have already completed. Health
/// must be read after those calls; their side effects are not predicted here.
/// This slice also requires that the conditional E622F0 callback does not alter
/// the health word or the relevant difficulty/definition fields before healing.
/// That callback's full effects have not been established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedingDispatchV1 {
    pub numeric: CellEatingNumerics,
    /// Present only when E7D0A0 reaches the progression call. The word is the
    /// raw stack argument read at E7D286, with no inferred semantic meaning.
    pub progression: Option<ProgressDispatchV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressDispatchV1 {
    pub amount: i32,
    pub source_byte: u8,
}

/// Caller admission, not a claim that the native callback has been modeled.
#[derive(Debug, Clone, Copy)]
pub struct CallbackPreservesInputsV1;

pub fn feeding_dispatch_v1(
    consumed: ConsumedV1<'_>,
    health_after_effect_calls: i32,
    difficulty_code: i32,
    raw_stack_word: u32,
    _callback_admission: CallbackPreservesInputsV1,
) -> FeedingDispatchV1 {
    let definition = match consumed {
        ConsumedV1::Definition(value) => Some(value),
        ConsumedV1::NativeNullOrSpecial => None,
    };
    let numeric = eating_numerics(health_after_effect_calls, definition, difficulty_code);
    let progression = numeric.food_value.map(|amount| {
        let source_byte = match definition {
            None => 1,
            Some(value) if value.flags() & 1 != 0 && raw_stack_word == 3 => 1,
            Some(value) if value.flags() & 1 != 0 && (raw_stack_word == 2 || raw_stack_word == 4) => 0,
            Some(value) => u8::from(value.cell_type() == 3),
        };
        ProgressDispatchV1 { amount, source_byte }
    });
    FeedingDispatchV1 { numeric, progression }
}

/// Complete only the measured integer slice. The supplied state must be read
/// after E7D0A0's intervening calls and the conditional E79750 call in E7CE40.
/// The native calls after accounting remain outside this model.
#[derive(Debug, Clone, Copy)]
pub struct PostInterveningCallsV1(pub ProgressStateV1);

pub fn account_dispatch_v1(
    dispatch: ProgressDispatchV1,
    state_after_intervening_calls: PostInterveningCallsV1,
) -> ProgressAccountingV1 {
    account_progress_v1(state_after_intervening_calls.0, dispatch.amount, dispatch.source_byte)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_definition::{parse, DEFINITION_SIZE};

    fn bytes(flags: u32, kind: u32, food: i32) -> [u8; DEFINITION_SIZE] {
        let mut data = [0; DEFINITION_SIZE];
        data[0xb0..0xb4].copy_from_slice(&flags.to_le_bytes());
        data[0xb4..0xb8].copy_from_slice(&kind.to_le_bytes());
        data[0x30c..0x310].copy_from_slice(&food.to_le_bytes());
        data
    }

    #[test]
    fn source_selection_retains_flag_and_stack_word_branches() {
        for (flags, kind, word, expected) in [
            (1, 3, 2, 0), (1, 0, 3, 1), (1, 3, 4, 0),
            (1, 3, 5, 1), (1, 0, 5, 0), (2, 3, 2, 1), (0, 0, 3, 0),
        ] {
            let data = bytes(flags, kind, 7);
            let result = feeding_dispatch_v1(ConsumedV1::Definition(parse(&data).unwrap()), 4, 1, word, CallbackPreservesInputsV1);
            assert_eq!(result.progression.unwrap(), ProgressDispatchV1 { amount: 7, source_byte: expected });
        }
    }

    #[test]
    fn zero_food_stops_before_progress_and_null_uses_ten() {
        let data = bytes(0, 0, 0);
        let zero = feeding_dispatch_v1(ConsumedV1::Definition(parse(&data).unwrap()), 4, 1, 0, CallbackPreservesInputsV1);
        assert_eq!(zero.progression, None);
        let null = feeding_dispatch_v1(ConsumedV1::NativeNullOrSpecial, 4, 1, 0, CallbackPreservesInputsV1);
        assert_eq!(null.numeric.health_after, 6);
        assert_eq!(null.progression, Some(ProgressDispatchV1 { amount: 10, source_byte: 1 }));
    }

    #[test]
    fn accounting_uses_the_explicit_later_state() {
        let dispatch = feeding_dispatch_v1(ConsumedV1::NativeNullOrSpecial, 2, 0, 0, CallbackPreservesInputsV1).progression.unwrap();
        let later = ProgressStateV1 { progress_1c: 995, counter_20: 2, counter_24: 3, counter_28: 4 };
        let result = account_dispatch_v1(dispatch, PostInterveningCallsV1(later));
        assert_eq!(result.state, ProgressStateV1 { progress_1c: 1000, counter_20: 7, counter_24: 8, counter_28: 4 });
        assert!(result.changed_route);
    }
}
