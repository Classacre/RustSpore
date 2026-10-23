//! Native 96-bit archive keys and the flag-driven packed index reader.
//!
//! Original `8DAFE0` consumes common type/scope/high words selected by bits
//! 0/1/2. This build requires a common, zero instance-high word. Stored-size
//! bit31 selects a metadata word after that same record; it is not compression.
//! The older `RecordShape` view remains a compatibility/diagnostic interface.

use crate::{decompress, DbpfError, DbpfHeader, RefPackError, HEADER_SIZE};
use std::{
    fmt,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// The original in-memory word order is instance-low, type, scope/group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NativeResourceKey {
    pub instance_id: u32,
    pub type_id: u32,
    pub group_id: u32,
}

impl NativeResourceKey {
    /// Decode the scope/low spelling used by source inspection CLIs and by
    /// existing PROP key accessors. Neither word is compression metadata.
    pub fn from_scoped_instance(type_id: u32, scoped: u64) -> Self {
        Self {
            instance_id: scoped as u32,
            type_id,
            group_id: (scoped >> 32) as u32,
        }
    }

    pub fn words(self) -> [u32; 3] {
        [self.instance_id, self.type_id, self.group_id]
    }

    /// Existing inspection CLIs spell scope and instance as one 16-digit ID.
    /// This is a convenience spelling, not a native 64-bit instance field.
    pub fn scoped_instance(self) -> u64 {
        (u64::from(self.group_id) << 32) | u64::from(self.instance_id)
    }
}

impl fmt::Display for NativeResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08X}:{:08X}:{:08X}",
            self.instance_id, self.type_id, self.group_id
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeResourceEntry {
    pub key: NativeResourceKey,
    pub offset: u32,
    pub stored_size: u32,
    pub decompressed_size: u32,
    /// Native ushort: zero is raw, FFFF is RefPack in the measured install.
    pub compression_type: u16,
    /// Persisted bit0 of metadata byte2, before the loader's override argument.
    pub disk_committed: bool,
    /// Exact optional on-disk word, including bits the native reader masks.
    pub metadata_word: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NativeIndexError {
    #[error("packed index truncated at byte {at}, needing {needed} more bytes")]
    Truncated { at: usize, needed: usize },
    #[error("this native reader requires shared instance-high flag4")]
    MissingCommonHigh,
    #[error("this native reader rejects nonzero instance-high {0:08X}")]
    NonZeroInstanceHigh(u32),
    #[error("packed index record count cannot fit in the available bytes")]
    RecordCountOutOfRange,
    #[error("packed index has {0} unconsumed bytes")]
    TrailingBytes(usize),
    #[error("exact lookup has more than one record for {0}")]
    DuplicateKey(NativeResourceKey),
}

#[derive(Debug, Clone)]
pub struct NativeIndex {
    pub shared_flags: u32,
    pub common_type: Option<u32>,
    pub common_group: Option<u32>,
    entries: Vec<NativeResourceEntry>,
}

fn next_word(bytes: &[u8], at: &mut usize) -> Result<u32, NativeIndexError> {
    let end = at
        .checked_add(4)
        .ok_or(NativeIndexError::RecordCountOutOfRange)?;
    let word = bytes
        .get(*at..end)
        .ok_or(NativeIndexError::Truncated { at: *at, needed: 4 })?;
    *at = end;
    Ok(u32::from_le_bytes(word.try_into().unwrap()))
}

impl NativeIndex {
    /// A bounded version of `8DAFE0`'s successful common-high-zero branch.
    /// Unused flag bits and metadata bits are retained, not assigned meanings.
    /// Unlike the pointer-based original, every read and the final extent are
    /// checked. Duplicate-key winner semantics are deliberately not invented.
    pub fn parse(bytes: &[u8], count: u32) -> Result<Self, NativeIndexError> {
        let mut at = 0;
        let shared_flags = next_word(bytes, &mut at)?;
        if shared_flags & 4 == 0 {
            return Err(NativeIndexError::MissingCommonHigh);
        }
        let common_type = if shared_flags & 1 != 0 {
            Some(next_word(bytes, &mut at)?)
        } else {
            None
        };
        let common_group = if shared_flags & 2 != 0 {
            Some(next_word(bytes, &mut at)?)
        } else {
            None
        };
        let common_high = next_word(bytes, &mut at)?;
        if common_high != 0 {
            return Err(NativeIndexError::NonZeroInstanceHigh(common_high));
        }
        let minimum_record =
            16 + usize::from(common_type.is_none()) * 4 + usize::from(common_group.is_none()) * 4;
        let count = usize::try_from(count).map_err(|_| NativeIndexError::RecordCountOutOfRange)?;
        if count > bytes.len().saturating_sub(at) / minimum_record {
            return Err(NativeIndexError::RecordCountOutOfRange);
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let type_id = match common_type {
                Some(value) => value,
                None => next_word(bytes, &mut at)?,
            };
            let group_id = match common_group {
                Some(value) => value,
                None => next_word(bytes, &mut at)?,
            };
            let instance_id = next_word(bytes, &mut at)?;
            let offset = next_word(bytes, &mut at)?;
            let stored_word = next_word(bytes, &mut at)?;
            let stored_size = stored_word & 0x7fff_ffff;
            let decompressed_size = next_word(bytes, &mut at)?;
            let metadata_word = if stored_word & 0x8000_0000 != 0 {
                Some(next_word(bytes, &mut at)?)
            } else {
                None
            };
            let compression_type = metadata_word.map(|word| word as u16).unwrap_or(
                if stored_size == decompressed_size {
                    0
                } else {
                    0xffff
                },
            );
            let disk_committed = metadata_word.is_some_and(|word| word & 0x0001_0000 != 0);
            entries.push(NativeResourceEntry {
                key: NativeResourceKey {
                    instance_id,
                    type_id,
                    group_id,
                },
                offset,
                stored_size,
                decompressed_size,
                compression_type,
                disk_committed,
                metadata_word,
            });
        }
        if at != bytes.len() {
            return Err(NativeIndexError::TrailingBytes(bytes.len() - at));
        }
        Ok(Self {
            shared_flags,
            common_type,
            common_group,
            entries,
        })
    }

    pub fn entries(&self) -> &[NativeResourceEntry] {
        &self.entries
    }

    pub fn find_unique(
        &self,
        key: NativeResourceKey,
    ) -> Result<Option<&NativeResourceEntry>, NativeIndexError> {
        let mut matches = self.entries.iter().filter(|entry| entry.key == key);
        let first = matches.next();
        if matches.next().is_some() {
            return Err(NativeIndexError::DuplicateKey(key));
        }
        Ok(first)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NativePackageError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("archive header: {0}")]
    Header(#[from] DbpfError),
    #[error("archive index: {0}")]
    Index(#[from] NativeIndexError),
    #[error("resource {key} spans {offset}..{end} outside {file_len} bytes")]
    ResourceOutOfRange {
        key: NativeResourceKey,
        offset: u32,
        end: u64,
        file_len: u64,
    },
    #[error("resource {key} has unsupported native compression tag {tag:04X}")]
    UnsupportedCompression { key: NativeResourceKey, tag: u16 },
    #[error("resource {key} could not be decompressed: {source}")]
    Decompression {
        key: NativeResourceKey,
        #[source]
        source: RefPackError,
    },
}

/// Canonical index-only archive access, separate from historical shape views.
pub struct NativePackageIndex {
    path: PathBuf,
    header: DbpfHeader,
    index: NativeIndex,
    file_len: u64,
}

impl NativePackageIndex {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, NativePackageError> {
        let path = path.as_ref().to_owned();
        let mut file = std::fs::File::open(&path)?;
        let file_len = file.metadata()?.len();
        let mut bytes = [0u8; HEADER_SIZE];
        file.read_exact(&mut bytes)?;
        let header = DbpfHeader::parse(&bytes)?;
        if header.version_0x04 != 3 {
            return Err(DbpfError::UnsupportedVersion {
                major: header.version_0x04,
                minor: header.version_0x08,
            }
            .into());
        }
        // Bound allocation before trusting an index-size word from a file.
        if u64::from(header.index_offset) + u64::from(header.index_size) > file_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "index extends beyond archive",
            )
            .into());
        }
        file.seek(SeekFrom::Start(u64::from(header.index_offset)))?;
        let mut block = vec![0; header.index_size as usize];
        file.read_exact(&mut block)?;
        let index = NativeIndex::parse(&block, header.index_entry_count)?;
        for entry in index.entries() {
            let end = u64::from(entry.offset) + u64::from(entry.stored_size);
            if end > file_len {
                return Err(NativePackageError::ResourceOutOfRange {
                    key: entry.key,
                    offset: entry.offset,
                    end,
                    file_len,
                });
            }
        }
        Ok(Self {
            path,
            header,
            index,
            file_len,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn header(&self) -> &DbpfHeader {
        &self.header
    }
    pub fn index(&self) -> &NativeIndex {
        &self.index
    }
    pub fn entries(&self) -> &[NativeResourceEntry] {
        self.index.entries()
    }
    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    pub fn read(&self, entry: &NativeResourceEntry) -> Result<Vec<u8>, NativePackageError> {
        if !matches!(entry.compression_type, 0 | 0xffff) {
            return Err(NativePackageError::UnsupportedCompression {
                key: entry.key,
                tag: entry.compression_type,
            });
        }
        let end = u64::from(entry.offset) + u64::from(entry.stored_size);
        if end > self.file_len {
            return Err(NativePackageError::ResourceOutOfRange {
                key: entry.key,
                offset: entry.offset,
                end,
                file_len: self.file_len,
            });
        }
        let mut file = std::fs::File::open(&self.path)?;
        file.seek(SeekFrom::Start(u64::from(entry.offset)))?;
        let mut stored = vec![0; entry.stored_size as usize];
        file.read_exact(&mut stored)?;
        if entry.compression_type == 0 {
            return Ok(stored);
        }
        let result = decompress(&stored, Some(entry.decompressed_size as usize), true).map_err(
            |source| NativePackageError::Decompression {
                key: entry.key,
                source,
            },
        )?;
        Ok(result.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words(values: &[u32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    #[test]
    fn current_metadata_belongs_to_current_record_and_scope_is_a_key_word() {
        let bytes = words(&[
            4,
            0,
            0xa,
            0xb,
            0xc,
            96,
            0x8000_0010,
            0x20,
            0x1ffff,
            0xd,
            0xe,
            0xf,
            112,
            0x8000_0008,
            8,
            0x10000,
        ]);
        let index = NativeIndex::parse(&bytes, 2).unwrap();
        assert_eq!(index.entries()[0].key.words(), [0xc, 0xa, 0xb]);
        assert_eq!(index.entries()[0].compression_type, 0xffff);
        assert_eq!(index.entries()[1].key.words(), [0xf, 0xd, 0xe]);
        assert_eq!(index.entries()[1].compression_type, 0);
        assert!(index.entries().iter().all(|entry| entry.disk_committed));
    }

    #[test]
    fn common_type_and_scope_decode_even_for_a_single_ambiguous_shape_record() {
        let index = NativeIndex::parse(
            &words(&[7, 0xa, 0xb, 0, 0xc, 96, 0x8000_0010, 16, 0x10000]),
            1,
        )
        .unwrap();
        assert_eq!(index.common_type, Some(0xa));
        assert_eq!(index.common_group, Some(0xb));
        assert_eq!(index.entries()[0].key.words(), [0xc, 0xa, 0xb]);
        let scoped = NativeIndex::parse(
            &words(&[6, 0xb, 0, 0xa, 0xc, 96, 0x8000_0010, 16, 0x10000]),
            1,
        )
        .unwrap();
        assert_eq!(scoped.entries()[0].key, index.entries()[0].key);
    }

    #[test]
    fn stored_high_bit_selects_metadata_while_absent_metadata_uses_native_size_rule() {
        let index = NativeIndex::parse(
            &words(&[
                7,
                0xa,
                0xb,
                0,
                1,
                96,
                4,
                4,
                2,
                100,
                4,
                8,
                3,
                104,
                0x8000_0004,
                4,
                0xfffe_abcd,
            ]),
            3,
        )
        .unwrap();
        assert_eq!(
            index
                .entries()
                .iter()
                .map(|entry| entry.compression_type)
                .collect::<Vec<_>>(),
            [0, 0xffff, 0xabcd]
        );
        assert_eq!(index.entries()[2].metadata_word, Some(0xfffe_abcd));
        assert!(!index.entries()[2].disk_committed);
    }

    #[test]
    fn unsupported_highs_truncations_and_excess_counts_fail_before_allocation() {
        assert_eq!(
            NativeIndex::parse(&words(&[0]), 0).unwrap_err(),
            NativeIndexError::MissingCommonHigh
        );
        assert_eq!(
            NativeIndex::parse(&words(&[4, 1]), 0).unwrap_err(),
            NativeIndexError::NonZeroInstanceHigh(1)
        );
        assert_eq!(
            NativeIndex::parse(&words(&[7, 1, 2, 0]), u32::MAX).unwrap_err(),
            NativeIndexError::RecordCountOutOfRange
        );
        let complete = words(&[7, 1, 2, 0, 3, 96, 0x8000_0004, 4, 0x10000]);
        assert!(matches!(
            NativeIndex::parse(&complete[..complete.len() - 1], 1),
            Err(NativeIndexError::Truncated { .. })
        ));
        assert!(matches!(
            NativeIndex::parse(&complete, 0),
            Err(NativeIndexError::TrailingBytes(_))
        ));
    }

    #[test]
    fn empty_indexes_unused_flags_and_ambiguous_exact_lookups_remain_explicit() {
        assert!(NativeIndex::parse(&words(&[0xffff_fff7, 1, 2, 0]), 0)
            .unwrap()
            .entries()
            .is_empty());
        let index =
            NativeIndex::parse(&words(&[7, 1, 2, 0, 3, 96, 4, 4, 3, 100, 4, 4]), 2).unwrap();
        let key = NativeResourceKey {
            instance_id: 3,
            type_id: 1,
            group_id: 2,
        };
        assert_eq!(key.scoped_instance(), 0x0000_0002_0000_0003);
        assert_eq!(
            index.find_unique(key),
            Err(NativeIndexError::DuplicateKey(key))
        );
    }
}
