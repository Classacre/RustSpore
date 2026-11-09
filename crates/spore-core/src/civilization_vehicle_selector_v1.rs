//! Bounded projection of original `FF0FB0`'s supplied triple selection.
//!
//! This accepts caller-supplied stable tables. It does not identify a live
//! vehicle, select an installed resource, or establish ownership of a table.

/// Decode a complete successful-stream `mModelKeys` payload under a caller's
/// explicit element bound. The source reads count followed by each triple's
/// words in +8, +4, +0 order. Exact end and bounded allocation are narrower
/// admissions; no file, descriptor dispatch or short-read behavior is modeled.
pub fn decode_model_keys(bytes: &[u8], max_entries: usize) -> Result<Vec<[u32; 3]>, &'static str> {
    if bytes.len() < 4 { return Err("missing count"); }
    let count = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
    if count > max_entries { return Err("count exceeds limit"); }
    let size = count.checked_mul(12).and_then(|n| n.checked_add(4)).ok_or("length overflow")?;
    if bytes.len() != size { return Err("length mismatch"); }
    let mut triples = Vec::with_capacity(count);
    for chunk in bytes[4..].chunks_exact(12) {
        let word = |offset: usize| u32::from_be_bytes(chunk[offset..offset+4].try_into().unwrap());
        triples.push([word(8), word(4), word(0)]);
    }
    Ok(triples)
}

/// Select a caller-owned triple using the source's first-match/default-zero
/// behavior. The caller admits readable, stable 16-entry tables and normal,
/// non-mutating helper behavior throughout the selection.
pub fn select<'a>(
    record_word: u32,
    selectors: &[u32; 16],
    triples: &'a [[u32; 3]; 16],
) -> (usize, &'a [u32; 3]) {
    let wanted = match record_word {
        0x018e_a1eb => Some(0x99e9_2f05),
        0x018e_b106 => Some(0x4e3f_7777),
        0x018e_a2cc => Some(0x47c1_0953),
        0x01a5_6aba => Some(0x72c4_9181),
        _ => None,
    };
    let index = wanted
        .and_then(|key| selectors.iter().position(|&v| v == key))
        .unwrap_or(0);
    (index, &triples[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_model_keys_reverse_source_word_order() {
        let bytes = [0,0,0,1, 0,0,0,3, 0,0,0,2, 0,0,0,1];
        assert_eq!(decode_model_keys(&bytes, 1).unwrap(), vec![[1,2,3]]);
        assert!(decode_model_keys(&bytes, 0).is_err());
        assert!(decode_model_keys(&bytes[..15], 1).is_err());
        let mut trailing = bytes.to_vec(); trailing.push(0);
        assert!(decode_model_keys(&trailing, 1).is_err());
    }

    #[test]
    fn first_match_and_source_fallback() {
        let mut selectors = [0u32; 16];
        selectors[3] = 0x47c1_0953;
        selectors[7] = 0x47c1_0953;
        let mut triples = [[0u32; 3]; 16];
        triples[3] = [11, 22, 33];
        assert_eq!(select(0x018e_a2cc, &selectors, &triples), (3, &triples[3]));
        assert_eq!(select(0x018e_a1eb, &selectors, &triples).0, 0);
        assert_eq!(select(0xffff_ffff, &selectors, &triples).0, 0);
    }

    #[test]
    fn four_supplied_selector_indices() {
        let mut selectors = [0u32; 16];
        selectors[..4].copy_from_slice(&[0x99e9_2f05, 0x4e3f_7777, 0x47c1_0953, 0x72c4_9181]);
        let triples = [[0u32; 3]; 16];
        for (record, index) in [
            (0x018e_a1eb, 0),
            (0x018e_b106, 1),
            (0x018e_a2cc, 2),
            (0x01a5_6aba, 3),
        ] {
            assert_eq!(select(record, &selectors, &triples).0, index);
        }
    }
}
