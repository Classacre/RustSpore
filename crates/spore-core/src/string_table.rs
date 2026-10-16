//! Parser for the SPORE string table format (type `0xDFAD9F51`).
//!
//! Status: 100% decoded. All 446 samples are exactly 796 bytes.
//!
//! Layout:
//! ```text
//! offset  0: LE u32 (hash/ID, varies per string)
//! offset  4: UTF-16LE string (null-terminated)
//! offset X: null padding to 796 bytes
//! ```
//!
//! Maximum string length: (796 - 4) / 2 = 396 characters.

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
    /// Hash/ID for looking up the string.
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

    // Decode UTF-16LE string starting at offset 4
    let mut text = String::new();
    let mut i = 4;
    while i + 2 <= ENTRY_SIZE {
        let ch = u16::from_le_bytes([data[i], data[i + 1]]);
        if ch == 0 {
            break;
        }
        if let Some(c) = char::from_u32(ch as u32) {
            text.push(c);
        }
        i += 2;
    }

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
        let text = "A".repeat(396);
        let data = build_test_entry(&text);
        let entry = parse(&data).unwrap();
        assert_eq!(entry.text.len(), 396);
    }

    #[test]
    fn wrong_size() {
        let data = vec![0u8; 100];
        assert_eq!(parse(&data), Err(StringTableError::WrongSize));
    }
}
