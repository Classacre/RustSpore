//! Legacy label reader for SPORE Cell definitions (type `0xDFAD9F51`).
//!
//! The old "string table, 100% decoded" identification was wrong. The first
//! word links a Cell structure and the text is an embedded developer label.
//! The remaining bytes contain other data and are not decoded by this module.
//! Source-backed Cell resource resolution lives in spore-bevy/source_cell.rs.
//!
//! Layout:
//! ```text
//! offset  0: LE u32 (hash/ID, varies per string)
//! offset  4: 80 UTF-16LE code units (stops at first null)
//! offset A4: unnamed cLocalizedString integer
//! remainder: Cell data, not established as null padding
//! ```
//!
//! The native cLocalizedString text member occupies 0xA0 bytes. The following
//! numeric data must never be consumed as developer-label characters.

use std::fmt;

/// Errors that can occur when parsing a string table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringTableError {
    /// The entry is not exactly 796 bytes.
    WrongSize,
}

impl fmt::Display for StringTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringTableError::WrongSize => write!(f, "string table entry must be 796 bytes"),
        }
    }
}

impl std::error::Error for StringTableError {}

/// Size of a string table entry in bytes.
pub const ENTRY_SIZE: usize = 796;

/// A parsed string table entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringEntry {
    /// Legacy field name: this word is the referenced Cell structure ID.
    pub hash: u32,
    /// The decoded UTF-16LE string.
    pub text: String,
}

/// Parse a string table entry.
pub fn parse(data: &[u8]) -> Result<StringEntry, StringTableError> {
    if data.len() != ENTRY_SIZE {
        return Err(StringTableError::WrongSize);
    }

    let hash = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    // Keep this legacy lossy API, while honoring the recovered fixed extent and
    // decoding surrogate pairs. The typed Cell parser also retains exact units.
    let units: Vec<u16> = data[4..0xa4]
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    let text = String::from_utf16_lossy(&units);

    Ok(StringEntry { hash, text })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_entry(text: &str) -> Vec<u8> {
        let mut data = vec![0u8; ENTRY_SIZE];
        let hash = 0x12345678u32;
        data[0..4].copy_from_slice(&hash.to_le_bytes());

        let mut offset = 4;
        for ch in text.encode_utf16() {
            let bytes = ch.to_le_bytes();
            data[offset] = bytes[0];
            data[offset + 1] = bytes[1];
            offset += 2;
        }
        // Null terminator already zero from initialization

        data
    }

    #[test]
    fn parse_basic() {
        let data = build_test_entry("PLACEHOLDER_Maa");
        let entry = parse(&data).unwrap();
        assert_eq!(entry.hash, 0x12345678);
        assert_eq!(entry.text, "PLACEHOLDER_Maa");
    }

    #[test]
    fn parse_empty() {
        let data = build_test_entry("");
        let entry = parse(&data).unwrap();
        assert_eq!(entry.hash, 0x12345678);
        assert_eq!(entry.text, "");
    }

    #[test]
    fn parse_long() {
        let text = format!("{}\u{1f30d}", "A".repeat(78));
        let mut data = build_test_entry(&text);
        data[0xa4..0xa8].copy_from_slice(&0x0042_0042u32.to_le_bytes());
        let entry = parse(&data).unwrap();
        assert_eq!(entry.text, text);
    }

    #[test]
    fn wrong_size() {
        let data = vec![0u8; 100];
        assert_eq!(parse(&data), Err(StringTableError::WrongSize));
    }
}
