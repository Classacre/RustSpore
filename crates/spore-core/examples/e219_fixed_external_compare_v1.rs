//! Narrow one-bone 797970 comparison: supplied identity-axis channel, zero X scale.
//! Source quaternion has zero vector part, and all translations are zero. This
//! special case does not model animation sampling, hierarchy or general SSE paths.
use sha2::{Digest, Sha256};
fn word(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn project(record: &[u8], sample: &[u32; 12]) -> Option<[u32; 12]> {
    if record.len() != 160
        || [0, 4, 8, 16, 20, 24, 44, 48, 52]
            .iter()
            .any(|&i| word(record, i) != 0)
        || word(record, 28) != 0x3f7fffff
        || [32, 36, 40]
            .iter()
            .any(|&i| word(record, i) != 1f32.to_bits())
        || record[56] != 0
        || *sample
            != [
                0,
                0,
                0,
                1f32.to_bits(),
                0,
                0,
                0,
                0,
                1f32.to_bits(),
                1f32.to_bits(),
                1f32.to_bits(),
                0,
            ]
    {
        return None;
    }
    // The admitted zero-vector quaternions yield the identity axis basis.
    // Original scale inputs are row +1C/+20/+24, not +20/+24/+28.
    let s = [
        f32::from_bits(sample[7]),
        f32::from_bits(sample[8]),
        f32::from_bits(sample[9]),
    ];
    let mut out = [0u32; 12];
    for i in 0..3 {
        out[5 * i] = (s[i] * f32::from_bits(word(record, 32 + 4 * i))).to_bits();
    }
    Some(out)
}
fn main() {
    let mut a = std::env::args().skip(1);
    let record = std::fs::read(a.next().expect("record")).unwrap();
    let native = std::fs::read(a.next().expect("native output")).unwrap();
    assert!(a.next().is_none());
    assert_eq!(
        format!("{:x}", Sha256::digest(&record)),
        "60ee1a68b0acf0d541b1c4cc4517e59256f9677c9f99d8c4ec26c563cd69fe14"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&native)),
        "d968a2ff2edd02cf81ac49e378a341fc4d5b65b3b83d53dc44cc8a6b1c2d7b25"
    );
    assert_eq!(native.len(), 64);
    let sample = [
        0,
        0,
        0,
        1f32.to_bits(),
        0,
        0,
        0,
        0,
        1f32.to_bits(),
        1f32.to_bits(),
        1f32.to_bits(),
        0,
    ];
    let expected = project(&record, &sample).unwrap();
    for (i, x) in expected.iter().enumerate() {
        assert_eq!(word(&native, 4 * i), *x, "matrix word {i}");
    }
    assert_eq!(word(&native, 48), 0x0000027f);
    assert_eq!(word(&native, 52), 0x1fa4);
    assert_eq!(word(&native, 56), 1);
    assert_eq!(word(&native, 60), 0);
    println!("matched fixed one-bone 12-word result; synthetic zero-X scale, no native sampling or general evaluator claim");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unadmitted_sample() {
        let r = [0; 160];
        assert!(project(&r, &[0; 12]).is_none());
    }
}
