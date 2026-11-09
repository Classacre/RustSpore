//! Fixed synthetic 9C5B40 scalar-consumer comparison, not live animation.
//!
//! Four supplied scalar aggregates and scripted query limits mirror the
//! independently reviewed source fixture. Angular kinds and other state are
//! intentionally outside this example's admitted domain.
use sha2::{Digest, Sha256};
use spore_core::native_sse::{self, Policy};
use std::{env, error::Error, fs};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const NATIVE_SHA: &str = "ce7d29ea3b217e67e6318f433be287541a5cd6e401771e87cece848aa8b45688";
const KEYS: [u32; 4] = [0x89e0_6a31, 0x114b_b90c, 0x1505_4351, 0xc229_9aa7];
// Raw numerator/weight words. These are supplied controlled aggregate inputs,
// not observed live samples or source-derived controller state.
const INPUT: [(u32, u32); 4] = [
    (0x3d80_0000, 0x3e80_0000),
    (0x3e80_0000, 0x3f00_0000),
    (0x3dc0_0000, 0x3e00_0000),
    (0x3ec0_0000, 0x3ec0_0000),
];

fn compute(keys: [u32; 4], rows: [(u32, u32); 4], limits: (u32, u32)) -> Result<Vec<u8>> {
    if keys != KEYS || rows != INPUT || limits != (0, 1.0f32.to_bits()) {
        return Err("outside fixed synthetic scalar fixture".into());
    }
    native_sse::scoped(Policy::Mxcsr1f80, || -> Result<Vec<u8>> {
        let mut total = 0.0f32;
        for &(_, weight) in &rows {
            total = f32::from_bits(weight) + total;
        }
        if !total.is_finite() || total <= 0.0 {
            return Err("unsupported aggregate total".into());
        }
        let epsilon = f32::from_bits(0x322b_cc77);
        let mut out = Vec::with_capacity(48);
        for (&key, &(numerator, weight)) in keys.iter().zip(&rows) {
            let weight = f32::from_bits(weight);
            let scalar = f32::from_bits(numerator) / (weight + epsilon);
            let normalized_weight = weight / total;
            // Scripted source +80 supplies min=0, max=1 and returns AL=0.
            // The source consumer ignores AL and calls +74/+78 regardless.
            let value = (1.0f32 - 0.0) * scalar + 0.0;
            out.extend_from_slice(&key.to_le_bytes());
            out.extend_from_slice(&value.to_bits().to_le_bytes());
            out.extend_from_slice(&normalized_weight.to_bits().to_le_bytes());
        }
        Ok(out)
    })
}

fn compare(native: &[u8]) -> Result<()> {
    if format!("{:x}", Sha256::digest(native)) != NATIVE_SHA || native.len() != 56 {
        return Err("native stdout hash or length mismatch".into());
    }
    let cw = u16::from_le_bytes(native[48..50].try_into()?);
    let sw = u16::from_le_bytes(native[50..52].try_into()?);
    let mx = u32::from_le_bytes(native[52..56].try_into()?);
    if cw != 0x027f || sw & 0x3840 != 0 || mx & !0x3f != 0x1f80 {
        return Err("native FP controls or x87 stack status mismatch".into());
    }
    if compute(KEYS, INPUT, (0, 1.0f32.to_bits()))? != native[..48] {
        return Err("fixed callback projection differs".into());
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 || args[1] != "--admit-synthetic-aggregate" {
        return Err("usage: --admit-synthetic-aggregate <retained-native-stdout>".into());
    }
    compare(&fs::read(&args[2])?)?;
    println!("matched 4 synthetic scalar rows; scripted callbacks; no live pose claim");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_projection_and_wrong_domains() {
        let output = compute(KEYS, INPUT, (0, 1.0f32.to_bits())).unwrap();
        assert_eq!(output.len(), 48);
        assert_eq!(&output[..4], &KEYS[0].to_le_bytes());
        assert_eq!(u32::from_le_bytes(output[8..12].try_into().unwrap()), 0x3e4c_cccd);
        let mut altered = INPUT;
        altered[0].1 = 0;
        assert!(compute(KEYS, altered, (0, 1.0f32.to_bits())).is_err());
        assert!(compute(KEYS, INPUT, (0, 0)).is_err());
        let mut keys = KEYS;
        keys[0] ^= 1;
        assert!(compute(keys, INPUT, (0, 1.0f32.to_bits())).is_err());
    }

    #[test]
    fn rejects_unpinned_native_data() {
        assert!(compare(&[0; 56]).is_err());
        assert!(compare(&[]).is_err());
    }
}
