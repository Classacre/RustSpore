//! The DBPF file header.
//!
//! Only fields measured against real shipped packages are modelled. See
//! `docs/dbpf-format.md` for the evidence behind each one.

use crate::DbpfError;

/// Total size of the DBPF header in bytes, as observed on shipped packages.
pub const HEADER_SIZE: usize = 0x60;

/// The DBPF header.
///
/// Every field below was read at the stated offset from at least two real
/// packages (`Data/Spore_Game.package` and `Data/PatchData.package`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbpfHeader {
    /// File magic. Always the ASCII bytes `DBPF` (`0x44 0x42 0x50 0x46`).
    pub magic: [u8; 4],

    /// The u32 at header offset `0x04`. It is `3` in all 13 packages inspected.
    ///
    /// The name "major version" is inherited from the conventional DBPF layout,
    /// not derived from measurement, and public documentation disagrees with it:
    /// see `docs/dbpf-format.md` section 2.1. Nothing in this crate branches on
    /// this value, so the uncertainty is harmless here.
    pub version_0x04: u32,

    /// The u32 at header offset `0x08`. It is `0` in all 13 packages inspected.
    pub version_0x08: u32,

    /// Number of entries in the resource index.
    ///
    /// Measured: 11,529 for `Data/Spore_Game.package` and 25,535 for
    /// `Data/PatchData.package`. Confirmed structurally: the index found at
    /// [`DbpfHeader::index_offset`] contains exactly this many valid records.
    pub index_entry_count: u32,

    /// Unidentified. Measured at header offset `0x28`. Zero in every package
    /// inspected.
    pub unknown_0x28: u32,

    /// Size of the index block in bytes, at header offset `0x2C`.
    ///
    /// Confirmed on three packages by `index_offset + index_size == file size`.
    pub index_size: u32,

    /// Unidentified. Measured at header offset `0x3C`. It is `3` in every
    /// package inspected, the same value as `version_0x04`.
    pub unknown_0x3c: u32,

    /// Absolute file offset of the index block, at header offset `0x40`.
    ///
    /// Confirmed on three packages by `index_offset + index_size == file size`.
    pub index_offset: u32,
}

impl DbpfHeader {
    /// Minimum number of bytes needed to read every modelled field.
    ///
    /// This is the offset just past `index_offset`. It is deliberately smaller
    /// than [`HEADER_SIZE`] so a package shorter than the full measured header
    /// window still parses as long as the fields we understand are present.
    pub const MIN_LEN: usize = 0x44;

    /// Parse a header from the start of `bytes`.
    ///
    /// All multi byte fields are little endian, matching the observed files.
    pub fn parse(bytes: &[u8]) -> Result<Self, DbpfError> {
        if bytes.len() < Self::MIN_LEN {
            return Err(DbpfError::TruncatedHeader {
                actual: bytes.len(),
                needed: Self::MIN_LEN,
            });
        }

        let magic: [u8; 4] = bytes[0x00..0x04].try_into().expect("4 bytes checked above");
        if &magic != b"DBPF" {
            return Err(DbpfError::BadMagic { found: magic });
        }

        let u32_at = |offset: usize| -> u32 {
            u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("offset within MIN_LEN"),
            )
        };

        Ok(Self {
            magic,
            version_0x04: u32_at(0x04),
            version_0x08: u32_at(0x08),
            index_entry_count: u32_at(0x24),
            unknown_0x28: u32_at(0x28),
            index_size: u32_at(0x2c),
            unknown_0x3c: u32_at(0x3c),
            index_offset: u32_at(0x40),
        })
    }

    /// The offset just past the end of the index block.
    ///
    /// On every package whose index parsed, this equals the file size exactly.
    pub fn index_end(&self) -> u64 {
        u64::from(self.index_offset) + u64::from(self.index_size)
    }
}
