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
//! In the previously measured corpus, the data length correlates with the 8-byte
//! context at offsets 28..36. Let `c` be the count of `0x04` bytes in that context.
//! Then the measured size exponent is `L = c - 1` (and `L` is undefined when `c == 0`,
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
//! Native routines instead read leading nonzero context bytes as dimensions and
//! access three-byte samples in a last-axis-fastest table. The older parser below
//! retains its empirical grouping for compatibility; `NativeSamples` exposes the
//! bounded source layout. Neither establishes terrain or collision semantics.

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
    /// Empirical size exponent `L` (`None` when no context byte equals `0x04`).
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

/// Compute the measured size exponent from the 8-byte flag context.
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
/// A checked view of the packed samples used by native routines 414FC0/415440.
/// Dimensions end at the first zero. No gameplay or interpolation role is assumed.
#[derive(Debug)]
pub struct NativeSamples<'a> {
    data: &'a [u8],
    dimensions: &'a [u8],
    count: usize,
}

/// The checked view rejects unsupported stream sizes and non-exact payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSamplesError {
    TooShort,
    TooLarge,
    SizeOverflow,
    SizeMismatch,
    IndexOutOfBounds,
    CoordinateOutOfBounds,
}

impl<'a> NativeSamples<'a> {
    /// Validate the consumer's 36..100000 byte stream bound and exact table extent.
    /// Exact extent is a safety restriction, not a native validation claim.
    pub fn new(data: &'a [u8]) -> Result<Self, NativeSamplesError> {
        if data.len() < HEADER_SIZE { return Err(NativeSamplesError::TooShort); }
        if data.len() >= 100_000 { return Err(NativeSamplesError::TooLarge); }
        let context = &data[28..36];
        let dimension_count = context.iter().position(|&n| n == 0).unwrap_or(8);
        let dimensions = &context[..dimension_count];
        let count = if dimensions.is_empty() { 0 } else {
            dimensions.iter().try_fold(1usize, |n, &d| n.checked_mul(usize::from(d)))
                .ok_or(NativeSamplesError::SizeOverflow)?
        };
        let expected = count.checked_mul(3).and_then(|n| HEADER_SIZE.checked_add(n))
            .ok_or(NativeSamplesError::SizeOverflow)?;
        if data.len() != expected { return Err(NativeSamplesError::SizeMismatch); }
        Ok(Self { data, dimensions, count })
    }

    pub fn dimensions(&self) -> &[u8] { self.dimensions }
    pub fn sample_count(&self) -> usize { self.count }

    /// Checked mixed-radix index; the last dimension changes fastest.
    pub fn index(&self, coordinates: &[u8]) -> Result<usize, NativeSamplesError> {
        if coordinates.len() != self.dimensions.len() || self.dimensions.is_empty() {
            return Err(NativeSamplesError::CoordinateOutOfBounds);
        }
        let mut index = 0usize;
        for (&coordinate, &dimension) in coordinates.iter().zip(self.dimensions) {
            if coordinate >= dimension { return Err(NativeSamplesError::CoordinateOutOfBounds); }
            index = index * usize::from(dimension) + usize::from(coordinate);
        }
        Ok(index)
    }

    /// Expand one packed sample into two triples using the observed arithmetic.
    /// This is not interpolated output, and exceptional floats and alternate
    /// floating-point control states have not been compared with native execution.
    pub fn sample(&self, index: usize) -> Result<([f32; 3], [f32; 3]), NativeSamplesError> {
        if index >= self.count { return Err(NativeSamplesError::IndexOutOfBounds); }
        let mut first = [0.; 3];
        let mut second = [0.; 3];
        let value = |at: usize| f32::from_le_bytes(self.data[at..at + 4].try_into().unwrap());
        let scale = value(24);
        for axis in 0..3 {
            let packed = self.data[HEADER_SIZE + index * 3 + axis];
            first[axis] = value(axis * 4) - f32::from(packed >> 4) * scale;
            second[axis] = (f32::from(packed & 15) * scale) + value(12 + axis * 4);
        }
        Ok((first, second))
    }

    /// Header triples, including the zero-dimension native fallback.
    pub fn header_triples(&self) -> ([f32; 3], [f32; 3]) {
        let value = |at: usize| f32::from_le_bytes(self.data[at..at + 4].try_into().unwrap());
        ([value(0), value(4), value(8)], [value(12), value(16), value(20)])
    }
}

#[cfg(test)]
mod native_samples_tests {
    use super::*;

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0; 36 + 3 * 6];
        for (i, value) in [1.0f32, 2., 3., 4., 5., 6., 0.5].iter().enumerate() {
            bytes[i*4..i*4+4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[28..31].copy_from_slice(&[2, 3, 0]);
        bytes[36 + 5*3..36 + 6*3].copy_from_slice(&[0x12, 0xf0, 0x0f]);
        bytes
    }

    #[test]
    fn mixed_radix_and_nibble_expansion() {
        let data = fixture();
        let samples = NativeSamples::new(&data).unwrap();
        assert_eq!(samples.dimensions(), &[2, 3]);
        assert_eq!(samples.sample_count(), 6);
        assert_eq!(samples.index(&[1, 2]), Ok(5));
        assert_eq!(samples.sample(5), Ok(([0.5, -5.5, 3.], [5., 5., 13.5])));
        assert_eq!(samples.index(&[2, 0]), Err(NativeSamplesError::CoordinateOutOfBounds));
        assert_eq!(samples.sample(6), Err(NativeSamplesError::IndexOutOfBounds));
    }

    #[test]
    fn first_zero_terminates_dimensions_and_bare_fallback() {
        let mut data = vec![0; 36];
        data[29] = 4; // Native scan stops at byte 28; this suffix is ignored.
        let samples = NativeSamples::new(&data).unwrap();
        assert!(samples.dimensions().is_empty());
        assert_eq!(samples.sample_count(), 0);
        assert_eq!(samples.header_triples(), ([0.; 3], [0.; 3]));
        assert_eq!(samples.sample(0), Err(NativeSamplesError::IndexOutOfBounds));
    }

    #[test]
    fn malformed_lengths_and_limits_are_rejected() {
        let mut data = fixture();
        data.pop();
        assert!(matches!(NativeSamples::new(&data), Err(NativeSamplesError::SizeMismatch)));
        assert!(matches!(NativeSamples::new(&[0; 35]), Err(NativeSamplesError::TooShort)));
        assert!(matches!(NativeSamples::new(&vec![0; 100_000]), Err(NativeSamplesError::TooLarge)));
    }

    #[test]
    fn zero_dimension_header_preserves_all_float_bits() {
        let words = [0x8000_0000u32, 0x7f80_0000, 0xff80_0000,
                     0x7fc1_2345, 0x7f80_0001, 0x0000_0001];
        let mut data = [0u8; 36];
        for (index, bits) in words.iter().enumerate() {
            data[index * 4..index * 4 + 4].copy_from_slice(&bits.to_le_bytes());
        }
        data[29..36].copy_from_slice(&[255, 1, 2, 3, 4, 5, 6]);
        let samples = NativeSamples::new(&data).unwrap();
        let (first, second) = samples.header_triples();
        assert_eq!(first.map(f32::to_bits), [words[0], words[1], words[2]]);
        assert_eq!(second.map(f32::to_bits), [words[3], words[4], words[5]]);
        assert_eq!(samples.index(&[]), Err(NativeSamplesError::CoordinateOutOfBounds));
    }

    #[test]
    fn exact_length_and_dimension_products_are_checked() {
        let mut longer = fixture();
        longer.push(0);
        assert!(matches!(NativeSamples::new(&longer), Err(NativeSamplesError::SizeMismatch)));

        let mut too_many_samples = [0u8; 36];
        too_many_samples[28..36].copy_from_slice(&[255, 255, 1, 1, 1, 1, 1, 1]);
        assert!(matches!(NativeSamples::new(&too_many_samples), Err(NativeSamplesError::SizeMismatch)));

        let mut overflow = [0u8; 36];
        overflow[28..36].fill(255);
        assert!(matches!(NativeSamples::new(&overflow), Err(NativeSamplesError::SizeOverflow)));

        let mut eight_dimensions = [0u8; 39];
        eight_dimensions[28..36].fill(1);
        let samples = NativeSamples::new(&eight_dimensions).unwrap();
        assert_eq!(samples.dimensions(), &[1; 8]);
        assert_eq!(samples.sample_count(), 1);
        assert_eq!(samples.index(&[0; 8]), Ok(0));
        assert_eq!(samples.index(&[0; 9]), Err(NativeSamplesError::CoordinateOutOfBounds));
    }

    #[test]
    fn mixed_radix_unit_dimension_keeps_other_coordinates_distinct() {
        let mut data = vec![0u8; 36 + 3 * 6];
        data[28..32].copy_from_slice(&[2, 1, 3, 0]);
        let samples = NativeSamples::new(&data).unwrap();
        assert_eq!(samples.index(&[0, 0, 0]), Ok(0));
        assert_eq!(samples.index(&[0, 0, 2]), Ok(2));
        assert_eq!(samples.index(&[1, 0, 0]), Ok(3));
        assert_eq!(samples.index(&[1, 0, 2]), Ok(5));
        assert_eq!(samples.index(&[1, 1, 0]), Err(NativeSamplesError::CoordinateOutOfBounds));
        assert_eq!(samples.index(&[1, 0]), Err(NativeSamplesError::CoordinateOutOfBounds));
    }
}
