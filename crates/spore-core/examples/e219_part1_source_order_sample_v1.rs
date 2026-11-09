//! Controlled E219 part sampler comparison for one source fixture.
//!
//! This is a fixed-window diagnostic with synthetic selection/threshold; it
//! does not establish live animation, general quaternion, CRT or FSIN parity.
use sha2::{Digest, Sha256};
use spore_core::{
    native_crt_acos::{self, AcosPolicy, ConstantWindow},
    native_sse::{self, Policy},
    native_x87::{Context, Extended, Precision, Rounding},
    rw4,
    rw4_animation::{self, TrackValues},
};
use std::{env, error::Error, fs};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ASSET_SHA: &str = "aed300c20813388a06b3dfa86644e53d573bb18e34167ba9b385563314037cb2";
const CRT_SHA: &str = "76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df";
const EXPANDED_NATIVE_SHA: &str =
    "f492de9e59a21452142b21055fe8363c9c33c523c76c1bbdf69e41adebba466b";
const NATIVE_SHA: &str = "78fcb5d214e1f9523916aa1565ed349d75be6baf4303063979f9397c9322bcec";

fn pinned(bytes: &[u8], want: &str) -> Result<()> {
    if format!("{:x}", Sha256::digest(bytes)) != want {
        return Err("input SHA-256 mismatch".into());
    }
    Ok(())
}

fn compute(bytes: &[u8], constants: &[u8], time: f32) -> Result<Vec<[f32; 7]>> {
    if time.to_bits() != 0x3ea0_0000 {
        return Err("only the admitted midpoint is supported".into());
    }
    pinned(bytes, ASSET_SHA)?;
    pinned(constants, CRT_SHA)?;
    native_sse::scoped(Policy::Mxcsr1f80, || {
        compute_scoped(bytes, constants, false)
    })
}

fn compute_expanded(bytes: &[u8], constants: &[u8]) -> Result<Vec<[f32; 7]>> {
    pinned(bytes, ASSET_SHA)?;
    pinned(constants, CRT_SHA)?;
    native_sse::scoped(Policy::Mxcsr1f80, || compute_scoped(bytes, constants, true))
}

fn compute_scoped(bytes: &[u8], constants: &[u8], expanded: bool) -> Result<Vec<[f32; 7]>> {
    let rw = rw4::parse(bytes)?;
    let set = rw4_animation::parse(&rw, bytes)?;
    let clip = &set
        .clips
        .iter()
        .find(|c| c.section_index == 1)
        .ok_or("missing clip")?
        .value;
    if clip.id != 0x8573_4356 || clip.tracks.len() != 7 {
        return Err("unsupported clip".into());
    }
    let constants = ConstantWindow::new(constants)?;
    let x87 = Context {
        precision: Precision::Bits53,
        rounding: Rounding::NearestEven,
    };
    let sin = |v: f32| -> Result<f32> {
        Ok(x87.store_f32(x87.sin_magnitude_at_most_two(Extended::from_f32(v)?)?)?)
    };
    let cases: &[(usize, f32)] = if expanded {
        &[(0, 0.0), (0, 1.0), (1, 0.5), (1, 1.0)]
    } else {
        &[(0, 0.5)]
    };
    let mut output = Vec::with_capacity(cases.len() * 7);
    for &(key, f) in cases {
        for (i, track) in clip.tracks.iter().enumerate() {
            let TrackValues::Transform(k) = &track.values else {
                return Err("unsupported track".into());
            };
            if track.kind != 0x101
                || track.stride != 32
                || k.len() != 3
                || k[0].time != 0.
                || k[1].time != 0.625
                || k[2].time != 1.25
            {
                return Err("unsupported key window".into());
            }
            let a = k[key].value;
            let b = k[key + 1].value;
            let d0 = a.rotation[0] * b.rotation[0];
            let d1 = a.rotation[1] * b.rotation[1];
            let d2 = a.rotation[2] * b.rotation[2];
            let d3 = a.rotation[3] * b.rotation[3];
            let dot = ((d0 + d1) + d2) + d3;
            let q = if i == 0 {
                if a.rotation != [0., 0., 0., 1.] || b.rotation != a.rotation {
                    return Err("unsupported near branch".into());
                }
                a.rotation
            } else {
                if !(0.23..0.95).contains(&dot) {
                    return Err("unsupported acos domain".into());
                }
                let theta = native_crt_acos::finite_nonnegative(
                    f64::from(dot),
                    constants,
                    AcosPolicy::Sse2Mxcsr1f80,
                )? as f32;
                let denominator = sin(theta)?;
                let first = sin((1.0 - f) * theta)? / denominator;
                let second = sin(f * theta)? / denominator;
                std::array::from_fn(|j| a.rotation[j] * first + b.rotation[j] * second)
            };
            let mut row = [0.; 7];
            row[..4].copy_from_slice(&q);
            for j in 0..3 {
                row[4 + j] = a.translation[j] + (b.translation[j] - a.translation[j]) * f;
            }
            if !row.iter().all(|v| v.is_finite()) {
                return Err("nonfinite output".into());
            }
            output.push(row);
        }
    }
    Ok(output)
}

fn compare(rows: &[[f32; 7]], native: &[u8], expanded: bool) -> Result<()> {
    let (want, times, trailer): (&str, &[u32], usize) = if expanded {
        (
            EXPANDED_NATIVE_SHA,
            &[0, 0x3f20_0000, 0x3f70_0000, 0x3fa0_0000],
            896,
        )
    } else {
        (NATIVE_SHA, &[0x3ea0_0000], 224)
    };
    pinned(native, want)?;
    if rows.len() != times.len() * 7 || native.len() != trailer + 8 {
        return Err("unexpected output extent".into());
    }
    for (i, row) in rows.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            let at = i * 32 + j * 4;
            if value.to_bits() != u32::from_le_bytes(native[at..at + 4].try_into()?) {
                return Err("source comparison differs".into());
            }
        }
        if u32::from_le_bytes(native[i * 32 + 28..i * 32 + 32].try_into()?) != times[i / 7] {
            return Err("source time differs".into());
        }
    }
    let cw = u16::from_le_bytes(native[trailer..trailer + 2].try_into()?);
    let sw = u16::from_le_bytes(native[trailer + 2..trailer + 4].try_into()?);
    let mx = u32::from_le_bytes(native[trailer + 4..trailer + 8].try_into()?);
    if cw != 0x027f || sw & 0x3840 != 0 || mx & !0x3f != 0x1f80 {
        return Err("source FP controls differ".into());
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 5
        || !["--admit-synthetic-midpoint", "--admit-synthetic-windows"].contains(&args[1].as_str())
    {
        return Err("usage: e219_part1_source_order_sample_v1 <--admit-synthetic-midpoint|--admit-synthetic-windows> <external-1.rw4> <crt-constants.bin> <retained-native-stdout.bin>".into());
    }
    let asset = fs::read(&args[2])?;
    let constants = fs::read(&args[3])?;
    let native = fs::read(&args[4])?;
    let expanded = args[1] == "--admit-synthetic-windows";
    let rows = if expanded {
        compute_expanded(&asset, &constants)?
    } else {
        compute(&asset, &constants, 0.3125)?
    };
    compare(&rows, &native, expanded)?;
    println!("{} fixed component words matched; synthetic windows and threshold, not live or general interpolation", rows.len() * 7);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unsupported_time_before_reading_inputs() {
        assert!(compute(&[], &[], 0.625).is_err());
        assert!(compute(&[], &[], f32::NAN).is_err());
    }
    #[test]
    fn rejects_tampered_reference_and_inputs() {
        assert!(pinned(b"changed", ASSET_SHA).is_err());
        assert!(compare(&[[0.; 7]; 7], &[0; 232], false).is_err());
        assert!(compare(&[[0.; 7]; 28], &[0; 904], true).is_err());
    }
}
