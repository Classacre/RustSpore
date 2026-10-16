//! Parser for the SPORE blockdata format (type `0x5CAA5E28`).
//!
//! Status: header partially decoded. Bounding box, flags, and size-based
//! structure identified. Full variable data structure not yet decoded.
//!
//! Layout:
//! ```text
//! offset  0: LE float: min X
//! offset  4: LE float: min Y
//! offset  8: LE float: min Z
//! offset 12: LE float: max X
//! offset 16: LE float: max Y
//! offset 20: LE float: max Z
//! offset 24: LE float (unknown, varies)
//! offset 28: 4 flag bytes (0x04 = true, 0x00 = false)
//! offset 32: variable data (size-dependent)
//! ```
//!
//! Size-based structure after flags:
//! - 36-byte: 3 u32 zeros (padding only)
//! - 48-byte: 4 zeros + 4-byte value + 8-byte hash
//! - 84-byte: 4 zeros + 48 bytes of structured data (4 groups of 12 bytes)

use crate::Vec3;
use std::fmt;

/// Errors that can occur when parsing a blockdata resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockdataError {
    /// The resource is too short for the bounding box.
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

/// A parsed blockdata resource (header only; variable data not fully decoded).
#[derive(Debug, Clone)]
pub struct Blockdata {
    /// Bounding box minimum corner.
    pub min: Vec3,
    /// Bounding box maximum corner.
    pub max: Vec3,
    /// Unknown float at offset 24.
    pub unknown_float: f32,
    /// Flag bytes at offset 28 (0x04 = true, 0x00 = false).
    pub flags: [u8; 4],
    /// Total resource size.
    pub total_size: usize,
    /// Variable data after the flags (raw bytes).
    pub extra_data: Vec<u8>,
}

/// Parse a blockdata resource.
pub fn parse(data: &[u8]) -> Result<Blockdata, BlockdataError> {
    if data.len() < 32 {
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
    let flags = [data[28], data[29], data[30], data[31]];

    let extra_data = if data.len() > 32 {
        data[32..].to_vec()
    } else {
        Vec::new()
    };

    Ok(Blockdata {
        min,
        max,
        unknown_float,
        flags,
        total_size: data.len(),
        extra_data,
    })
}

impl Blockdata {
    /// Check if flag at index `i` is set.
    pub fn flag(&self, i: usize) -> bool {
        i < 4 && self.flags[i] == 0x04
    }

    /// Number of set flags.
    pub fn flag_count(&self) -> usize {
        self.flags.iter().filter(|&&f| f == 0x04).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_blockdata(size: usize) -> Vec<u8> {
        let mut data = vec![0u8; size];
        // Bounding box
        let min = [-0.1f32, -0.2, -0.3];
        let max = [0.1f32, 0.2, 0.3];
        for i in 0..3 {
            data[i * 4..i * 4 + 4].copy_from_slice(&min[i].to_le_bytes());
            data[12 + i * 4..12 + i * 4 + 4].copy_from_slice(&max[i].to_le_bytes());
        }
        // Unknown float
        data[24..28].copy_from_slice(&0.05f32.to_le_bytes());
        // Flags
        data[28] = 0x04;
        data[29] = 0x04;
        data
    }

    #[test]
    fn parse_36byte() {
        let data = build_test_blockdata(36);
        let bd = parse(&data).unwrap();
        assert!((bd.min.x - (-0.1)).abs() < 1e-6);
        assert!((bd.max.z - 0.3).abs() < 1e-6);
        assert!(bd.flag(0));
        assert!(bd.flag(1));
        assert!(!bd.flag(2));
        assert_eq!(bd.flag_count(), 2);
        assert_eq!(bd.total_size, 36);
    }

    #[test]
    fn parse_48byte() {
        let data = build_test_blockdata(48);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.total_size, 48);
        assert_eq!(bd.extra_data.len(), 16);
    }

    #[test]
    fn parse_84byte() {
        let data = build_test_blockdata(84);
        let bd = parse(&data).unwrap();
        assert_eq!(bd.total_size, 84);
        assert_eq!(bd.extra_data.len(), 52);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 10];
        assert!(matches!(parse(&data), Err(BlockdataError::TooShort)));
    }
}
