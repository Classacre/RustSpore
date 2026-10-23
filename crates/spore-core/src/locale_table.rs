//! Original locale-table tokenization for the checked well-formed UTF8 domain.
//!
//! `6B6CC0` reads type02FAC0B6/scope02FABF01, optionally consumes EFBBBF,
//! recognizes comments/explicit hex IDs and assigns automatic IDs after stored
//! entries. The source converter stores one 16-bit unit per UTF8 character,
//! truncating four-byte values. Final UI detokenization is separate; invalid
//! UTF8 is an explicit checked boundary here.

use std::fmt;

pub const RESOURCE_TYPE: u32 = 0x02fa_c0b6;
pub const RESOURCE_SCOPE: u32 = 0x02fa_bf01;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocaleError {
    InvalidBom,
    InvalidHexPrefix { at: usize },
    InvalidUtf8 { at: usize },
    AutomaticIdSentinel { at: usize },
}
impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source locale table: {self:?}")
    }
}
impl std::error::Error for LocaleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleEntry {
    pub id: u32,
    /// Source-converted units. Markup, escapes and trailing spaces are retained.
    pub text: Vec<u16>,
    pub source_text_range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleTable {
    /// Stored entries in original order, including overwritten duplicate IDs.
    pub entries: Vec<LocaleEntry>,
    /// EOF or the first NUL byte, which the original byte reader treats as EOF.
    pub stopped_at: usize,
}
impl LocaleTable {
    /// Native map assignment overwrites the offset for a repeated identifier.
    pub fn lookup(&self, id: u32) -> Option<&[u16]> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.id == id)
            .map(|entry| entry.text.as_slice())
    }
}

fn whitespace(value: u8) -> bool {
    value == b' ' || (b'\t'..=b'\r').contains(&value)
}
fn hex(value: u8) -> Option<u32> {
    match value {
        b'0'..=b'9' => Some(u32::from(value - b'0')),
        b'a'..=b'f' => Some(u32::from(value - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(value - b'A' + 10)),
        _ => None,
    }
}

/// Preserve the original three-state byte grammar instead of splitting lines.
///
/// Whitespace/comments can separate an ID from its text. Explicit IDs have at
/// most eight hex digits; a ninth digit starts the text. Unprefixed text uses
/// the previous stored ID plus one with wrapping. FFFFFFFF is the original
/// pending-ID sentinel, so it does not become a stored explicit identifier.
/// A trailing ID without text succeeds without adding an entry. Invalid UTF8
/// fails rather than silently inventing replacement glyphs; native malformed
/// conversion policy and UI substitutions are not established by this parser.
pub fn parse(data: &[u8]) -> Result<LocaleTable, LocaleError> {
    let mut cursor = 0;
    if data.first() == Some(&0xef) {
        if !data.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(LocaleError::InvalidBom);
        }
        cursor = 3;
    }
    let mut entries = Vec::new();
    let mut pending = u32::MAX;
    let mut previous = u32::MAX;
    while cursor < data.len() && data[cursor] != 0 {
        if whitespace(data[cursor]) {
            cursor += 1;
            continue;
        }
        if data[cursor] == b'#' {
            while cursor < data.len() && !matches!(data[cursor], 0 | b'\r' | b'\n') {
                cursor += 1;
            }
            continue;
        }
        if pending == u32::MAX {
            if data[cursor] == b'0' {
                let start = cursor;
                if !matches!(data.get(cursor + 1), Some(b'x' | b'X')) {
                    return Err(LocaleError::InvalidHexPrefix { at: start });
                }
                cursor += 2;
                pending = 0;
                for _ in 0..8 {
                    let Some(digit) = data.get(cursor).copied().and_then(hex) else {
                        break;
                    };
                    pending = pending.wrapping_mul(16).wrapping_add(digit);
                    cursor += 1;
                }
            } else {
                pending = previous.wrapping_add(1);
                // The source revisits state0 forever when automatic numbering
                // reaches its pending-ID sentinel. Keep this outside the
                // bounded parser rather than reproducing an infinite loop.
                if pending == u32::MAX {
                    return Err(LocaleError::AutomaticIdSentinel { at: cursor });
                }
            }
            continue;
        }
        let start = cursor;
        while cursor < data.len() && !matches!(data[cursor], 0 | b'\r' | b'\n') {
            cursor += 1;
        }
        let text = std::str::from_utf8(&data[start..cursor]).map_err(|error| {
            LocaleError::InvalidUtf8 {
                at: start + error.valid_up_to(),
            }
        })?;
        entries.push(LocaleEntry {
            id: pending,
            // 93C8A2..93C8BA uses CX-sized shifts, followed by one word
            // store at93C8D1. Four-byte UTF8 therefore becomes its low16
            // bits, rather than a modern UTF16 surrogate pair. All current
            // installed tables are BMP-only, but keep the native quirk.
            text: text
                .chars()
                .map(|character| character as u32 as u16)
                .collect(),
            source_text_range: start..cursor,
        });
        previous = pending;
        pending = u32::MAX;
    }
    Ok(LocaleTable {
        entries,
        stopped_at: cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_grammar_keeps_comments_pending_ids_auto_ids_and_duplicate_assignment() {
        let source = "\u{feff}# header\r\n0x00000010 # pending comment\r\n  Σ世界  \r\nnext # literal\n0X10 replacement\n0x123456789 tail\n0xFFFFFFFF automatic";
        let table = parse(source.as_bytes()).unwrap();
        let actual: Vec<_> = table
            .entries
            .iter()
            .map(|entry| (entry.id, String::from_utf16(&entry.text).unwrap()))
            .collect();
        assert_eq!(
            actual,
            vec![
                (16, "Σ世界  ".into()),
                (17, "next # literal".into()),
                (16, "replacement".into()),
                (0x12345678, "9 tail".into()),
                (0x12345679, "automatic".into())
            ]
        );
        assert_eq!(
            String::from_utf16(table.lookup(16).unwrap()).unwrap(),
            "replacement"
        );
        assert!(table.lookup(99).is_none());
    }

    #[test]
    fn nul_stops_without_consuming_suffix_and_checked_encoding_errors_are_explicit() {
        let source = b"0x1 value\0unread\xff";
        let table = parse(source).unwrap();
        assert_eq!(table.stopped_at, 9);
        assert_eq!(table.entries.len(), 1);
        assert!(parse(b"0x42").unwrap().entries.is_empty());
        assert_eq!(parse(b"\xefwrong"), Err(LocaleError::InvalidBom));
        assert_eq!(
            parse(b"0 value"),
            Err(LocaleError::InvalidHexPrefix { at: 0 })
        );
        assert_eq!(parse(b"0x1 \xff"), Err(LocaleError::InvalidUtf8 { at: 4 }));
        assert!(matches!(
            parse(b"0xFFFFFFFE value\nnext"),
            Err(LocaleError::AutomaticIdSentinel { .. })
        ));
    }

    #[test]
    fn source_four_byte_conversion_stores_one_truncated_word() {
        let table = parse("0x1 \u{1f600}\u{10000}\u{10ffff}".as_bytes()).unwrap();
        assert_eq!(table.entries[0].text, [0xf600, 0, 0xffff]);
    }
}
