//! Exact source-row intake for the canonical CreatureData rest-matrix inputs.
//!
//! This is the bounded geometry region of `9CBFA0`, including the mixed
//! SSE/x87 matrix-to-quaternion helper `9A4DF0`. The caller supplies the x87
//! arithmetic context; the original process control word is not inferred.
//! This does not finalize a runtime hierarchy or evaluate animation channels.

use crate::anim_body_pose::{RestBlock, MAX_PALETTE_COUNT};
use crate::creature_data::{CreatureData, PART_ROW_SIZE};
use crate::native_x87::{Context, Extended, X87Error};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodySourceError {
    TooManyRows(usize),
    RowStorageMismatch { rows: usize, bytes: usize },
    InvalidRowLength { palette_id: u32, bytes: usize },
    NonFiniteInput { palette_id: u32, row_offset: usize },
    X87 { palette_id: u32, error: X87Error },
    NonFiniteOutput { palette_id: u32, component: usize },
}

impl fmt::Display for BodySourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyRows(rows) => write!(f, "{rows} source rows exceed {MAX_PALETTE_COUNT}"),
            Self::RowStorageMismatch { rows, bytes } => write!(
                f,
                "{rows} typed rows disagree with {bytes} raw source-row bytes"
            ),
            Self::InvalidRowLength { palette_id, bytes } => write!(
                f,
                "source row {palette_id} has {bytes} bytes; expected {PART_ROW_SIZE}"
            ),
            Self::NonFiniteInput {
                palette_id,
                row_offset,
            } => write!(
                f,
                "source row {palette_id} has nonfinite geometry at +{row_offset:X}"
            ),
            Self::X87 { palette_id, error } => write!(f, "source row {palette_id}: {error}"),
            Self::NonFiniteOutput {
                palette_id,
                component,
            } => write!(
                f,
                "source row {palette_id} produced nonfinite rest input {component}"
            ),
        }
    }
}

impl std::error::Error for BodySourceError {}

// 9A4DF0. Trace and quaternion numerators use SSE binary32. The square-root
// expression stays in x87 across FST, followed by a separate retained *0.5.
fn matrix_quaternion(matrix: [f32; 9], context: Context) -> Result<[f32; 4], X87Error> {
    let one = Extended::from_f32(1.0)?;
    let half = Extended::from_f32(0.5)?;
    let trace = (matrix[0] + matrix[4]) + matrix[8];
    if trace > 0.0 {
        let radicand = context.add(Extended::from_f32(trace)?, one)?;
        let root = context.sqrt(radicand)?;
        let committed_root = context.store_f32(root)?;
        let factor = 0.5 / committed_root;
        let w = context.store_f32(context.mul(root, half)?)?;
        Ok([
            (matrix[7] - matrix[5]) * factor,
            (matrix[2] - matrix[6]) * factor,
            (matrix[3] - matrix[1]) * factor,
            w,
        ])
    } else {
        let mut i = usize::from(matrix[4] > matrix[0]);
        if matrix[8] > matrix[i * 4] {
            i = 2;
        }
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        // FLD m[k,k]; FADD m[j,j]; FSUBR m[i,i]; FADD 1; FSQRT.
        let others = context.add(
            Extended::from_f32(matrix[k * 4])?,
            Extended::from_f32(matrix[j * 4])?,
        )?;
        let diagonal = context.sub(Extended::from_f32(matrix[i * 4])?, others)?;
        let radicand = context.add(diagonal, one)?;
        let root = context.sqrt(radicand)?;
        let committed_root = context.store_f32(root)?;
        let factor = 0.5 / committed_root;
        let mut result = [0.0; 4];
        result[i] = context.store_f32(context.mul(root, half)?)?;
        result[j] = (matrix[j * 3 + i] + matrix[i * 3 + j]) * factor;
        result[k] = (matrix[k * 3 + i] + matrix[i * 3 + k]) * factor;
        result[3] = (matrix[k * 3 + j] - matrix[j * 3 + k]) * factor;
        Ok(result)
    }
}

/// Recover one canonical loader row's rest center and xyzw orientation.
///
/// `palette_id` is its source array ordinal, not a runtime block index or an
/// embedded row-name field. The complete original 140-byte row is required.
/// Geometry reads raw source bytes: public edits to typed CreaturePart fields
/// do not alter this source-pinned intake. No rotation-quality tolerance or
/// normalization is applied; finite imperfect matrices retain native behavior.
pub fn source_rest_block(
    palette_id: u32,
    row: &[u8],
    context: Context,
) -> Result<RestBlock, BodySourceError> {
    if row.len() != PART_ROW_SIZE {
        return Err(BodySourceError::InvalidRowLength {
            palette_id,
            bytes: row.len(),
        });
    }
    let mut values = [0.0; 19];
    for (index, value) in values.iter_mut().enumerate() {
        let offset = 20 + index * 4;
        *value = f32::from_le_bytes(row[offset..offset + 4].try_into().unwrap());
        if !value.is_finite() {
            return Err(BodySourceError::NonFiniteInput {
                palette_id,
                row_offset: offset,
            });
        }
    }
    let scale = values[6];
    // 9CC295..9CC2E0 commits six products before the center additions.
    let min: [f32; 3] = std::array::from_fn(|i| values[i] * scale);
    let max: [f32; 3] = std::array::from_fn(|i| values[i + 3] * scale);
    let local_center: [f32; 3] = std::array::from_fn(|i| (max[i] + min[i]) * 0.5);
    // 9CC340..9CC35F transposes source row+30's stored matrix before conversion.
    let transposed = std::array::from_fn(|i| values[7 + (i % 3) * 3 + i / 3]);
    let orientation_xyzw = matrix_quaternion(transposed, context)
        .map_err(|error| BodySourceError::X87 { palette_id, error })?;
    let [x, y, z, w] = orientation_xyzw;
    let [cx, cy, cz] = local_center;
    let xxn = -(x * x);
    let yyn = -(y * y);
    let zzn = -(z * z);
    let xy = y * x;
    let xz = z * x;
    let yz = z * y;
    let wx = x * w;
    let wy = y * w;
    let wz = z * w;
    // Expanded source order at 9CC467..9CC5A0, not a pre-rounded matrix product.
    let rx = (((xz + wy) * cz + (xy - wz) * cy) + (zzn + yyn) * cx) * 2.0;
    let ry = (((zzn + xxn) * cy + (yz - wx) * cz) + (xy + wz) * cx) * 2.0;
    let rz = (((yyn + xxn) * cz + (yz + wx) * cy) + (xz - wy) * cx) * 2.0;
    let center = [
        (rx + cx) + values[16],
        (ry + cy) + values[17],
        (rz + cz) + values[18],
    ];
    if let Some(component) = center
        .iter()
        .chain(&orientation_xyzw)
        .position(|v| !v.is_finite())
    {
        return Err(BodySourceError::NonFiniteOutput {
            palette_id,
            component,
        });
    }
    Ok(RestBlock {
        palette_id,
        center,
        orientation_xyzw,
    })
}

/// Prepare every raw CreatureData row in retained source-palette order.
///
/// Class-one rows are included. This recovers the canonical rest inputs only;
/// root nomination, fake-foot insertion, rerooting, ordering, capability/channel
/// binding and evaluated runtime poses remain separate operations.
pub fn source_rest_blocks(
    source: &CreatureData,
    context: Context,
) -> Result<Vec<RestBlock>, BodySourceError> {
    let count = source.parts.len();
    if count > MAX_PALETTE_COUNT {
        return Err(BodySourceError::TooManyRows(count));
    }
    if source.data0.len() != count * PART_ROW_SIZE {
        return Err(BodySourceError::RowStorageMismatch {
            rows: count,
            bytes: source.data0.len(),
        });
    }
    source
        .data0
        .chunks_exact(PART_ROW_SIZE)
        .enumerate()
        .map(|(index, row)| source_rest_block(index as u32, row, context))
        .collect()
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

    fn row(rotation: [f32; 9]) -> [u8; PART_ROW_SIZE] {
        let mut row = [0; PART_ROW_SIZE];
        let values = [-1., -2., -3., 3., 4., 5., 2.];
        for (i, value) in values
            .into_iter()
            .chain(rotation)
            .chain([10., 20., 30.])
            .enumerate()
        {
            row[20 + i * 4..24 + i * 4].copy_from_slice(&value.to_le_bytes());
        }
        row
    }

    #[test]
    fn identity_uses_scaled_bounds_center_then_translation() {
        let source = row([1., 0., 0., 0., 1., 0., 0., 0., 1.]);
        let value = source_rest_block(7, &source, context()).unwrap();
        assert_eq!(value.palette_id, 7);
        assert_eq!(value.center, [12., 22., 32.]);
        assert_eq!(value.orientation_xyzw, [0., 0., 0., 1.]);
    }

    #[test]
    fn largest_diagonal_branches_preserve_native_axis_choice() {
        for (matrix, axis) in [
            ([1., 0., 0., 0., -1., 0., 0., 0., -1.], 0),
            ([-1., 0., 0., 0., 1., 0., 0., 0., -1.], 1),
            ([-1., 0., 0., 0., -1., 0., 0., 0., 1.], 2),
        ] {
            let value = source_rest_block(0, &row(matrix), context()).unwrap();
            let mut expected = [0.; 4];
            expected[axis] = 1.;
            assert_eq!(value.orientation_xyzw, expected);
        }
    }

    #[test]
    fn imperfect_matrix_is_used_without_quality_filter_or_normalization() {
        let value = source_rest_block(
            0,
            &row([1.01, 0., 0., 0., 1.01, 0., 0., 0., 1.01]),
            context(),
        )
        .unwrap();
        assert!(value.orientation_xyzw[3] > 1.);
        assert_eq!(&value.orientation_xyzw[..3], &[0.; 3]);
    }

    #[test]
    fn row_extent_and_nonfinite_geometry_are_typed_errors() {
        assert!(matches!(
            source_rest_block(0, &[0; 68], context()),
            Err(BodySourceError::InvalidRowLength { .. })
        ));
        let mut source = row([1., 0., 0., 0., 1., 0., 0., 0., 1.]);
        source[44..48].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            source_rest_block(3, &source, context()),
            Err(BodySourceError::NonFiniteInput {
                palette_id: 3,
                row_offset: 44
            })
        );
    }
}
