//! Exact synthetic three-field CellSaveGame fixture codec.
//!
//! This models one bounded source-instruction fixture, not a general Spore save
//! format, real file or live object. It accepts only this exact three-field frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fields {
    /// Object +10, +14, +18 respectively; source serializes them in reverse order.
    pub player_key: [u32; 3],
    pub food: u32,
    pub plant_food: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError { Length, Header }

/// Encode the supplied synthetic class prefix and fixed three-field synthetic frame.
/// Successful complete callbacks, sorted fixed metadata and stable object state
/// are fixture admissions; no source initialization or filesystem I/O occurs.
pub fn encode(fields: Fields) -> [u8; 60] {
    let words = [0xabcdef00, 0x01a80d26, 48, 3,
        0xabcd0000, 12, 0xabcd0001, 4, 0xabcd0011, 4,
        fields.player_key[2], fields.player_key[1], fields.player_key[0],
        fields.food, fields.plant_food];
    let mut out = [0u8; 60];
    for (i, word) in words.into_iter().enumerate() {
        out[i*4..i*4+4].copy_from_slice(&word.to_be_bytes());
    }
    out
}
/// Decode only the exact fixed fixture framing; reject unknown fields, altered
/// lengths, wrong order and trailing bytes rather than infer general compatibility.
pub fn decode(bytes: &[u8]) -> Result<Fields, DecodeError> {
    if bytes.len() != 60 { return Err(DecodeError::Length); }
    let word = |i: usize| u32::from_be_bytes(bytes[i*4..i*4+4].try_into().unwrap());
    let header = [0xabcdef00,0x01a80d26,48,3,0xabcd0000,12,0xabcd0001,4,0xabcd0011,4];
    if header.into_iter().enumerate().any(|(i,v)| word(i) != v) { return Err(DecodeError::Header); }
    Ok(Fields { player_key: [word(12),word(11),word(10)], food: word(13), plant_food: word(14) })
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn synthetic_roundtrip_and_field_order() {
        let x=Fields{player_key:[0x11223344,0x55667788,0x99aabbcc],food:1,plant_food:1001};
        let b=encode(x); assert_eq!(&b[40..52],&[0x99,0xaa,0xbb,0xcc,0x55,0x66,0x77,0x88,0x11,0x22,0x33,0x44]);
        assert_eq!(decode(&b),Ok(x));
    }
    #[test] fn rejects_wrong_or_incomplete_frame() {
        let b=encode(Fields{player_key:[1,2,3],food:4,plant_food:5});
        assert_eq!(decode(&b[..59]),Err(DecodeError::Length));
        let mut extra=b.to_vec();extra.push(0);assert_eq!(decode(&extra),Err(DecodeError::Length));
        for offset in [0,4,8,12,16,20,24,28,32,36] {let mut bad=b;bad[offset]^=1;assert_eq!(decode(&bad),Err(DecodeError::Header));}
    }
}
