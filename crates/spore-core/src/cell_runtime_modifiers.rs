//! E563F0's data2 multiplier, with E51760's actor-link gates explicit.
//!
//! The caller supplies the resolved animation object's aggregate; this module
//! does not create actors, resolve manager handles or choose an animation clip.
//! Nonfinite reached values remain checked errors from the aggregate reader.

use crate::{
    cell_runtime_data::{CellRuntimeData, CellRuntimeDataError, DATA2_ID},
    native_x87::{Context, Extended},
};

/// Explicit inputs to the original actor248/FC -> resolvedObject24 route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorAnimationData<'a> {
    pub actor_handle_248: u32,
    pub actor_word_fc: u32,
    pub resolved_aggregate: Option<CellRuntimeData<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Data2Route {
    ExcludedActorState,
    MissingActorLink,
    MissingResolvedObject,
    MissingData,
    Found { record: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data2Multiplier {
    /// Native ST0: either FLD1 or FLD of E833B0's committed binary32 value.
    pub retained: Extended,
    pub route: Data2Route,
    /// Whether E51760 reaches its B71D00 manager lookup.
    pub manager_lookup_called: bool,
}

/// Complete E563F0 state gating and finite E833B0 data2 consumption.
///
/// Only the four literal states 12/13/14/30 are excluded. A found positive or
/// negative zero is returned as zero; it must not be replaced by the missing
/// value's literal one. Unused aggregate fields are not read on early exits.
pub fn data2_multiplier(
    actor_state_18c: i32,
    animation: ActorAnimationData<'_>,
    context: Context,
) -> Result<Data2Multiplier, CellRuntimeDataError> {
    let fallback = |route, manager_lookup_called| Data2Multiplier {
        retained: Extended::from_i32(1),
        route,
        manager_lookup_called,
    };
    if matches!(actor_state_18c, 0x12 | 0x13 | 0x14 | 0x30) {
        return Ok(fallback(Data2Route::ExcludedActorState, false));
    }
    if animation.actor_handle_248 == 0 || animation.actor_word_fc == 0 {
        return Ok(fallback(Data2Route::MissingActorLink, false));
    }
    let Some(aggregate) = animation.resolved_aggregate else {
        return Ok(fallback(Data2Route::MissingResolvedObject, true));
    };
    let sample = aggregate.lookup(DATA2_ID, context)?;
    if !sample.found {
        return Ok(fallback(Data2Route::MissingData, true));
    }
    Ok(Data2Multiplier {
        retained: Extended::from_f32(sample.value)?,
        route: Data2Route::Found {
            record: sample.matched_record.expect("found lookup has an ordered row"),
        },
        manager_lookup_called: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::X87Error;

    fn context() -> Context {
        Context::from_control_word(0x027f).unwrap()
    }

    #[test]
    fn four_literal_states_skip_unread_malformed_aggregate() {
        let count_only = 8i32.to_le_bytes();
        let input = ActorAnimationData {
            actor_handle_248: u32::MAX,
            actor_word_fc: 2,
            resolved_aggregate: Some(CellRuntimeData::from_aggregate_a0(&count_only).unwrap()),
        };
        for state in [0x12, 0x13, 0x14, 0x30] {
            let result = data2_multiplier(state, input, context()).unwrap();
            assert_eq!(result.retained, Extended::from_i32(1));
            assert_eq!(result.route, Data2Route::ExcludedActorState);
            assert!(!result.manager_lookup_called);
        }
        for state in [i32::MIN, -1, 0, 0x11, 0x15, 0x2f, 0x31, i32::MAX] {
            assert!(matches!(
                data2_multiplier(state, input, context()),
                Err(CellRuntimeDataError::TruncatedRecords { .. })
            ));
        }
    }

    #[test]
    fn nonzero_link_words_and_missing_results_keep_distinct_branches() {
        let mut input = ActorAnimationData {
            actor_handle_248: 0,
            actor_word_fc: u32::MAX,
            resolved_aggregate: None,
        };
        let missing = data2_multiplier(0, input, context()).unwrap();
        assert_eq!(missing.route, Data2Route::MissingActorLink);
        assert!(!missing.manager_lookup_called);
        input.actor_handle_248 = 2;
        let missing = data2_multiplier(0, input, context()).unwrap();
        assert_eq!(missing.route, Data2Route::MissingResolvedObject);
        assert!(missing.manager_lookup_called);
        let empty = 0i32.to_le_bytes();
        input.resolved_aggregate = Some(CellRuntimeData::from_aggregate_a0(&empty).unwrap());
        let missing = data2_multiplier(0, input, context()).unwrap();
        assert_eq!(missing.route, Data2Route::MissingData);
        assert_eq!(missing.retained, Extended::from_i32(1));
        input.actor_word_fc = 0;
        assert_eq!(
            data2_multiplier(0, input, context()).unwrap().route,
            Data2Route::MissingActorLink
        );
    }

    #[test]
    fn found_signed_zero_is_used_and_reached_nonfinite_is_an_error() {
        let mut raw = [0xa5; 24];
        raw[..4].copy_from_slice(&1i32.to_le_bytes());
        raw[4..8].copy_from_slice(&DATA2_ID.to_le_bytes());
        raw[12..16].copy_from_slice(&(-0f32).to_bits().to_le_bytes());
        raw[20..24].copy_from_slice(&1f32.to_bits().to_le_bytes());
        fn make(bytes: &[u8]) -> ActorAnimationData<'_> {
            ActorAnimationData {
                actor_handle_248: 1,
                actor_word_fc: 1,
                resolved_aggregate: Some(CellRuntimeData::from_aggregate_a0(bytes).unwrap()),
            }
        }
        let result = data2_multiplier(0, make(&raw), context()).unwrap();
        assert_eq!(result.route, Data2Route::Found { record: 0 });
        assert_eq!(context().store_f32(result.retained).unwrap().to_bits(), 0x8000_0000);
        raw[12..16].copy_from_slice(&0x7f80_0001u32.to_le_bytes());
        assert_eq!(
            data2_multiplier(0, make(&raw), context()),
            Err(CellRuntimeDataError::X87(X87Error::NonFiniteInput))
        );
    }
}
