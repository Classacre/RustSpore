//! Reader for the DBPF container format used by SPORE `.package` files.
//!
//! DBPF ("Database Packed File") is the archive format the game ships its assets
//! in. This crate implements the header, the resource index and payload
//! extraction, all measured against real shipped packages. The evidence behind
//! every layout decision is in `docs/dbpf-format.md`, and the analysis scripts
//! that produced it are in `tools/`.
//!
//! Nothing here guesses at the on-disk format. Where a layout is not
//! established, the reader returns an explicit error rather than inventing one:
//! see [`DbpfError::UnsupportedIndexLayout`], which is what the three packages
//! documented in `docs/dbpf-format.md` section 6 will produce.

use std::fmt;
use std::path::{Path, PathBuf};

pub mod header;
pub mod refpack;

pub use header::{DbpfHeader, HEADER_SIZE};
pub use refpack::{decompress, RefPackError, RefPackInfo, RefPackOutput};

/// Size of one resource index record, in bytes.
///
/// Measured, not assumed: a structural search for a long run of records with
/// in-range `(offset, size)` spans found exactly 11,529 consecutive records at a
/// 28-byte stride in `Data/Spore_Game.package`, matching that file's header
/// count at a ratio of 1.0000.
pub const INDEX_RECORD_SIZE: usize = 28;

/// Bytes before the first index record, and after the last one.
///
/// Measured: the prefix is `4` in all ten packages whose index parsed, and the
/// trailer is `0x0001FFFF` or `0x00010000`. Their meaning is not established;
/// they are skipped.
pub const INDEX_PREFIX_SIZE: usize = 4;
pub const INDEX_TRAILER_SIZE: usize = 4;

/// A DBPF resource key: the triple that uniquely identifies a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceKey {
    pub type_id: u32,
    pub group_id: u32,
    pub instance_id: u64,
}

impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08X}:{:08X}:{:016X}",
            self.type_id, self.group_id, self.instance_id
        )
    }
}

/// One entry in the resource index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceEntry {
    /// The resource key.
    pub key: ResourceKey,
    /// Absolute file offset of the payload.
    pub offset: u32,
    /// Stored payload size in bytes.
    pub stored_size: u32,
    /// Size after decompression. Equal to `stored_size` when the payload is raw.
    pub decompressed_size: u32,
}

impl ResourceEntry {
    /// Whether the payload is RefPack-coded rather than stored raw.
    ///
    /// The rule is measured, not assumed: across `Spore_Game.package`, 2,215 of
    /// 11,529 resources have `stored_size == decompressed_size` and 9,314 have
    /// `stored_size < decompressed_size`. Bit 31 of the stored size is set on
    /// 100% of records and therefore carries no per-resource information.
    pub fn is_coded(&self) -> bool {
        self.decompressed_size > self.stored_size
    }
}

/// Everything that can go wrong while opening or reading a package.
#[derive(Debug, thiserror::Error)]
pub enum DbpfError {
    /// The underlying file could not be read.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// The first four bytes were not the ASCII magic `DBPF`.
    #[error("not a DBPF file: magic was {found:?}, expected [0x44, 0x42, 0x50, 0x46] (\"DBPF\")")]
    BadMagic { found: [u8; 4] },

    /// The file ended before the measured header fields could be read.
    #[error("file too small: {actual} bytes, need at least {needed} bytes for the header")]
    TruncatedHeader { actual: usize, needed: usize },

    /// The version is outside the range this reader understands.
    #[error("unsupported DBPF version {major}.{minor}; only major version 3 is handled")]
    UnsupportedVersion { major: u32, minor: u32 },

    /// The index block does not fit the measured record layout.
    ///
    /// Three packages in a retail install land here; see
    /// `docs/dbpf-format.md` section 6. The reader refuses to guess rather than
    /// silently mis-parsing them.
    #[error(
        "index layout not supported for this package: {count} entries would need \
         {needed} bytes but the index block is {available} bytes \
         (index at {index_offset}, size {index_size})"
    )]
    UnsupportedIndexLayout {
        count: u32,
        needed: u64,
        available: u64,
        index_offset: u32,
        index_size: u32,
    },

    /// A resource record pointed outside the file.
    #[error("resource {key} spans {offset}..{end}, which is outside the {file_len} byte file")]
    ResourceOutOfRange {
        key: ResourceKey,
        offset: u32,
        end: u64,
        file_len: usize,
    },

    /// A coded payload could not be decompressed.
    #[error("resource {key} could not be decompressed: {source}")]
    Decompression {
        key: ResourceKey,
        #[source]
        source: RefPackError,
    },
}

/// An open DBPF package.
///
/// The whole file is held in memory for now. This is simple and correct; it can
/// be replaced with a memory map later without changing the public surface.
pub struct DbpfFile {
    path: PathBuf,
    bytes: Vec<u8>,
    header: DbpfHeader,
    entries: Vec<ResourceEntry>,
}

impl DbpfFile {
    /// Open a package, parse its header and read its resource index.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbpfError> {
        let path = path.as_ref().to_path_buf();
        let bytes = std::fs::read(&path)?;
        let header = DbpfHeader::parse(&bytes)?;

        if header.version_0x04 != 3 {
            return Err(DbpfError::UnsupportedVersion {
                major: header.version_0x04,
                minor: header.version_0x08,
            });
        }

        let entries = parse_index(&bytes, &header)?;
        Ok(Self {
            path,
            bytes,
            header,
            entries,
        })
    }

    /// The parsed header.
    pub fn header(&self) -> &DbpfHeader {
        &self.header
    }

    /// The path the package was opened from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every resource index entry, in file order.
    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }

    /// The resource keys in the package.
    pub fn resource_keys(&self) -> impl Iterator<Item = ResourceKey> + '_ {
        self.entries.iter().map(|e| e.key)
    }

    /// Find the entry for a key.
    pub fn find(&self, key: &ResourceKey) -> Option<&ResourceEntry> {
        self.entries.iter().find(|e| e.key == *key)
    }

    /// The total size of the package in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the package is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// The raw bytes of the package as loaded.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Read and, if needed, decompress the payload of one resource.
    pub fn resource_data(&self, entry: &ResourceEntry) -> Result<Vec<u8>, DbpfError> {
        let start = entry.offset as usize;
        let end = start
            .checked_add(entry.stored_size as usize)
            .ok_or(DbpfError::ResourceOutOfRange {
                key: entry.key,
                offset: entry.offset,
                end: u64::from(entry.offset) + u64::from(entry.stored_size),
                file_len: self.bytes.len(),
            })?;

        if end > self.bytes.len() {
            return Err(DbpfError::ResourceOutOfRange {
                key: entry.key,
                offset: entry.offset,
                end: end as u64,
                file_len: self.bytes.len(),
            });
        }

        let stored = &self.bytes[start..end];

        if !entry.is_coded() {
            return Ok(stored.to_vec());
        }

        // Coded payloads carry their own RefPack size header. The declared size
        // is checked against the index record, which is a free consistency test:
        // on real data the two agree for every resource sampled.
        let out = decompress(stored, Some(entry.decompressed_size as usize), true).map_err(
            |source| DbpfError::Decompression {
                key: entry.key,
                source,
            },
        )?;
        Ok(out.data)
    }
}

/// Parse the resource index described by `header`.
///
/// Layout, all measured: a 4-byte prefix, `index_entry_count` records of
/// [`INDEX_RECORD_SIZE`] bytes, then a 4-byte trailer. The block begins at
/// `header.index_offset` and runs for `header.index_size` bytes.
fn parse_index(bytes: &[u8], header: &DbpfHeader) -> Result<Vec<ResourceEntry>, DbpfError> {
    let count = header.index_entry_count as u64;
    let needed = (INDEX_PREFIX_SIZE + INDEX_TRAILER_SIZE) as u64 + count * INDEX_RECORD_SIZE as u64;
    let available = u64::from(header.index_size);

    if needed > available {
        return Err(DbpfError::UnsupportedIndexLayout {
            count: header.index_entry_count,
            needed,
            available,
            index_offset: header.index_offset,
            index_size: header.index_size,
        });
    }

    let base = header.index_offset as usize + INDEX_PREFIX_SIZE;
    let end = base + header.index_entry_count as usize * INDEX_RECORD_SIZE;
    if end > bytes.len() {
        return Err(DbpfError::TruncatedHeader {
            actual: bytes.len(),
            needed: end,
        });
    }

    let mut entries = Vec::with_capacity(header.index_entry_count as usize);
    for i in 0..header.index_entry_count as usize {
        let at = base + i * INDEX_RECORD_SIZE;
        let u = |o: usize| -> u32 {
            u32::from_le_bytes(
                bytes[at + o..at + o + 4]
                    .try_into()
                    .expect("bounds checked above"),
            )
        };
        // Field order measured by cardinality: group and type are
        // low-cardinality, the two instance words are not.
        let group_id = u(0);
        let type_id = u(4);
        let instance_hi = u(8);
        let instance_lo = u(12);
        let offset = u(16);
        // Bit 31 of the size word is set on every record, so it is masked off.
        let stored_size = u(20) & 0x7FFF_FFFF;
        let decompressed_size = u(24);

        entries.push(ResourceEntry {
            key: ResourceKey {
                type_id,
                group_id,
                instance_id: ((instance_hi as u64) << 32) | instance_lo as u64,
            },
            offset,
            stored_size,
            decompressed_size,
        });
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Measured against a retail `Data/Spore_Game.package`.
    const SPORE_GAME_INDEX_ENTRY_COUNT: u32 = 11_529;

    /// Measured against a retail `Data/PatchData.package`.
    const PATCH_DATA_INDEX_ENTRY_COUNT: u32 = 25_535;

    /// Build a synthetic 0x60 byte header so the tests do not depend on the real
    /// game files being present. Only the measured offsets are populated.
    fn synthetic_header(index_entry_count: u32) -> Vec<u8> {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[0x00..0x04].copy_from_slice(b"DBPF");
        buf[0x04..0x08].copy_from_slice(&3u32.to_le_bytes());
        buf[0x08..0x0C].copy_from_slice(&0u32.to_le_bytes());
        buf[0x24..0x28].copy_from_slice(&index_entry_count.to_le_bytes());
        buf[0x28..0x2C].copy_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
        buf[0x2C..0x30].copy_from_slice(&0x0000_00FFu32.to_le_bytes());
        buf[0x3C..0x40].copy_from_slice(&0x0BAD_F00Du32.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(&0x0000_0060u32.to_le_bytes());
        buf
    }

    #[test]
    fn parses_magic_and_version_from_synthetic_header() {
        let header = DbpfHeader::parse(&synthetic_header(0)).expect("header should parse");
        assert_eq!(&header.magic, b"DBPF");
        assert_eq!(header.version_0x04, 3);
        assert_eq!(header.version_0x08, 0);
    }

    #[test]
    fn reads_measured_entry_counts() {
        let h = DbpfHeader::parse(&synthetic_header(SPORE_GAME_INDEX_ENTRY_COUNT)).expect("parses");
        assert_eq!(h.index_entry_count, 11_529);
        let h = DbpfHeader::parse(&synthetic_header(PATCH_DATA_INDEX_ENTRY_COUNT)).expect("parses");
        assert_eq!(h.index_entry_count, 25_535);
    }

    #[test]
    fn reads_index_fields_at_their_offsets() {
        let header = DbpfHeader::parse(&synthetic_header(0)).expect("parses");
        assert_eq!(header.unknown_0x28, 0xDEAD_BEEF);
        assert_eq!(header.index_size, 0x0000_00FF);
        assert_eq!(header.unknown_0x3c, 0x0BAD_F00D);
        assert_eq!(header.index_offset, 0x0000_0060);
        assert_eq!(header.index_end(), 0x0000_0060 + 0x0000_00FF);
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut buf = synthetic_header(0);
        buf[0x00..0x04].copy_from_slice(b"XXXX");
        match DbpfHeader::parse(&buf) {
            Err(DbpfError::BadMagic { found }) => assert_eq!(&found, b"XXXX"),
            other => panic!("expected BadMagic, got {other:?}"),
        }
    }

    #[test]
    fn rejects_truncated_header() {
        let buf = vec![b'D', b'B', b'P', b'F'];
        match DbpfHeader::parse(&buf) {
            Err(DbpfError::TruncatedHeader { actual, needed }) => {
                assert_eq!(actual, 4);
                assert_eq!(needed, DbpfHeader::MIN_LEN);
            }
            other => panic!("expected TruncatedHeader, got {other:?}"),
        }
    }

    #[test]
    fn resource_key_display_is_type_group_instance() {
        let key = ResourceKey {
            type_id: 0x0000_1234,
            group_id: 0x0000_ABCD,
            instance_id: 0x0000_0000_0000_0001,
        };
        assert_eq!(key.to_string(), "00001234:0000ABCD:0000000000000001");
    }

    /// Build a complete, valid little package in memory: two raw resources and a
    /// well-formed index. This exercises the whole path from bytes to payload.
    fn synthetic_package() -> Vec<u8> {
        const PAYLOAD_OFFSET: u32 = 0x60;
        let mut buf = synthetic_header(2);

        // Two 4-byte raw payloads starting straight after the header.
        buf.extend_from_slice(b"ABCD");
        buf.extend_from_slice(b"EFGH");

        let index_offset = buf.len() as u32; // 0x68
        let index_size = (INDEX_PREFIX_SIZE + 2 * INDEX_RECORD_SIZE + INDEX_TRAILER_SIZE) as u32;

        // Patch the header now that the index position is known.
        buf[0x2C..0x30].copy_from_slice(&index_size.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(&index_offset.to_le_bytes());

        buf.extend_from_slice(&4u32.to_le_bytes()); // prefix, measured as 4

        let mut push_record =
            |group: u32, type_id: u32, inst: u64, offset: u32, size: u32, decomp: u32| {
                buf.extend_from_slice(&group.to_le_bytes());
                buf.extend_from_slice(&type_id.to_le_bytes());
                buf.extend_from_slice(&((inst >> 32) as u32).to_le_bytes());
                buf.extend_from_slice(&(inst as u32).to_le_bytes());
                buf.extend_from_slice(&offset.to_le_bytes());
                // Bit 31 is set on every real record, so set it here too.
                buf.extend_from_slice(&(size | 0x8000_0000).to_le_bytes());
                buf.extend_from_slice(&decomp.to_le_bytes());
            };

        push_record(0x0001_FFFF, 0x2F7D_0004, 1, PAYLOAD_OFFSET, 4, 4);
        push_record(0x0001_FFFF, 0x00B1_B104, 2, PAYLOAD_OFFSET + 4, 4, 4);

        buf.extend_from_slice(&0x0001_FFFFu32.to_le_bytes()); // trailer
        buf
    }

    #[test]
    fn parses_index_and_extracts_raw_payloads() {
        let path = std::env::temp_dir().join("rustspore_dbpf_synthetic_index.package");
        std::fs::write(&path, synthetic_package()).expect("write synthetic package");

        let file = DbpfFile::open(&path).expect("open synthetic package");
        assert_eq!(file.header().index_entry_count, 2);
        assert_eq!(file.entries().len(), 2);

        let first = file.entries()[0];
        assert_eq!(first.key.type_id, 0x2F7D_0004);
        assert_eq!(first.key.group_id, 0x0001_FFFF);
        assert_eq!(first.key.instance_id, 1);
        assert_eq!(first.offset, 0x60);
        assert_eq!(first.stored_size, 4);
        assert!(!first.is_coded(), "stored == decompressed means raw");

        assert_eq!(file.resource_data(&first).expect("raw payload"), b"ABCD");

        let second = file.entries()[1];
        assert_eq!(file.resource_data(&second).expect("raw payload"), b"EFGH");

        assert_eq!(file.find(&first.key).map(|e| e.offset), Some(0x60));
        assert_eq!(file.resource_keys().count(), 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn index_that_does_not_fit_the_record_layout_is_refused() {
        // Mimic the real failing case: a count too large for the index block.
        let mut buf = synthetic_header(4);
        buf[0x2C..0x30].copy_from_slice(&96u32.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(&0x60u32.to_le_bytes());
        buf.resize(0x60 + 96, 0);

        match parse_index(&buf, &DbpfHeader::parse(&buf).expect("header")) {
            Err(DbpfError::UnsupportedIndexLayout {
                count,
                needed,
                available,
                ..
            }) => {
                assert_eq!(count, 4);
                assert_eq!(needed, 120);
                assert_eq!(available, 96);
            }
            other => panic!("expected UnsupportedIndexLayout, got {other:?}"),
        }
    }
}
