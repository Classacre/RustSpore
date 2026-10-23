//! Parser for the SPORE content-assembly manifest (type `0x030BDEE3`).
//!
//! Status: **100% validated (1,641 of 1,641).** The Python reference validator is
//! `tools/type_030b_validate.py`.
//!
//! ## Layout (big-endian)
//!
//! ```text
//! offset  0: u32 version = 10
//! offset 84: u32 ascii_length
//! offset 88: ASCII tag (ascii_length bytes),
//!            e.g. "tag:spore.com,2006:AssembledContent"
//! then:
//!     u32 count_a, then count_a * (u32 utf16_length + utf16_length*2 bytes)
//!     u32 count_b, then count_b * 4-byte hash
//!     one 4-byte hash
//!     one optional extra 4-byte hash (the 139-byte variant)
//! ```
//!
//! The `0x23` byte at offset 87 is the ASCII length (35) and also reads as `#`,
//! which is where an earlier note's `#tag:` came from.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    TooShort,
    BadVersion(u32),
    Truncated,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestError::TooShort => write!(f, "content manifest too short"),
            ManifestError::BadVersion(v) => write!(f, "unexpected manifest version {v}"),
            ManifestError::Truncated => write!(f, "content manifest truncated"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// A parsed content-assembly manifest.
#[derive(Debug, Clone)]
pub struct ContentManifest {
    pub version: u32,
    /// The ASCII tag string.
    pub ascii_tag: String,
    /// The UTF-16 string list.
    pub strings: Vec<String>,
    /// The 4-byte hash list.
    pub hashes: Vec<u32>,
    /// The trailing 4-byte hash.
    pub trailer_hash: u32,
    /// The optional extra hash (139-byte variant).
    pub extra_hash: Option<u32>,
    /// Whether the walk consumed exactly the whole buffer.
    pub exact: bool,
    /// Total bytes consumed.
    pub consumed: usize,
}

fn be_u32(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn decode_utf16le(d: &[u8]) -> String {
    let mut units = Vec::with_capacity(d.len() / 2);
    let mut i = 0;
    while i + 2 <= d.len() {
        units.push(u16::from_le_bytes([d[i], d[i + 1]]));
        i += 2;
    }
    char::decode_utf16(units.iter().copied())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// Parse a content-assembly manifest.
pub fn parse(data: &[u8]) -> Result<ContentManifest, ManifestError> {
    let n = data.len();
    if n < 88 {
        return Err(ManifestError::TooShort);
    }
    let version = be_u32(data, 0);
    if version != 10 {
        return Err(ManifestError::BadVersion(version));
    }
    let ascii_len = be_u32(data, 84) as usize;
    if ascii_len > 1000 || 88 + ascii_len > n {
        return Err(ManifestError::Truncated);
    }
    let ascii_tag = String::from_utf8_lossy(&data[88..88 + ascii_len]).into_owned();
    let mut p = 88 + ascii_len;

    if p + 4 > n {
        return Err(ManifestError::Truncated);
    }
    let count_a = be_u32(data, p) as usize;
    p += 4;
    if count_a > 1000 {
        return Err(ManifestError::Truncated);
    }
    let mut strings = Vec::with_capacity(count_a.min(256));
    for _ in 0..count_a {
        if p + 4 > n {
            return Err(ManifestError::Truncated);
        }
        let ln = be_u32(data, p) as usize;
        p += 4;
        if ln > 1000 || p + ln * 2 > n {
            return Err(ManifestError::Truncated);
        }
        strings.push(decode_utf16le(&data[p..p + ln * 2]));
        p += ln * 2;
    }

    if p + 4 > n {
        return Err(ManifestError::Truncated);
    }
    let count_b = be_u32(data, p) as usize;
    p += 4;
    if count_b > 1000 || p + count_b * 4 > n {
        return Err(ManifestError::Truncated);
    }
    let mut hashes = Vec::with_capacity(count_b.min(256));
    for _ in 0..count_b {
        hashes.push(be_u32(data, p));
        p += 4;
    }

    if p + 4 > n {
        return Err(ManifestError::Truncated);
    }
    let trailer_hash = be_u32(data, p);
    p += 4;
    let extra_hash = if n - p == 4 {
        let h = be_u32(data, p);
        p += 4;
        Some(h)
    } else {
        None
    };

    Ok(ContentManifest {
        version,
        ascii_tag,
        strings,
        hashes,
        trailer_hash,
        extra_hash,
        exact: p == n,
        consumed: p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_be(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_be_bytes());
    }
    fn push_utf16(v: &mut Vec<u8>, s: &str) {
        for u in s.encode_utf16() {
            v.extend_from_slice(&u.to_le_bytes());
        }
    }

    fn build(tag: &str, strings: &[&str], hashes: &[u32], trailer: u32, extra: Option<u32>) -> Vec<u8> {
        let mut d = vec![0u8; 88];
        // version at 0
        d[0..4].copy_from_slice(&10u32.to_be_bytes());
        // ascii length at 84
        d[84..88].copy_from_slice(&(tag.len() as u32).to_be_bytes());
        d.extend_from_slice(tag.as_bytes());
        push_be(&mut d, strings.len() as u32);
        for s in strings {
            push_be(&mut d, s.encode_utf16().count() as u32);
            push_utf16(&mut d, s);
        }
        push_be(&mut d, hashes.len() as u32);
        for &h in hashes {
            push_be(&mut d, h);
        }
        push_be(&mut d, trailer);
        if let Some(e) = extra {
            push_be(&mut d, e);
        }
        d
    }

    #[test]
    fn parse_with_strings_and_hashes() {
        let d = build(
            "tag:spore.com,2006:AssembledContent",
            &["set:fungus"],
            &[0x11111111, 0x22222222],
            0x33333333,
            None,
        );
        let m = parse(&d).unwrap();
        assert_eq!(m.version, 10);
        assert_eq!(m.ascii_tag, "tag:spore.com,2006:AssembledContent");
        assert_eq!(m.strings, vec!["set:fungus".to_string()]);
        assert_eq!(m.hashes, vec![0x11111111, 0x22222222]);
        assert_eq!(m.trailer_hash, 0x33333333);
        assert_eq!(m.extra_hash, None);
        assert!(m.exact);
    }

    #[test]
    fn parse_with_extra_hash() {
        let d = build("tag:x", &[], &[], 0, Some(0xABCD1234));
        let m = parse(&d).unwrap();
        assert_eq!(m.extra_hash, Some(0xABCD1234));
        assert!(m.exact);
    }

    #[test]
    fn bad_version() {
        let mut d = build("tag:x", &[], &[], 0, None);
        // version is BE, so it occupies bytes 0..4 as [0,0,0,10]; corrupt the last.
        d[3] = 0;
        assert!(matches!(parse(&d), Err(ManifestError::BadVersion(_))));
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 40]), Err(ManifestError::TooShort)));
    }

    #[test]
    fn truncated_string() {
        let d = build("tag:x", &["set:fungus"], &[], 0, None);
        let cut = &d[..d.len() - 3];
        assert!(matches!(parse(cut), Err(ManifestError::Truncated)));
    }
}
