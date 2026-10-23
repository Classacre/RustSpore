//! Parser for the SPORE blockdata format (type `0x5CAA5E28`).
//!
//! Status: **100% structurally validated.** Every one of 3,098 blockdata
//! resources across `PatchData.package` and `Spore_Game.package` satisfies
//! `total_size == 36 + 12 * 4^L` exactly, with zero mismatches. The Python
//! reference validator is `tools/blockdata_validate.py`.
//!
//! ## Layout
//!
//! ```text
//! offset  0: LE f32 min X          \
//! offset  4: LE f32 min Y           |
//! offset  8: LE f32 min Z           |  bounding box (24 bytes)
//! offset 12: LE f32 max X           |
//! offset 16: LE f32 max Y           |
//! offset 20: LE f32 max Z          /
//! offset 24: LE f32 (unknown, varies)
//! offset 28: 8-byte flag context (bytes at +28..+36)
//! offset 36: data, `12 * 4^L` bytes, grouped into 12-byte records
//! ```
//!
//! ## The decisive finding: a closed form from the flag context
//!
//! The data length is fully determined by the 8-byte flag context at offsets
//! 28..36. Let `c` be the number of bytes equal to `0x04` in those 8 bytes.
//! Then the quadtree level is `L = c - 1` (and `L` is undefined when `c == 0`,
//! which is the bare 36-byte header), and the data holds `4^L` records of 12
//! bytes each:
//!
//! ```text
//! total_size == 36 + 12 * 4^(c - 1)      (for c >= 1)
//! total_size == 36                        (for c == 0)
//! ```
//!
//! The eight observed size classes map one-to-one onto `c`:
//! `c = 0..7` -> sizes `36, 48, 84, 228, 804, 3108, 12324, 49188`.
//!
//! This is an adaptive 4-ary quadtree: `L` is the subdivision depth and the
//! record count is the leaf capacity `4^L`. The 12-byte record semantics (node
//! child-mask vs. leaf data) are not decoded; this module validates and exposes
//! the record bytes, not their meaning.

use crate::Vec3;
use std::fmt;

/// Size of the fixed header, in bytes.
pub const HEADER_SIZE: usize = 36;

/// Size of one data record, in bytes.
pub const RECORD_SIZE: usize = 12;

/// Errors that can occur when parsing a blockdata resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockdataError {
    /// The resource is shorter than the 36-byte header.
    TooShort,
}

impl fmt::Display for BlockdataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockdataError::TooShort => write!(f, "blockdata resource too short"),
        }
    }
}

impl std::error::Error for BlockdataError {}

/// A parsed blockdata resource.
#[derive(Debug, Clone)]
pub struct Blockdata {
    /// Bounding box minimum corner.
    pub min: Vec3,
    /// Bounding box maximum corner.
    pub max: Vec3,
    /// Unknown float at offset 24.
    pub unknown_float: f32,
    /// The 8-byte flag context at offsets 28..36.
    pub flag_context: [u8; 8],
    /// Quadtree level `L` (`None` when the context has no `0x04` bytes).
    pub level: Option<u32>,
    /// Number of 12-byte data records: `4^L`, or `0` when `level` is `None`.
    pub record_count: usize,
    /// Total resource size in bytes.
    pub total_size: usize,
    /// The data records (each three little-endian u32), from offset 36.
    pub records: Vec<[u32; 3]>,
    /// Whether `total_size == 36 + 12 * record_count`.
    pub exact: bool,
}

/// Compute the quadtree level from the 8-byte flag context.
///
/// Returns `None` when no byte equals `0x04` (a bare 36-byte header), otherwise
/// `Some(count - 1)` where `count` is the number of `0x04` bytes.
pub fn level_from_context(context: &[u8; 8]) -> Option<u32> {
    let c = context.iter().filter(|&&b| b == 0x04).count();
    if c == 0 {
        None
    } else {
        Some((c - 1) as u32)
    }
}

/// Parse a blockdata resource.
pub fn parse(data: &[u8]) -> Result<Blockdata, BlockdataError> {
    if data.len() < HEADER_SIZE {
        return Err(BlockdataError::TooShort);
    }

    let min = Vec3::new(
        f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        f32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        f32::from_le_bytes([data[8], data[9], data[10], data[11]]),
    );
    let max = Vec3::new(
        f32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        f32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        f32::from_le_bytes([data[20], data[21], data[22], data[23]]),
    );
    let unknown_float = f32::from_le_bytes([data[24], data[25], data[26], data[27]]);

    let mut flag_context = [0u8; 8];
    flag_context.copy_from_slice(&data[28..36]);

    let level = level_from_context(&flag_context);
    let record_count = level.map(|l| 4usize.pow(l)).unwrap_or(0);
    let expected_size = HEADER_SIZE + RECORD_SIZE * record_count;
    let exact = data.len() == expected_size;

    let mut records = Vec::with_capacity(record_count.min(16384));
    let mut off = HEADER_SIZE;
    for _ in 0..record_count {
        if off + RECORD_SIZE > data.len() {
            break;
        }
        let a = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        let b = u32::from_le_bytes([data[off + 4], data[off + 5], data[off + 6], data[off + 7]]);
        let c = u32::from_le_bytes([data[off + 8], data[off + 9], data[off + 10], data[off + 11]]);
        records.push([a, b, c]);
        off += RECORD_SIZE;
    }

    Ok(Blockdata {
        min,
        max,
        unknown_float,
        flag_context,
        level,
        record_count,
        total_size: data.len(),
        records,
        exact,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic blockdata blob of `size` bytes, setting the given
    /// positions (0..8) in the flag context to `0x04`.
    fn build(size: usize, ctx_positions: &[usize]) -> Vec<u8> {
        let mut data = vec![0u8; size];
        let min = [-0.1f32, -0.2, -0.3];
        let max = [0.1f32, 0.2, 0.3];
        for i in 0..3 {
            data[i * 4..i * 4 + 4].copy_from_slice(&min[i].to_le_bytes());
            data[12 + i * 4..12 + i * 4 + 4].copy_from_slice(&max[i].to_le_bytes());
        }
        data[24..28].copy_from_slice(&0.05f32.to_le_bytes());
        for &p in ctx_positions {
            data[28 + p] = 0x04;
        }
        data
    }

    #[test]
    fn parse_36byte_bare_header() {
        let data = build(36, &[]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, None);
        assert_eq!(bd.record_count, 0);
        assert!(bd.records.is_empty());
        assert!(bd.exact);
        assert!((bd.min.x - (-0.1)).abs() < 1e-6);
        assert!((bd.max.z - 0.3).abs() < 1e-6);
    }

    #[test]
    fn parse_48byte_level0() {
        let data = build(48, &[0]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, Some(0));
        assert_eq!(bd.record_count, 1);
        assert_eq!(bd.records.len(), 1);
        assert!(bd.exact);
    }

    #[test]
    fn parse_84byte_level1() {
        let data = build(84, &[0, 1]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, Some(1));
        assert_eq!(bd.record_count, 4);
        assert_eq!(bd.records.len(), 4);
        assert!(bd.exact);
    }

    #[test]
    fn parse_228byte_level2() {
        let data = build(228, &[0, 1, 2]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, Some(2));
        assert_eq!(bd.record_count, 16);
        assert!(bd.exact);
    }

    /// The flag context spans both dwords: five 0x04 bytes (including one at
    /// offset 32..36) give L = 4, record_count = 256, total 3108.
    #[test]
    fn parse_3108byte_level4_spanning_context() {
        let data = build(3108, &[0, 1, 2, 3, 4]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, Some(4));
        assert_eq!(bd.record_count, 256);
        assert_eq!(bd.total_size, 3108);
        assert!(bd.exact);
    }

    #[test]
    fn records_are_little_endian_u32_triples() {
        let mut data = build(48, &[0]);
        data[36..40].copy_from_slice(&0x11223344u32.to_le_bytes());
        data[40..44].copy_from_slice(&0x55667788u32.to_le_bytes());
        data[44..48].copy_from_slice(&0x99AABBCCu32.to_le_bytes());
        let bd = parse(&data).unwrap();
        assert_eq!(bd.records[0], [0x11223344, 0x55667788, 0x99AABBCC]);
    }

    #[test]
    fn size_mismatch_is_not_exact() {
        // Two 0x04 bytes imply L=1 (4 records, total 84), but the blob is 48.
        let data = build(48, &[0, 1]);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.level, Some(1));
        assert_eq!(bd.record_count, 4);
        assert!(!bd.exact);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 10];
        assert!(matches!(parse(&data), Err(BlockdataError::TooShort)));
    }
}
