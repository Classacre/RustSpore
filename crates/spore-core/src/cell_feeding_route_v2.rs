//! Bounded E7D0A0 feeding-to-E7CE40 accounting composition, version 2.
//!
//! Evidence: `target/source-cell/native-cell-eating-sdk.asm`, E7D119-E7D2BA;
//! `native-cell-progression-add-sdk.asm`, E7CE40-E7CF13; REA dossiers
//! ev_a087b3ecd9eeb1cc0636f12c2f76929366b2e7863a6ce2b60c854a7052ca2291
//! and ev_55ac184edc91c220ea244bd9da72c956e1d9d0987780c9782497bbe256e122c4.
//! This model does not execute the intervening native calls or integrate gameplay.

use crate::cell_progression_v1::{account_progress_v1, ProgressAccountingV1, ProgressStateV1};

/// Snapshot after bomb/nova calls and before the healing branch. A present
/// definition is the valid E4CAA0 result, not an inferred missing-resource fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreHealDefinitionV2 {
    NativeNullOrSpecial,
    Present { hp_value: i32, food_value: i32 },
}

/// Fresh snapshot at E7D26D, after E514B0 and any E532C0 call. A present
/// snapshot refers to the same native definition object selected earlier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDefinitionV2 {
    NativeNullOrSpecial,
    Present { flags: u32, cell_type: u32 },
}

/// Explicit caller admission: global player reference exists, B71D00 resolved
/// a valid player object, and all relevant pointers remain valid at each read.
/// If E622F0 is called, it preserves the health, difficulty, hp and food values
/// used by the numeric slice through their native reads. Effect calls have
/// already completed. This is a precondition, not a claim about those calls.
#[derive(Debug, Clone, Copy)]
pub struct PreHealAdmissionV2;

/// Explicit caller admission: the native definition pointer and player/save
/// pointers remain valid through E514B0 and conditional E532C0. The routing
/// snapshot is read after those calls, from the same selected definition object.
#[derive(Debug, Clone, Copy)]
pub struct RoutingAdmissionV2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedingSliceV2 {
    /// E82160 cleanup is still pending; its effects and completion are unknown.
    ZeroFoodCleanupPending { health_after: i32 },
    Continue { health_after: i32, amount: i32, definition_present: bool },
}

/// Evaluate only healing and food selection, excluding all native callbacks.
/// No bomb or nova request is inferred from this later snapshot.
pub fn pre_heal_slice_v2(
    health_after_effect_calls: i32,
    difficulty_code: i32,
    definition: PreHealDefinitionV2,
    _admission: PreHealAdmissionV2,
) -> FeedingSliceV2 {
    let mut health = health_after_effect_calls;
    if health < 6 {
        match definition {
            PreHealDefinitionV2::NativeNullOrSpecial => health = health.wrapping_add(2),
            PreHealDefinitionV2::Present { hp_value, .. } if hp_value > 0 => {
                let gain = if difficulty_code == 0 { hp_value.wrapping_mul(2) } else { hp_value };
                health = health.wrapping_add(gain);
            }
            PreHealDefinitionV2::Present { .. } => {}
        }
    }
    let health_after = health.min(6);
    match definition {
        PreHealDefinitionV2::Present { food_value: 0, .. } => FeedingSliceV2::ZeroFoodCleanupPending { health_after },
        PreHealDefinitionV2::Present { food_value, .. } => FeedingSliceV2::Continue { health_after, amount: food_value, definition_present: true },
        PreHealDefinitionV2::NativeNullOrSpecial => FeedingSliceV2::Continue { health_after, amount: 10, definition_present: false },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressDispatchV2 { pub amount: i32, pub source_byte: u8 }

/// Rejects an inconsistent null/present snapshot; pointer identity and lifetime
/// are caller admissions because these plain snapshots cannot establish them.
pub fn route_v2(
    feeding: FeedingSliceV2,
    routing: RoutingDefinitionV2,
    raw_stack_word: u32,
    _admission: RoutingAdmissionV2,
) -> Option<ProgressDispatchV2> {
    let (amount, definition_present) = match feeding {
        FeedingSliceV2::Continue { amount, definition_present, .. } => (amount, definition_present),
        FeedingSliceV2::ZeroFoodCleanupPending { .. } => return None,
    };
    let source_byte = match (definition_present, routing) {
        (false, RoutingDefinitionV2::NativeNullOrSpecial) => 1,
        (true, RoutingDefinitionV2::Present { flags, cell_type }) => {
            if flags & 1 != 0 && raw_stack_word == 3 { 1 }
            else if flags & 1 != 0 && (raw_stack_word == 2 || raw_stack_word == 4) { 0 }
            else { u8::from(cell_type == 3) }
        }
        _ => return None,
    };
    Some(ProgressDispatchV2 { amount, source_byte })
}

/// State actually captured after all intervening calls, including E79750 if
/// its E7CE40 gate called it. No E79750 side effects are predicted here.
#[derive(Debug, Clone, Copy)]
pub struct PostInterveningCallsV2(pub ProgressStateV1);

pub fn account_dispatch_v2(dispatch: ProgressDispatchV2, state: PostInterveningCallsV2) -> ProgressAccountingV1 {
    account_progress_v1(state.0, dispatch.amount, dispatch.source_byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_uses_fresh_snapshot_and_overrides_type_for_words_two_and_four() {
        let feeding = pre_heal_slice_v2(4, 1, PreHealDefinitionV2::Present { hp_value: 1, food_value: 7 }, PreHealAdmissionV2);
        for (flags, cell_type, word, source_byte) in [
            (1, 3, 2, 0), (1, 0, 3, 1), (1, 3, 4, 0),
            (1, 3, 5, 1), (1, 0, 5, 0), (2, 3, 2, 1), (0, 0, 3, 0),
        ] {
            assert_eq!(route_v2(feeding, RoutingDefinitionV2::Present { flags, cell_type }, word, RoutingAdmissionV2), Some(ProgressDispatchV2 { amount: 7, source_byte }));
        }
    }

    #[test]
    fn zero_food_retains_pending_cleanup_and_cannot_route() {
        let feeding = pre_heal_slice_v2(4, 1, PreHealDefinitionV2::Present { hp_value: 1, food_value: 0 }, PreHealAdmissionV2);
        assert_eq!(feeding, FeedingSliceV2::ZeroFoodCleanupPending { health_after: 5 });
        assert_eq!(route_v2(feeding, RoutingDefinitionV2::Present { flags: 1, cell_type: 3 }, 3, RoutingAdmissionV2), None);
    }

    #[test]
    fn null_route_and_snapshot_mismatch() {
        let feeding = pre_heal_slice_v2(4, 0, PreHealDefinitionV2::NativeNullOrSpecial, PreHealAdmissionV2);
        assert_eq!(route_v2(feeding, RoutingDefinitionV2::NativeNullOrSpecial, 0, RoutingAdmissionV2), Some(ProgressDispatchV2 { amount: 10, source_byte: 1 }));
        assert_eq!(route_v2(feeding, RoutingDefinitionV2::Present { flags: 0, cell_type: 0 }, 0, RoutingAdmissionV2), None);
    }

    #[test]
    fn accounting_uses_the_post_call_state() {
        let dispatch = ProgressDispatchV2 { amount: 10, source_byte: 1 };
        let state = ProgressStateV1 { progress_1c: 995, counter_20: 2, counter_24: 3, counter_28: 4 };
        let result = account_dispatch_v2(dispatch, PostInterveningCallsV2(state));
        assert_eq!(result.state, ProgressStateV1 { progress_1c: 1000, counter_20: 7, counter_24: 8, counter_28: 4 });
        assert!(result.changed_route);
    }
}
