//! Bounded original E56330/E4ECC0 size scaling before the Cell integrator.
//!
//! Actor size index and progress are explicit source words. The twenty source
//! scale values can be supplied verbatim; source table construction/live actor
//! binding remain external. The result retains the original x87 register value.

use crate::{
    native_sse::{self, Policy, SseError},
    native_x87::{Context, Extended, X87Error},
};
use std::fmt;

/// Exact binary32 words read at original 1484680..14846CF, not decimal reparsing.
pub const ORIGINAL_SCALE_TABLE_BITS: [u32; 20] = [
    0x4120_0000,
    0x41f0_0000,
    0x42c8_0000,
    0x4396_0000,
    0x447a_0000,
    0x453b_8000,
    0x461c_4000,
    0x46ea_6000,
    0x47c3_5000,
    0x4892_7c00,
    0x4974_2400,
    0x4a37_1b00,
    0x4b18_9680,
    0x4be4_e1c0,
    0x4cbe_bc20,
    0x4d8f_0d18,
    0x4e6e_6b28,
    0x4f32_d05e,
    0x5015_02f9,
    0x50df_8476,
];

/// Literal threshold/selected-word pairs reached by E4ECC0 for progress0..999.
/// Their source addresses are 1484700/14846E0 plus row*1C.
const PROGRESS_ROWS: [(i32, usize); 20] = [
    (50, 1),
    (100, 1),
    (150, 2),
    (200, 2),
    (250, 3),
    (300, 3),
    (350, 4),
    (400, 4),
    (450, 5),
    (500, 5),
    (550, 6),
    (600, 6),
    (650, 7),
    (700, 7),
    (750, 8),
    (800, 8),
    (850, 9),
    (900, 9),
    (950, 10),
    (1000, 10),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellMovementScalingContext {
    pub x87: Context,
    pub sse: Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellMovementScale {
    pub retained: Extended,
    /// None on the original index-1 early return, before progress is read.
    pub selected_scale_index: Option<usize>,
    /// Exact committed DIVSS word, including any quieted NaN payload.
    pub ratio_bits: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellMovementScalingError {
    UnsupportedActorSizeIndex(i32),
    UnsupportedProgress(i32),
    MissingScaleTable,
    Sse(SseError),
    X87(X87Error),
}
impl fmt::Display for CellMovementScalingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded native Cell size scaling: {self:?}")
    }
}
impl std::error::Error for CellMovementScalingError {}
impl From<SseError> for CellMovementScalingError {
    fn from(value: SseError) -> Self {
        Self::Sse(value)
    }
}
impl From<X87Error> for CellMovementScalingError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}

/// Complete E4ECC0 within its explicit admitted progress range.
///
/// Exactly1000 uses the original independent special-case word10. For0..999,
/// the native loop advances to the first threshold strictly greater than the
/// raw signed progress and reads that row's selected word. Other raw values
/// are unsupported by this bounded API; they are never clamped or fabricated.
pub fn progress_scale_index(progress: i32) -> Result<usize, CellMovementScalingError> {
    if progress == 1000 {
        return Ok(10);
    }
    if !(0..1000).contains(&progress) {
        return Err(CellMovementScalingError::UnsupportedProgress(progress));
    }
    Ok(PROGRESS_ROWS
        .into_iter()
        .find(|(threshold, _)| progress < *threshold)
        .unwrap()
        .1)
}

/// Original finite retained x87 output, with an explicit scoped SSE policy.
///
/// Actor index-1 returns exact one before reading progress or scale values.
/// Otherwise only actor indices0..19 and progress0..1000 are admitted. DIVSS
/// commits a binary32 ratio before four inclusive comparisons. Even raw table
/// NaNs/infinities and finite0/0 or overflow ratios have literal native branch
/// outcomes: unordered comparisons reach one; negative infinity reaches four.
/// The table is read-only, and complete caller MXCSR/x87 environments survive.
pub fn size_movement_scale(
    actor_size_index: i32,
    progress: i32,
    scale_table: Option<&[f32; 20]>,
    context: CellMovementScalingContext,
) -> Result<Extended, CellMovementScalingError> {
    Ok(size_movement_scale_details(actor_size_index, progress, scale_table, context)?.retained)
}

/// Same expression with its independently observable native scalar inputs.
pub fn size_movement_scale_details(
    actor_size_index: i32,
    progress: i32,
    scale_table: Option<&[f32; 20]>,
    context: CellMovementScalingContext,
) -> Result<CellMovementScale, CellMovementScalingError> {
    if actor_size_index == -1 {
        return Ok(CellMovementScale {
            retained: Extended::from_f32(1.)?,
            selected_scale_index: None,
            ratio_bits: None,
        });
    }
    if !(0..20).contains(&actor_size_index) {
        return Err(CellMovementScalingError::UnsupportedActorSizeIndex(
            actor_size_index,
        ));
    }
    let denominator_index = progress_scale_index(progress)?;
    let scale_table = scale_table.ok_or(CellMovementScalingError::MissingScaleTable)?;
    native_sse::scoped(context.sse, || {
        let ratio = native_divide(
            scale_table[actor_size_index as usize],
            scale_table[denominator_index],
        );
        let one = Extended::from_f32(1.)?;
        let two = Extended::from_f32(2.)?;
        let four = Extended::from_f32(4.)?;
        let half = Extended::from_f32(0.5)?;
        let retained = if ratio <= 0.5 {
            four
        } else if ratio <= 1. {
            let delta = context.x87.sub(Extended::from_f32(ratio)?, half)?;
            // FMUL ST1,ST0 overwrites delta while preserving four for FSUBRP.
            let product = context.x87.mul(four, delta)?;
            context.x87.sub(four, product)?
        } else if ratio <= 2. {
            two
        } else if ratio <= 4. {
            let delta = context.x87.sub(Extended::from_f32(ratio)?, two)?;
            let product = context.x87.mul(delta, half)?;
            context.x87.sub(two, product)?
        } else {
            one
        };
        Ok(CellMovementScale {
            retained,
            selected_scale_index: Some(denominator_index),
            ratio_bits: Some(ratio.to_bits()),
        })
    })
}

// Native DIVSS operand order also determines observable NaN payload selection.
// Spell out that single operation rather than relying on Rust's unspecified
// arithmetic NaN payload propagation. The outer scope checked SSE2 beforehand.
fn native_divide(numerator: f32, denominator: f32) -> f32 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut quotient = numerator;
        // SAFETY: SSE2 was checked by native_sse::scoped, exceptions are masked,
        // both operands are values and the enclosing guard restores all MXCSR.
        unsafe {
            std::arch::asm!("divss {quotient}, {denominator}",
                quotient = inout(xmm_reg) quotient, denominator = in(xmm_reg) denominator,
                options(nostack));
        }
        quotient
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        // The scoped architecture check prevents this branch being reached.
        let _ = (numerator, denominator);
        unreachable!("native scaling requires the checked SSE2 scope")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn context() -> CellMovementScalingContext {
        CellMovementScalingContext {
            x87: Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            sse: Policy::Mxcsr1f80,
        }
    }

    #[test]
    fn exact_progress_rows_and_early_unused_inputs_preserve_boundaries() {
        for (progress, selected) in [
            (0, 1),
            (49, 1),
            (50, 1),
            (99, 1),
            (100, 2),
            (949, 10),
            (950, 10),
            (999, 10),
            (1000, 10),
        ] {
            assert_eq!(progress_scale_index(progress), Ok(selected));
        }
        assert_eq!(
            progress_scale_index(-1),
            Err(CellMovementScalingError::UnsupportedProgress(-1))
        );
        assert_eq!(
            progress_scale_index(1001),
            Err(CellMovementScalingError::UnsupportedProgress(1001))
        );
        assert_eq!(
            size_movement_scale(-1, i32::MIN, None, context()).unwrap(),
            Extended::from_f32(1.).unwrap()
        );
        assert_eq!(
            size_movement_scale(20, 0, None, context()),
            Err(CellMovementScalingError::UnsupportedActorSizeIndex(20))
        );
    }

    #[test]
    fn native_ratio_commits_then_literal_inclusive_or_unordered_branches() {
        let mut table = [1.; 20];
        for (ratio, expected) in [
            (0.5, 4.),
            (0.75, 3.),
            (1., 2.),
            (2., 2.),
            (3., 1.5),
            (4., 1.),
            (f32::NAN, 1.),
            (f32::NEG_INFINITY, 4.),
        ] {
            table[0] = ratio;
            let actual = size_movement_scale(0, 0, Some(&table), context()).unwrap();
            assert_eq!(actual, Extended::from_f32(expected).unwrap());
        }
        table[0] = 0.;
        table[1] = 0.;
        assert_eq!(
            size_movement_scale(0, 0, Some(&table), context()).unwrap(),
            Extended::from_f32(1.).unwrap()
        );
    }
}
