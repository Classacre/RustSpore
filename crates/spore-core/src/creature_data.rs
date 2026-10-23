//! Parser for the SPORE creature editor save (type `0x0F43029A`, `.creaturedata`).
//!
//! The Python reference validator is `tools/type_0f43_validate.py`. The original
//! container layout follows SporeModder FX's `CreatureData` (no code copied).
//! Native reader/writer evidence and 1,365 shipped resources establish the full
//! 128-byte header, native RLE and checked per-part capability/morph ownership.
//!
//! ## Layout (little-endian)
//!
//! ```text
//! i32 header = 0xABB455B7
//! i32 version (9 or 10)
//! 3 counts (v9) or 4 counts (v10), i32 each
//! RLE chunk buffer, decompressed length 0x80 for versions 9 and 10:
//!     repeat: u16 chunk_size, then chunk_size bytes, decoded until 0x80 output
//!     RLE: 0xFE is an escape; next byte is a count (0 = literal 0xFE, else repeat)
//!          repeat uses the last decoded byte, including across chunk boundaries
//! counts[0] * 0x8C bytes (data0)
//! counts[1] * 4 bytes (capability ids, ASCII)
//! counts[1] * 1 byte  (capability numbers)
//! counts[2] * 4 bytes (morph ids, u32)
//! counts[2] * 4 bytes (value1 f32)
//! counts[2] * 4 bytes (value2 f32)
//! 4 bytes (field110)
//! if version >= 10: counts[3] * 0x38 bytes (data3)
//! ```

use std::fmt;

/// The little-endian file header.
pub const HEADER: u32 = 0xABB455B7;
/// Native version-9/10 serialized header span, decoded into object +0x18.
pub const HEADER_BUFFER_SIZE: usize = 0x80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureDataError {
    TooShort,
    BadHeader(u32),
    BadVersion(u32),
    Truncated,
    InvalidHeaderRle,
    InvalidCount {
        index: usize,
        count: u32,
        limit: u32,
    },
    InvalidPartRange {
        part: usize,
        array: PartArray,
        start: usize,
        count: usize,
        expected_start: usize,
        len: usize,
    },
    IncompletePartRanges {
        array: PartArray,
        covered: usize,
        len: usize,
    },
}

/// The flat array addressed by a part's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartArray {
    Capabilities,
    Morphs,
}

impl fmt::Display for CreatureDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreatureDataError::TooShort => write!(f, "creaturedata too short"),
            CreatureDataError::BadHeader(h) => write!(f, "unexpected creaturedata header {h:#010X}"),
            CreatureDataError::BadVersion(v) => write!(f, "unsupported creaturedata version {v}"),
            CreatureDataError::Truncated => write!(f, "creaturedata truncated"),
            CreatureDataError::InvalidHeaderRle => write!(f, "invalid creaturedata header RLE"),
            CreatureDataError::InvalidCount { index, count, limit } =>
                write!(f, "creaturedata count {index} is {count}, exceeding {limit}"),
            CreatureDataError::InvalidPartRange { part, array, start, count, expected_start, len } =>
                write!(f, "part {part} {array:?} range {start}+{count} does not continue at {expected_start} within {len}"),
            CreatureDataError::IncompletePartRanges { array, covered, len } =>
                write!(f, "part {array:?} ranges cover {covered} of {len} entries"),
        }
    }
}

impl std::error::Error for CreatureDataError {}

#[derive(Debug, Clone)]
pub struct CreatureCapability {
    /// Exact four source bytes, including zero padding and unrecognized values.
    pub id_bytes: [u8; 4],
    /// Legacy lossy text view; use `id_bytes` or `packed_id()` for matching.
    pub id: String,
    pub number: u8,
}

impl CreatureCapability {
    /// The capability identifier used by ANIM selectors and PCTP dictionaries.
    pub fn packed_id(&self) -> u32 {
        u32::from_le_bytes(self.id_bytes)
    }
}

#[derive(Debug, Clone)]
pub struct CreatureMorph {
    pub id: u32,
    pub value1: f32,
    pub value2: f32,
}

/// One row of the rig table in `data0`: a part, where it sits, and what it is.
///
/// The row is 0x8C bytes. Only the fields a renderer needs are named; the rest are
/// kept as `reserved` so the layout stays visible without inventing meanings.
///
/// `[MEASURED]` The 3x3 at offset 48 is a rotation: it is orthonormal (row norms 1,
/// pairwise dots 0, tolerance 0.01) on **31,704 of 31,704 rows** across the 1,022
/// creature files in `Spore_Content`. The three floats at 84 are a translation, with
/// median largest component 0.90 and maximum 17.25, which is model scale, not
/// garbage. The two words at 132 and 136 are the high and low halves of the part's
/// prop instance: every one of the 31,704 resolves to a prop in the install.
#[derive(Debug, Clone, Copy)]
pub struct CreaturePart {
    /// The part's own index in the rig, from the first `u16`.
    pub index: u16,
    /// The second `u16` of the row.
    pub link: u16,
    /// Legacy name: parent part index, or `0xFFFF` for the root.
    pub flags_a: u16,
    /// Legacy name: symmetric partner index, or `0xFFFF` for no partner.
    pub flags_b: u16,
    /// Fifth and sixth `u16`.
    pub group: u16,
    pub kind: u16,
    /// Legacy raw word at +12. This packs two range starts, not a row ordinal.
    /// Prefer `capability_start` and `morph_start`.
    pub row: u32,
    /// Legacy raw word at +16. This packs two byte counts and an unknown word.
    /// Prefer `capability_count`, `morph_count` and `range_reserved`.
    pub count: u32,
    pub capability_start: u16,
    pub morph_start: u16,
    pub capability_count: u8,
    pub morph_count: u8,
    /// Uninterpreted source word at +18.
    pub range_reserved: u16,
    /// Bounding box of the part before the transform.
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    /// Rotation, row-major 3x3.
    pub rotation: [[f32; 3]; 3],
    /// Translation applied after the rotation.
    pub translation: [f32; 3],
    /// Five floats after the translation, meaning not established.
    pub reserved_floats: [f32; 5],
    /// The prop this part refers to.
    pub part_instance: u64,
}

impl CreaturePart {
    /// Native writer stores the parent row index at +4, with `0xFFFF` as root.
    pub fn parent_index(&self) -> Option<usize> {
        (self.flags_a != u16::MAX).then_some(usize::from(self.flags_a))
    }

    /// Native writer stores the reciprocal symmetric partner at +6.
    pub fn symmetric_index(&self) -> Option<usize> {
        (self.flags_b != u16::MAX).then_some(usize::from(self.flags_b))
    }

    pub fn capability_range(&self) -> std::ops::Range<usize> {
        let start = usize::from(self.capability_start);
        start..start + usize::from(self.capability_count)
    }

    pub fn morph_range(&self) -> std::ops::Range<usize> {
        let start = usize::from(self.morph_start);
        start..start + usize::from(self.morph_count)
    }
}

/// A checked per-part view. These slices belong to this placed part, even when
/// several parts reference the same prop instance.
#[derive(Debug, Clone, Copy)]
pub struct CreaturePartView<'a> {
    pub part: &'a CreaturePart,
    pub capabilities: &'a [CreatureCapability],
    pub morphs: &'a [CreatureMorph],
    /// Complete original 0x8C-byte row, retaining unnamed fields.
    pub raw_row: &'a [u8],
}

/// Bytes per rig row.
pub const PART_ROW_SIZE: usize = 0x8C;

/// Offset of the paint palette inside the decompressed header buffer.
pub const PALETTE_OFFSET: usize = 32;

impl CreatureData {
    /// Checked access remains safe if callers edit the public arrays after parse.
    pub fn part_view(&self, index: usize) -> Option<CreaturePartView<'_>> {
        let part = self.parts.get(index)?;
        let start = index.checked_mul(PART_ROW_SIZE)?;
        Some(CreaturePartView {
            part,
            capabilities: self.capabilities.get(part.capability_range())?,
            morphs: self.morphs.get(part.morph_range())?,
            raw_row: self.data0.get(start..start.checked_add(PART_ROW_SIZE)?)?,
        })
    }

    pub fn part_capabilities(&self, index: usize) -> Option<&[CreatureCapability]> {
        self.capabilities
            .get(self.parts.get(index)?.capability_range())
    }

    pub fn part_morphs(&self, index: usize) -> Option<&[CreatureMorph]> {
        self.morphs.get(self.parts.get(index)?.morph_range())
    }

    /// The shipped rows form contiguous, non-overlapping partitions, including
    /// zero-length ranges: measured on all 46,418 rows in 1,365 resources.
    pub fn validate_part_ranges(&self) -> Result<(), CreatureDataError> {
        validate_part_ranges(&self.parts, self.capabilities.len(), self.morphs.len())
    }

    /// The creature's three paint colours, each RGB in 0..1.
    ///
    /// `[MEASURED]` The decompressed header is 0x80 bytes and holds nine floats from
    /// offset 32, which are three RGB triples. Checked against the shipped skin of the
    /// creatures that have one: across 40 such creatures, all three colours appear
    /// (within 0.08 RGB) somewhere in the skin for 30 of them and at least one for 37.
    /// They are not a flat palette for that skin, though: only a median 14 percent of
    /// its opaque pixels lie that close to a palette colour. So these are the colours
    /// the creature was authored with, and the shipped skins paint shading and
    /// intermediate tones on top of them.
    pub fn palette(&self) -> Option<[[f32; 3]; 3]> {
        let b = &self.uncompressed_buffer;
        if b.len() < PALETTE_OFFSET + 36 {
            return None;
        }
        let f = |o: usize| f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let mut out = [[0.0f32; 3]; 3];
        for (i, c) in out.iter_mut().enumerate() {
            c[0] = f(PALETTE_OFFSET + i * 12);
            c[1] = f(PALETTE_OFFSET + i * 12 + 4);
            c[2] = f(PALETTE_OFFSET + i * 12 + 8);
        }
        Some(out)
    }
}

#[derive(Debug, Clone)]
pub struct CreatureData {
    pub version: i32,
    /// Raw `counts[0]` rows (0x8C bytes each).
    pub data0: Vec<u8>,
    /// The rig: one entry per part, decoded from `data0`.
    pub parts: Vec<CreaturePart>,
    pub capabilities: Vec<CreatureCapability>,
    pub morphs: Vec<CreatureMorph>,
    pub field110: i32,
    /// Version 10 only: `counts[3]` rows of 0x38 bytes.
    pub data3: Vec<u8>,
    /// The raw RLE chunk bytes (size prefixes and data), kept verbatim.
    pub compressed_buffer: Vec<u8>,
    /// Complete 128-byte native header. Unidentified fields remain raw bytes.
    pub uncompressed_buffer: Vec<u8>,
    /// Whether the parse consumed exactly the whole buffer.
    pub exact: bool,
    /// Total bytes consumed.
    pub consumed: usize,
}

struct R<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> R<'a> {
    fn i32(&mut self) -> Result<i32, CreatureDataError> {
        if self.p + 4 > self.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let v = i32::from_le_bytes([
            self.d[self.p],
            self.d[self.p + 1],
            self.d[self.p + 2],
            self.d[self.p + 3],
        ]);
        self.p += 4;
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, CreatureDataError> {
        Ok(self.i32()? as u32)
    }
    fn u8(&mut self) -> Result<u8, CreatureDataError> {
        if self.p + 1 > self.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let v = self.d[self.p];
        self.p += 1;
        Ok(v)
    }
    fn f32(&mut self) -> Result<f32, CreatureDataError> {
        if self.p + 4 > self.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let v = f32::from_le_bytes([
            self.d[self.p],
            self.d[self.p + 1],
            self.d[self.p + 2],
            self.d[self.p + 3],
        ]);
        self.p += 4;
        Ok(v)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], CreatureDataError> {
        if self.p + n > self.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let s = &self.d[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
}

/// Read the RLE chunk buffer, returning (raw bytes, decompressed bytes).
///
/// The stream is a sequence of `u16 chunk_size` followed by `chunk_size` bytes, read
/// until the output reaches the expected length. Inside a chunk, a byte `0xFE` is an
/// escape: the next byte is a count, where 0 emits a literal `0xFE` and anything else
/// repeats the last emitted byte that many times. Any other byte is emitted as itself.
///
/// `[MEASURED]` The previous version of this function emitted `current[0]` for every
/// literal and never advanced its index, so the "decompressed" buffer was one byte
/// repeated. Nothing consumed it, which is why that went unnoticed until the paint
/// palette was read out of it and came back as -8e18.
fn read_compressed(r: &mut R<'_>, length: usize) -> Result<(Vec<u8>, Vec<u8>), CreatureDataError> {
    let mut raw: Vec<u8> = Vec::new();
    let mut dst: Vec<u8> = Vec::with_capacity(length);
    let mut last_byte = 0u8;

    while dst.len() < length {
        if r.p + 2 > r.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let size = u16::from_le_bytes([r.d[r.p], r.d[r.p + 1]]) as usize;
        r.p += 2;
        if size < 1 || size > 0x800 || r.p + size > r.d.len() {
            return Err(CreatureDataError::Truncated);
        }
        let chunk = &r.d[r.p..r.p + size];
        raw.extend_from_slice(&(size as u16).to_le_bytes());
        raw.extend_from_slice(chunk);
        r.p += size;

        let mut i = 0usize;
        while i < chunk.len() && dst.len() < length {
            let b = chunk[i];
            i += 1;
            if b != 0xFE {
                dst.push(b);
                last_byte = b;
                continue;
            }
            if i >= chunk.len() {
                return Err(CreatureDataError::InvalidHeaderRle);
            }
            let count = chunk[i];
            i += 1;
            if count == 0 {
                dst.push(0xFE);
                last_byte = 0xFE;
            } else {
                if usize::from(count) > length - dst.len() {
                    return Err(CreatureDataError::InvalidHeaderRle);
                }
                for _ in 0..count {
                    dst.push(last_byte);
                }
            }
        }
    }

    Ok((raw, dst))
}

/// Parse a `.creaturedata` resource.
pub fn parse(data: &[u8]) -> Result<CreatureData, CreatureDataError> {
    if data.len() < 8 {
        return Err(CreatureDataError::TooShort);
    }
    let mut r = R { d: data, p: 0 };
    let header = r.u32()?;
    if header != HEADER {
        return Err(CreatureDataError::BadHeader(header));
    }
    let version = r.i32()?;
    if version < 9 || version > 10 {
        return Err(CreatureDataError::BadVersion(version as u32));
    }
    let cc = if version < 10 { 3 } else { 4 };
    let mut counts = Vec::with_capacity(cc);
    for index in 0..cc {
        let count = r.u32()?;
        // Native 0x4C0BB0 guards these three arrays before allocation. The
        // version-10 fourth array retains the existing defensive size cap.
        let limit = [256, 4096, 1024, 1_000_000][index];
        if count > limit {
            return Err(CreatureDataError::InvalidCount {
                index,
                count,
                limit,
            });
        }
        counts.push(count as usize);
    }

    let (compressed_buffer, uncompressed_buffer) = read_compressed(&mut r, HEADER_BUFFER_SIZE)?;

    let data0 = r.take(counts[0] * 0x8C)?.to_vec();

    let mut cap_ids = Vec::with_capacity(counts[1]);
    for _ in 0..counts[1] {
        let b = r.take(4)?;
        cap_ids.push(<[u8; 4]>::try_from(b).unwrap());
    }
    let mut capabilities = Vec::with_capacity(counts[1]);
    for id_bytes in cap_ids {
        capabilities.push(CreatureCapability {
            id: String::from_utf8_lossy(&id_bytes).into_owned(),
            id_bytes,
            number: r.u8()?,
        });
    }

    let mut morph_ids = Vec::with_capacity(counts[2]);
    for _ in 0..counts[2] {
        morph_ids.push(r.u32()?);
    }
    let mut values1 = Vec::with_capacity(counts[2]);
    for _ in 0..counts[2] {
        values1.push(r.f32()?);
    }
    let mut values2 = Vec::with_capacity(counts[2]);
    for _ in 0..counts[2] {
        values2.push(r.f32()?);
    }
    let mut morphs = Vec::with_capacity(counts[2]);
    for i in 0..counts[2] {
        morphs.push(CreatureMorph {
            id: morph_ids[i],
            value1: values1[i],
            value2: values2[i],
        });
    }

    let field110 = r.i32()?;
    let data3 = if version >= 10 {
        r.take(counts[3] * 0x38)?.to_vec()
    } else {
        Vec::new()
    };

    let parts = decode_parts(&data0);
    validate_part_ranges(&parts, capabilities.len(), morphs.len())?;

    Ok(CreatureData {
        version,
        data0,
        parts,
        capabilities,
        morphs,
        field110,
        data3,
        compressed_buffer,
        uncompressed_buffer,
        exact: r.p == data.len(),
        consumed: r.p,
    })
}

/// Decode the rig rows. A trailing partial row is ignored rather than panicking.
fn decode_parts(data0: &[u8]) -> Vec<CreaturePart> {
    data0
        .chunks_exact(PART_ROW_SIZE)
        .map(|r| {
            let f = |o: usize| f32::from_le_bytes([r[o], r[o + 1], r[o + 2], r[o + 3]]);
            let u16_at = |o: usize| u16::from_le_bytes([r[o], r[o + 1]]);
            let u32_at = |o: usize| u32::from_le_bytes([r[o], r[o + 1], r[o + 2], r[o + 3]]);
            let mut rotation = [[0.0f32; 3]; 3];
            for (row, slot) in rotation.iter_mut().enumerate() {
                for (col, v) in slot.iter_mut().enumerate() {
                    *v = f(48 + (row * 3 + col) * 4);
                }
            }
            CreaturePart {
                index: u16_at(0),
                link: u16_at(2),
                flags_a: u16_at(4),
                flags_b: u16_at(6),
                group: u16_at(8),
                kind: u16_at(10),
                row: u32_at(12),
                count: u32_at(16),
                capability_start: u16_at(12),
                morph_start: u16_at(14),
                capability_count: r[16],
                morph_count: r[17],
                range_reserved: u16_at(18),
                bounds_min: [f(20), f(24), f(28)],
                bounds_max: [f(32), f(36), f(40)],
                rotation,
                translation: [f(84), f(88), f(92)],
                reserved_floats: [f(108), f(112), f(116), f(120), f(124)],
                part_instance: ((u32_at(132) as u64) << 32) | u32_at(136) as u64,
            }
        })
        .collect()
}

fn validate_part_ranges(
    parts: &[CreaturePart],
    capabilities: usize,
    morphs: usize,
) -> Result<(), CreatureDataError> {
    for (array, len) in [
        (PartArray::Capabilities, capabilities),
        (PartArray::Morphs, morphs),
    ] {
        let mut expected_start = 0;
        for (part, row) in parts.iter().enumerate() {
            let range = match array {
                PartArray::Capabilities => row.capability_range(),
                PartArray::Morphs => row.morph_range(),
            };
            if range.start != expected_start || range.end > len {
                return Err(CreatureDataError::InvalidPartRange {
                    part,
                    array,
                    start: range.start,
                    count: range.len(),
                    expected_start,
                    len,
                });
            }
            expected_start = range.end;
        }
        if expected_start != len {
            return Err(CreatureDataError::IncompletePartRanges {
                array,
                covered: expected_start,
                len,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_i32(v: &mut Vec<u8>, x: i32) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    /// Build a version-9 file with a simple RLE chunk (all literals), one
    /// capability and one morph.
    fn build_v9() -> Vec<u8> {
        let mut d = Vec::new();
        push_i32(&mut d, HEADER as i32);
        push_i32(&mut d, 9);
        push_i32(&mut d, 1); // counts[0] data0 rows
        push_i32(&mut d, 1); // counts[1] capabilities
        push_i32(&mut d, 1); // counts[2] morphs
                             // One complete native header chunk, containing no escape bytes.
        let chunk = vec![0x11u8; HEADER_BUFFER_SIZE];
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);
        // data0
        let mut row = [0u8; PART_ROW_SIZE];
        row[16] = 1; // one capability on this part
        row[17] = 1; // one morph on this part
        d.extend_from_slice(&row);
        // capability id + number
        d.extend_from_slice(b"abcd");
        d.push(7);
        // morph id + two floats
        push_i32(&mut d, 0x1234_5678);
        d.extend_from_slice(&1.5f32.to_le_bytes());
        d.extend_from_slice(&2.5f32.to_le_bytes());
        // field110
        push_i32(&mut d, 0x99);
        d
    }

    #[test]
    fn parse_v9() {
        let d = build_v9();
        let c = parse(&d).unwrap();
        assert_eq!(c.version, 9);
        assert_eq!(c.data0.len(), 0x8C);
        assert_eq!(c.capabilities.len(), 1);
        assert_eq!(c.capabilities[0].id, "abcd");
        assert_eq!(c.capabilities[0].number, 7);
        assert_eq!(c.morphs.len(), 1);
        assert_eq!(c.morphs[0].id, 0x1234_5678);
        assert!((c.morphs[0].value1 - 1.5).abs() < 1e-6);
        assert_eq!(c.field110, 0x99);
        assert!(c.exact);
        assert_eq!(c.consumed, d.len());
    }

    fn ranged_fixture() -> Vec<u8> {
        let single = build_v9();
        let row_at = 20 + 2 + HEADER_BUFFER_SIZE;
        let mut d = single[..row_at].to_vec();
        d[8..12].copy_from_slice(&3i32.to_le_bytes());
        for (index, cap_start, morph_start, cap_count, morph_count, parent, symmetric) in [
            (0u16, 0u16, 0u16, 1u8, 0u8, u16::MAX, u16::MAX),
            (1, 1, 0, 0, 1, 0, 2),
            (2, 1, 1, 0, 0, 0, 1),
        ] {
            let mut row = [0u8; PART_ROW_SIZE];
            row[0..2].copy_from_slice(&index.to_le_bytes());
            row[4..6].copy_from_slice(&parent.to_le_bytes());
            row[6..8].copy_from_slice(&symmetric.to_le_bytes());
            row[12..14].copy_from_slice(&cap_start.to_le_bytes());
            row[14..16].copy_from_slice(&morph_start.to_le_bytes());
            row[16] = cap_count;
            row[17] = morph_count;
            row[18..20].copy_from_slice(&0xA55Au16.to_le_bytes());
            row[96..100].copy_from_slice(&0xCAFEBABEu32.to_le_bytes());
            d.extend_from_slice(&row);
        }
        d.extend_from_slice(&single[row_at + PART_ROW_SIZE..]);
        d
    }

    #[test]
    fn placed_parts_own_distinct_flat_array_ranges() {
        let c = parse(&ranged_fixture()).unwrap();
        assert!(c.validate_part_ranges().is_ok());
        assert_eq!(
            c.part_capabilities(0).unwrap()[0].packed_id(),
            u32::from_le_bytes(*b"abcd")
        );
        assert!(c.part_morphs(0).unwrap().is_empty());
        assert!(c.part_capabilities(1).unwrap().is_empty());
        assert_eq!(c.part_morphs(1).unwrap()[0].id, 0x12345678);
        assert!(c.part_morphs(2).unwrap().is_empty());
        assert_eq!(c.parts[0].parent_index(), None);
        assert_eq!(c.parts[1].parent_index(), Some(0));
        assert_eq!(c.parts[1].symmetric_index(), Some(2));
        assert_eq!(c.parts[2].symmetric_index(), Some(1));
        let view = c.part_view(1).unwrap();
        assert_eq!(view.part.row, 1); // two packed u16 starts
        assert_eq!(view.part.count, 0xA55A0100);
        assert_eq!(view.part.range_reserved, 0xA55A);
        assert_eq!(&view.raw_row[96..100], &0xCAFEBABEu32.to_le_bytes());
        assert!(c.part_view(3).is_none());
    }

    #[test]
    fn retains_capability_bytes_that_are_not_utf8() {
        let mut d = build_v9();
        let cap_at = 20 + 2 + HEADER_BUFFER_SIZE + PART_ROW_SIZE;
        d[cap_at..cap_at + 4].copy_from_slice(&[0xFF, b'a', 0, 0x80]);
        let c = parse(&d).unwrap();
        assert_eq!(c.capabilities[0].id_bytes, [0xFF, b'a', 0, 0x80]);
        assert_eq!(c.capabilities[0].packed_id(), 0x800061FF);
    }

    #[test]
    fn rejects_overlapping_gapped_and_out_of_bounds_part_ranges() {
        let row_at = 20 + 2 + HEADER_BUFFER_SIZE;
        for (part, offset, value, array) in [
            (1, 12, 0u16, PartArray::Capabilities), // overlap
            (1, 12, 2, PartArray::Capabilities),    // gap
            (0, 12, u16::MAX, PartArray::Capabilities),
            (1, 14, u16::MAX, PartArray::Morphs),
        ] {
            let mut d = ranged_fixture();
            let at = row_at + part * PART_ROW_SIZE + offset;
            d[at..at + 2].copy_from_slice(&value.to_le_bytes());
            assert!(
                matches!(parse(&d), Err(CreatureDataError::InvalidPartRange { part: found, array: found_array, .. }) if found == part && found_array == array)
            );
        }
        let mut d = build_v9();
        d[row_at + 16] = u8::MAX;
        assert!(matches!(
            parse(&d),
            Err(CreatureDataError::InvalidPartRange {
                array: PartArray::Capabilities,
                ..
            })
        ));
    }

    #[test]
    fn rejects_unassigned_flat_array_entries() {
        let row_at = 20 + 2 + HEADER_BUFFER_SIZE;
        for (offset, array) in [(16, PartArray::Capabilities), (17, PartArray::Morphs)] {
            let mut d = build_v9();
            d[row_at + offset] = 0;
            assert!(
                matches!(parse(&d), Err(CreatureDataError::IncompletePartRanges { array: found, covered: 0, len: 1 }) if found == array)
            );
        }
    }

    #[test]
    fn checked_views_handle_public_array_mutation() {
        let mut c = parse(&ranged_fixture()).unwrap();
        c.capabilities.clear();
        assert!(c.part_capabilities(0).is_none());
        assert!(c.part_view(0).is_none());
        c.parts[1].morph_start = u16::MAX;
        assert!(c.part_morphs(1).is_none());
        c.data0.clear();
        assert!(c.part_view(2).is_none());
    }

    #[test]
    fn bad_header() {
        let mut d = build_v9();
        d[0] = 0;
        assert!(matches!(parse(&d), Err(CreatureDataError::BadHeader(_))));
    }

    #[test]
    fn bad_version() {
        let mut d = build_v9();
        d[4] = 7;
        assert!(matches!(parse(&d), Err(CreatureDataError::BadVersion(_))));
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 4]), Err(CreatureDataError::TooShort)));
    }

    #[test]
    fn truncated() {
        let d = build_v9();
        let cut = &d[..d.len() - 2];
        assert!(matches!(parse(cut), Err(CreatureDataError::Truncated)));
    }

    /// A rig row decodes to a rotation, a translation and a part instance. The
    /// rotation is orthonormal in every real row, so a row that decodes to something
    /// else means the offsets have moved.
    #[test]
    fn decodes_a_rig_row() {
        let mut d = Vec::new();
        // header: magic, version 9, three counts, then the RLE chunk buffer
        push_i32(&mut d, HEADER as i32);
        push_i32(&mut d, 9);
        push_i32(&mut d, 1); // one rig row
        push_i32(&mut d, 0); // no capabilities
        push_i32(&mut d, 0); // no morphs
        let chunk = vec![0x11u8; HEADER_BUFFER_SIZE];
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);

        // one 0x8C row: identity rotation, translation (1, 2, 3), part instance
        let mut row = vec![0u8; PART_ROW_SIZE];
        row[0..2].copy_from_slice(&7u16.to_le_bytes()); // index
        row[2..4].copy_from_slice(&9u16.to_le_bytes()); // link
        let mut put = |at: usize, v: f32| {
            row[at..at + 4].copy_from_slice(&v.to_le_bytes());
        };
        put(48, 1.0);
        put(52, 0.0);
        put(56, 0.0);
        put(60, 0.0);
        put(64, 1.0);
        put(68, 0.0);
        put(72, 0.0);
        put(76, 0.0);
        put(80, 1.0);
        put(84, 1.0);
        put(88, 2.0);
        put(92, 3.0);
        row[132..136].copy_from_slice(&0x4066_6000u32.to_le_bytes()); // instance high
        row[136..140].copy_from_slice(&0xE6B1_310Au32.to_le_bytes()); // instance low
        d.extend_from_slice(&row);
        push_i32(&mut d, 0); // field110, always present

        let c = parse(&d).unwrap();
        assert!(c.exact, "fixture must consume exactly");
        assert_eq!(c.parts.len(), 1);
        let p = c.parts[0];
        assert_eq!(p.index, 7);
        assert_eq!(p.link, 9);
        assert_eq!(p.rotation[0], [1.0, 0.0, 0.0]);
        assert_eq!(p.rotation[1], [0.0, 1.0, 0.0]);
        assert_eq!(p.rotation[2], [0.0, 0.0, 1.0]);
        assert_eq!(p.translation, [1.0, 2.0, 3.0]);
        assert_eq!(p.part_instance, 0x4066_6000_E6B1_310A);
    }

    /// A trailing partial row is dropped rather than read past the end.
    #[test]
    fn ignores_a_partial_rig_row() {
        let mut d = Vec::new();
        push_i32(&mut d, HEADER as i32);
        push_i32(&mut d, 9);
        push_i32(&mut d, 1);
        push_i32(&mut d, 0);
        push_i32(&mut d, 0);
        let chunk = vec![0x11u8; HEADER_BUFFER_SIZE];
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);
        d.extend_from_slice(&vec![0u8; PART_ROW_SIZE + 10]);
        push_i32(&mut d, 0); // field110, always present

        let c = parse(&d).unwrap();
        assert_eq!(c.parts.len(), 1, "the ten trailing bytes are not a row");
    }

    /// A chunk with a repeat escape decodes to the right bytes. The previous decoder
    /// emitted the chunk's first byte for every literal and never advanced, so this is
    /// the test that would have caught it.
    #[test]
    fn rle_decodes_literals_and_repeats() {
        let mut d = Vec::new();
        push_i32(&mut d, HEADER as i32);
        push_i32(&mut d, 9);
        push_i32(&mut d, 0); // no rig rows
        push_i32(&mut d, 0);
        push_i32(&mut d, 0);
        // A B FE 02 emits four bytes, followed by enough C literals to fill
        // the complete 128-byte native header.
        let mut chunk = vec![0x41u8, 0x42, 0xFE, 0x02];
        chunk.extend(std::iter::repeat(0x43u8).take(HEADER_BUFFER_SIZE - 4));
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);
        push_i32(&mut d, 0); // field110

        let c = parse(&d).unwrap();
        let out = &c.uncompressed_buffer;
        assert_eq!(out.len(), HEADER_BUFFER_SIZE);
        assert_eq!(&out[..5], &[0x41, 0x42, 0x42, 0x42, 0x43]);
        assert_eq!(out[HEADER_BUFFER_SIZE - 1], 0x43);
    }

    /// Nine floats at offset 32 of the decompressed header are the paint palette.
    #[test]
    fn reads_the_paint_palette() {
        let mut d = Vec::new();
        push_i32(&mut d, HEADER as i32);
        push_i32(&mut d, 9);
        push_i32(&mut d, 0);
        push_i32(&mut d, 0);
        push_i32(&mut d, 0);
        // A chunk of literals holding the palette at offset 32.
        let mut chunk = vec![0u8; HEADER_BUFFER_SIZE];
        let colours = [
            0.25f32, 0.5, 0.75, //
            0.1, 0.2, 0.3, //
            0.9, 0.8, 0.7,
        ];
        for (i, v) in colours.iter().enumerate() {
            let at = PALETTE_OFFSET + i * 4;
            chunk[at..at + 4].copy_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);
        push_i32(&mut d, 0);

        let c = parse(&d).unwrap();
        let p = c.palette().expect("palette");
        assert_eq!(p[0], [0.25, 0.5, 0.75]);
        assert_eq!(p[1], [0.1, 0.2, 0.3]);
        assert_eq!(p[2], [0.9, 0.8, 0.7]);
    }

    fn empty_with_chunks(version: i32, chunks: &[&[u8]]) -> Vec<u8> {
        let mut data = Vec::new();
        for value in [HEADER as i32, version, 0, 0, 0] {
            push_i32(&mut data, value);
        }
        if version == 10 {
            push_i32(&mut data, 0);
        }
        for chunk in chunks {
            data.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
            data.extend_from_slice(chunk);
        }
        push_i32(&mut data, 0);
        data
    }

    #[test]
    fn both_native_versions_preserve_the_full_header_tail_and_literal_escape() {
        let mut expected = [0u8; HEADER_BUFFER_SIZE];
        expected[68] = 0xAB;
        expected[93] = 0xFE;
        expected[127] = 0xCD;
        let mut encoded = Vec::new();
        for byte in expected {
            encoded.push(byte);
            if byte == 0xFE {
                encoded.push(0);
            }
        }
        for version in [9, 10] {
            let source = empty_with_chunks(version, &[&encoded]);
            let creature = parse(&source).unwrap();
            assert_eq!(creature.uncompressed_buffer, expected);
            assert_eq!(creature.compressed_buffer.len(), encoded.len() + 2);
            assert_eq!(creature.consumed, source.len());
            assert!(creature.exact);
        }
    }

    #[test]
    fn native_repeat_state_uses_the_last_literal_across_chunk_boundaries() {
        let source = empty_with_chunks(9, &[&[0x11, 0x22], &[0xFE, 126]]);
        let creature = parse(&source).unwrap();
        assert_eq!(creature.uncompressed_buffer[0], 0x11);
        assert_eq!(&creature.uncompressed_buffer[1..], &[0x22; 127]);
        assert!(creature.exact);
    }

    #[test]
    fn malformed_native_rle_and_every_truncated_prefix_are_rejected() {
        for chunks in [
            vec![&[0xFE][..], &[0x11; 128][..]],
            vec![&[0xFE, 129][..]],
            vec![&[0x11, 0xFE, 128][..]],
        ] {
            assert_eq!(
                parse(&empty_with_chunks(9, &chunks)).unwrap_err(),
                CreatureDataError::InvalidHeaderRle
            );
        }
        let complete = empty_with_chunks(10, &[&[0x11, 0xFE, 127]]);
        for end in 0..complete.len() {
            assert!(parse(&complete[..end]).is_err(), "prefix {end}");
        }
        assert!(parse(&complete).unwrap().exact);
    }

    #[test]
    fn native_array_count_guards_reject_before_header_reads() {
        for (index, limit) in [256u32, 4096, 1024].into_iter().enumerate() {
            let mut source = vec![0u8; 20];
            source[..4].copy_from_slice(&HEADER.to_le_bytes());
            source[4..8].copy_from_slice(&9u32.to_le_bytes());
            source[8 + index * 4..12 + index * 4].copy_from_slice(&limit.to_le_bytes());
            assert_eq!(parse(&source).unwrap_err(), CreatureDataError::Truncated);
            source[8 + index * 4..12 + index * 4].copy_from_slice(&(limit + 1).to_le_bytes());
            assert_eq!(
                parse(&source).unwrap_err(),
                CreatureDataError::InvalidCount {
                    index,
                    count: limit + 1,
                    limit,
                }
            );
        }
    }
}
