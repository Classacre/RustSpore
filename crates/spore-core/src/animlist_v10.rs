//! Bounded, lossless reader for the observed AnimList v10 wire layout.
//!
//! Based on the local v10 census probe and `re/cell-animlist-resource-selection-v1.md`.
//! Order here is file order; it does not establish resource precedence or playback.
//! Strings are retained as little-endian UTF-16 bytes, including invalid surrogates.

#[derive(Debug, PartialEq, Eq)]
pub struct AnimList<'a> {
    pub records: Vec<Record<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a> {
    pub key: u32,
    pub name_utf16le: &'a [u8],
    pub words_before_flags: [u32; 2],
    pub flags: [u8; 3],
    pub words_after_flags: [u32; 3],
    pub variants: Vec<Variant<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Variant<'a> {
    pub threshold_bits: u32,
    pub entries: Vec<Entry<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    pub words: [u32; 3],
    pub path_utf16le: &'a [u8],
}

/// Resource ceilings are independent of structural checks against the input length.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub bytes: usize,
    pub records: usize,
    pub variants: usize,
    pub entries: usize,
    pub string_units: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { bytes: 16 * 1024 * 1024, records: 100_000, variants: 500_000,
            entries: 1_000_000, string_units: 4_000_000 }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    BadMagic,
    UnsupportedVersion(u32),
    Truncated,
    Overflow,
    LimitExceeded,
    AllocationFailed,
    TrailingBytes,
}

struct Reader<'a> { bytes: &'a [u8], pos: usize, limits: Limits, variants: usize, entries: usize, units: usize }

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        let end = self.pos.checked_add(n).ok_or(ParseError::Overflow)?;
        let result = self.bytes.get(self.pos..end).ok_or(ParseError::Truncated)?;
        self.pos = end;
        Ok(result)
    }
    fn word(&mut self) -> Result<u32, ParseError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn count(&mut self) -> Result<usize, ParseError> {
        usize::try_from(self.word()?).map_err(|_| ParseError::Overflow)
    }
    fn string(&mut self) -> Result<&'a [u8], ParseError> {
        let n = self.count()?;
        self.units = self.units.checked_add(n).ok_or(ParseError::Overflow)?;
        if self.units > self.limits.string_units { return Err(ParseError::LimitExceeded); }
        self.take(n.checked_mul(2).ok_or(ParseError::Overflow)?)
    }
    fn bounded_count(&mut self, total: usize, maximum: usize, minimum_size: usize) -> Result<(usize, usize), ParseError> {
        let n = self.count()?;
        let updated = total.checked_add(n).ok_or(ParseError::Overflow)?;
        if updated > maximum { return Err(ParseError::LimitExceeded); }
        let required = n.checked_mul(minimum_size).ok_or(ParseError::Overflow)?;
        if required > self.bytes.len() - self.pos { return Err(ParseError::Truncated); }
        Ok((n, updated))
    }
}

fn allocate<T>(count: usize) -> Result<Vec<T>, ParseError> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| ParseError::AllocationFailed)?;
    Ok(values)
}

/// Parse one complete v10 payload. No duplicate keys or variants are merged.
pub fn parse<'a>(bytes: &'a [u8], limits: Limits) -> Result<AnimList<'a>, ParseError> {
    if bytes.len() > limits.bytes { return Err(ParseError::LimitExceeded); }
    let mut r = Reader { bytes, pos: 0, limits, variants: 0, entries: 0, units: 0 };
    if r.take(4)? != b"tsla" { return Err(ParseError::BadMagic); }
    let version = r.word()?;
    if version != 10 { return Err(ParseError::UnsupportedVersion(version)); }
    // Minimum record: key, empty name length, five words, three bytes, variant count.
    let (count, _) = r.bounded_count(0, limits.records, 35)?;
    let mut records = allocate(count)?;
    for _ in 0..count {
        let key = r.word()?;
        let name_utf16le = r.string()?;
        let words_before_flags = [r.word()?, r.word()?];
        let flags_slice = r.take(3)?;
        let flags = [flags_slice[0], flags_slice[1], flags_slice[2]];
        let words_after_flags = [r.word()?, r.word()?, r.word()?];
        let (variant_count, total) = r.bounded_count(r.variants, limits.variants, 8)?;
        r.variants = total;
        let mut variants = allocate(variant_count)?;
        for _ in 0..variant_count {
            let threshold_bits = r.word()?;
            let (entry_count, total) = r.bounded_count(r.entries, limits.entries, 16)?;
            r.entries = total;
            let mut entries = allocate(entry_count)?;
            for _ in 0..entry_count {
                let words = [r.word()?, r.word()?, r.word()?];
                let path_utf16le = r.string()?;
                entries.push(Entry { words, path_utf16le });
            }
            variants.push(Variant { threshold_bits, entries });
        }
        records.push(Record { key, name_utf16le, words_before_flags, flags, words_after_flags, variants });
    }
    if r.pos != bytes.len() { return Err(ParseError::TrailingBytes); }
    Ok(AnimList { records })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(b: &mut Vec<u8>, n: u32) { b.extend_from_slice(&n.to_be_bytes()); }
    fn fixture() -> Vec<u8> {
        let mut b = b"tsla".to_vec();
        word(&mut b, 10); word(&mut b, 1); word(&mut b, 0x12345678);
        word(&mut b, 2); b.extend_from_slice(&[0x00, 0xd8, 0x41, 0x00]);
        word(&mut b, 1); word(&mut b, 2); b.extend_from_slice(&[3, 4, 5]);
        word(&mut b, 6); word(&mut b, 7); word(&mut b, 8);
        word(&mut b, 2);
        word(&mut b, 0x7fc01234); word(&mut b, 2);
        for n in [9, 10, 11] { word(&mut b, n); }
        word(&mut b, 1); b.extend_from_slice(&[0x42, 0]);
        for n in [12, 13, 14] { word(&mut b, n); }
        word(&mut b, 0);
        word(&mut b, 0x80000000); word(&mut b, 0);
        b
    }
    #[test]
    fn preserves_raw_values_and_file_order() {
        let b = fixture();
        let parsed = parse(&b, Limits::default()).unwrap();
        let r = &parsed.records[0];
        assert_eq!(r.key, 0x12345678);
        assert_eq!(r.name_utf16le, &[0, 0xd8, 0x41, 0]);
        assert_eq!(r.words_before_flags, [1, 2]);
        assert_eq!(r.flags, [3, 4, 5]);
        assert_eq!(r.words_after_flags, [6, 7, 8]);
        assert_eq!(r.variants[0].threshold_bits, 0x7fc01234);
        assert_eq!(r.variants[0].entries[0].words, [9, 10, 11]);
        assert_eq!(r.variants[0].entries[0].path_utf16le, &[0x42, 0]);
        assert_eq!(r.variants[0].entries[1].words, [12, 13, 14]);
        assert_eq!(r.variants[1].threshold_bits, 0x80000000);
        assert!(r.variants[1].entries.is_empty());
    }
    #[test]
    fn rejects_every_truncated_prefix_and_extra_data() {
        let b = fixture();
        for end in 0..b.len() { assert!(parse(&b[..end], Limits::default()).is_err(), "prefix {end}"); }
        let mut extended = b; extended.push(0);
        assert_eq!(parse(&extended, Limits::default()), Err(ParseError::TrailingBytes));
    }
    #[test]
    fn rejects_bad_header_and_excessive_counts() {
        let mut b = fixture(); b[0] = 0;
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::BadMagic));
        b[0] = b't'; b[7] = 7;
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::UnsupportedVersion(7)));
        b[7] = 10; b[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::LimitExceeded));
        let b = fixture(); let mut limits = Limits::default(); limits.entries = 1;
        assert_eq!(parse(&b, limits), Err(ParseError::LimitExceeded));
        limits = Limits::default(); limits.string_units = 1;
        assert_eq!(parse(&b, limits), Err(ParseError::LimitExceeded));
    }
    #[test]
    fn accepts_empty_list() {
        let mut b = b"tsla".to_vec(); word(&mut b, 10); word(&mut b, 0);
        assert!(parse(&b, Limits::default()).unwrap().records.is_empty());
    }
}
