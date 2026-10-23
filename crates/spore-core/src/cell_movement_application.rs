//! Bounded scalar application from original Cell function `00E55FB0`.
//!
//! Actor fields and the environment's indexed accumulators are explicit inputs.
//! This applies a supplied movement delta; it does not calculate steering, the
//! source actor scale, or a simulation time step. The supported arithmetic
//! domain is finite IEEE binary32 with nearest rounding and preserved subnormals.

use std::collections::BTreeMap;
use std::fmt;

/// Native application inputs, with source field names left unclassified.
#[derive(Clone, Copy, Debug)]
pub struct CellMovementDelta<'a> {
    /// Source actor word at `248`; zero disables indexed-part application.
    pub actor_word_248: u32,
    /// Signed source actor word at `250`; only values at least two use indices.
    pub part_count: i32,
    /// Source actor words from `254`, in original order. Repeats are significant.
    pub part_indices: &'a [i32],
    /// Source actor float at `BC`.
    pub scale: f32,
    /// Supplied three-float vector, independent of the destination accumulators.
    pub delta: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellMovementApplicationError {
    MissingIndices,
    InvalidPartIndex,
    NonFiniteInput,
    NonFiniteResult,
}

impl fmt::Display for CellMovementApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingIndices => "Cell movement count exceeds supplied index slice",
            Self::InvalidPartIndex => "Cell movement index lies outside supplied part vectors",
            Self::NonFiniteInput => "nonfinite Cell movement application input",
            Self::NonFiniteResult => "Cell movement application left its finite arithmetic domain",
        })
    }
}
impl std::error::Error for CellMovementApplicationError {}

fn finite(value: [f32; 3]) -> bool {
    value.iter().all(|component| component.is_finite())
}

/// Apply the complete finite-domain arithmetic and index branch of `E55FB0`.
///
/// `actor_accumulator` corresponds to actor `90..98`; `part_accumulators` is the
/// environment vector array at `4654`, before indexing. The source pointers must
/// describe disjoint input/output buffers, as at the recovered movement caller.
/// A nonzero `248` word with signed count at least two first adds to every listed
/// part, including repeated indices; the actor accumulator is always updated.
/// Other branches do not inspect the unused index slice or part vectors.
///
/// Invalid source memory domains and nonfinite arithmetic are explicit errors.
/// Errors leave all accumulators unchanged. This is a checked boundary around
/// the original operation order, not an original-game bounds check or fallback.
pub fn apply_movement_delta(
    input: CellMovementDelta<'_>,
    actor_accumulator: &mut [f32; 3],
    part_accumulators: &mut [[f32; 3]],
) -> Result<(), CellMovementApplicationError> {
    use CellMovementApplicationError as Error;
    if !input.scale.is_finite() || !finite(input.delta) || !finite(*actor_accumulator) {
        return Err(Error::NonFiniteInput);
    }
    // Preserve MULSS operand ordering, including signed zero.
    let amount = [
        input.scale * input.delta[0],
        input.delta[1] * input.scale,
        input.delta[2] * input.scale,
    ];
    if !finite(amount) {
        return Err(Error::NonFiniteResult);
    }
    let mut updates = BTreeMap::<usize, [f32; 3]>::new();
    if input.actor_word_248 != 0 && input.part_count >= 2 {
        let indices = input
            .part_indices
            .get(..input.part_count as usize)
            .ok_or(Error::MissingIndices)?;
        for &source_index in indices {
            let index = usize::try_from(source_index).map_err(|_| Error::InvalidPartIndex)?;
            let existing = *part_accumulators
                .get(index)
                .ok_or(Error::InvalidPartIndex)?;
            let current = updates.get(&index).copied().unwrap_or(existing);
            if !finite(current) {
                return Err(Error::NonFiniteInput);
            }
            let next = [
                current[0] + amount[0],
                current[1] + amount[1],
                current[2] + amount[2],
            ];
            if !finite(next) {
                return Err(Error::NonFiniteResult);
            }
            updates.insert(index, next);
        }
    }
    // The actor path reverses the ADDSS operands for Y/Z, as in the source.
    let next_actor = [
        actor_accumulator[0] + amount[0],
        amount[1] + actor_accumulator[1],
        amount[2] + actor_accumulator[2],
    ];
    if !finite(next_actor) {
        return Err(Error::NonFiniteResult);
    }
    for (index, next) in updates {
        part_accumulators[index] = next;
    }
    *actor_accumulator = next_actor;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_indices_accumulate_in_source_order_and_unused_indices_are_ignored() {
        let mut actor = [1., 2., 3.];
        let mut parts = [[16_777_216., 0., 0.], [10., 20., 30.]];
        let input = CellMovementDelta {
            actor_word_248: 1,
            part_count: 3,
            part_indices: &[0, 1, 0, -1],
            scale: 1.,
            delta: [1., 2., -3.],
        };
        apply_movement_delta(input, &mut actor, &mut parts).unwrap();
        // Two separate +1 stores both round away; replacing them with +2 differs.
        assert_eq!(parts, [[16_777_216., 4., -6.], [11., 22., 27.]]);
        assert_eq!(actor, [2., 4., 0.]);

        let input = CellMovementDelta {
            actor_word_248: 0,
            part_count: i32::MAX,
            part_indices: &[],
            scale: -0.,
            delta: [1., -1., 0.],
        };
        let mut unused_parts = [[f32::NAN; 3]];
        apply_movement_delta(input, &mut actor, &mut unused_parts).unwrap();
        assert!(unused_parts[0].iter().all(|x| x.is_nan()));
        for count in [i32::MIN, -1, 0, 1] {
            apply_movement_delta(
                CellMovementDelta {
                    actor_word_248: 2,
                    part_count: count,
                    ..input
                },
                &mut actor,
                &mut unused_parts,
            )
            .unwrap();
        }
    }

    #[test]
    fn unsupported_domains_do_not_commit_partial_updates() {
        let mut actor = [1., 2., 3.];
        let mut parts = [[10., 20., 30.]];
        let input = CellMovementDelta {
            actor_word_248: 255,
            part_count: 2,
            part_indices: &[0, 1],
            scale: 1.,
            delta: [1., 2., 3.],
        };
        assert_eq!(
            apply_movement_delta(input, &mut actor, &mut parts),
            Err(CellMovementApplicationError::InvalidPartIndex)
        );
        assert_eq!(actor, [1., 2., 3.]);
        assert_eq!(parts, [[10., 20., 30.]]);
        assert_eq!(
            apply_movement_delta(
                CellMovementDelta {
                    part_indices: &[0],
                    ..input
                },
                &mut actor,
                &mut parts
            ),
            Err(CellMovementApplicationError::MissingIndices)
        );
        assert_eq!(
            apply_movement_delta(
                CellMovementDelta {
                    part_indices: &[0, 0],
                    scale: f32::MAX,
                    delta: [2., 0., 0.],
                    ..input
                },
                &mut actor,
                &mut parts
            ),
            Err(CellMovementApplicationError::NonFiniteResult)
        );
        assert_eq!(actor, [1., 2., 3.]);
        assert_eq!(parts, [[10., 20., 30.]]);
    }
}
