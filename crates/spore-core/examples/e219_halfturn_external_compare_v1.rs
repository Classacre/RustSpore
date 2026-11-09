//! One fixed half-turn source reduction, with supplied synthetic channel.
//! This is not a general evaluator or animation sampler.
use sha2::{Digest, Sha256};
fn word(b: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}
#[inline(never)]
fn mul(a: f32, b: f32) -> f32 {
    std::hint::black_box(a) * std::hint::black_box(b)
}
#[inline(never)]
fn add(a: f32, b: f32) -> f32 {
    std::hint::black_box(a) + std::hint::black_box(b)
}
fn project(record: &[u8]) -> Option<[u32; 12]> {
    if record.len() != 160
        || [0, 4, 8, 16, 20, 24, 44, 48, 52]
            .iter()
            .any(|&i| word(record, i) != 0)
        || word(record, 28) != 0x3f7fffff
        || [32, 36, 40]
            .iter()
            .any(|&i| word(record, i) != 1f32.to_bits())
        || record[56] != 0
    {
        return None;
    }
    // Source 797B..797D: sampled (1,0,0,0) quaternion composed with
    // record (0,0,0,w); then sqrt(2)*x, (0-x)*x + .5, + .5.
    // Identity basis, scale 1 and record +2C/+30/+34=0 suppress translation.
    let qx = mul(1.0, f32::from_bits(word(record, 28)));
    let x = mul(f32::from_bits(0x3fb504f3), qx);
    let half = add(mul(0.0 - x, x), 0.5);
    let diagonal = add(half, 0.5);
    Some([
        1f32.to_bits(),
        0,
        0,
        0,
        0,
        diagonal.to_bits(),
        0,
        0,
        0,
        0,
        diagonal.to_bits(),
        0,
    ])
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
        "c685f1f0ddbfb1095767b6ebf467304c8d72fb7c79404404482895b80e91e365"
    );
    assert_eq!(native.len(), 64);
    let expected = project(&record).unwrap();
    for (i, x) in expected.iter().enumerate() {
        assert_eq!(word(&native, 4 * i), *x, "matrix word {i}");
    }
    assert_eq!(word(&native, 48), 0x027f);
    assert_eq!(word(&native, 52), 0x1fa0);
    assert_eq!(word(&native, 56), 1);
    assert_eq!(word(&native, 60), 0);
    println!("matched fixed half-turn source reduction; supplied synthetic channel, no native sampling claim");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unadmitted_record() {
        assert!(project(&[0; 160]).is_none());
    }
}
