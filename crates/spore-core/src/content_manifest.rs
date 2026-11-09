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

/// The version-10 localized-name prefix read by `54CFAB..54D085`.
///
/// This is only a prefix: subsequent manifest reads and resource-provider
/// admission are not modeled. The caller must establish the ordinary stream
/// framing, including absence of the version-10 exceptional seek.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalizedNamePrefix {
    pub table_id: u32,
    pub first_id: u32,
    pub name_id: u32,
    pub optional_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalizedNamePrefixError {
    Truncated,
    UnsupportedVersion(u32),
    RawStringBranch,
}

/// Parse only the ordinary-framing version-10 localized branch. The boolean
/// is an explicit admission of the source stream/seek context; this function
/// cannot derive it from the package bytes.
pub fn localized_name_prefix(
    data: &[u8],
    ordinary_version10_framing: bool,
) -> Result<Option<LocalizedNamePrefix>, LocalizedNamePrefixError> {
    if !ordinary_version10_framing {
        return Ok(None);
    }
    if data.len() < 80 {
        return Err(LocalizedNamePrefixError::Truncated);
    }
    let version = be_u32(data, 0);
    if version != 10 {
        return Err(LocalizedNamePrefixError::UnsupportedVersion(version));
    }
    if be_u32(data, 60) == 0 {
        return Err(LocalizedNamePrefixError::RawStringBranch);
    }
    Ok(Some(LocalizedNamePrefix {
        table_id: be_u32(data, 64),
        first_id: be_u32(data, 68),
        name_id: be_u32(data, 72),
        optional_id: be_u32(data, 76),
    }))
}

/// Caller-admitted provider selection; a parsed table alone does not establish
/// that the source runtime would select it.
pub enum AdmittedLocale<'a> {
    Unresolved,
    Selected { table_id: u32, table: &'a crate::locale_table::LocaleTable },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalizedLookupStage { First, Name, Optional }

/// Results are conditional on the supplied provider and prefix context.
/// Even `RawCandidates` does not establish complete reader success or the
/// processed string returned by `6B4FF0`.
#[derive(Debug, PartialEq, Eq)]
pub enum LocalizedNameLookup<'a> {
    ProviderUnresolved,
    SelectedTableMismatch { expected: u32, supplied: u32 },
    LookupFailed { stage: LocalizedLookupStage, id: u32 },
    RawCandidates { first: &'a [u16], name: &'a [u16], optional: Option<&'a [u16]> },
}

impl LocalizedNamePrefix {
    /// Resolve in source order; missing/empty entries and required sentinel
    /// IDs follow `6B4EE0`'s false result. No locale output processing occurs.
    pub fn lookup<'a>(&self, provider: AdmittedLocale<'a>) -> LocalizedNameLookup<'a> {
        if self.first_id == u32::MAX {
            return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::First, id: self.first_id };
        }
        // Each reader lookup initializes a fresh locale object; 6B4EE0
        // does not request a table when its ID is the sentinel.
        if self.table_id == u32::MAX {
            return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::First, id: self.first_id };
        }
        let (table_id, table) = match provider {
            AdmittedLocale::Unresolved => return LocalizedNameLookup::ProviderUnresolved,
            AdmittedLocale::Selected { table_id, table } => (table_id, table),
        };
        if table_id != self.table_id {
            return LocalizedNameLookup::SelectedTableMismatch { expected: self.table_id, supplied: table_id };
        }
        let get = |id: u32| table.lookup(id).filter(|text| text.first().is_some_and(|unit| *unit != 0));
        let Some(first) = get(self.first_id) else {
            return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::First, id: self.first_id };
        };
        if self.name_id == u32::MAX {
            return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::Name, id: self.name_id };
        }
        let Some(name) = get(self.name_id) else {
            return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::Name, id: self.name_id };
        };
        let optional = if self.optional_id == u32::MAX { None } else {
            let Some(value) = get(self.optional_id) else {
                return LocalizedNameLookup::LookupFailed { stage: LocalizedLookupStage::Optional, id: self.optional_id };
            };
            Some(value)
        };
        LocalizedNameLookup::RawCandidates { first, name, optional }
    }
}

/// Version-10, zero-discriminator raw-name prefix on the ordinary stream
/// path. Bytes are preserved exactly as read into the x86 UTF-16 containers.
/// This does not prove the remainder of the manifest or provider is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawNamePrefix {
    pub field_68: u64,
    pub first_utf16le: Vec<u8>,
    pub name_utf16le: Vec<u8>,
    pub optional_utf16le: Vec<u8>,
    pub consumed: usize,
}
impl RawNamePrefix {
    /// Units copied by `E845D0` before the first NUL, assuming the successful
    /// native container resize supplies its trailing NUL and a valid pointer.
    pub fn name_units_before_nul(&self) -> Vec<u16> {
        self.name_utf16le.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]]))
            .take_while(|unit| *unit != 0).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawNamePrefixError {
    Truncated,
    UnsupportedVersion(u32),
    LocalizedBranch,
    LengthOutOfRange { field: u8, units: u32 },
    /// Source calls stream +30 for a zero-length first or name read; its
    /// zero-byte return semantics are not established by this parser.
    ZeroLengthReadUnresolved { field: u8 },
}

/// Parses a conservative, full-read subset of `54CC93..54CFA6`.
/// `ordinary_version10_framing` admits the stream's big-endian integer mode
/// and absence of the exceptional seek. `Ok(None)` leaves framing unresolved.
/// Native raw reads test nonzero rather than exact byte counts; truncated or
/// partial reads are intentionally rejected here, not represented as success.
pub fn raw_name_prefix(
    data: &[u8], ordinary_version10_framing: bool,
) -> Result<Option<RawNamePrefix>, RawNamePrefixError> {
    if !ordinary_version10_framing { return Ok(None); }
    if data.len() < 72 { return Err(RawNamePrefixError::Truncated); }
    let version = be_u32(data, 0);
    if version != 10 { return Err(RawNamePrefixError::UnsupportedVersion(version)); }
    if be_u32(data, 60) != 0 { return Err(RawNamePrefixError::LocalizedBranch); }
    let field_68 = u64::from_be_bytes(data[64..72].try_into().unwrap());
    let mut position = 72usize;
    fn read_field(data: &[u8], position: &mut usize, field: u8, limit: u32, required: bool)
        -> Result<Vec<u8>, RawNamePrefixError> {
        let count_end = position.checked_add(4).ok_or(RawNamePrefixError::Truncated)?;
        if count_end > data.len() { return Err(RawNamePrefixError::Truncated); }
        let count = be_u32(data, *position);
        *position = count_end;
        if count >= limit { return Err(RawNamePrefixError::LengthOutOfRange { field, units: count }); }
        if required && count == 0 { return Err(RawNamePrefixError::ZeroLengthReadUnresolved { field }); }
        let end = position.checked_add(count as usize * 2).ok_or(RawNamePrefixError::Truncated)?;
        if end > data.len() { return Err(RawNamePrefixError::Truncated); }
        let result = data[*position..end].to_vec();
        *position = end;
        Ok(result)
    }
    let first_utf16le = read_field(data, &mut position, 0x58, 256, true)?;
    let name_utf16le = read_field(data, &mut position, 0x78, 256, true)?;
    let optional_utf16le = read_field(data, &mut position, 0x88, 4096, false)?;
    Ok(Some(RawNamePrefix { field_68, first_utf16le, name_utf16le, optional_utf16le, consumed: position }))
}

/// Explicit source state at entry to `6B4FF0`. The caller must obtain these
/// values; a locale file does not establish runtime classifier or global state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleOutputState {
    pub cached_flag: bool,
    pub cached_pointer_present: bool,
    pub classifier_nonzero: bool,
    pub global_nonzero: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleOutputDecision {
    /// The source returns its `+08` raw pointer without processing.
    Raw,
    /// A cached pointer or the processor supplies output; its contents are not modeled.
    ProcessedUnresolved,
}

/// Decide only the return branch; this does not validate pointer lifetime,
/// string termination, service behavior, or the origin of supplied state.
pub fn locale_output_decision(state: LocaleOutputState) -> LocaleOutputDecision {
    if state.cached_flag {
        if state.cached_pointer_present {
            LocaleOutputDecision::ProcessedUnresolved
        } else {
            LocaleOutputDecision::Raw
        }
    } else if !state.classifier_nonzero && !state.global_nonzero {
        LocaleOutputDecision::Raw
    } else {
        LocaleOutputDecision::ProcessedUnresolved
    }
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
    fn raw_name_prefix_checks_bounds_and_preserves_embedded_zero() {
        let mut bytes = vec![0; 72];
        bytes[..4].copy_from_slice(&10u32.to_be_bytes());
        bytes[64..72].copy_from_slice(&0x0123456789abcdefu64.to_be_bytes());
        push_be(&mut bytes, 1); bytes.extend_from_slice(&[0x41, 0]);
        push_be(&mut bytes, 3); bytes.extend_from_slice(&[0x42, 0, 0, 0, 0x43, 0]);
        push_be(&mut bytes, 0);
        assert_eq!(raw_name_prefix(&bytes, false).unwrap(), None);
        let raw = raw_name_prefix(&bytes, true).unwrap().unwrap();
        assert_eq!(raw.field_68, 0x0123456789abcdef);
        assert_eq!(raw.name_utf16le, [0x42, 0, 0, 0, 0x43, 0]);
        assert_eq!(raw.name_units_before_nul(), [0x42]);
        assert_eq!(raw.consumed, bytes.len());
        assert_eq!(raw_name_prefix(&bytes[..bytes.len()-1], true), Err(RawNamePrefixError::Truncated));
        bytes[72..76].copy_from_slice(&0u32.to_be_bytes());
        assert_eq!(raw_name_prefix(&bytes, true), Err(RawNamePrefixError::ZeroLengthReadUnresolved { field: 0x58 }));
        bytes[72..76].copy_from_slice(&256u32.to_be_bytes());
        assert_eq!(raw_name_prefix(&bytes, true), Err(RawNamePrefixError::LengthOutOfRange { field: 0x58, units: 256 }));
    }

    #[test]
    fn output_branch_uses_explicit_state() {
        for cached in [false, true] {
            for present in [false, true] {
                for classifier in [false, true] {
                    for global in [false, true] {
                        let state = LocaleOutputState { cached_flag: cached, cached_pointer_present: present,
                            classifier_nonzero: classifier, global_nonzero: global };
                        let expected = if (cached && !present) || (!cached && !classifier && !global) {
                            LocaleOutputDecision::Raw
                        } else { LocaleOutputDecision::ProcessedUnresolved };
                        assert_eq!(locale_output_decision(state), expected);
                    }
                }
            }
        }
    }

    #[test]
    fn localized_prefix_and_lookup_are_conditional() {
        use LocalizedNameLookup::*;
        let mut bytes = vec![0; 80];
        bytes[0..4].copy_from_slice(&10u32.to_be_bytes());
        for (offset, value) in [(60, u32::MAX), (64, 0x04dc2f94), (68, 1), (72, 0x22), (76, u32::MAX)] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
        }
        assert_eq!(localized_name_prefix(&bytes, false).unwrap(), None);
        let prefix = localized_name_prefix(&bytes, true).unwrap().unwrap();
        assert_eq!(prefix.lookup(AdmittedLocale::Unresolved), ProviderUnresolved);
        let table = crate::locale_table::parse(b"0x1 Fictor\n0x22 Zib\n").unwrap();
        assert!(matches!(prefix.lookup(AdmittedLocale::Selected { table_id: 0, table: &table }), SelectedTableMismatch { .. }));
        assert_eq!(prefix.lookup(AdmittedLocale::Selected { table_id: prefix.table_id, table: &table }), RawCandidates {
            first: &[70, 105, 99, 116, 111, 114], name: &[90, 105, 98], optional: None,
        });
        let mut missing = prefix;
        missing.first_id = u32::MAX;
        assert_eq!(missing.lookup(AdmittedLocale::Unresolved), LookupFailed { stage: LocalizedLookupStage::First, id: u32::MAX });
        missing = prefix;
        missing.optional_id = 7;
        assert_eq!(missing.lookup(AdmittedLocale::Selected { table_id: prefix.table_id, table: &table }), LookupFailed { stage: LocalizedLookupStage::Optional, id: 7 });
        bytes[60..64].fill(0);
        assert_eq!(localized_name_prefix(&bytes, true), Err(LocalizedNamePrefixError::RawStringBranch));
        assert_eq!(localized_name_prefix(&bytes[..79], true), Err(LocalizedNamePrefixError::Truncated));
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
