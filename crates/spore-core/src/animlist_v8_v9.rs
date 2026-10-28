//! Bounded readers for the AnimList v8 and v9 layouts inferred from A0E8C0.
//! No installed v8 or v9 resource was available for validation. Strings remain
//! UTF-16LE bytes, words remain raw bits, and record order is preserved.

#[derive(Debug, PartialEq, Eq)]
pub struct AnimList<'a> { pub version: u32, pub records: Vec<Record<'a>> }
#[derive(Debug, PartialEq, Eq)]
pub enum Record<'a> { V8(RecordV8<'a>), V9(RecordV9<'a>) }
#[derive(Debug, PartialEq, Eq)]
pub struct RecordV8<'a> {
    pub key: u32,
    pub word_after_key: u32,
    pub paths_utf16le: Vec<&'a [u8]>,
    pub path_words: Vec<u32>,
    pub shared_words: [u32; 2],
    pub flag_before_word: u8,
    pub word_between_flags: u32,
    pub flag_after_word: u8,
    pub trailing_words: [u32; 3],
}
#[derive(Debug, PartialEq, Eq)]
pub struct RecordV9<'a> {
    pub key: u32,
    pub name_utf16le: &'a [u8],
    pub words_before_flags: [u32; 2],
    pub flags: [u8; 2],
    pub words_after_flags: [u32; 3],
    pub variants: Vec<Variant<'a>>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Variant<'a> { pub threshold_bits: u32, pub entries: Vec<Entry<'a>> }
#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> { pub words: [u32; 3], pub path_utf16le: &'a [u8] }

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub bytes: usize, pub records: usize, pub paths: usize,
    pub variants: usize, pub entries: usize, pub string_units: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self { bytes: 16 * 1024 * 1024, records: 100_000, paths: 1_000_000,
            variants: 500_000, entries: 1_000_000, string_units: 4_000_000 }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    BadMagic, UnsupportedVersion(u32), Truncated, Overflow,
    LimitExceeded, AllocationFailed, TrailingBytes,
}
struct Reader<'a> {
    bytes: &'a [u8], pos: usize, limits: Limits,
    paths: usize, variants: usize, entries: usize, units: usize,
}
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
    fn count(&mut self, total: usize, maximum: usize, minimum_size: usize) -> Result<(usize, usize), ParseError> {
        let n = usize::try_from(self.word()?).map_err(|_| ParseError::Overflow)?;
        let sum = total.checked_add(n).ok_or(ParseError::Overflow)?;
        if sum > maximum { return Err(ParseError::LimitExceeded); }
        if n.checked_mul(minimum_size).ok_or(ParseError::Overflow)? > self.bytes.len() - self.pos {
            return Err(ParseError::Truncated);
        }
        Ok((n, sum))
    }
    fn string(&mut self) -> Result<&'a [u8], ParseError> {
        let n = usize::try_from(self.word()?).map_err(|_| ParseError::Overflow)?;
        self.units = self.units.checked_add(n).ok_or(ParseError::Overflow)?;
        if self.units > self.limits.string_units { return Err(ParseError::LimitExceeded); }
        self.take(n.checked_mul(2).ok_or(ParseError::Overflow)?)
    }
}
fn allocate<T>(n: usize) -> Result<Vec<T>, ParseError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| ParseError::AllocationFailed)?;
    Ok(v)
}
/// Parse one complete v8 or v9 payload; no records are merged.
pub fn parse<'a>(bytes: &'a [u8], limits: Limits) -> Result<AnimList<'a>, ParseError> {
    if bytes.len() > limits.bytes { return Err(ParseError::LimitExceeded); }
    let mut r = Reader { bytes, pos: 0, limits, paths: 0, variants: 0, entries: 0, units: 0 };
    if r.take(4)? != b"tsla" { return Err(ParseError::BadMagic); }
    let version = r.word()?;
    if version != 8 && version != 9 { return Err(ParseError::UnsupportedVersion(version)); }
    let (count, _) = r.count(0, limits.records, if version == 8 { 38 } else { 34 })?;
    let mut records = allocate(count)?;
    for _ in 0..count {
        let key = r.word()?;
        if version == 8 {
            let word_after_key = r.word()?;
            let (n, total) = r.count(r.paths, limits.paths, 8)?;
            r.paths = total;
            let mut paths_utf16le = allocate(n)?;
            for _ in 0..n { paths_utf16le.push(r.string()?); }
            let mut path_words = allocate(n)?;
            for _ in 0..n { path_words.push(r.word()?); }
            let shared_words = [r.word()?, r.word()?];
            let flag_before_word = r.take(1)?[0];
            let word_between_flags = r.word()?;
            let flag_after_word = r.take(1)?[0];
            let trailing_words = [r.word()?, r.word()?, r.word()?];
            records.push(Record::V8(RecordV8 { key, word_after_key, paths_utf16le, path_words,
                shared_words, flag_before_word, word_between_flags, flag_after_word, trailing_words }));
        } else {
            let name_utf16le = r.string()?;
            let words_before_flags = [r.word()?, r.word()?];
            let b = r.take(2)?;
            let flags = [b[0], b[1]];
            let words_after_flags = [r.word()?, r.word()?, r.word()?];
            let (n, total) = r.count(r.variants, limits.variants, 8)?;
            r.variants = total;
            let mut variants = allocate(n)?;
            for _ in 0..n {
                let threshold_bits = r.word()?;
                let (n, total) = r.count(r.entries, limits.entries, 16)?;
                r.entries = total;
                let mut entries = allocate(n)?;
                for _ in 0..n {
                    let words = [r.word()?, r.word()?, r.word()?];
                    let path_utf16le = r.string()?;
                    entries.push(Entry { words, path_utf16le });
                }
                variants.push(Variant { threshold_bits, entries });
            }
            records.push(Record::V9(RecordV9 { key, name_utf16le, words_before_flags,
                flags, words_after_flags, variants }));
        }
    }
    if r.pos != bytes.len() { return Err(ParseError::TrailingBytes); }
    Ok(AnimList { version, records })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(b: &mut Vec<u8>, n: u32) { b.extend_from_slice(&n.to_be_bytes()); }
    fn fixture(version: u32) -> Vec<u8> {
        let mut b = b"tsla".to_vec(); word(&mut b, version); word(&mut b, 1); word(&mut b, 42);
        if version == 8 {
            word(&mut b, 0x7fc01234); word(&mut b, 2);
            word(&mut b, 1); b.extend_from_slice(&[0, 0xd8]); word(&mut b, 0);
            word(&mut b, 11); word(&mut b, 12); word(&mut b, 13); word(&mut b, 14);
            b.push(15); word(&mut b, 16); b.push(17);
            for x in [18, 19, 20] { word(&mut b, x); }
        } else {
            word(&mut b, 1); b.extend_from_slice(&[0, 0xd8]);
            word(&mut b, 11); word(&mut b, 12); b.extend_from_slice(&[13, 14]);
            for x in [15, 16, 17] { word(&mut b, x); }
            word(&mut b, 1); word(&mut b, 0x80000000); word(&mut b, 1);
            for x in [18, 19, 20] { word(&mut b, x); }
            word(&mut b, 1); b.extend_from_slice(&[0x41, 0]);
        }
        b
    }
    #[test]
    fn preserves_v8_wire_fields() {
        let b = fixture(8); let p = parse(&b, Limits::default()).unwrap();
        let Record::V8(r) = &p.records[0] else { panic!("wrong version") };
        assert_eq!((r.key, r.word_after_key), (42, 0x7fc01234));
        assert_eq!(r.paths_utf16le, vec![&[0, 0xd8][..], &[][..]]);
        assert_eq!(r.path_words, [11, 12]); assert_eq!(r.shared_words, [13, 14]);
        assert_eq!((r.flag_before_word, r.word_between_flags, r.flag_after_word), (15, 16, 17));
        assert_eq!(r.trailing_words, [18, 19, 20]);
    }
    #[test]
    fn preserves_v9_wire_fields() {
        let b = fixture(9); let p = parse(&b, Limits::default()).unwrap();
        let Record::V9(r) = &p.records[0] else { panic!("wrong version") };
        assert_eq!(r.key, 42); assert_eq!(r.name_utf16le, &[0, 0xd8]);
        assert_eq!(r.words_before_flags, [11, 12]); assert_eq!(r.flags, [13, 14]);
        assert_eq!(r.words_after_flags, [15, 16, 17]);
        assert_eq!(r.variants[0].threshold_bits, 0x80000000);
        assert_eq!(r.variants[0].entries[0].words, [18, 19, 20]);
        assert_eq!(r.variants[0].entries[0].path_utf16le, &[0x41, 0]);
    }
    #[test]
    fn rejects_truncated_and_extra_data() {
        for version in [8, 9] {
            let mut b = fixture(version);
            for end in 0..b.len() { assert!(parse(&b[..end], Limits::default()).is_err(), "v{version} prefix {end}"); }
            b.push(0); assert_eq!(parse(&b, Limits::default()), Err(ParseError::TrailingBytes));
        }
    }
    #[test]
    fn empty_lists_retain_source_version() {
        for version in [8, 9] {
            let mut b = b"tsla".to_vec(); word(&mut b, version); word(&mut b, 0);
            let parsed = parse(&b, Limits::default()).unwrap();
            assert_eq!(parsed.version, version);
            assert!(parsed.records.is_empty());
        }
    }
    #[test]
    fn rejects_versions_and_limits() {
        let mut b = fixture(8); b[7] = 7;
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::UnsupportedVersion(7)));
        let mut limits = Limits::default(); limits.paths = 1;
        assert_eq!(parse(&fixture(8), limits), Err(ParseError::LimitExceeded));
        limits = Limits::default(); limits.entries = 0;
        assert_eq!(parse(&fixture(9), limits), Err(ParseError::LimitExceeded));
        limits = Limits::default(); limits.string_units = 0;
        assert_eq!(parse(&fixture(9), limits), Err(ParseError::LimitExceeded));
    }
}
