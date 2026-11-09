//! Bounded parsing of the observed version-1 resource-index framing.
//!
//! This describes a source-backed index, not a Cell save format. The third
//! header word and pair values are retained without assigning semantics.
//! Callers must supply a complete, stable decoded resource snapshot; this
//! parser does not read files or establish archive or runtime identity.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceIndex {
    pub header_word: u32,
    pub pairs: Vec<[u32; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexError {
    Truncated,
    UnsupportedVersion,
    CountLimit,
    LengthMismatch,
}

/// A validated sequence of big-endian length-prefixed spans. The count is
/// supplied by the caller; this type does not identify or decode the contents.
pub struct Spans<'a> {
    remaining: &'a [u8],
    count: usize,
}

/// Validate a complete sequence before iteration. Each span includes its own
/// four-byte length and has at least the two words read by the source loop.
pub fn spans(bytes: &[u8], count: usize, max_count: usize) -> Result<Spans<'_>, IndexError> {
    if count > max_count {
        return Err(IndexError::CountLimit);
    }
    let mut tail = bytes;
    for _ in 0..count {
        if tail.len() < 8 {
            return Err(IndexError::Truncated);
        }
        let len = u32::from_be_bytes(tail[..4].try_into().unwrap()) as usize;
        if len < 8 || len > tail.len() {
            return Err(IndexError::LengthMismatch);
        }
        tail = &tail[len..];
    }
    if !tail.is_empty() {
        return Err(IndexError::LengthMismatch);
    }
    Ok(Spans {
        remaining: bytes,
        count,
    })
}

impl<'a> Iterator for Spans<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }
        let len = u32::from_be_bytes(self.remaining[..4].try_into().unwrap()) as usize;
        let (span, rest) = self.remaining.split_at(len);
        self.remaining = rest;
        self.count -= 1;
        Some(span)
    }
}

/// Parse exactly one complete index, rejecting trailing bytes and counts above
/// the caller's explicit resource limit. Words are big-endian.
pub fn parse(bytes: &[u8], max_pairs: usize) -> Result<ResourceIndex, IndexError> {
    if bytes.len() < 12 {
        return Err(IndexError::Truncated);
    }
    let word = |offset: usize| u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
    if word(0) != 1 {
        return Err(IndexError::UnsupportedVersion);
    }
    let count = word(4) as usize;
    if count > max_pairs {
        return Err(IndexError::CountLimit);
    }
    let expected = count
        .checked_mul(8)
        .and_then(|n| n.checked_add(12))
        .ok_or(IndexError::CountLimit)?;
    if bytes.len() != expected {
        return Err(IndexError::LengthMismatch);
    }
    let pairs = (0..count)
        .map(|i| [word(12 + i * 8), word(16 + i * 8)])
        .collect();
    Ok(ResourceIndex {
        header_word: word(8),
        pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_complete_frame_and_limits() {
        let bytes = [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0, 2, 0, 0, 0, 3];
        assert_eq!(
            parse(&bytes, 1).unwrap(),
            ResourceIndex {
                header_word: 7,
                pairs: vec![[2, 3]]
            }
        );
        assert_eq!(parse(&bytes, 0), Err(IndexError::CountLimit));
        assert_eq!(parse(&bytes[..19], 1), Err(IndexError::LengthMismatch));
        assert_eq!(parse(&bytes[..8], 1), Err(IndexError::Truncated));
        let mut trailing = bytes.to_vec();
        trailing.push(0);
        assert_eq!(parse(&trailing, 1), Err(IndexError::LengthMismatch));
        let mut wrong = bytes;
        wrong[3] = 2;
        assert_eq!(parse(&wrong, 1), Err(IndexError::UnsupportedVersion));
    }

    #[test]
    fn validates_all_spans_before_iteration() {
        let bytes = [0, 0, 0, 8, 0, 0, 0, 4, 0, 0, 0, 9, 0, 0, 0, 5, 7];
        assert_eq!(
            spans(&bytes, 2, 2)
                .unwrap()
                .map(|x| x.len())
                .collect::<Vec<_>>(),
            [8, 9]
        );
        assert!(matches!(spans(&bytes, 2, 1), Err(IndexError::CountLimit)));
        assert!(matches!(
            spans(&bytes[..16], 2, 2),
            Err(IndexError::LengthMismatch)
        ));
        assert!(matches!(
            spans(&bytes, 1, 2),
            Err(IndexError::LengthMismatch)
        ));
        assert!(matches!(
            spans(&[0, 0, 0, 4, 0, 0, 0, 0], 1, 1),
            Err(IndexError::LengthMismatch)
        ));
    }
}
