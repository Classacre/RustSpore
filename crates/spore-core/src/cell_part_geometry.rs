//! Original Cell part geometry from `E83E60`, with classification supplied.
//!
//! Creature loading, property-list inheritance and `E827A0` classification are
//! separate operations. This module consumes the original 140-byte source row
//! and preserves the scalar operation order and copied opaque source words.

use std::fmt;

pub const NATIVE_CELL_PART_LIMIT: usize = 64;

#[derive(Clone, Copy, Debug)]
pub struct NativeCellPart {
    /// Native byte zero, derived from source word `06 == FFFF`.
    pub word_06_is_ffff: bool,
    /// Literal result of the original ordered class predicate, `1..=12`.
    pub class: u32,
    pub center: [f32; 3],
    /// Copied source `30..53`; unused matrix components remain opaque bits.
    pub matrix_bits: [u32; 9],
    pub radius: f32,
    /// Copied source `60..6B`, without interpreting the values.
    pub tail_bits: [u32; 3],
}

impl NativeCellPart {
    /// Write all defined native output fields, retaining padding bytes `1..3`.
    pub fn write_native(&self, destination: &mut [u8; 72]) {
        destination[0] = u8::from(self.word_06_is_ffff);
        destination[4..8].copy_from_slice(&self.class.to_le_bytes());
        for (offset, value) in self.center.iter().enumerate() {
            destination[8 + offset * 4..12 + offset * 4]
                .copy_from_slice(&value.to_bits().to_le_bytes());
        }
        for (offset, bits) in self.matrix_bits.iter().enumerate() {
            destination[20 + offset * 4..24 + offset * 4].copy_from_slice(&bits.to_le_bytes());
        }
        destination[56..60].copy_from_slice(&self.radius.to_bits().to_le_bytes());
        for (offset, bits) in self.tail_bits.iter().enumerate() {
            destination[60 + offset * 4..64 + offset * 4].copy_from_slice(&bits.to_le_bytes());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellPartGeometryError {
    MissingClasses,
    InvalidClass,
    NonFiniteInput,
    NonFiniteIntermediate,
}

impl fmt::Display for CellPartGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell part geometry: {self:?}")
    }
}
impl std::error::Error for CellPartGeometryError {}

fn word(row: &[u8; 140], offset: usize) -> u32 {
    u32::from_le_bytes(row[offset..offset + 4].try_into().unwrap())
}
fn input(row: &[u8; 140], offset: usize) -> Result<f32, CellPartGeometryError> {
    let value = f32::from_bits(word(row, offset));
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CellPartGeometryError::NonFiniteInput)
    }
}
fn result(value: f32) -> Result<f32, CellPartGeometryError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CellPartGeometryError::NonFiniteIntermediate)
    }
}

/// Build one ordinary `E83E60` row from an explicitly resolved native class.
///
/// `creation_instance_low` is the source creation key's first word, used by the
/// literal `D9007204` radius exception. Source `E827A0` classification is not
/// repeated or replaced here. Scalar SSE arithmetic requires MXCSR control
/// `1F80`; the two x87 `FABS` load/store pairs are exact for finite binary32.
/// Only components consumed by arithmetic must be finite. Matrix and tail words
/// that native code only copies are retained even if they encode nonfinite data.
pub fn build_cell_part(
    row: &[u8; 140],
    class: u32,
    creation_instance_low: u32,
) -> Result<NativeCellPart, CellPartGeometryError> {
    if !(1..=12).contains(&class) {
        return Err(CellPartGeometryError::InvalidClass);
    }
    let scale = input(row, 0x2c)?;
    // MULSS bound,2.5 precedes MULSS scale,bound for every source bound.
    let mut bounds = [0.; 6];
    for (index, bound) in bounds.iter_mut().enumerate() {
        *bound = result(scale * result(input(row, 0x14 + index * 4)? * 2.5)?)?;
    }
    let [min_x, min_y, min_z, max_x, max_y, max_z] = bounds;
    let center_z = result(result(max_z + min_z)? * 0.5)?;
    let half_x = result(result(max_x - min_x)? * 0.5)?;
    let center_x = result(result(max_x + min_x)? * 0.5)?;
    let center_y = result(result(max_y + min_y)? * 0.5)?;
    let half_z = result(result(max_z - min_z)? * 0.5)?;

    let y_into_x = result(input(row, 0x3c)? * center_y)?;
    let z_into_x = result(input(row, 0x48)? * center_z)?;
    let zy_into_x = result(z_into_x + y_into_x)?;
    let x_into_x = result(input(row, 0x30)? * center_x)?;
    let sum_x = result(zy_into_x + x_into_x)?;
    let y_into_y = result(input(row, 0x40)? * center_y)?;
    let z_into_y = result(input(row, 0x4c)? * center_z)?;
    let zy_into_y = result(z_into_y + y_into_y)?;
    let x_into_y = result(input(row, 0x34)? * center_x)?;
    let sum_y = result(zy_into_y + x_into_y)?;
    let translated_x = result(input(row, 0x54)? * 2.5)?;
    let translated_y = result(input(row, 0x58)? * 2.5)?;
    let output_x = result(translated_x + sum_x)?;
    let output_y = result(translated_y + sum_y)?;

    let abs_z = half_z.abs();
    let abs_x = half_x.abs();
    let radius = if abs_z > abs_x { abs_z } else { abs_x };
    let factor = if class == 2 && creation_instance_low != 0xd900_7204 {
        0.5
    } else {
        1.
    };
    Ok(NativeCellPart {
        word_06_is_ffff: u16::from_le_bytes(row[6..8].try_into().unwrap()) == 0xffff,
        class,
        center: [output_x, output_y, 0.],
        matrix_bits: std::array::from_fn(|index| word(row, 0x30 + 4 * index)),
        radius: result(radius * factor)?,
        tail_bits: std::array::from_fn(|index| word(row, 0x60 + 4 * index)),
    })
}

/// Apply the native first-64-row limit before inspecting rows or their classes.
pub fn build_cell_parts(
    source_rows: &[[u8; 140]],
    classes: &[u32],
    creation_instance_low: u32,
) -> Result<Vec<NativeCellPart>, CellPartGeometryError> {
    let count = source_rows.len().min(NATIVE_CELL_PART_LIMIT);
    let classes = classes
        .get(..count)
        .ok_or(CellPartGeometryError::MissingClasses)?;
    source_rows[..count]
        .iter()
        .zip(classes)
        .map(|(row, &class)| build_cell_part(row, class, creation_instance_low))
        .collect()
}

/// Literal `476A98C7` special-resource branch, with the mutable source globals
/// supplied as raw words. No default identity/origin is invented for those words.
pub fn build_special_cell_part(origin_bits: [u32; 3], matrix_bits: [u32; 9]) -> NativeCellPart {
    NativeCellPart {
        word_06_is_ffff: true,
        class: 11,
        center: origin_bits.map(f32::from_bits),
        matrix_bits,
        radius: 1.,
        tail_bits: origin_bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_copies_padding_and_special_globals_preserve_exact_bits() {
        let mut row = [0; 140];
        row[44..48].copy_from_slice(&1f32.to_bits().to_le_bytes());
        row[56..60].copy_from_slice(&0x7fc0_1234u32.to_le_bytes());
        row[96..100].copy_from_slice(&0xff80_0000u32.to_le_bytes());
        let part = build_cell_part(&row, 1, 0).unwrap();
        let mut output = [0xa5; 72];
        part.write_native(&mut output);
        assert_eq!(&output[1..4], &[0xa5; 3]);
        assert_eq!(&output[28..32], &row[56..60]);
        assert_eq!(&output[60..64], &row[96..100]);
        assert_eq!(&output[16..20], &[0; 4]);
        let origin = [0x8000_0000, 0x7fc0_1234, 0xff80_0000];
        let special = build_special_cell_part(origin, [0x7fc0_9876; 9]);
        assert_eq!(special.center.map(f32::to_bits), origin);
        assert_eq!(special.tail_bits, origin);
    }

    #[test]
    fn native_limit_precedes_unused_rows_and_class_validation() {
        let mut rows = vec![[0; 140]; 65];
        rows[64] = [0xff; 140];
        let mut classes = vec![1; 65];
        classes[64] = 0;
        assert_eq!(build_cell_parts(&rows, &classes, 0).unwrap().len(), 64);
        assert!(matches!(
            build_cell_parts(&rows, &classes[..63], 0),
            Err(CellPartGeometryError::MissingClasses)
        ));
    }
}
