//! Bounded projections of observed candidate world and population disk layouts.
//!
//! All fields are raw on-disk words; correspondence to post-fixup runtime fields
//! is unverified. Other sections and array overlap are not checked. Nonempty
//! projected arrays require an offset of at least 16 as a conservative policy;
//! this is not a claim about every valid source file.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldRow {
    pub raw_word_00: u32,
    pub raw_bytes_04: [u8; 4],
    pub raw_word_08: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldView {
    pub rows: Vec<WorldRow>,
    pub raw_word_08: u32,
    pub raw_word_0c: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopulationRow {
    pub bytes: [u8; 0x4c],
}
impl PopulationRow {
    pub fn word(&self, offset: usize) -> Option<u32> {
        let b: [u8; 4] = self.bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?;
        Some(u32::from_le_bytes(b))
    }
}

fn word(data: &[u8], offset: usize) -> Result<u32, &'static str> {
    let bytes: [u8; 4] = data.get(offset..offset.checked_add(4).ok_or("offset overflow")?)
        .ok_or("truncated word")?.try_into().map_err(|_| "truncated word")?;
    Ok(u32::from_le_bytes(bytes))
}
fn rows(data: &[u8], count: u32, offset: u32, stride: usize) -> Result<&[u8], &'static str> {
    let count = usize::try_from(count).map_err(|_| "count overflow")?;
    if count == 0 { return Ok(&[]); }
    let start = usize::try_from(offset).map_err(|_| "offset overflow")?;
    if start < 16 { return Err("array overlaps header"); }
    let len = count.checked_mul(stride).ok_or("array size overflow")?;
    let end = start.checked_add(len).ok_or("array extent overflow")?;
    data.get(start..end).ok_or("truncated array")
}

/// Project 12-byte rows in an observed disk layout. Runtime correspondence is
/// unverified; the second section body and overlap are not checked.
pub fn parse_world(data: &[u8]) -> Result<WorldView, &'static str> {
    let count = word(data, 0)?;
    let offset = word(data, 4)?;
    let raw_word_08 = word(data, 8)?;
    let raw_word_0c = word(data, 12)?;
    if raw_word_08 != 0 && (raw_word_0c < 16 || raw_word_0c as usize >= data.len()) {
        return Err("invalid opaque section offset");
    }
    let body = rows(data, count, offset, 12)?;
    let mut result = Vec::new();
    for row in body.chunks_exact(12) {
        result.push(WorldRow { raw_word_00: word(row, 0)?, raw_bytes_04: row[4..8].try_into().map_err(|_| "truncated row")?, raw_word_08: word(row, 8)? });
    }
    Ok(WorldView { rows: result, raw_word_08, raw_word_0c })
}

/// Project 0x4c-byte rows in an observed disk layout. Runtime correspondence is
/// unverified; the first section body and overlap are not checked.
pub fn parse_population_rows(data: &[u8]) -> Result<Vec<PopulationRow>, &'static str> {
    word(data, 0)?;
    word(data, 4)?;
    let count = word(data, 8)?;
    let offset = word(data, 12)?;
    rows(data, count, offset, 0x4c)?.chunks_exact(0x4c).map(|row|
        Ok(PopulationRow { bytes: row.try_into().map_err(|_| "truncated row")? })
    ).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    #[test]
    fn world_reads_ordered_rows_and_retains_opaque_header() {
        let mut data = vec![0u8; 44];
        put(&mut data, 0, 2); put(&mut data, 4, 16);
        put(&mut data, 8, 1); put(&mut data, 12, 40);
        put(&mut data, 16, 0x1234); put(&mut data, 20, 0x01020304);
        put(&mut data, 24, u32::MAX); put(&mut data, 28, 0x5678);
        let v = parse_world(&data).unwrap();
        assert_eq!(v.rows.len(), 2);
        assert_eq!(v.rows[0].raw_word_00, 0x1234);
        assert_eq!(v.rows[0].raw_bytes_04, [4, 3, 2, 1]);
        assert_eq!(v.rows[0].raw_word_08, u32::MAX);
        assert_eq!(v.rows[1].raw_word_00, 0x5678);
        assert_eq!((v.raw_word_08, v.raw_word_0c), (1, 40));
    }
    #[test]
    fn rejects_truncated_and_overflowing_rows() {
        assert!(parse_world(&[0; 15]).is_err());
        let mut data = vec![0; 28]; put(&mut data, 0, 2); put(&mut data, 4, 16);
        assert!(parse_world(&data).is_err());
        put(&mut data, 0, u32::MAX); put(&mut data, 4, u32::MAX);
        assert!(parse_world(&data).is_err());
        put(&mut data, 0, 1); put(&mut data, 4, 8);
        assert!(parse_world(&data).is_err());
        put(&mut data, 0, 0); put(&mut data, 8, 1); put(&mut data, 12, 28);
        assert!(parse_world(&data).is_err());
    }
    #[test]
    fn population_keeps_words_in_source_order_and_checks_bounds() {
        let mut data = vec![0; 16 + 0x4c * 2];
        put(&mut data, 8, 2); put(&mut data, 12, 16);
        put(&mut data, 16 + 0x1c, 0x12345678);
        put(&mut data, 16 + 0x28, 1);
        put(&mut data, 16 + 0x4c + 0x3c, 2);
        let result = parse_population_rows(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].word(0x1c), Some(0x12345678));
        assert_eq!(result[0].word(0x28), Some(1));
        assert_eq!(result[1].word(0x3c), Some(2));
        assert_eq!(result[0].word(0x4a), None);
        data.pop();
        assert!(parse_population_rows(&data).is_err());
    }
}
