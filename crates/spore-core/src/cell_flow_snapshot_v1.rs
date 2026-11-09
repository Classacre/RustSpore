//! Controlled composition of source-oriented binning and aggregation.
//!
//! The real `E4E730` force update is not executed. Its preserved-scratch
//! condition must be admitted separately; a caller must supply the actual slot
//! snapshots and frame. This is not a live environment or player update.

use crate::cell_flow_aggregation_v1::{self as aggregation, Bin, Output};
use crate::cell_flow_binning_v1::{self as binning, Slot, X87CopyAdmission};
use crate::cell_flow_field::{CellFlowFrame, CELL_FLOW_SAMPLES};
use crate::native_sse::Policy;
use crate::native_x87::Context;

/// Admit that intervening native work did not change the copied scratch bins.
/// This requires the valid, bounded, nonaliasing conditions in
/// `re/cell-flow-intervening-force-static-v1.md`, including the neighborhood
/// output capacity and the absence of external writers. It does not execute or
/// validate that work.
#[derive(Clone, Copy, Debug)]
pub struct PreservedScratchAdmission;

#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub outputs: [Output; CELL_FLOW_SAMPLES],
    pub counts: [u8; CELL_FLOW_SAMPLES],
    pub outside: usize,
    pub full_bin: usize,
}

impl Snapshot {
    /// Extract the generated vector field for the existing source sampler.
    /// The caller must still admit its runtime frame and the sampler's FP policy.
    pub fn field_vectors(&self) -> [[f32; 3]; CELL_FLOW_SAMPLES] {
        std::array::from_fn(|index| self.outputs[index].vector)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Binning(binning::Error),
    Aggregation(aggregation::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "controlled Cell flow snapshot: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Build and aggregate one caller-owned snapshot. Both substeps use checked,
/// finite domains and return no partial output on error. FP sticky flags and
/// intervening force effects are not represented.
pub fn build(
    slots: &[Slot],
    frame: CellFlowFrame,
    policy: Policy,
    x87: Context,
    x87_copy: X87CopyAdmission,
    _preserved: PreservedScratchAdmission,
) -> Result<Snapshot, Error> {
    let copied = binning::build(slots, frame, policy, x87_copy).map_err(Error::Binning)?;
    let records: Vec<Vec<aggregation::Record>> = copied
        .bins
        .iter()
        .map(|bin| {
            bin.iter()
                .map(|r| aggregation::Record {
                    vector: r.vector,
                    scalar: r.scalar,
                })
                .collect()
        })
        .collect();
    let bins: Vec<Bin<'_>> = records
        .iter()
        .map(|row| Bin {
            count: row.len() as i32,
            records: row,
        })
        .collect();
    let outputs = aggregation::aggregate(&bins, policy, x87).map_err(Error::Aggregation)?;
    let counts = std::array::from_fn(|i| copied.bins[i].len() as u8);
    Ok(Snapshot {
        outputs,
        counts,
        outside: copied.outside,
        full_bin: copied.full_bin,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn context() -> Context {
        Context {
            precision: Precision::Bits64,
            rounding: Rounding::NearestEven,
        }
    }
    fn frame() -> CellFlowFrame {
        CellFlowFrame {
            origin: [0.0, 0.0],
            scale: [1.0, 1.0],
        }
    }

    #[test]
    fn composes_source_ordered_bins_and_field_outputs() {
        let mut slots = vec![
            Slot {
                position: [0.25, 0.25, 0.0],
                vector: [3.0, 4.0, 0.0],
                scalar: 2.0
            };
            10
        ];
        slots.push(Slot {
            position: [2.0, 1.0, 0.0],
            vector: [0.0, 0.0, 1.0],
            scalar: 4.0,
        });
        let result = build(
            &slots,
            frame(),
            Policy::Mxcsr1f80,
            context(),
            X87CopyAdmission,
            PreservedScratchAdmission,
        )
        .unwrap();
        assert_eq!(
            (result.counts[0], result.full_bin, result.outside),
            (8, 2, 0)
        );
        assert_eq!(result.outputs[0].vector, [3.0, 4.0, 0.0]);
        assert_eq!(
            (
                result.outputs[0].magnitude,
                result.outputs[0].mean_reciprocal
            ),
            (5.0, 0.5)
        );
        assert_eq!(result.counts[22], 1);
        assert_eq!(result.outputs[22].mean_reciprocal, 0.25);
        assert_eq!(result.outputs[1].vector.map(f32::to_bits), [0; 3]);
        let sample =
            crate::cell_flow_field::sample_flow_field([2.5, 1.5], frame(), &result.field_vectors())
                .unwrap();
        assert_eq!(sample, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn reached_zero_scalar_is_an_explicit_aggregation_error() {
        let slots = [Slot {
            position: [0.0; 3],
            vector: [1.0; 3],
            scalar: 0.0,
        }];
        assert_eq!(
            build(
                &slots,
                frame(),
                Policy::Mxcsr1f80,
                context(),
                X87CopyAdmission,
                PreservedScratchAdmission
            ),
            Err(Error::Aggregation(aggregation::Error::ZeroDenominator))
        );
    }
}
