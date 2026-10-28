//! Bounded reader for the observed AnimList v7 wire layout.
//!
//! The pre-v9 branch of executable function A0E8C0 and a read-only scan of the
//! two installed v7 resources support this layout. Fields and ordering are wire
//! observations, not claims about playback or resource precedence. UTF-16LE is
//! retained as bytes, including invalid surrogates; floating values remain bits.

#[derive(Debug, PartialEq, Eq)]
pub struct AnimList<'a> { pub records: Vec<Record<'a>> }

#[derive(Debug, PartialEq, Eq)]
pub struct Record<'a> {
    pub key: u32,
    /// The paths occur together on the wire, before the corresponding words.
    pub paths_utf16le: Vec<&'a [u8]>,
    pub path_words: Vec<u32>,
    pub shared_words: [u32; 2],
    pub flag_before_word: u8,
    pub word_between_flags: u32,
    pub flag_after_word: u8,
    pub trailing_words: [u32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub bytes: usize,
    pub records: usize,
    pub paths: usize,
    pub string_units: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { bytes: 16 * 1024 * 1024, records: 100_000,
            paths: 1_000_000, string_units: 4_000_000 }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    BadMagic, UnsupportedVersion(u32), Truncated, Overflow,
    LimitExceeded, AllocationFailed, TrailingBytes,
}

struct Reader<'a> { bytes: &'a [u8], pos: usize, limits: Limits, paths: usize, units: usize }
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

/// Parse a complete v7 payload, preserving duplicates and file order.
pub fn parse<'a>(bytes: &'a [u8], limits: Limits) -> Result<AnimList<'a>, ParseError> {
    if bytes.len() > limits.bytes { return Err(ParseError::LimitExceeded); }
    let mut r = Reader { bytes, pos: 0, limits, paths: 0, units: 0 };
    if r.take(4)? != b"tsla" { return Err(ParseError::BadMagic); }
    let version = r.word()?;
    if version != 7 { return Err(ParseError::UnsupportedVersion(version)); }
    let (count, _) = r.count(0, limits.records, 34)?;
    let mut records = allocate(count)?;
    for _ in 0..count {
        let key = r.word()?;
        // Each path contributes at least a length word and an associated word.
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
        records.push(Record { key, paths_utf16le, path_words, shared_words,
            flag_before_word, word_between_flags, flag_after_word, trailing_words });
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
        word(&mut b, 7); word(&mut b, 1); word(&mut b, 0x12345678); word(&mut b, 2);
        word(&mut b, 2); b.extend_from_slice(&[0, 0xd8, 0x41, 0]);
        word(&mut b, 1); b.extend_from_slice(&[0x42, 0]);
        word(&mut b, 0x7fc01234); word(&mut b, 0x80000000);
        word(&mut b, 3); word(&mut b, 4); b.push(5); word(&mut b, 6); b.push(7);
        word(&mut b, 8); word(&mut b, 9); word(&mut b, 10); b
    }
    #[test]
    fn preserves_raw_fields_and_order() {
        let b = fixture(); let parsed = parse(&b, Limits::default()).unwrap();
        let r = &parsed.records[0];
        assert_eq!(r.key, 0x12345678);
        assert_eq!(r.paths_utf16le, vec![&[0, 0xd8, 0x41, 0][..], &[0x42, 0][..]]);
        assert_eq!(r.path_words, [0x7fc01234, 0x80000000]);
        assert_eq!(r.shared_words, [3, 4]);
        assert_eq!((r.flag_before_word, r.word_between_flags, r.flag_after_word), (5, 6, 7));
        assert_eq!(r.trailing_words, [8, 9, 10]);
    }
    #[test]
    fn rejects_truncation_and_trailing_bytes() {
        let mut b = fixture();
        for end in 0..b.len() { assert!(parse(&b[..end], Limits::default()).is_err(), "prefix {end}"); }
        b.push(0); assert_eq!(parse(&b, Limits::default()), Err(ParseError::TrailingBytes));
    }
    #[test]
    fn rejects_bad_headers_and_limits() {
        let mut b = fixture(); b[0] = 0;
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::BadMagic));
        b[0] = b't'; b[7] = 10;
        assert_eq!(parse(&b, Limits::default()), Err(ParseError::UnsupportedVersion(10)));
        let b = fixture(); let mut limits = Limits::default(); limits.paths = 1;
        assert_eq!(parse(&b, limits), Err(ParseError::LimitExceeded));
        limits = Limits::default(); limits.string_units = 2;
        assert_eq!(parse(&b, limits), Err(ParseError::LimitExceeded));
    }
    #[test]
    fn accepts_empty_list() {
        let mut b = b"tsla".to_vec(); word(&mut b, 7); word(&mut b, 0);
        assert!(parse(&b, Limits::default()).unwrap().records.is_empty());
    }
}
