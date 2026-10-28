//! Bounded projection of the 131 records traversed by FUN_00e6cc30.
//! Evidence: target/rea/20261028-source-runtime-v6/783-analyze_function.json,
//! SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! The caller supplies bytes; no shipped bytes are embedded here. The table
//! alone does not reproduce the actor-dependent filtering or selection.

pub const TABLE_VA: u32 = 0x0148_4a20;
pub const RECORD_BYTES: usize = 36;
pub const RECORD_COUNT: usize = 131;
pub const TABLE_BYTES: usize = RECORD_BYTES * RECORD_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// All nine little-endian words in source order, including the two below.
    pub words: [u32; 9],
}

impl Record {
    /// Compared against the function's third argument; its semantic enum is unknown.
    pub fn state_word(&self) -> u32 { self.words[0] }
    /// Returned on selection and used by callers as an animation request key.
    pub fn request_key(&self) -> u32 { self.words[1] }
}

/// Accept exactly the traced table extent. This rejects both truncation and
/// unrelated trailing bytes; identity of the supplied executable is the caller's duty.
pub fn parse_table(bytes: &[u8]) -> Option<Vec<Record>> {
    if bytes.len() != TABLE_BYTES { return None; }
    Some(bytes.chunks_exact(RECORD_BYTES).map(|chunk| {
        let mut words = [0u32; 9];
        for (word, raw) in words.iter_mut().zip(chunk.chunks_exact(4)) {
            *word = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        }
        Record { words }
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_extent_and_little_endian_projection() {
        let mut bytes = vec![0u8; TABLE_BYTES];
        bytes[0..4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x90ab_cdefu32.to_le_bytes());
        bytes[TABLE_BYTES - 4..].copy_from_slice(&0x7654_3210u32.to_le_bytes());
        let rows = parse_table(&bytes).unwrap();
        assert_eq!(rows.len(), 131);
        assert_eq!(rows[0].state_word(), 0x1234_5678);
        assert_eq!(rows[0].request_key(), 0x90ab_cdef);
        assert_eq!(rows[130].words[8], 0x7654_3210);
        assert!(parse_table(&bytes[..TABLE_BYTES - 1]).is_none());
        bytes.push(0);
        assert!(parse_table(&bytes).is_none());
    }
}
