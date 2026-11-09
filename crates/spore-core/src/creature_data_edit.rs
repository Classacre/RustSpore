//! Source-preserving edits to a CreatureData part's stored rest transform.
//!
//! The source rows contain absolute rest transforms, not parent-relative ones.
//! This operation only patches the 12 existing float words in one row; it does
//! not assemble geometry, update a baked GMDL, create a shareable creation, or
//! establish that the edited data will work in the original game. Keep source
//! assets immutable and write any returned bytes to a separate destination.

use crate::creature_data::{self, CreatureDataError, PART_ROW_SIZE};
use std::fmt;

const ROTATION_OFFSET: usize = 48;
const TRANSLATION_OFFSET: usize = 84;
const TRANSFORM_END: usize = 96;

/// Values are stored as the source row's 3x3 row-major rotation and absolute
/// translation. No hierarchy composition or normalization is applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestTransform {
    pub rotation: [[f32; 3]; 3],
    pub translation: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    InvalidSource(CreatureDataError),
    TrailingBytes,
    PartOutOfRange { index: usize, count: usize },
    NonFiniteTransform,
    InvalidRowExtent,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSource(error) => write!(f, "invalid CreatureData: {error}"),
            Self::TrailingBytes => write!(f, "CreatureData has unconsumed trailing bytes"),
            Self::PartOutOfRange { index, count } => {
                write!(f, "part index {index} is outside {count} source rows")
            }
            Self::NonFiniteTransform => write!(f, "rest transform contains a nonfinite value"),
            Self::InvalidRowExtent => write!(f, "CreatureData row extent is inconsistent"),
        }
    }
}

impl std::error::Error for EditError {}

/// Return a modified copy of a complete version-9/10 CreatureData payload.
///
/// The ordinal selects the serialized row; the row's stored `index` field is
/// not substituted for it. The input is parsed and its part ranges validated
/// before any output is created. All bytes outside the selected row's rotation
/// and translation are retained exactly, including unknown fields, the original
/// compressed header and version-10 data. Values must be finite, but this does
/// not assert that the rotation is physically valid or that the game's editor
/// would permit the resulting placement.
pub fn patch_rest_transform(
    source: &[u8],
    ordinal: usize,
    transform: RestTransform,
) -> Result<Vec<u8>, EditError> {
    let parsed = creature_data::parse(source).map_err(EditError::InvalidSource)?;
    if !parsed.exact || parsed.consumed != source.len() {
        return Err(EditError::TrailingBytes);
    }
    if ordinal >= parsed.parts.len() {
        return Err(EditError::PartOutOfRange { index: ordinal, count: parsed.parts.len() });
    }
    if transform.rotation.iter().flatten().chain(transform.translation.iter()).any(|v| !v.is_finite()) {
        return Err(EditError::NonFiniteTransform);
    }

    // Header + version + three/four array counts precede the original RLE
    // chunk buffer. Preserve that exact compressed representation.
    let counts_bytes = if parsed.version == 9 { 12usize } else { 16usize };
    let data_start = 8usize.checked_add(counts_bytes)
        .and_then(|n| n.checked_add(parsed.compressed_buffer.len()))
        .ok_or(EditError::InvalidRowExtent)?;
    let row_start = ordinal.checked_mul(PART_ROW_SIZE)
        .and_then(|n| data_start.checked_add(n))
        .ok_or(EditError::InvalidRowExtent)?;
    let row_end = row_start.checked_add(PART_ROW_SIZE).ok_or(EditError::InvalidRowExtent)?;
    let all_rows_end = data_start.checked_add(parsed.data0.len()).ok_or(EditError::InvalidRowExtent)?;
    if row_end > all_rows_end || source.get(data_start..all_rows_end) != Some(parsed.data0.as_slice()) {
        return Err(EditError::InvalidRowExtent);
    }
    let mut output = source.to_vec();
    let row = output.get_mut(row_start..row_end).ok_or(EditError::InvalidRowExtent)?;
    for (index, value) in transform.rotation.iter().flatten().enumerate() {
        let start = ROTATION_OFFSET + index * 4;
        row[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, value) in transform.translation.iter().enumerate() {
        let start = TRANSLATION_OFFSET + index * 4;
        row[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }
    debug_assert_eq!(TRANSLATION_OFFSET + 3 * 4, TRANSFORM_END);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(version: i32) -> (Vec<u8>, usize) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&creature_data::HEADER.to_le_bytes());
        bytes.extend_from_slice(&version.to_le_bytes());
        for count in [2u32, 0, 0] { bytes.extend_from_slice(&count.to_le_bytes()); }
        if version == 10 { bytes.extend_from_slice(&1u32.to_le_bytes()); }
        // One zero followed by 127 repeats: a complete 128-byte header.
        // Version 10 uses two chunks to exercise the preceding-byte RLE state
        // and row-offset calculation across a chunk boundary.
        if version == 9 {
            bytes.extend_from_slice(&3u16.to_le_bytes());
            bytes.extend_from_slice(&[0, 0xfe, 127]);
        } else {
            bytes.extend_from_slice(&1u16.to_le_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&2u16.to_le_bytes());
            bytes.extend_from_slice(&[0xfe, 127]);
        }
        let data_start = bytes.len();
        for index in 0..2u16 {
            let mut row = [0u8; PART_ROW_SIZE];
            row[0..2].copy_from_slice(&index.to_le_bytes());
            row[4..6].copy_from_slice(&u16::MAX.to_le_bytes());
            row[6..8].copy_from_slice(&u16::MAX.to_le_bytes());
            // Unknown bytes exercise preservation around the edited range.
            row[20..24].copy_from_slice(&[0x91, 0x82, 0x73, 0x64]);
            row[128..132].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
            bytes.extend_from_slice(&row);
        }
        bytes.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        if version == 10 { bytes.extend_from_slice(&[0xa5; 0x38]); }
        (bytes, data_start)
    }

    fn transform() -> RestTransform {
        RestTransform { rotation: [[1.0, -0.0, 0.25], [0.0, 1.0, 0.5], [0.0, 0.0, 1.0]],
            translation: [3.5, -2.0, 0.125] }
    }

    #[test]
    fn patches_only_selected_source_words_in_both_versions() {
        for version in [9, 10] {
            let (source, start) = source(version);
            let edited = patch_rest_transform(&source, 1, transform()).unwrap();
            let first = start + PART_ROW_SIZE + ROTATION_OFFSET;
            let last = start + PART_ROW_SIZE + TRANSFORM_END;
            assert_eq!(&edited[..first], &source[..first]);
            assert_eq!(&edited[last..], &source[last..]);
            let parsed = creature_data::parse(&edited).unwrap();
            assert!(parsed.exact);
            assert_eq!(parsed.parts[1].rotation, transform().rotation);
            assert_eq!(parsed.parts[1].translation, transform().translation);
            assert_eq!(parsed.parts[1].rotation[0][1].to_bits(), (-0.0f32).to_bits());
            assert_eq!(parsed.parts[0].rotation, [[0.0; 3]; 3]);
        }
    }

    #[test]
    fn unchanged_transform_is_byte_exact() {
        let (source, _) = source(9);
        let parsed = creature_data::parse(&source).unwrap();
        let row = parsed.parts[0];
        let edited = patch_rest_transform(&source, 0, RestTransform {
            rotation: row.rotation, translation: row.translation,
        }).unwrap();
        assert_eq!(edited, source);
    }

    #[test]
    fn rejects_invalid_input_and_nonfinite_values() {
        let (source, _) = source(10);
        assert_eq!(patch_rest_transform(&source, 2, transform()),
            Err(EditError::PartOutOfRange { index: 2, count: 2 }));
        let mut invalid = transform(); invalid.translation[1] = f32::NAN;
        assert_eq!(patch_rest_transform(&source, 0, invalid), Err(EditError::NonFiniteTransform));
        let mut trailing = source.clone(); trailing.push(0x7f);
        assert_eq!(patch_rest_transform(&trailing, 0, transform()), Err(EditError::TrailingBytes));
        assert!(matches!(patch_rest_transform(&source[..source.len()-1], 0, transform()), Err(EditError::InvalidSource(_))));
    }

    #[test]
    #[ignore = "requires an explicitly supplied local CreatureData payload"]
    fn retained_source_with_owned_capabilities_and_morphs_preserves_other_bytes() {
        let path = std::env::var_os("RUSTSPORE_CREATUREDATA_FIXTURE")
            .expect("set RUSTSPORE_CREATUREDATA_FIXTURE to a local payload");
        let source = std::fs::read(path).unwrap();
        let parsed = creature_data::parse(&source).unwrap();
        assert!(parsed.exact);
        assert_eq!((parsed.version, parsed.parts.len(), parsed.capabilities.len(), parsed.morphs.len()),
            (9, 10, 15, 24));
        let original = parsed.parts[9];
        let same = RestTransform { rotation: original.rotation, translation: original.translation };
        assert_eq!(patch_rest_transform(&source, 9, same).unwrap(), source);
        let mut changed = same;
        changed.translation[0] = 123.125;
        let edited = patch_rest_transform(&source, 9, changed).unwrap();
        let data_start = 8 + 12 + parsed.compressed_buffer.len();
        let begin = data_start + 9 * PART_ROW_SIZE + ROTATION_OFFSET;
        let end = data_start + 9 * PART_ROW_SIZE + TRANSFORM_END;
        assert_eq!(&edited[..begin], &source[..begin]);
        assert_eq!(&edited[end..], &source[end..]);
        let checked = creature_data::parse(&edited).unwrap();
        assert_eq!(checked.parts[9].translation[0], 123.125);
        assert_eq!(checked.capabilities.len(), parsed.capabilities.len());
        assert_eq!(checked.morphs.len(), parsed.morphs.len());
    }
}
