//! Parser for the SPORE creature body editor model (type `0x1A99B06B`, `.bem`).
//!
//! Status: **100% validated (1,733 of 1,733).** The Python reference validator is
//! `tools/type_1a99_validate.py`. Layout follows SporeModder's `BEMMain`
//! (reimplemented; no code copied).
//!
//! ## Layout (little-endian)
//!
//! ```text
//! u32 magic = 0x48657ED3
//! u32 version        (3 < version <= 5)
//! u32 unk1           (part count, <= 1024)
//! u32 unk2           (<= 128)
//! RLE chunk buffer, decompressed length 128 (version >= 5) or 68
//! RLE chunk buffer, decompressed length unk1 * 472
//! ```
//!
//! RLE: repeat `u16 chunk_size` (1..=2048) then `chunk_size` bytes. Inside a chunk
//! byte `0xFE` is an escape; the next byte is a count (0 = a literal `0xFE`, else
//! repeat the previous byte that many times). Chunks are read until the requested
//! output length is reached.
//!
//! `[MEASURED]` Every file in the install is version 5, and every one needs the
//! 128-byte first section. Decompressing 68 bytes instead still resynchronises on
//! the next chunk boundary, so a 68-byte read looks plausible on 1,128 samples and
//! fails on the other 605. 128 is the size that holds for the whole population.

use std::fmt;

/// The little-endian file magic.
pub const MAGIC: u32 = 0x4865_7ED3;

const MAX_CHUNK: usize = 0x800;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BemError {
    TooShort,
    BadMagic(u32),
    BadVersion(u32),
    BadHeaderField,
    Truncated,
}

impl fmt::Display for BemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BemError::TooShort => write!(f, "bem too short"),
            BemError::BadMagic(m) => write!(f, "unexpected bem magic {m:#010X}"),
            BemError::BadVersion(v) => write!(f, "unsupported bem version {v}"),
            BemError::BadHeaderField => write!(f, "bem header field out of range"),
            BemError::Truncated => write!(f, "bem truncated"),
        }
    }
}

impl std::error::Error for BemError {}

/// One decompressed RLE section, with the raw chunk bytes it came from.
#[derive(Debug, Clone, Default)]
pub struct BemSection {
    /// The raw chunk bytes (size prefixes and data), kept verbatim.
    pub compressed: Vec<u8>,
    /// The decompressed bytes. Informational; the format keeps the raw form.
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Bem {
    pub version: u32,
    /// Part count. Scales the second section (`unk1 * 472` bytes).
    pub unk1: u32,
    pub unk2: u32,
    /// The 128-byte (version 5) header section.
    pub header_section: BemSection,
    /// The `unk1 * 472`-byte part section.
    pub parts_section: BemSection,
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
    fn u32(&mut self) -> Result<u32, BemError> {
        if self.p + 4 > self.d.len() {
            return Err(BemError::Truncated);
        }
        let v = u32::from_le_bytes([
            self.d[self.p],
            self.d[self.p + 1],
            self.d[self.p + 2],
            self.d[self.p + 3],
        ]);
        self.p += 4;
        Ok(v)
    }
}

/// Read one RLE chunk buffer producing exactly `length` decompressed bytes.
fn read_rle(r: &mut R<'_>, length: usize) -> Result<BemSection, BemError> {
    let mut section = BemSection {
        compressed: Vec::new(),
        data: vec![0u8; length],
    };
    let mut produced = 0usize;

    while produced < length {
        if r.p + 2 > r.d.len() {
            return Err(BemError::Truncated);
        }
        let size = u16::from_le_bytes([r.d[r.p], r.d[r.p + 1]]) as usize;
        r.p += 2;
        if size < 1 || size > MAX_CHUNK || r.p + size > r.d.len() {
            return Err(BemError::Truncated);
        }
        let chunk = &r.d[r.p..r.p + size];
        r.p += size;
        section.compressed.extend_from_slice(&(size as u16).to_le_bytes());
        section.compressed.extend_from_slice(chunk);

        // The SporeModder quirk: a run repeats the chunk's first byte, not the
        // byte immediately preceding the escape.
        let last_byte = chunk[0];

        let mut i = 0usize;
        while i < size && produced < length {
            if chunk[i] != 0xFE {
                section.data[produced] = chunk[i];
                produced += 1;
                i += 1;
                continue;
            }
            // Escape.
            i += 1;
            if i >= size {
                break;
            }
            let count = chunk[i];
            if count == 0 {
                section.data[produced] = 0xFE;
                produced += 1;
            } else {
                let n = (count as usize).min(length - produced);
                for _ in 0..n {
                    section.data[produced] = last_byte;
                    produced += 1;
                }
            }
            i += 1;
        }
    }

    Ok(section)
}

/// Decompressed size of the first section for a given version.
fn header_section_len(version: u32) -> usize {
    if version >= 5 {
        128
    } else {
        68
    }
}

/// Parse a `.bem` resource.
pub fn parse(data: &[u8]) -> Result<Bem, BemError> {
    if data.len() < 16 {
        return Err(BemError::TooShort);
    }
    let mut r = R { d: data, p: 0 };
    let magic = r.u32()?;
    if magic != MAGIC {
        return Err(BemError::BadMagic(magic));
    }
    let version = r.u32()?;
    if version <= 3 || version > 5 {
        return Err(BemError::BadVersion(version));
    }
    let unk1 = r.u32()?;
    let unk2 = r.u32()?;
    if unk1 > 1024 || unk2 > 128 {
        return Err(BemError::BadHeaderField);
    }

    let header_section = read_rle(&mut r, header_section_len(version))?;
    let parts_section = read_rle(&mut r, unk1 as usize * 472)?;

    Ok(Bem {
        version,
        unk1,
        unk2,
        header_section,
        parts_section,
        exact: r.p == data.len(),
        consumed: r.p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One chunk holding `length` literal bytes that never equal the escape byte.
    /// A zero-length section writes nothing at all, matching the reader.
    fn literal_chunk(out: &mut Vec<u8>, length: usize) {
        if length == 0 {
            return;
        }
        out.extend_from_slice(&(length as u16).to_le_bytes());
        out.extend(std::iter::repeat(0x11u8).take(length));
    }

    fn build(version: u32, unk1: u32, unk2: u32) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend_from_slice(&MAGIC.to_le_bytes());
        d.extend_from_slice(&version.to_le_bytes());
        d.extend_from_slice(&unk1.to_le_bytes());
        d.extend_from_slice(&unk2.to_le_bytes());
        literal_chunk(&mut d, header_section_len(version));
        literal_chunk(&mut d, unk1 as usize * 472);
        d
    }

    #[test]
    fn parse_v5() {
        let d = build(5, 2, 0);
        let b = parse(&d).unwrap();
        assert_eq!(b.version, 5);
        assert_eq!(b.unk1, 2);
        assert_eq!(b.unk2, 0);
        assert_eq!(b.header_section.data.len(), 128);
        assert_eq!(b.parts_section.data.len(), 944);
        assert!(b.exact);
        assert_eq!(b.consumed, d.len());
    }

    #[test]
    fn zero_parts() {
        let d = build(5, 0, 0);
        let b = parse(&d).unwrap();
        assert_eq!(b.parts_section.data.len(), 0);
        assert!(b.exact);
    }

    #[test]
    fn escape_run_expands() {
        let mut d = Vec::new();
        d.extend_from_slice(&MAGIC.to_le_bytes());
        d.extend_from_slice(&5u32.to_le_bytes());
        d.extend_from_slice(&0u32.to_le_bytes());
        d.extend_from_slice(&0u32.to_le_bytes());
        // One chunk: 0x41, then 0xFE 0x05 (repeat the chunk's first byte 5 times),
        // then enough literals to reach 128 output bytes: 1 + 5 + 122 = 128.
        let mut chunk = vec![0x41u8, 0xFE, 0x05];
        chunk.extend(std::iter::repeat(0x22u8).take(122));
        d.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        d.extend_from_slice(&chunk);

        let b = parse(&d).unwrap();
        assert_eq!(b.header_section.data.len(), 128);
        assert_eq!(&b.header_section.data[..6], &[0x41, 0x41, 0x41, 0x41, 0x41, 0x41]);
        assert_eq!(b.header_section.data[6], 0x22);
        assert!(b.exact);
    }

    #[test]
    fn bad_magic() {
        let mut d = build(5, 1, 0);
        d[0] = 0;
        assert!(matches!(parse(&d), Err(BemError::BadMagic(_))));
    }

    #[test]
    fn bad_version() {
        let mut d = build(5, 1, 0);
        d[4] = 3;
        assert!(matches!(parse(&d), Err(BemError::BadVersion(3))));
    }

    #[test]
    fn header_field_out_of_range() {
        let mut d = build(5, 1, 0);
        // unk1 = 2000, above the 1024 cap.
        d[8..12].copy_from_slice(&2000u32.to_le_bytes());
        assert!(matches!(parse(&d), Err(BemError::BadHeaderField)));
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 8]), Err(BemError::TooShort)));
    }

    #[test]
    fn truncated_section() {
        let d = build(5, 1, 0);
        let cut = &d[..d.len() - 40];
        assert!(matches!(parse(cut), Err(BemError::Truncated)));
    }
}
