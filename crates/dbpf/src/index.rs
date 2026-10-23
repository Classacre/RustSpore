//! Resource index layout detection.
//!
//! Historical compatibility view: its group/prefix/trailer labels do not
//! describe native resource identity. See `native_index` for the flag-driven
//! reader recovered from8DAFE0 and `re/archive-index-native.md` for the correction.
//!
//! The record shape is **not** constant across retail packages. Three variants were
//! measured in one install, and they differ in record size, in where the prefix and
//! trailer sit, and in field order. See `docs/dbpf-format.md` section 6 for the evidence.
//!
//! The detection method is the one that was validated against all 13 packages:
//!
//! 1. Exact fit. A candidate must consume the index block exactly:
//!    `prefix + count * record_size + trailer == index_size`.
//!    This narrows the candidates but does not decide between them, because for one
//!    real package both `(prefix 16, record 20)` and `(prefix 32, record 16)` satisfy it.
//! 2. Contiguity. Payloads are packed, so when records are sorted by offset each
//!    resource should end where the next begins: `offset_i + size_i == offset_{i+1}`.
//!    The candidate with the best contiguity score wins. On the real packages the
//!    correct shape scores 100% and the wrong one scores far lower.

use crate::{DbpfError, ResourceEntry, ResourceKey};

/// Which field layout a record uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// 28-byte record, the common case in 10 of 13 packages.
    /// `group, type, instance_hi, instance_lo, offset, stored_size, decompressed_size`
    Wide28,
    /// 24-byte record, seen in `Spore_Audio1.package`.
    /// `type, instance, offset, stored_size, decompressed_size, group`
    Audio24,
    /// 20-byte record, seen in the two EP1 locale packages.
    /// `hash, offset, stored_size, decompressed_size, group`
    Locale20,
}

/// A complete index record shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordShape {
    /// Bytes before the first record.
    pub prefix_size: usize,
    /// Bytes per record.
    pub record_size: usize,
    /// Bytes after the last record.
    pub trailer_size: usize,
    /// Which field layout applies.
    pub kind: RecordKind,
}

impl RecordShape {
    /// The shapes observed in retail packages, in preference order.
    pub const CANDIDATES: [RecordShape; 3] = [
        RecordShape { prefix_size: 4, record_size: 28, trailer_size: 4, kind: RecordKind::Wide28 },
        RecordShape { prefix_size: 12, record_size: 24, trailer_size: 0, kind: RecordKind::Audio24 },
        RecordShape { prefix_size: 16, record_size: 20, trailer_size: 0, kind: RecordKind::Locale20 },
    ];

    /// Total bytes this shape needs for `count` records.
    pub fn block_size(&self, count: u32) -> u64 {
        (self.prefix_size + self.trailer_size) as u64 + count as u64 * self.record_size as u64
    }

    /// Whether the shape consumes the index block exactly.
    pub fn fits_exactly(&self, count: u32, index_size: u32) -> bool {
        self.block_size(count) == u64::from(index_size)
    }
}

/// Parse the records of one shape out of `bytes`.
pub fn parse_records(
    bytes: &[u8],
    index_offset: u32,
    count: u32,
    shape: RecordShape,
) -> Option<Vec<ResourceEntry>> {
    let base = index_offset as usize + shape.prefix_size;
    let end = base + count as usize * shape.record_size;
    if end > bytes.len() {
        return None;
    }

    let mut entries = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let at = base + i * shape.record_size;
        let u = |o: usize| -> u32 {
            u32::from_le_bytes(bytes[at + o..at + o + 4].try_into().expect("bounds checked"))
        };

        // Field order differs per kind, so each is spelled out rather than
        // parameterised, to keep the mapping readable against the spec.
        let (key, offset, stored_size, decompressed_size) = match shape.kind {
            RecordKind::Wide28 => (
                ResourceKey {
                    group_id: u(0),
                    type_id: u(4),
                    instance_id: ((u(8) as u64) << 32) | u(12) as u64,
                },
                u(16),
                u(20) & 0x7FFF_FFFF,
                u(24),
            ),
            RecordKind::Audio24 => (
                ResourceKey {
                    type_id: u(0),
                    group_id: u(20),
                    instance_id: u(4) as u64,
                },
                u(8),
                u(12) & 0x7FFF_FFFF,
                u(16),
            ),
            RecordKind::Locale20 => (
                ResourceKey {
                    // This variant carries no type id; the leading word is
                    // treated as the instance. Unconfirmed, see the spec.
                    type_id: 0,
                    group_id: u(16),
                    instance_id: u(0) as u64,
                },
                u(4),
                u(8) & 0x7FFF_FFFF,
                u(12),
            ),
        };

        entries.push(ResourceEntry {
            key,
            offset,
            stored_size,
            decompressed_size,
        });
    }
    Some(entries)
}

/// Fraction of consecutive offset-sorted records that are exactly contiguous.
///
/// Payloads are packed back to back, so a correct parse scores close to 1.0 and a
/// mis-parsed record layout scores much lower.
pub fn contiguity_score(entries: &[ResourceEntry], file_len: u64) -> f64 {
    if entries.len() < 2 {
        return 0.0;
    }
    let mut spans: Vec<(u64, u64)> = entries
        .iter()
        .filter(|e| {
            let start = u64::from(e.offset);
            let end = start + u64::from(e.stored_size);
            e.stored_size > 0 && start >= 0x60 && end <= file_len
        })
        .map(|e| (u64::from(e.offset), u64::from(e.stored_size)))
        .collect();
    if spans.len() < 2 {
        return 0.0;
    }
    spans.sort_unstable();
    let chains = spans
        .windows(2)
        .filter(|w| w[0].0 + w[0].1 == w[1].0)
        .count();
    chains as f64 / (spans.len() - 1) as f64
}

/// Choose the index shape for a package.
///
/// Returns the shape and the parsed records. Errors when no candidate fits, or when
/// none of the fitting candidates parses to something plausibly contiguous.
pub fn detect(
    bytes: &[u8],
    index_offset: u32,
    index_size: u32,
    count: u32,
) -> Result<(RecordShape, Vec<ResourceEntry>), DbpfError> {
    detect_with_len(bytes, index_offset, index_size, count, bytes.len())
}

/// As [`detect`], for a buffer that holds only the index block.
///
/// Resource offsets are absolute file offsets, so the contiguity score needs the
/// real file length rather than the length of the slice being parsed. Passing the
/// slice's own length scores every candidate as out of range and rejects layouts
/// that are perfectly valid.
pub fn detect_with_len(
    bytes: &[u8],
    index_offset: u32,
    index_size: u32,
    count: u32,
    file_len: usize,
) -> Result<(RecordShape, Vec<ResourceEntry>), DbpfError> {
    let mut best: Option<(f64, RecordShape, Vec<ResourceEntry>)> = None;

    for shape in RecordShape::CANDIDATES {
        if !shape.fits_exactly(count, index_size) {
            continue;
        }
        let Some(entries) = parse_records(bytes, index_offset, count, shape) else {
            continue;
        };
        let score = contiguity_score(&entries, file_len as u64);
        let better = match &best {
            None => true,
            Some((best_score, _, _)) => score > *best_score,
        };
        if better {
            best = Some((score, shape, entries));
        }
    }

    match best {
        // Contiguity is the arbiter between shapes that all fit the block, but it
        // is undefined for a single record: a one-resource package would score 0.0
        // and be rejected even when the shape fits the block exactly. Fall back to
        // the fit itself in that case.
        Some((score, shape, entries)) if score > 0.0 || entries.len() < 2 => {
            Ok((shape, entries))
        }
        _ => Err(DbpfError::UnsupportedIndexLayout {
            count,
            needed: RecordShape::CANDIDATES[0].block_size(count),
            available: u64::from(index_size),
            index_offset,
            index_size,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape_of(kind: RecordKind) -> RecordShape {
        RecordShape::CANDIDATES
            .iter()
            .copied()
            .find(|s| s.kind == kind)
            .expect("candidate exists")
    }

    #[test]
    fn measured_block_sizes_match_the_real_packages() {
        // 4 + 11529*28 + 4 = 322820, the measured index size of Spore_Game.package.
        let wide = shape_of(RecordKind::Wide28);
        assert_eq!(wide.block_size(11_529), 322_820);
        assert!(wide.fits_exactly(11_529, 322_820));

        // 12 + 17651*24 = 423636, the measured index size of Spore_Audio1.package.
        let audio = shape_of(RecordKind::Audio24);
        assert_eq!(audio.block_size(17_651), 423_636);
        assert!(audio.fits_exactly(17_651, 423_636));

        // 16 + 4*20 = 96, the measured index size of Spore_EP1_Locale_02.package.
        let locale = shape_of(RecordKind::Locale20);
        assert_eq!(locale.block_size(4), 96);
        assert!(locale.fits_exactly(4, 96));

        // The invariant measured across all 13 packages.
        for s in RecordShape::CANDIDATES {
            assert_eq!(
                s.prefix_size + s.record_size + s.trailer_size,
                36,
                "prefix + record + trailer should be 36"
            );
        }
    }

    #[test]
    fn contiguity_detects_a_packed_sequence() {
        let entries = vec![
            ResourceEntry { key: ResourceKey { type_id: 1, group_id: 1, instance_id: 1 },
                            offset: 0x60, stored_size: 0x10, decompressed_size: 0x10 },
            ResourceEntry { key: ResourceKey { type_id: 1, group_id: 1, instance_id: 2 },
                            offset: 0x70, stored_size: 0x20, decompressed_size: 0x20 },
            ResourceEntry { key: ResourceKey { type_id: 1, group_id: 1, instance_id: 3 },
                            offset: 0x90, stored_size: 0x08, decompressed_size: 0x08 },
        ];
        assert_eq!(contiguity_score(&entries, 0x1000), 1.0);
    }

    #[test]
    fn contiguity_is_low_for_a_broken_sequence() {
        let entries = vec![
            ResourceEntry { key: ResourceKey { type_id: 1, group_id: 1, instance_id: 1 },
                            offset: 0x60, stored_size: 0x10, decompressed_size: 0x10 },
            ResourceEntry { key: ResourceKey { type_id: 1, group_id: 1, instance_id: 2 },
                            offset: 0x900, stored_size: 0x20, decompressed_size: 0x20 },
        ];
        assert_eq!(contiguity_score(&entries, 0x1000), 0.0);
    }
}
