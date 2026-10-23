//! Reader for the DBPF container format used by SPORE `.package` files.
//!
//! DBPF ("Database Packed File") is the archive format the game ships its assets
//! in. This crate implements the header, the resource index and payload
//! extraction, all measured against real shipped packages. The evidence behind
//! every layout decision is in `docs/dbpf-format.md`, and the analysis scripts
//! that produced it are in `tools/`.
//!
//! `native_index` provides the canonical flag-driven keys/metadata recovered
//! from the original reader. The earlier `RecordShape`/`ResourceKey` API is a
//! historical compatibility view; its group labels are superseded. See
//! `re/archive-index-native.md` before using it for native resource identity.

use std::fmt;
use std::path::{Path, PathBuf};

pub mod header;
pub mod index;
pub mod native_index;
pub mod refpack;

pub use header::{DbpfHeader, HEADER_SIZE};
pub use index::{RecordKind, RecordShape};
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

/// Historical inspection key, retained for compatibility.
/// Its combined instance spelling and old shape-derived group field must not
/// be treated as native identity; use `native_index::NativeResourceKey`.
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
    shape: RecordShape,
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

        let (shape, entries) = parse_index(&bytes, &header)?;
        Ok(Self {
            path,
            bytes,
            header,
            entries,
            shape,
        })
    }

    /// The record shape that was detected for this package.
    pub fn record_shape(&self) -> RecordShape {
        self.shape
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
        let out = decompress(&stored, Some(entry.decompressed_size as usize), true).map_err(
            |source| DbpfError::Decompression {
                key: entry.key,
                source,
            },
        )?;
        Ok(out.data)
    }
}


/// A package's index without its payload.
///
/// `DbpfFile::open` reads the whole file, which is fine for one package and hopeless
/// for a directory of them: a retail install is 5.75 GB across 82 packages, and a
/// model's texture can live in any of them. This reads the header and the index block
/// only, then seeks to a resource when one is actually needed.
///
/// Measured: `Spore_Graphics.package` is 995 MB on disk and its index is 1.2 MB, so a
/// cross-package lookup costs a few hundred kilobytes of I/O per package instead of a
/// gigabyte.
pub struct PackageIndex {
    path: PathBuf,
    header: DbpfHeader,
    shape: RecordShape,
    entries: Vec<ResourceEntry>,
    file_len: u64,
}

impl PackageIndex {
    /// Read the header and index block of a package.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbpfError> {
        use std::io::{Read, Seek, SeekFrom};

        let path = path.as_ref().to_path_buf();
        let mut file = std::fs::File::open(&path)?;
        let file_len = file.metadata()?.len();

        let mut head = [0u8; HEADER_SIZE];
        file.read_exact(&mut head)?;
        let header = DbpfHeader::parse(&head)?;
        if header.version_0x04 != 3 {
            return Err(DbpfError::UnsupportedVersion {
                major: header.version_0x04,
                minor: header.version_0x08,
            });
        }

        let mut block = vec![0u8; header.index_size as usize];
        file.seek(SeekFrom::Start(u64::from(header.index_offset)))?;
        file.read_exact(&mut block)?;

        let (shape, entries) = index::detect_with_len(
            &block,
            0,
            header.index_size,
            header.index_entry_count,
            file_len as usize,
        )?;
        Ok(Self {
            path,
            header,
            shape,
            entries,
            file_len,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn header(&self) -> &DbpfHeader {
        &self.header
    }

    pub fn shape(&self) -> RecordShape {
        self.shape
    }

    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    /// Historical combined scope/low lookup. The shape-derived group field is
    /// ignored because it is not native group identity. New native consumers
    /// should use `native_index::NativeIndex::find_unique` with all three words.
    pub fn find_instance(&self, type_id: u32, instance_id: u64) -> Option<&ResourceEntry> {
        self.entries
            .iter()
            .find(|e| e.key.type_id == type_id && e.key.instance_id == instance_id)
    }

    /// Find a resource by instance alone, for a key whose type is 0.
    ///
    /// A type of 0 means "unspecified", not "absent": one measured example is
    /// `0x00000000BE37F695`, which is an RW4 in `Spore_Graphics`. Matching the
    /// instance alone resolves it; matching type 0 does not.
    pub fn find_any_type(&self, instance_id: u64) -> Option<&ResourceEntry> {
        self.entries
            .iter()
            .find(|e| e.key.instance_id == instance_id)
    }

    /// Read one resource by seeking, then decompress it if it is coded.
    pub fn read(&self, entry: &ResourceEntry) -> Result<Vec<u8>, DbpfError> {
        use std::io::{Read, Seek, SeekFrom};

        let end = u64::from(entry.offset) + u64::from(entry.stored_size);
        if end > self.file_len {
            return Err(DbpfError::ResourceOutOfRange {
                key: entry.key,
                offset: entry.offset,
                end,
                file_len: self.file_len as usize,
            });
        }
        let mut file = std::fs::File::open(&self.path)?;
        file.seek(SeekFrom::Start(u64::from(entry.offset)))?;
        let mut stored = vec![0u8; entry.stored_size as usize];
        file.read_exact(&mut stored)?;
        if !entry.is_coded() {
            return Ok(stored);
        }
        let out = decompress(&stored, Some(entry.decompressed_size as usize), true).map_err(
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
/// The record shape is **not** constant across retail packages. Three variants were
/// measured, differing in record size and in field order. `index::detect` picks the
/// shape that consumes the index block exactly and whose records chain contiguously
/// when sorted by offset. See `docs/dbpf-format.md` section 6.
fn parse_index(    bytes: &[u8],
    header: &DbpfHeader,
) -> Result<(RecordShape, Vec<ResourceEntry>), DbpfError> {
    index::detect(
        bytes,
        header.index_offset,
        header.index_size,
        header.index_entry_count,
    )
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

    /// A package holding exactly one resource. Contiguity needs two records, so
    /// this used to be refused even though the 28-byte shape fits the 36-byte
    /// block exactly. Retail installs contain such packages (`Locale/en-us/UI`).
    #[test]
    fn single_entry_package_is_accepted() {
        let mut buf = synthetic_header(1);
        buf.extend_from_slice(b"ABCD"); // one 4-byte raw payload at 0x60
        let index_offset = buf.len() as u32; // 0x64
        let index_size = (INDEX_PREFIX_SIZE + INDEX_RECORD_SIZE + INDEX_TRAILER_SIZE) as u32;
        buf[0x2C..0x30].copy_from_slice(&index_size.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(&index_offset.to_le_bytes());

        buf.extend_from_slice(&4u32.to_le_bytes()); // prefix
        buf.extend_from_slice(&0x0001_FFFFu32.to_le_bytes()); // group
        buf.extend_from_slice(&0x2F7D_0004u32.to_le_bytes()); // type
        buf.extend_from_slice(&0u32.to_le_bytes()); // instance high
        buf.extend_from_slice(&1u32.to_le_bytes()); // instance low
        buf.extend_from_slice(&0x60u32.to_le_bytes()); // offset
        buf.extend_from_slice(&(4u32 | 0x8000_0000).to_le_bytes()); // stored
        buf.extend_from_slice(&4u32.to_le_bytes()); // decompressed
        buf.extend_from_slice(&0x0001_FFFFu32.to_le_bytes()); // trailer

        let path = std::env::temp_dir().join("rustspore_dbpf_single_entry.package");
        std::fs::write(&path, &buf).expect("write synthetic package");
        let file = DbpfFile::open(&path).expect("single-entry package should open");
        assert_eq!(file.entries().len(), 1);
        assert_eq!(file.resource_data(&file.entries()[0]).expect("payload"), b"ABCD");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn index_that_no_shape_can_consume_is_refused() {        // 4 entries in a 100-byte block. No candidate fits: the shapes need
        // 120 (28-byte), 108 (24-byte) or 96 (20-byte) bytes for 4 records.
        let mut buf = synthetic_header(4);
        buf[0x2C..0x30].copy_from_slice(&100u32.to_le_bytes());
        buf[0x40..0x44].copy_from_slice(&0x60u32.to_le_bytes());
        buf.resize(0x60 + 100, 0);

        match parse_index(&buf, &DbpfHeader::parse(&buf).expect("header")) {
            Err(DbpfError::UnsupportedIndexLayout { count, available, .. }) => {
                assert_eq!(count, 4);
                assert_eq!(available, 100);
            }
            other => panic!("expected UnsupportedIndexLayout, got {other:?}"),
        }
    }

    /// The index-only reader must agree with the full reader on the same file: same
    /// entries, same payloads. It exists so a directory of packages can be searched
    /// without holding 5.75 GB of game data in memory.
    #[test]
    fn package_index_matches_full_open() {
        let path = std::env::temp_dir().join("rustspore_dbpf_index_only.package");
        std::fs::write(&path, synthetic_package()).expect("write synthetic package");

        let full = DbpfFile::open(&path).expect("open full");
        let index = PackageIndex::open(&path).expect("open index only");

        assert_eq!(index.entries().len(), full.entries().len());
        assert_eq!(index.shape(), full.record_shape());
        assert_eq!(index.file_len(), full.len() as u64);
        for (a, b) in index.entries().iter().zip(full.entries()) {
            assert_eq!(a.key, b.key);
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.stored_size, b.stored_size);
            assert_eq!(
                index.read(a).expect("index read"),
                full.resource_data(b).expect("full read")
            );
        }

        let _ = std::fs::remove_file(&path);
    }

    /// A prop key carries a type and an instance but no group, so lookup ignores it.
    #[test]
    fn package_index_finds_by_instance_ignoring_group() {
        let path = std::env::temp_dir().join("rustspore_dbpf_find_instance.package");
        std::fs::write(&path, synthetic_package()).expect("write synthetic package");

        let index = PackageIndex::open(&path).expect("open index only");
        let found = index
            .find_instance(0x00B1_B104, 2)
            .expect("second resource by instance");
        assert_eq!(found.key.group_id, 0x0001_FFFF);
        assert_eq!(index.read(found).expect("read"), b"EFGH");
        assert!(index.find_instance(0x00B1_B104, 99).is_none());

        let _ = std::fs::remove_file(&path);
    }
}
