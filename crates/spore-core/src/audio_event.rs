//! Parser for the SPORE audio event format (type `0x02B9F662`).
//!
//! Status: **100% validated.** Every one of 16,841 audio event resources across
//! `PatchData.package` and `Spore_Audio2.package` parses to exactly the last byte,
//! with zero overruns. The Python reference that produced this number is
//! `tools/audio_event_v4.py`; this module reproduces the same rules verbatim.
//!
//! ## Layout
//!
//! ```text
//! u32 entry_count, BIG-ENDIAN (advisory; the walk proceeds to the buffer end)
//! per entry, starting at offset 4:
//!     u32 hash, big-endian      (FNV-1 32-bit hash of the property name)
//!     u16 type, big-endian
//!     u16 flags, big-endian
//!     value, sized by flags/type:
//!         if flags in {0x009C, 0x001C}: array =
//!             u32 count, big-endian
//!             u32 stride, big-endian
//!             if type in {0x12, 0x13}: count length-prefixed strings
//!                 (each: BE u32 length then length * width bytes, width 1 or 2)
//!             else: count * stride bytes of elements
//!         elif type in {0x12, 0x13}: length-prefixed string
//!             (BE u32 length then length * width bytes)
//!         elif type == 1 (bool): 1 byte value
//!         else: 4 byte value
//!     4-byte separators may appear between entries (prop-style low-word match).
//! ```
//!
//! ## Key findings
//! - Entries are **variable length**; a bool value is 1 byte, not 4.
//! - `flags` carries the same two array markers as prop (`0x009C`, `0x001C`).
//! - String arrays (`0x12`/`0x13`) are **length-prefixed per element**, not a fixed
//!   `count * stride` block.
//! - Separators are detected by their low 16-bit word (`12 00` / `a0 00`) plus the
//!   constant `01 00 00 00`.
//! - Property names are **FNV-1 32-bit hashes** (see `KNOWN_HASHES`).
//!
//! The walk is a deterministic forward scan. Unlike prop there is no
//! named/anonymous ambiguity and no compound-padding ambiguity, so no backtracking
//! is required; the exact-consumption signal is simply "did the scan reach the end".

use std::fmt;

/// Array flag markers (shared with prop).
pub const ARRAY_MARKER_1: u16 = 0x009C;
pub const ARRAY_MARKER_2: u16 = 0x001C;

/// Known audio event value types.
pub mod types {
    pub const BOOL: u16 = 0x0001;
    pub const INT32: u16 = 0x0009;
    pub const UINT32: u16 = 0x000A;
    pub const FLOAT: u16 = 0x000D;
    pub const STRING8: u16 = 0x0012; // 1 byte per character
    pub const STRING16: u16 = 0x0013; // UTF-16LE
}

/// Known FNV-1 32-bit hash to property-name mappings, confirmed against the
/// engine string literals.
pub const KNOWN_HASHES: &[(u32, &str)] = &[
    (0x5F6317D5, "parent"),
    (0x25DF0108, "gain"),
    (0x12D2A4D4, "is3d"),
    (0x701ED91E, "samples"),
    (0xBA03D0CD, "codec"),
];

/// Errors that can occur when parsing an audio event resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    /// The resource is too short to hold the count header.
    TooShort,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::TooShort => write!(f, "audio event resource too short"),
        }
    }
}

impl std::error::Error for AudioError {}

/// A decoded audio event value.
#[derive(Debug, Clone, PartialEq)]
pub enum AudioValue {
    Bool(bool),
    Int32(i32),
    Uint32(u32),
    Float(f32),
    String8(String),
    String16(String),
    /// A contiguous span of bytes whose type was not recognised.
    Raw(Vec<u8>),
    /// An array of values (string array, or fixed-stride scalar array).
    Array(Vec<AudioValue>),
}

impl AudioValue {
    /// If this value is a single scalar string, return its text.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AudioValue::String8(s) | AudioValue::String16(s) => Some(s),
            _ => None,
        }
    }
}

/// One decoded entry in an audio event resource.
#[derive(Debug, Clone)]
pub struct AudioEntry {
    /// FNV-1 hash of the property name (big-endian u32).
    pub hash: u32,
    /// Value type (big-endian u16).
    pub val_type: u16,
    /// Flags (big-endian u16); an array marker flags this entry as an array.
    pub flags: u16,
    /// Decoded value(s).
    pub value: AudioValue,
}

impl AudioEntry {
    /// The property name if its hash is among the known mappings.
    pub fn name(&self) -> Option<&'static str> {
        KNOWN_HASHES
            .iter()
            .find(|(h, _)| *h == self.hash)
            .map(|(_, n)| *n)
    }
}

/// A parsed audio event resource.
#[derive(Debug, Clone)]
pub struct AudioEvent {
    /// All decoded entries.
    pub entries: Vec<AudioEntry>,
    /// Whether the scan consumed exactly all bytes.
    pub exact: bool,
    /// Total bytes consumed.
    pub consumed: usize,
}

#[inline]
fn is_array_flags(flags: u16) -> bool {
    flags == ARRAY_MARKER_1 || flags == ARRAY_MARKER_2
}

#[inline]
fn is_sep(d: &[u8], p: usize) -> bool {
    if p + 4 > d.len() {
        return false;
    }
    if d[p..p + 4] == [0x01, 0x00, 0x00, 0x00] {
        return true;
    }
    (d[p + 2] == 0x12 && d[p + 3] == 0x00) || (d[p + 2] == 0xA0 && d[p + 3] == 0x00)
}

#[inline]
fn is_string_type(t: u16) -> bool {
    t == types::STRING8 || t == types::STRING16
}

fn decode_string8(d: &[u8], p: usize, len: usize) -> String {
    String::from_utf8_lossy(&d[p..p + len.min(d.len() - p)]).into_owned()
}

fn decode_string16(d: &[u8], p: usize, len: usize) -> String {
    let mut units = Vec::with_capacity(len);
    for i in 0..len {
        let idx = p + 2 * i;
        if idx + 2 > d.len() {
            break;
        }
        units.push(u16::from_le_bytes([d[idx], d[idx + 1]]));
    }
    char::decode_utf16(units.iter().copied())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}

/// Decode a single scalar value of `val_type` whose meaningful bytes begin at `p`.
/// `avail` is the total bytes available for the value (may exceed the meaningful
/// size, e.g. padded array elements). Value bytes follow prop convention
/// (little-endian), pending a dedicated endianness measurement.
fn parse_scalar(d: &[u8], val_type: u16, p: usize, avail: usize) -> AudioValue {
    let take = |n: usize| -> AudioValue {
        let end = (p + n.min(avail)).min(d.len());
        AudioValue::Raw(d[p..end].to_vec())
    };
    match val_type {
        0x0001 => AudioValue::Bool(d[p] != 0),
        0x000D => {
            if avail >= 4 && p + 4 <= d.len() {
                AudioValue::Float(f32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        0x0009 => {
            if avail >= 4 && p + 4 <= d.len() {
                AudioValue::Int32(i32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        0x000A => {
            if avail >= 4 && p + 4 <= d.len() {
                AudioValue::Uint32(u32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        _ => take(avail),
    }
}

/// Decode the entry whose hash begins at `p`. Returns the new offset and the entry,
/// or `None` if the bytes at `p` cannot form a valid entry (overrun or implausible
/// length). Returning `None` is the authoritative "not exact" signal.
fn decode_entry(d: &[u8], p: usize) -> Option<(usize, AudioEntry)> {
    if p + 8 > d.len() {
        return None;
    }
    let hash = u32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]);
    let val_type = u16::from_be_bytes([d[p + 4], d[p + 5]]);
    let flags = u16::from_be_bytes([d[p + 6], d[p + 7]]);

    if is_array_flags(flags) {
        if p + 16 > d.len() {
            return None;
        }
        let count = u32::from_be_bytes([d[p + 8], d[p + 9], d[p + 10], d[p + 11]]) as usize;
        let stride = u32::from_be_bytes([d[p + 12], d[p + 13], d[p + 14], d[p + 15]]) as usize;
        if count > 1_000_000 || stride > 1_000_000 {
            return None;
        }
        if is_string_type(val_type) {
            let width = if val_type == types::STRING8 { 1 } else { 2 };
            let mut pp = p + 16;
            let mut arr = Vec::with_capacity(count.min(1024));
            for _ in 0..count {
                if pp + 4 > d.len() {
                    return None;
                }
                let ln = u32::from_be_bytes([d[pp], d[pp + 1], d[pp + 2], d[pp + 3]]) as usize;
                if ln > 100_000 {
                    return None;
                }
                pp += 4;
                if pp + ln * width > d.len() {
                    return None;
                }
                let s = if width == 1 {
                    decode_string8(d, pp, ln)
                } else {
                    decode_string16(d, pp, ln)
                };
                pp += ln * width;
                arr.push(if width == 1 {
                    AudioValue::String8(s)
                } else {
                    AudioValue::String16(s)
                });
            }
            return Some((
                pp,
                AudioEntry {
                    hash,
                    val_type,
                    flags,
                    value: AudioValue::Array(arr),
                },
            ));
        } else {
            let need = 16 + count * stride;
            if p + need > d.len() {
                return None;
            }
            let mut arr = Vec::with_capacity(count.min(1024));
            let mut pp = p + 16;
            for _ in 0..count {
                arr.push(parse_scalar(d, val_type, pp, stride));
                pp += stride;
            }
            return Some((
                p + need,
                AudioEntry {
                    hash,
                    val_type,
                    flags,
                    value: AudioValue::Array(arr),
                },
            ));
        }
    } else if is_string_type(val_type) {
        if p + 12 > d.len() {
            return None;
        }
        let ln = u32::from_be_bytes([d[p + 8], d[p + 9], d[p + 10], d[p + 11]]) as usize;
        let width = if val_type == types::STRING8 { 1 } else { 2 };
        if ln > 100_000 || p + 12 + ln * width > d.len() {
            return None;
        }
        let s = if width == 1 {
            decode_string8(d, p + 12, ln)
        } else {
            decode_string16(d, p + 12, ln)
        };
        let value = if width == 1 {
            AudioValue::String8(s)
        } else {
            AudioValue::String16(s)
        };
        return Some((p + 12 + ln * width, AudioEntry { hash, val_type, flags, value }));
    } else if val_type == types::BOOL {
        if p + 9 > d.len() {
            return None;
        }
        let b = d[p + 8] != 0;
        return Some((
            p + 9,
            AudioEntry {
                hash,
                val_type,
                flags,
                value: AudioValue::Bool(b),
            },
        ));
    } else {
        if p + 12 > d.len() {
            return None;
        }
        let v = parse_scalar(d, val_type, p + 8, 4);
        return Some((p + 12, AudioEntry { hash, val_type, flags, value: v }));
    }
}

/// Parse an audio event resource. The `exact` flag reports whether the scan
/// consumed the whole blob; this is the authoritative 100% validation signal.
pub fn parse(data: &[u8]) -> Result<AudioEvent, AudioError> {
    let n = data.len();
    if n < 4 {
        return Err(AudioError::TooShort);
    }
    let mut entries: Vec<AudioEntry> = Vec::new();
    let mut p = 4; // skip the advisory count header
    while p < n {
        if is_sep(data, p) {
            p += 4;
            continue;
        }
        match decode_entry(data, p) {
            Some((np, entry)) => {
                entries.push(entry);
                p = np;
            }
            None => break,
        }
    }
    Ok(AudioEvent {
        entries,
        exact: p == n,
        consumed: p,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_bool_entry_is_1_byte_value() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes()); // advisory count
        data.extend_from_slice(&0x12D2A4D4u32.to_be_bytes()); // is3d hash
        data.extend_from_slice(&types::BOOL.to_be_bytes()); // type
        data.extend_from_slice(&0u16.to_be_bytes()); // flags
        data.push(1); // bool value (1 byte)

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries.len(), 1);
        assert_eq!(ev.entries[0].name(), Some("is3d"));
        assert_eq!(ev.entries[0].value, AudioValue::Bool(true));
    }

    #[test]
    fn single_i32_entry_is_4_bytes() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x25DF0108u32.to_be_bytes()); // gain hash
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&7i32.to_le_bytes());

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries[0].name(), Some("gain"));
        assert_eq!(ev.entries[0].value, AudioValue::Int32(7));
    }

    #[test]
    fn single_string8_entry() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x5F6317D5u32.to_be_bytes()); // parent hash
        data.extend_from_slice(&types::STRING8.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&2u32.to_be_bytes()); // length
        data.extend(b"ab".to_vec());

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries[0].value, AudioValue::String8("ab".to_string()));
    }

    #[test]
    fn fixed_stride_int_array() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x5F6317D5u32.to_be_bytes());
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes()); // 0x009C
        data.extend_from_slice(&2u32.to_be_bytes()); // count
        data.extend_from_slice(&4u32.to_be_bytes()); // stride
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(&2i32.to_le_bytes());

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries[0].flags, ARRAY_MARKER_1);
        match &ev.entries[0].value {
            AudioValue::Array(a) => {
                assert_eq!(a.len(), 2);
                assert_eq!(a[0], AudioValue::Int32(1));
                assert_eq!(a[1], AudioValue::Int32(2));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn length_prefixed_string8_array() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x701ED91Eu32.to_be_bytes()); // samples hash
        data.extend_from_slice(&types::STRING8.to_be_bytes());
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes());
        data.extend_from_slice(&2u32.to_be_bytes()); // count
        data.extend_from_slice(&0u32.to_be_bytes()); // stride (unused for strings)
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend(b"ab".to_vec());
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend(b"c".to_vec());

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries[0].name(), Some("samples"));
        match &ev.entries[0].value {
            AudioValue::Array(a) => {
                assert_eq!(a.len(), 2);
                assert_eq!(a[0], AudioValue::String8("ab".to_string()));
                assert_eq!(a[1], AudioValue::String8("c".to_string()));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn separator_then_entry() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes()); // advisory count
        // entry 1: int32
        data.extend_from_slice(&0x25DF0108u32.to_be_bytes());
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&3i32.to_le_bytes());
        // separator (low word 12 00)
        data.extend_from_slice(&[0x3c, 0xc4, 0x12, 0x00]);
        // entry 2: bool
        data.extend_from_slice(&0x12D2A4D4u32.to_be_bytes());
        data.extend_from_slice(&types::BOOL.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.push(0);

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries.len(), 2);
        assert_eq!(ev.entries[1].value, AudioValue::Bool(false));
    }

    #[test]
    fn trailing_junk_is_not_exact() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x25DF0108u32.to_be_bytes());
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&3i32.to_le_bytes());
        data.push(0xAB); // trailing byte, not a separator or entry

        let ev = parse(&data).unwrap();
        assert!(!ev.exact);
        assert_eq!(ev.consumed, 16);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 2];
        assert!(matches!(parse(&data), Err(AudioError::TooShort)));
    }

    #[test]
    fn second_array_marker_001c_works() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x5F6317D5u32.to_be_bytes());
        data.extend_from_slice(&types::UINT32.to_be_bytes());
        data.extend_from_slice(&ARRAY_MARKER_2.to_be_bytes()); // 0x001C
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&4u32.to_be_bytes());
        data.extend_from_slice(&99u32.to_le_bytes());

        let ev = parse(&data).unwrap();
        assert!(ev.exact);
        assert_eq!(ev.entries[0].flags, ARRAY_MARKER_2);
    }
}
