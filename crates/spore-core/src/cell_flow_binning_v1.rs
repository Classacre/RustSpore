//! Bounded `E4DC40` bin construction from explicitly supplied environment slots.
//!
//! This is the snapshot *before* `E4E730`, not a complete field update. The
//! intervening source calls, environment lifecycle and actor scheduling remain
//! outside this operation. It retains source input order and the eight-record
//! capacity of each of the 20 by 15 bins. All data and FP policy are supplied;
//! this does not establish live game state or source-instruction conformance.

use crate::cell_flow_field::{CellFlowFrame, CELL_FLOW_HEIGHT, CELL_FLOW_SAMPLES, CELL_FLOW_WIDTH};
use crate::native_sse::{self, Policy, SseError};

pub const MAX_ENVIRONMENT_SLOTS: usize = 1500;
pub const MAX_BIN_RECORDS: usize = 8;

/// The native finite scalar copies use x87 FLD/FSTP. The caller admits masked
/// x87 exceptions and sufficient stack space; this model does not expose the
/// status flags that native loads and stores may set, including for subnormals.
#[derive(Clone, Copy, Debug)]
pub struct X87CopyAdmission;

/// One caller-supplied slot from the three native environment arrays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub position: [f32; 3],
    pub vector: [f32; 3],
    pub scalar: f32,
}

/// A copied source record. `index` is the original slot ordinal, not an actor ID.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Record {
    pub position: [f32; 3],
    pub vector: [f32; 3],
    pub scalar: f32,
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bins {
    pub bins: [Vec<Record>; CELL_FLOW_SAMPLES],
    pub outside: usize,
    pub full_bin: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManySlots,
    NonFiniteFrameOrPosition,
    NonFiniteCoordinate,
    NonFiniteCopiedRecord,
    Sse(SseError),
}
impl From<SseError> for Error {
    fn from(error: SseError) -> Self {
        Self::Sse(error)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bounded Cell flow binning: {self:?}")
    }
}
impl std::error::Error for Error {}

// The scoped MXCSR is an implicit input, so these instructions are neither
// `pure` nor `nomem`; their source operand order and individual commits matter.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn sub(mut left: f32, right: f32) -> f32 {
    // SAFETY: scalar XMM registers only; the surrounding scope checks SSE2.
    unsafe {
        std::arch::asm!("subss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut left: f32, right: f32) -> f32 {
    // SAFETY: scalar XMM registers only; the surrounding scope checks SSE2.
    unsafe {
        std::arch::asm!("mulss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}

/// Construct caller-owned bins under the declared nearest-even SSE policy.
///
/// This checked domain allows at most 1,500 slots and finite frame/XY words.
/// It rejects nonfinite arithmetic and accepted records with nonfinite copied
/// fields; the native routine does not provide these checked errors. The x87
/// admission is required for finite scalar copies. Source x87 status/traps and
/// MXCSR sticky flags are not reproduced; the caller's MXCSR is restored. Rejected
/// and full-bin slots do not consume their vector, scalar or Z. The native
/// caller passes 1,500 slots; smaller snapshots are controlled inputs. Input
/// arrays, frame and results must be stable and nonaliasing for comparison.
pub fn build(
    slots: &[Slot],
    frame: CellFlowFrame,
    policy: Policy,
    _x87: X87CopyAdmission,
) -> Result<Bins, Error> {
    if slots.len() > MAX_ENVIRONMENT_SLOTS {
        return Err(Error::TooManySlots);
    }
    if !frame
        .origin
        .into_iter()
        .chain(frame.scale)
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteFrameOrPosition);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut result = Bins {
                bins: std::array::from_fn(|_| Vec::new()),
                outside: 0,
                full_bin: 0,
            };
            for (index, slot) in slots.iter().enumerate() {
                if !slot.position[0].is_finite() || !slot.position[1].is_finite() {
                    return Err(Error::NonFiniteFrameOrPosition);
                }
                let x = mul(sub(slot.position[0], frame.origin[0]), frame.scale[0]);
                let y = mul(sub(slot.position[1], frame.origin[1]), frame.scale[1]);
                if !x.is_finite() || !y.is_finite() {
                    return Err(Error::NonFiniteCoordinate);
                }
                // For finite coordinates in this accepted range, the source's
                // CVTSS2SI/compare/correction is exactly floor. Outside it the
                // unsigned source checks reject the bin, including negatives.
                if x < 0.0 || x >= CELL_FLOW_WIDTH as f32 || y < 0.0 || y >= CELL_FLOW_HEIGHT as f32
                {
                    result.outside += 1;
                    continue;
                }
                let bin = x.floor() as usize + CELL_FLOW_WIDTH * y.floor() as usize;
                if result.bins[bin].len() == MAX_BIN_RECORDS {
                    result.full_bin += 1;
                    continue;
                }
                if !slot.position[2].is_finite()
                    || !slot.vector.iter().all(|v| v.is_finite())
                    || !slot.scalar.is_finite()
                {
                    return Err(Error::NonFiniteCopiedRecord);
                }
                // Native copies position and vector words, and FLD/FSTP copies
                // finite binary32 scalar words exactly. Exception flags are not
                // modeled by this checked API.
                result.bins[bin].push(Record {
                    position: slot.position,
                    vector: slot.vector,
                    scalar: slot.scalar,
                    index: index as u32,
                });
            }
            Ok(result)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = slots;
            Err(Error::Sse(SseError::UnsupportedArchitecture))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slot(x: f32, y: f32, tag: f32) -> Slot {
        Slot {
            position: [x, y, tag],
            vector: [tag, -tag, 1.0],
            scalar: tag,
        }
    }
    fn frame() -> CellFlowFrame {
        CellFlowFrame {
            origin: [0.0, 0.0],
            scale: [1.0, 1.0],
        }
    }

    #[test]
    fn bounds_capacity_and_input_order() {
        let mut slots: Vec<_> = (0..10).map(|i| slot(0.25, 0.25, i as f32)).collect();
        slots.extend([
            slot(-0.01, 0.0, 1.0),
            slot(20.0, 0.0, 2.0),
            slot(19.999, 14.999, 3.0),
            slot(-0.0, 1.0, 4.0),
        ]);
        let bins = build(&slots, frame(), Policy::Mxcsr1f80, X87CopyAdmission).unwrap();
        assert_eq!((bins.outside, bins.full_bin), (2, 2));
        assert_eq!(
            bins.bins[0].iter().map(|r| r.index).collect::<Vec<_>>(),
            (0..8).collect::<Vec<_>>()
        );
        assert_eq!(bins.bins[299][0].index, 12);
        assert_eq!(bins.bins[20][0].position[0].to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn source_transform_and_unconsumed_fields() {
        let transform = CellFlowFrame {
            origin: [10.0, -2.0],
            scale: [2.0, 0.5],
        };
        let mut outside = slot(100.0, 100.0, f32::NAN);
        outside.vector = [f32::NAN; 3];
        let accepted = slot(10.75, 2.0, -0.0);
        let bins = build(
            &[outside, accepted],
            transform,
            Policy::Mxcsr1f80,
            X87CopyAdmission,
        )
        .unwrap();
        assert_eq!(bins.outside, 1);
        let record = bins.bins[1 + 2 * CELL_FLOW_WIDTH][0];
        assert_eq!(record.index, 1);
        assert_eq!(record.scalar.to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn rejects_reached_invalid_domains_without_output() {
        assert_eq!(
            build(
                &vec![slot(0.0, 0.0, 1.0); 1501],
                frame(),
                Policy::Mxcsr1f80,
                X87CopyAdmission
            ),
            Err(Error::TooManySlots)
        );
        assert_eq!(
            build(
                &[slot(f32::NAN, 0.0, 1.0)],
                frame(),
                Policy::Mxcsr1f80,
                X87CopyAdmission
            ),
            Err(Error::NonFiniteFrameOrPosition)
        );
        assert_eq!(
            build(
                &[slot(f32::MAX, 0.0, 1.0)],
                CellFlowFrame {
                    origin: [-f32::MAX, 0.0],
                    scale: [1.0; 2]
                },
                Policy::Mxcsr1f80,
                X87CopyAdmission
            ),
            Err(Error::NonFiniteCoordinate)
        );
        assert_eq!(
            build(
                &[slot(0.0, 0.0, f32::NAN)],
                frame(),
                Policy::Mxcsr1f80,
                X87CopyAdmission
            ),
            Err(Error::NonFiniteCopiedRecord)
        );
    }

    #[test]
    fn full_bin_does_not_consume_later_invalid_fields_and_accepts_1500_slots() {
        let mut slots = vec![slot(0.25, 0.25, 1.0); MAX_ENVIRONMENT_SLOTS];
        slots[8] = Slot {
            position: [0.25, 0.25, f32::NAN],
            vector: [f32::NAN; 3],
            scalar: f32::NAN,
        };
        let bins = build(&slots, frame(), Policy::Mxcsr1f80, X87CopyAdmission).unwrap();
        assert_eq!(
            (bins.bins[0].len(), bins.full_bin, bins.outside),
            (8, 1492, 0)
        );
        assert_eq!(bins.bins[0].last().unwrap().index, 7);
    }

    #[test]
    fn adjacent_binary32_coordinates_use_source_floor_boundaries() {
        let below_one = f32::from_bits(1.0f32.to_bits() - 1);
        let above_one = f32::from_bits(1.0f32.to_bits() + 1);
        let below_twenty = f32::from_bits(20.0f32.to_bits() - 1);
        let slots = [
            slot(below_one, 0.0, 1.0),
            slot(1.0, 0.0, 2.0),
            slot(above_one, 0.0, 3.0),
            slot(below_twenty, 0.0, 4.0),
            slot(20.0, 0.0, 5.0),
            slot(-f32::from_bits(1), 0.0, 6.0),
        ];
        let bins = build(&slots, frame(), Policy::Mxcsr1f80, X87CopyAdmission).unwrap();
        assert_eq!(bins.bins[0][0].index, 0);
        assert_eq!(
            bins.bins[1].iter().map(|r| r.index).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(bins.bins[19][0].index, 3);
        assert_eq!(bins.outside, 2);
    }
}
