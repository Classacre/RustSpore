//! Exact canonical source extents and origin fields after the rest boundary.
//!
//! This is the scalar-SSE tail `9CC5A8..9CC800` in `9CBFA0`. It retains
//! source arithmetic order and clamp behavior. It does not construct anchors,
//! optional feet, finalized capabilities, selector classes or runtime poses.

use crate::anim_body_pose::RestBlock;
use crate::anim_body_source::{source_rest_block, source_rest_blocks, BodySourceError};
use crate::creature_data::{CreatureData, PART_ROW_SIZE};
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceBodyFields {
    pub rest: RestBlock,
    /// Canonical block+138/13C/140, after native nonpositive clamp.
    pub half_extents: [f32; 3],
    /// Block+144/148/14C, expanded in source order from the inverse quaternion.
    pub local_origin: [f32; 3],
    /// Three literal positive-zero fields at block+324/328/32C.
    pub field324: [f32; 3],
    /// Block+330/334/338, source row60/64/68 scaled and added to local origin.
    pub effects_origin: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceBodyFieldsError {
    Source(BodySourceError),
    NonFiniteExtraInput {
        palette_id: u32,
        row_offset: usize,
    },
    NonFiniteOutput {
        palette_id: u32,
        block_offset: usize,
    },
    Sse(SseError),
}
impl From<SseError> for SourceBodyFieldsError {
    fn from(error: SseError) -> Self {
        Self::Sse(error)
    }
}
impl From<BodySourceError> for SourceBodyFieldsError {
    fn from(error: BodySourceError) -> Self {
        Self::Source(error)
    }
}
impl fmt::Display for SourceBodyFieldsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "canonical source body fields: {self:?}")
    }
}
impl std::error::Error for SourceBodyFieldsError {}

// The caller has checked the complete row and rest block and installed1F80.
fn tail(row: &[u8], rest: RestBlock) -> Result<SourceBodyFields, SourceBodyFieldsError> {
    let word = |at: usize| f32::from_le_bytes(row[at..at + 4].try_into().unwrap());
    let mut extra = [0.0; 3];
    for (i, value) in extra.iter_mut().enumerate() {
        let row_offset = 0x60 + i * 4;
        *value = word(row_offset);
        if !value.is_finite() {
            return Err(SourceBodyFieldsError::NonFiniteExtraInput {
                palette_id: rest.palette_id,
                row_offset,
            });
        }
    }
    let scale = word(0x2C);
    // Same six committed source products at9CC295..9CC2E0 as the head.
    let min: [f32; 3] = std::array::from_fn(|i| word(0x14 + i * 4) * scale);
    let max: [f32; 3] = std::array::from_fn(|i| word(0x20 + i * 4) * scale);
    let [dx, dy, dz] = std::array::from_fn(|i| word(0x54 + i * 4) - rest.center[i]);
    let [x, y, z, w] = rest.orientation_xyzw;
    let nx = -(x * x);
    let ny = -(y * y);
    let nz = -(z * z);
    let nwx = -(x * w);
    let nwy = -(y * w);
    let nwz = -(z * w);
    let xy = y * x;
    let xz = z * x;
    let yz = z * y;
    // Literal119-instruction tail order, not a generic inverse-rotate call.
    let local_origin = [
        (((nz + ny) * dx + (xy - nwz) * dy) + (xz + nwy) * dz) * 2.0 + dx,
        (((xy + nwz) * dx + (nz + nx) * dy) + (yz - nwx) * dz) * 2.0 + dy,
        (((xz - nwy) * dx + (ny + nx) * dz) + (yz + nwx) * dy) * 2.0 + dz,
    ];
    let mut half_extents: [f32; 3] = std::array::from_fn(|i| (max[i] - min[i]) * 0.5);
    for value in &mut half_extents {
        // COMISS0,extent/JB preserves every positive subnormal. Zero and
        // negative values get the original readonly1489324 word3DCCCCCD.
        if *value <= 0.0 {
            *value = f32::from_bits(0x3DCC_CCCD);
        }
    }
    let effects_origin = [
        extra[0] * scale + local_origin[0],
        local_origin[1] + extra[1] * scale,
        local_origin[2] + extra[2] * scale,
    ];
    for (values, start) in [
        (&half_extents, 0x138),
        (&local_origin, 0x144),
        (&effects_origin, 0x330),
    ] {
        for (i, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(SourceBodyFieldsError::NonFiniteOutput {
                    palette_id: rest.palette_id,
                    block_offset: start + i * 4,
                });
            }
        }
    }
    Ok(SourceBodyFields {
        rest,
        half_extents,
        local_origin,
        field324: [0.0; 3],
        effects_origin,
    })
}

/// Recover one full raw source row's bounded rest/extents/origin fields.
///
/// The x87 context governs matrix-to-quaternion conversion in the existing
/// strict head. This operation scopes all scalar SSE to1F80 and restores the
/// caller's complete MXCSR on success/error. No matrix-quality check or
/// quaternion normalization is added. The finite extra-input/output checks
/// are explicit interface bounds rather than native error diagnostics.
pub fn source_body_fields(
    palette_id: u32,
    row: &[u8],
    context: Context,
) -> Result<SourceBodyFields, SourceBodyFieldsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        let rest = source_rest_block(palette_id, row, context)?;
        tail(row, rest)
    })
}

/// Recover every retained source row, including class-one rows, in source
/// palette order. Existing strict row-count/255-row bounds remain in force.
pub fn source_body_fields_for_creature(
    source: &CreatureData,
    context: Context,
) -> Result<Vec<SourceBodyFields>, SourceBodyFieldsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        let rest = source_rest_blocks(source, context)?;
        source
            .data0
            .chunks_exact(PART_ROW_SIZE)
            .zip(rest)
            .map(|(row, rest)| tail(row, rest))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn context() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }
    fn row(
        min: [f32; 3],
        max: [f32; 3],
        scale: f32,
        translation: [f32; 3],
        extra: [f32; 3],
    ) -> [u8; 140] {
        let mut result = [0u8; 140];
        for (at, values) in [
            (0x14, min.to_vec()),
            (0x20, max.to_vec()),
            (0x2C, vec![scale]),
            (0x30, vec![1., 0., 0., 0., 1., 0., 0., 0., 1.]),
            (0x54, translation.to_vec()),
            (0x60, extra.to_vec()),
        ] {
            for (i, value) in values.into_iter().enumerate() {
                result[at + i * 4..at + i * 4 + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
        result
    }

    #[test]
    fn source_identity_and_translation_preserve_extent_origin_and_effect_order() {
        let input = row([0., 0., 0.], [2., 4., 6.], 2., [3., 4., 5.], [0.5, 1., 1.5]);
        let value = source_body_fields(7, &input, context()).unwrap();
        assert_eq!(value.rest.palette_id, 7);
        assert_eq!(value.rest.center, [5., 8., 11.]);
        assert_eq!(value.half_extents, [2., 4., 6.]);
        assert_eq!(value.local_origin, [-2., -4., -6.]);
        assert_eq!(value.field324.map(f32::to_bits), [0; 3]);
        assert_eq!(value.effects_origin, [-1., -2., -3.]);
    }

    #[test]
    fn native_nonpositive_clamp_does_not_filter_positive_subnormal_extent() {
        let input = row(
            [0., 0., 0.],
            [-2., 0., f32::from_bits(8)],
            1.,
            [0.; 3],
            [0.; 3],
        );
        let value = source_body_fields(0, &input, context()).unwrap();
        assert_eq!(
            value.half_extents.map(f32::to_bits),
            [0x3DCC_CCCD, 0x3DCC_CCCD, 4]
        );
    }

    #[test]
    fn malformed_rows_and_nonfinite_new_parameters_have_explicit_errors() {
        let input = row([0.; 3], [1.; 3], 1., [0.; 3], [0.; 3]);
        assert!(matches!(
            source_body_fields(0, &input[..139], context()),
            Err(SourceBodyFieldsError::Source(
                BodySourceError::InvalidRowLength { .. }
            ))
        ));
        let input = row([0.; 3], [1.; 3], 1., [0.; 3], [0., f32::NAN, 0.]);
        assert_eq!(
            source_body_fields(3, &input, context()),
            Err(SourceBodyFieldsError::NonFiniteExtraInput {
                palette_id: 3,
                row_offset: 0x64
            })
        );
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn hostile_ftz_caller_retains_positive_subnormal_extent_and_restores_state() {
        fn current() -> u32 {
            let mut value = 0u32;
            // SAFETY: SSE2 target and bounded local storage.
            unsafe {
                std::arch::asm!("stmxcsr [{value}]",value=in(reg)&mut value,
                options(nostack,preserves_flags));
            }
            value
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                // SAFETY: exact same-thread saved state, also on assertion failure.
                unsafe {
                    std::arch::asm!("ldmxcsr [{value}]",value=in(reg)&self.0,
                    options(nostack,preserves_flags));
                }
            }
        }
        let _restore = Restore(current());
        let input =
            std::hint::black_box(row([0.; 3], [f32::from_bits(8); 3], 1., [0.; 3], [0.; 3]));
        let hostile = 0xDF81u32;
        // SAFETY: defined control/status bits, with no optional DAZ request.
        unsafe {
            std::arch::asm!("ldmxcsr [{value}]",value=in(reg)&hostile,
            options(nostack,preserves_flags));
        }
        let value = source_body_fields(0, &input, context()).unwrap();
        assert_eq!(value.half_extents.map(f32::to_bits), [4; 3]);
        assert_eq!(current(), hostile);
        assert!(source_body_fields(0, &input[..139], context()).is_err());
        assert_eq!(current(), hostile);
    }
}
