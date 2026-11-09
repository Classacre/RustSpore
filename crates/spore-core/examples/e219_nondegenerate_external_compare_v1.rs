//! Fixed one-bone 797970 source reduction; synthetic channel and record edits.
use sha2::{Digest, Sha256};
fn word(b: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}
fn project(record: &[u8], row: &[u32; 12]) -> Option<[u32; 12]> {
    let expected_row = [
        0,
        0,
        0,
        1f32.to_bits(),
        5f32.to_bits(),
        6f32.to_bits(),
        7f32.to_bits(),
        2f32.to_bits(),
        3f32.to_bits(),
        4f32.to_bits(),
        0,
        0,
    ];
    if record.len() != 160
        || *row != expected_row
        || [0, 4, 8, 16, 20, 24].iter().any(|&i| word(record, i) != 0)
        || word(record, 28) != 0x3f7fffff
        || [32, 36, 40, 44, 48, 52]
            .iter()
            .any(|&i| word(record, i) != 1f32.to_bits())
        || record[56] != 0
    {
        return None;
    }
    // With zero quaternion vector, zero record translation and identity supplied
    // basis, this branch reduces to its supplied diagonal and translation.
    let mut out = [0u32; 12];
    for i in 0..3 {
        out[5 * i] =
            (f32::from_bits(row[7 + i]) * f32::from_bits(word(record, 32 + 4 * i))).to_bits();
        out[4 * i + 3] =
            (f32::from_bits(row[4 + i]) * f32::from_bits(word(record, 44 + 4 * i))).to_bits();
    }
    Some(out)
}
fn main() {
    let mut args = std::env::args().skip(1);
    let record = std::fs::read(args.next().expect("synthetic record")).unwrap();
    let native = std::fs::read(args.next().expect("native output")).unwrap();
    assert!(args.next().is_none());
    assert_eq!(
        format!("{:x}", Sha256::digest(&record)),
        "4da3d50272cf393f87361e2d2bf0f31f96d1a7dc81a44c12a41a7effb66ea238"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&native)),
        "e408f5a2fba738e9701dfead3f927af091cc1010afca42dc46c987be0bdad3b2"
    );
    assert_eq!(native.len(), 64);
    let row = [
        0,
        0,
        0,
        1f32.to_bits(),
        5f32.to_bits(),
        6f32.to_bits(),
        7f32.to_bits(),
        2f32.to_bits(),
        3f32.to_bits(),
        4f32.to_bits(),
        0,
        0,
    ];
    let expected = project(&record, &row).unwrap();
    for (i, w) in expected.iter().enumerate() {
        assert_eq!(word(&native, 4 * i), *w, "matrix word {i}");
    }
    assert_eq!(word(&native, 48), 0x027f);
    assert_eq!(word(&native, 52), 0x1fa0);
    assert_eq!(word(&native, 56), 1);
    assert_eq!(word(&native, 60), 0);
    println!("matched fixed nonuniform-scale and translation case; modified record and channel synthetic");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unadmitted_row() {
        assert!(project(&[0; 160], &[0; 12]).is_none());
    }
}
