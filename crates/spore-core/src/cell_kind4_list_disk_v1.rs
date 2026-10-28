//! Provisional disk projection of a kind-4 list. Runtime fixup and selection are unknown.
//!
//! The observed header contains a little-endian count and byte offset; each
//! projected row is 28 opaque bytes. This module deliberately does not interpret
//! row words or assert that a projected disk offset is a runtime pointer.

pub const HEADER_BYTES: usize = 8;
pub const ROW_BYTES: usize = 0x1c;
pub const MAX_ROWS: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum Invalid {
    TruncatedHeader,
    CountLimit,
    HeaderOverlap,
    TruncatedRows,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Projection<'a> {
    pub count: usize,
    pub array_offset: usize,
    /// Every original row byte, including words whose meanings are unknown.
    pub rows: &'a [u8],
    /// The original payload, including gaps and trailing bytes.
    pub payload: &'a [u8],
}

pub fn parse(data: &[u8]) -> Result<Projection<'_>, Invalid> {
    let header = data.get(..HEADER_BYTES).ok_or(Invalid::TruncatedHeader)?;
    let count = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
    let offset = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    if count > MAX_ROWS {
        return Err(Invalid::CountLimit);
    }
    if count == 0 && offset == 0 {
        return Ok(Projection { count, array_offset: offset, rows: &data[0..0], payload: data });
    }
    if offset < HEADER_BYTES {
        return Err(Invalid::HeaderOverlap);
    }
    let end = count.checked_mul(ROW_BYTES).and_then(|n| offset.checked_add(n))
        .ok_or(Invalid::TruncatedRows)?;
    let rows = data.get(offset..end).ok_or(Invalid::TruncatedRows)?;
    Ok(Projection { count, array_offset: offset, rows, payload: data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_observed_one_row_without_interpreting_it() {
        let mut data = vec![0; 36];
        data[..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..8].copy_from_slice(&8u32.to_le_bytes());
        let words = [1u32, 0, 0x3f800000, u32::MAX, 4, 2, 5];
        for (dst, word) in data[8..].chunks_exact_mut(4).zip(words) {
            dst.copy_from_slice(&word.to_le_bytes());
        }
        let parsed = parse(&data).unwrap();
        assert_eq!(parsed.count, 1);
        assert_eq!(parsed.array_offset, 8);
        assert_eq!(parsed.rows, &data[8..]);
        assert_eq!(parsed.payload, data);
    }

    #[test]
    fn rejects_bad_header_count_overlap_and_truncation() {
        assert_eq!(parse(&[0; 7]), Err(Invalid::TruncatedHeader));
        let mut data = [0u8; 36];
        data[..4].copy_from_slice(&((MAX_ROWS + 1) as u32).to_le_bytes());
        assert_eq!(parse(&data), Err(Invalid::CountLimit));
        data[..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..8].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(parse(&data), Err(Invalid::HeaderOverlap));
        data[4..8].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(parse(&data), Err(Invalid::TruncatedRows));
        data[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(parse(&data), Err(Invalid::TruncatedRows));
    }

    #[test]
    fn permits_empty_and_preserves_gaps_and_trailing_bytes() {
        let empty = [0u8; 8];
        assert_eq!(parse(&empty).unwrap().rows.len(), 0);
        let mut data = vec![0xa5; 41];
        data[..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..8].copy_from_slice(&12u32.to_le_bytes());
        let parsed = parse(&data).unwrap();
        assert_eq!(parsed.rows, &data[12..40]);
        assert_eq!(parsed.payload, data);
    }
}
