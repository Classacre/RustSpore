//! Parser for the SPORE hash-pair table (type `0x02D5C9AF`).
//!
//! The historical Python validator `tools/type_02d5_validate.py` observed
//! version-4 framing in 1,631 samples from its measured package set:
//!
//! ```text
//! BE u32 version = 4
//! section 1: BE u32 count_a, then count_a * 12-byte records
//! section 2: BE u32 count_b, then count_b * 12-byte records
//! ```
//!
//! Each projected 12-byte record contains three raw words. Native code
//! distinguishes record types in the first section; the meaning of the middle
//! word in other sections remains unverified. Preserve all words and sections.

use std::fmt;

/// A source-observed integer record marker.
pub const RECORD_MARKER: u32 = 0x02E1A75D;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashPairsError {
    /// The resource is too short to hold the version.
    TooShort,
    /// The version is not 4.
    BadVersion(u32),
    /// A section count or record run runs past the end of the buffer.
    Truncated,
}

impl fmt::Display for HashPairsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashPairsError::TooShort => write!(f, "hash-pair table too short"),
            HashPairsError::BadVersion(v) => write!(f, "unexpected hash-pair version {v}"),
            HashPairsError::Truncated => write!(f, "hash-pair section truncated"),
        }
    }
}

impl std::error::Error for HashPairsError {}

/// One projected 12-byte record, preserving its middle word and raw value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashPair {
    pub hash: u32,
    pub marker: u32,
    pub value: u32,
}

/// A parsed hash-pair table.
#[derive(Debug, Clone)]
pub struct HashPairs {
    pub version: u32,
    /// Section 1 records.
    pub section_a: Vec<HashPair>,
    /// Section 2 records.
    pub section_b: Vec<HashPair>,
    /// Subsequent sections, if any, in their original order.
    pub extra_sections: Vec<Vec<HashPair>>,
    /// Number of sections encountered, including any empty sections.
    pub section_count: usize,
    /// Whether the walk consumed exactly the whole buffer.
    pub exact: bool,
    /// Total bytes consumed.
    pub consumed: usize,
}

fn be_u32(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

/// Parse a hash-pair table.
pub fn parse(data: &[u8]) -> Result<HashPairs, HashPairsError> {
    let n = data.len();
    if n < 8 {
        return Err(HashPairsError::TooShort);
    }
    let version = be_u32(data, 0);
    if version != 4 {
        return Err(HashPairsError::BadVersion(version));
    }

    let mut p = 4usize;
    let mut sections: Vec<Vec<HashPair>> = Vec::new();
    while p < n {
        if p + 4 > n {
            return Err(HashPairsError::Truncated);
        }
        let count = be_u32(data, p) as usize;
        p += 4;
        if count > 1_000_000 {
            return Err(HashPairsError::Truncated);
        }
        if p + count * 12 > n {
            return Err(HashPairsError::Truncated);
        }
        let mut recs = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            let hash = be_u32(data, p);
            let marker = be_u32(data, p + 4);
            let value = be_u32(data, p + 8);
            recs.push(HashPair { hash, marker, value });
            p += 12;
        }
        sections.push(recs);
    }

    let exact = p == n;
    let section_count = sections.len();
    let mut it = sections.into_iter();
    let section_a = it.next().unwrap_or_default();
    let section_b = it.next().unwrap_or_default();
    let extra_sections = it.collect();

    Ok(HashPairs {
        version,
        section_a,
        section_b,
        extra_sections,
        section_count,
        exact,
        consumed: p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_be(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_be_bytes());
    }

    fn build(section_a: &[(u32, u32)], section_b: &[(u32, u32)]) -> Vec<u8> {
        let mut d = Vec::new();
        push_be(&mut d, 4); // version
        push_be(&mut d, section_a.len() as u32);
        for &(h, val) in section_a {
            push_be(&mut d, h);
            push_be(&mut d, RECORD_MARKER);
            push_be(&mut d, val);
        }
        push_be(&mut d, section_b.len() as u32);
        for &(h, val) in section_b {
            push_be(&mut d, h);
            push_be(&mut d, RECORD_MARKER);
            push_be(&mut d, val);
        }
        d
    }

    #[test]
    fn parse_two_sections() {
        let d = build(&[(0x02DD90AF, 0x438F6347)], &[(0x05FB4759, 0), (0x03CC89B1, 0x913B23BE)]);
        let t = parse(&d).unwrap();
        assert_eq!(t.version, 4);
        assert_eq!(t.section_a.len(), 1);
        assert_eq!(t.section_b.len(), 2);
        assert_eq!(t.section_a[0], HashPair { hash: 0x02DD90AF, marker: RECORD_MARKER, value: 0x438F6347 });
        assert_eq!(t.section_b[1], HashPair { hash: 0x03CC89B1, marker: RECORD_MARKER, value: 0x913B23BE });
        assert!(t.exact);
        assert_eq!(t.consumed, d.len());
    }

    #[test]
    fn empty_sections() {
        let d = build(&[], &[]);
        let t = parse(&d).unwrap();
        assert!(t.section_a.is_empty());
        assert!(t.section_b.is_empty());
        assert!(t.exact);
    }

    #[test]
    fn bad_version() {
        let mut d = build(&[], &[]);
        d[3] = 3;
        assert!(matches!(parse(&d), Err(HashPairsError::BadVersion(_))));
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 4]), Err(HashPairsError::TooShort)));
    }

    #[test]
    fn truncated_section() {
        let d = build(&[(1, 2)], &[]);
        let cut = &d[..d.len() - 1];
        assert!(matches!(parse(cut), Err(HashPairsError::Truncated)));
    }

    #[test]
    fn preserves_unknown_markers_and_additional_sections() {
        let mut data = build(&[(1, 2)], &[]);
        data[12..16].copy_from_slice(&0x02E1A7FFu32.to_be_bytes());
        push_be(&mut data, 1);
        push_be(&mut data, 3);
        push_be(&mut data, 0xDEADBEEF);
        push_be(&mut data, 4);
        let table = parse(&data).unwrap();
        assert_eq!(table.section_a[0].marker, 0x02E1A7FF);
        assert_eq!(table.extra_sections, vec![vec![HashPair { hash: 3, marker: 0xDEADBEEF, value: 4 }]]);
        assert_eq!(table.section_count, 3);
        assert!(table.exact);
        assert_eq!(table.consumed, data.len());
    }

    #[test]
    fn reports_missing_second_section_and_rejects_partial_extra_section() {
        let mut data = Vec::new();
        push_be(&mut data, 4);
        push_be(&mut data, 0);
        let table = parse(&data).unwrap();
        assert_eq!(table.section_count, 1);
        assert!(table.section_b.is_empty());
        data.push(0);
        assert_eq!(parse(&data).unwrap_err(), HashPairsError::Truncated);
    }
}
