//! Bounded 46E776 corner record projection with explicitly supplied indices.
//!
//! Each index selects a 20-byte record; four byte values become u16 multiples
//! of three and four finite binary32 payloads are preserved. The validated
//! native comparison uses preallocated append stubs and x87 control 027F.
//! This does not derive actual corner triples, palette identities or live state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { IndexOutOfRange, NonFiniteWeight }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attributes { pub offsets: [u16; 4], pub weight_words: [u32; 4] }

pub fn join(records: &[[u8; 20]], indices: &[u32]) -> Result<Vec<Attributes>, Error> {
    let mut result = Vec::with_capacity(indices.len());
    for &index in indices {
        let record = records.get(index as usize).ok_or(Error::IndexOutOfRange)?;
        let mut offsets = [0; 4];
        let mut weight_words = [0; 4];
        for i in 0..4 {
            offsets[i] = u16::from(record[i]) * 3;
            let at = 4 + i * 4;
            let word = u32::from_le_bytes(record[at..at + 4].try_into().unwrap());
            if !f32::from_bits(word).is_finite() { return Err(Error::NonFiniteWeight); }
            weight_words[i] = word;
        }
        result.push(Attributes { offsets, weight_words });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_duplicates_and_preserves_payloads() {
        let mut a = [0u8; 20]; a[..4].copy_from_slice(&[0, 1, 128, 255]);
        a[4..8].copy_from_slice(&0x80000000u32.to_le_bytes());
        a[8..12].copy_from_slice(&1u32.to_le_bytes());
        let got = join(&[a], &[0, 0]).unwrap();
        assert_eq!(got[0].offsets, [0, 3, 384, 765]);
        assert_eq!(got[0].weight_words, [0x80000000, 1, 0, 0]);
        assert_eq!(got[0], got[1]);
        assert_eq!(join(&[a], &[1]), Err(Error::IndexOutOfRange));
        a[4..8].copy_from_slice(&0x7fc00000u32.to_le_bytes());
        assert_eq!(join(&[a], &[0]), Err(Error::NonFiniteWeight));
    }
}
