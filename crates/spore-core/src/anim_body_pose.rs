//! Native generated-body rest and pose matrices from explicit runtime inputs.
//!
//! `9C3150` stores rest matrices by block+21C before rerooting and ordering;
//! `9C6E40` later writes the live table by that same retained palette ID. These
//! operations do not construct a hierarchy, map animation channels, normalize
//! quaternions, or apply the model world's separate first-group count clamp.
//! See `re/anim-rig-native.md` for the constructor scope and instruction order.

use crate::anim_palette::BoneMatrix;
use std::fmt;

/// Blocks without a generated-body palette row, including optional fake feet.
pub const NO_PALETTE: u32 = u32::MAX;
/// The successful native body-finalization wrapper permits at most 255 rows.
pub const MAX_PALETTE_COUNT: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RestBlock {
    /// Explicit source palette ID retained in finalized block+21C.
    pub palette_id: u32,
    /// Native block+10C, before rerooting and runtime ordering.
    pub center: [f32; 3],
    /// Native block+118, in x/y/z/w order. Used without normalization.
    pub orientation_xyzw: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeBlockPose {
    /// The finalized definition block's retained +21C, not its runtime ordinal.
    pub palette_id: u32,
    /// Corresponding runtime block+10, before creature-scale removal.
    pub position: [f32; 3],
    /// Corresponding runtime block+1C, in x/y/z/w order.
    pub orientation_xyzw: [f32; 4],
}

/// The native intermediate stores nine basis floats, then three translations.
/// This differs from the output's three affine rows of four floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RestMatrix {
    pub basis: [f32; 9],
    pub translation: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestPalette {
    matrices: Vec<RestMatrix>,
}

impl RestPalette {
    /// Rest matrices in retained palette-ID order, independent of block order.
    pub fn matrices(&self) -> &[RestMatrix] {
        &self.matrices
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseStage {
    Rest,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyPoseError {
    PaletteCountOutOfRange(usize),
    PaletteIdOutOfRange {
        stage: PoseStage,
        block: usize,
        palette_id: u32,
        palette_count: usize,
    },
    DuplicatePaletteId {
        stage: PoseStage,
        block: usize,
        palette_id: u32,
    },
    MissingPaletteId {
        stage: PoseStage,
        palette_id: u32,
    },
    NonFiniteInput {
        stage: PoseStage,
        block: usize,
        field: &'static str,
        component: usize,
    },
    InvalidCreatureScale,
    NonFiniteResult {
        stage: PoseStage,
        palette_id: u32,
        component: usize,
    },
}

impl fmt::Display for BodyPoseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PaletteCountOutOfRange(count) => {
                write!(f, "body palette count {count} exceeds {MAX_PALETTE_COUNT}")
            }
            Self::PaletteIdOutOfRange {
                stage,
                block,
                palette_id,
                palette_count,
            } => write!(
                f,
                "{stage:?} block {block} has palette ID {palette_id}, outside {palette_count} rows"
            ),
            Self::DuplicatePaletteId {
                stage,
                block,
                palette_id,
            } => write!(f, "{stage:?} block {block} repeats palette ID {palette_id}"),
            Self::MissingPaletteId { stage, palette_id } => {
                write!(
                    f,
                    "{stage:?} input has no block for palette ID {palette_id}"
                )
            }
            Self::NonFiniteInput {
                stage,
                block,
                field,
                component,
            } => write!(
                f,
                "{stage:?} block {block} has nonfinite {field}[{component}]"
            ),
            Self::InvalidCreatureScale => {
                write!(
                    f,
                    "creature scale and its binary32 reciprocal must be finite and nonzero"
                )
            }
            Self::NonFiniteResult {
                stage,
                palette_id,
                component,
            } => write!(
                f,
                "{stage:?} palette ID {palette_id} produced nonfinite matrix component {component}"
            ),
        }
    }
}

impl std::error::Error for BodyPoseError {}

fn validate_components(
    values: &[f32],
    stage: PoseStage,
    block: usize,
    field: &'static str,
) -> Result<(), BodyPoseError> {
    if let Some(component) = values.iter().position(|value| !value.is_finite()) {
        return Err(BodyPoseError::NonFiniteInput {
            stage,
            block,
            field,
            component,
        });
    }
    Ok(())
}

fn checked_index<T>(
    entries: &[Option<T>],
    stage: PoseStage,
    block: usize,
    palette_id: u32,
) -> Result<Option<usize>, BodyPoseError> {
    if palette_id == NO_PALETTE {
        return Ok(None);
    }
    let index = palette_id as usize;
    if index >= entries.len() {
        return Err(BodyPoseError::PaletteIdOutOfRange {
            stage,
            block,
            palette_id,
            palette_count: entries.len(),
        });
    }
    if entries[index].is_some() {
        return Err(BodyPoseError::DuplicatePaletteId {
            stage,
            block,
            palette_id,
        });
    }
    Ok(Some(index))
}

fn complete<T>(entries: Vec<Option<T>>, stage: PoseStage) -> Result<Vec<T>, BodyPoseError> {
    entries
        .into_iter()
        .enumerate()
        .map(|(palette_id, value)| {
            value.ok_or(BodyPoseError::MissingPaletteId {
                stage,
                palette_id: palette_id as u32,
            })
        })
        .collect()
}

// Literal scalar-SSE binary32 order from 9A4580. Do not replace with a generic
// quaternion conversion, normalization, fused multiply-add, or reassociation.
fn quaternion_basis([x, y, z, w]: [f32; 4]) -> [f32; 9] {
    let yz = z * y;
    let zz = z * z;
    let xy = y * x;
    let xx = x * x;
    let xz = z * x;
    let wx = w * x;
    let yy = y * y;
    let wy = w * y;
    let wz = w * z;
    [
        1.0 - (zz + yy) * 2.0,
        (xy - wz) * 2.0,
        (wy + xz) * 2.0,
        (wz + xy) * 2.0,
        1.0 - (zz + xx) * 2.0,
        (yz - wx) * 2.0,
        (xz - wy) * 2.0,
        (wx + yz) * 2.0,
        1.0 - (yy + xx) * 2.0,
    ]
}

fn rest_matrix(block: &RestBlock) -> RestMatrix {
    let [x, y, z, w] = block.orientation_xyzw;
    let [px, py, pz] = block.center;
    let xxn = -(x * x);
    let yyn = -(y * y);
    let zzn = -(z * z);
    let xy = y * x;
    let xz = z * x;
    let yz = z * y;
    let xwn = -(x * w);
    let ywn = -(y * w);
    let zwn = -(z * w);

    // 9C347A..9C3562 expands the quaternion expression before adding the
    // unrotated center. Multiplication by a pre-rounded basis differs in f32.
    let tx = (((zzn + yyn) * px + (xz + ywn) * pz) + (xy - zwn) * py) * 2.0;
    let ty = (((xy + zwn) * px + (yz - xwn) * pz) + (zzn + xxn) * py) * 2.0;
    let tz = (((xz - ywn) * px + (yyn + xxn) * pz) + (yz + xwn) * py) * 2.0;
    RestMatrix {
        basis: quaternion_basis([-x, -y, -z, w]),
        translation: [-(tx + px), -(ty + py), -(tz + pz)],
    }
}

/// Prepare all retained source rows before rerooting, using native rest math.
///
/// `NO_PALETTE` blocks are skipped. Other IDs must cover `0..palette_count`
/// exactly once, independently of supplied block order. Native out-of-range IDs
/// are skipped and duplicate IDs overwrite storage; this bounded interface
/// rejects those ambiguous inputs instead of returning partially written rows.
/// Nonfinite inputs/results are likewise rejected. Finite nonunit quaternions
/// retain their literal native behavior without normalization.
pub fn prepare_rest_palette(
    palette_count: usize,
    blocks: &[RestBlock],
) -> Result<RestPalette, BodyPoseError> {
    if palette_count > MAX_PALETTE_COUNT {
        return Err(BodyPoseError::PaletteCountOutOfRange(palette_count));
    }
    let stage = PoseStage::Rest;
    let mut entries = vec![None; palette_count];
    for (block_index, block) in blocks.iter().enumerate() {
        let Some(index) = checked_index(&entries, stage, block_index, block.palette_id)? else {
            continue;
        };
        validate_components(&block.center, stage, block_index, "center")?;
        validate_components(
            &block.orientation_xyzw,
            stage,
            block_index,
            "orientation_xyzw",
        )?;
        let value = rest_matrix(block);
        let components: Vec<_> = value
            .basis
            .iter()
            .chain(&value.translation)
            .copied()
            .collect();
        if let Some(component) = components.iter().position(|value| !value.is_finite()) {
            return Err(BodyPoseError::NonFiniteResult {
                stage,
                palette_id: block.palette_id,
                component,
            });
        }
        entries[index] = Some(value);
    }
    Ok(RestPalette {
        matrices: complete(entries, stage)?,
    })
}

// 9C7AD0..9C7DF8, preserving its asymmetric accumulation and translation order.
fn compose_pose(rest: RestMatrix, basis: [f32; 9], translation: [f32; 3]) -> BoneMatrix {
    let a = rest.basis;
    let t = rest.translation;
    let p = basis;
    [
        (a[6] * p[2] + p[0] * a[0]) + p[1] * a[3],
        (a[7] * p[2] + p[0] * a[1]) + p[1] * a[4],
        (a[8] * p[2] + p[0] * a[2]) + p[1] * a[5],
        ((t[2] * p[2] + p[0] * t[0]) + p[1] * t[1]) + translation[0],
        (p[3] * a[0] + p[4] * a[3]) + p[5] * a[6],
        (p[3] * a[1] + p[4] * a[4]) + p[5] * a[7],
        (p[3] * a[2] + p[4] * a[5]) + p[5] * a[8],
        translation[1] + ((p[3] * t[0] + p[5] * t[2]) + p[4] * t[1]),
        (p[6] * a[0] + p[7] * a[3]) + p[8] * a[6],
        (p[6] * a[1] + p[7] * a[4]) + p[8] * a[7],
        (p[6] * a[2] + p[7] * a[5]) + p[8] * a[8],
        ((p[6] * t[0] + p[8] * t[2]) + p[7] * t[1]) + translation[2],
    ]
}

/// Feed explicit evaluated runtime poses to the generated body's matrix table.
///
/// Runtime array order may differ from source order: each pose uses the retained
/// finalized block+21C. Every real row must be present exactly once. Translation
/// uses one binary32 reciprocal (`9C722C`), then three multiplications. Negative
/// finite scale follows the measured arithmetic; zero, nonfinite, reciprocal
/// underflow/overflow and nonfinite output are rejected at this API boundary.
/// The model world's first-group count clamp belongs to `anim_palette`.
pub fn evaluate_body_palette(
    rest: &RestPalette,
    blocks: &[RuntimeBlockPose],
    creature_scale: f32,
) -> Result<Vec<BoneMatrix>, BodyPoseError> {
    if !creature_scale.is_finite() || creature_scale == 0.0 {
        return Err(BodyPoseError::InvalidCreatureScale);
    }
    let inverse_scale = 1.0 / creature_scale;
    if !inverse_scale.is_finite() || inverse_scale == 0.0 {
        return Err(BodyPoseError::InvalidCreatureScale);
    }
    let stage = PoseStage::Runtime;
    let mut entries = vec![None; rest.matrices.len()];
    for (block_index, block) in blocks.iter().enumerate() {
        let Some(index) = checked_index(&entries, stage, block_index, block.palette_id)? else {
            continue;
        };
        validate_components(&block.position, stage, block_index, "position")?;
        validate_components(
            &block.orientation_xyzw,
            stage,
            block_index,
            "orientation_xyzw",
        )?;
        let translation = block.position.map(|value| value * inverse_scale);
        let matrix = compose_pose(
            rest.matrices[index],
            quaternion_basis(block.orientation_xyzw),
            translation,
        );
        if let Some(component) = matrix.iter().position(|value| !value.is_finite()) {
            return Err(BodyPoseError::NonFiniteResult {
                stage,
                palette_id: block.palette_id,
                component,
            });
        }
        entries[index] = Some(matrix);
    }
    complete(entries, stage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim_palette::IDENTITY;

    fn rest(id: u32, center: [f32; 3]) -> RestBlock {
        RestBlock {
            palette_id: id,
            center,
            orientation_xyzw: [0., 0., 0., 1.],
        }
    }

    fn pose(id: u32, position: [f32; 3]) -> RuntimeBlockPose {
        RuntimeBlockPose {
            palette_id: id,
            position,
            orientation_xyzw: [0., 0., 0., 1.],
        }
    }

    #[test]
    fn reordered_ids_keep_rest_and_live_rows_together() {
        let rest =
            prepare_rest_palette(2, &[rest(1, [4., 5., 6.]), rest(0, [1., 2., 3.])]).unwrap();
        let output =
            evaluate_body_palette(&rest, &[pose(1, [8., 10., 12.]), pose(0, [2., 4., 6.])], 2.)
                .unwrap();
        assert_eq!(output, vec![IDENTITY; 2]);
    }

    #[test]
    fn sentinel_rows_are_skipped_without_reading_pose_fields() {
        let mut skipped = rest(NO_PALETTE, [f32::NAN; 3]);
        skipped.orientation_xyzw = [f32::INFINITY; 4];
        let rest = prepare_rest_palette(1, &[skipped, rest(0, [0.; 3])]).unwrap();
        let mut skipped = pose(NO_PALETTE, [f32::NAN; 3]);
        skipped.orientation_xyzw = [f32::INFINITY; 4];
        assert_eq!(
            evaluate_body_palette(&rest, &[skipped, pose(0, [0.; 3])], 1.).unwrap(),
            vec![IDENTITY]
        );
    }

    #[test]
    fn complete_coverage_is_required_at_both_stages() {
        assert!(matches!(
            prepare_rest_palette(2, &[rest(0, [0.; 3])]),
            Err(BodyPoseError::MissingPaletteId { palette_id: 1, .. })
        ));
        assert!(matches!(
            prepare_rest_palette(1, &[rest(0, [0.; 3]), rest(0, [0.; 3])]),
            Err(BodyPoseError::DuplicatePaletteId { .. })
        ));
        assert!(matches!(
            prepare_rest_palette(1, &[rest(1, [0.; 3])]),
            Err(BodyPoseError::PaletteIdOutOfRange { .. })
        ));
        let rest = prepare_rest_palette(1, &[rest(0, [0.; 3])]).unwrap();
        assert!(matches!(
            evaluate_body_palette(&rest, &[], 1.),
            Err(BodyPoseError::MissingPaletteId { .. })
        ));
        assert!(matches!(
            evaluate_body_palette(&rest, &[pose(0, [0.; 3]), pose(0, [0.; 3])], 1.),
            Err(BodyPoseError::DuplicatePaletteId { .. })
        ));
        assert!(matches!(
            evaluate_body_palette(&rest, &[pose(1, [0.; 3])], 1.),
            Err(BodyPoseError::PaletteIdOutOfRange { .. })
        ));
    }

    #[test]
    fn invalid_numeric_context_is_reported() {
        assert!(matches!(
            prepare_rest_palette(256, &[]),
            Err(BodyPoseError::PaletteCountOutOfRange(256))
        ));
        assert!(matches!(
            prepare_rest_palette(1, &[rest(0, [f32::NAN; 3])]),
            Err(BodyPoseError::NonFiniteInput { .. })
        ));
        let rest = prepare_rest_palette(1, &[rest(0, [0.; 3])]).unwrap();
        for scale in [0., -0., f32::NAN, f32::INFINITY, f32::from_bits(1)] {
            assert_eq!(
                evaluate_body_palette(&rest, &[pose(0, [0.; 3])], scale),
                Err(BodyPoseError::InvalidCreatureScale)
            );
        }
        assert!(matches!(
            evaluate_body_palette(&rest, &[pose(0, [f32::INFINITY; 3])], 1.),
            Err(BodyPoseError::NonFiniteInput { .. })
        ));
        let mut large = pose(0, [0.; 3]);
        large.orientation_xyzw = [f32::MAX; 4];
        assert!(matches!(
            evaluate_body_palette(&rest, &[large], 1.),
            Err(BodyPoseError::NonFiniteResult { .. })
        ));
    }

    #[test]
    fn finite_nonunit_quaternion_is_not_normalized() {
        let rest = prepare_rest_palette(1, &[rest(0, [0.; 3])]).unwrap();
        let mut live = pose(0, [3., 4., 5.]);
        live.orientation_xyzw = [0., 0., 1., 1.];
        assert_eq!(
            evaluate_body_palette(&rest, &[live], 1.).unwrap()[0],
            [-1., -2., 0., 3., 2., -1., 0., 4., 0., 0., 1., 5.]
        );
    }

    #[test]
    fn empty_palette_and_negative_scale_are_explicit() {
        let empty = prepare_rest_palette(0, &[]).unwrap();
        assert!(evaluate_body_palette(&empty, &[], 1.).unwrap().is_empty());
        let rest = prepare_rest_palette(1, &[rest(0, [0.; 3])]).unwrap();
        let output = evaluate_body_palette(&rest, &[pose(0, [2., 4., 6.])], -2.).unwrap();
        assert_eq!([output[0][3], output[0][7], output[0][11]], [-1., -2., -3.]);
    }
}
