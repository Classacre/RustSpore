//! Compare fixed synthetic native ranking with bounded Rust ranking.
//! Native host used a checked fixed-capacity insertion substitute.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_ranking_v1::{self as model, SnapshotAdmission},
    native_sse::Policy,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT: &str = "8dbfc96db366d0b43182ec040893c936ee31410cda9ef64f971e66c579a39f19";
const CASES: &[&[u32]] = &[
    &[
        0x40400000, 0x40a00000, 0x40a00000, 0x3f800000, 0x41100000, 0x40a00000,
    ],
    &[0x80000000, 0xbf800000, 0, 0x40800000],
    &[0, 0, 0, 0],
    &[0x60ad78ec, 0x40400000, 0xe0ad78ec, 0x40000000, 0x3f800000],
];
fn word(v: &serde_json::Value, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = v.as_str().ok_or("missing hex word")?;
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hex word".into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT) {
        return Err(
            "usage: creature_generated_skin_ranking_compare_v1 <receipt.json> <receipt-sha256>"
                .into(),
        );
    }
    let mut b = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut b)?;
    if b.len() > 65536 || format!("{:x}", Sha256::digest(&b)) != RECEIPT {
        return Err("receipt size/hash mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&b)?;
    for (k, x) in [
        ("schema", "creature-skin-ranking-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "72a9a1cd1969b157c821d96902d4417ef7441b4bb265e3a3055968aa66bccebb",
        ),
        (
            "instruction_sha256",
            "d25b8f8c45e606b8cda910922639061d3001d12a9103aa7c52ee3b4504c7fc18",
        ),
    ] {
        if v[k].as_str() != Some(x) {
            return Err(format!("receipt {k} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("invalid receipt status".into());
    }
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != CASES.len() {
        return Err("case count mismatch".into());
    }
    for (i, row) in cases.iter().enumerate() {
        if row["id"].as_u64() != Some(i as u64) {
            return Err(format!("case {i}: id mismatch").into());
        }
        let input = row["input_bits"].as_array().ok_or("missing inputs")?;
        if input.len() != CASES[i].len() {
            return Err("input count mismatch".into());
        }
        for (x, expected) in input.iter().zip(CASES[i]) {
            if word(x, 8)? != *expected {
                return Err(format!("case {i}: input mismatch").into());
            }
        }
        let cw = word(&row["cw"], 4)?;
        let sw = word(&row["sw"], 4)?;
        let mx = word(&row["mxcsr"], 8)?;
        let flags = word(&row["eflags"], 8)?;
        if cw != 0x27f
            || sw & 0x38ff != 0
            || mx & !0x3f != 0x1f80
            || mx & 0x1f != 0
            || flags & 0x400 != 0
        {
            return Err(format!("case {i}: FP metadata mismatch").into());
        }
        let expected = row["ranked_words"]
            .as_array()
            .ok_or("missing ranked words")?;
        if expected.len() != 4 {
            return Err("rank count mismatch".into());
        }
        let weights: Vec<_> = CASES[i].iter().map(|&v| f32::from_bits(v)).collect();
        let got = model::rank(&weights, Policy::Mxcsr1f80, SnapshotAdmission)?;
        for j in 0..4 {
            let p = expected[j].as_array().ok_or("rank entry not pair")?;
            if p.len() != 2
                || (word(&p[0], 8)?, word(&p[1], 8)?)
                    != (got[j].index_word, got[j].weight.to_bits())
            {
                return Err(format!("case {i}, rank {j}: mismatch").into());
            }
        }
    }
    println!("MATCH: all 32 index/weight words in four synthetic ranking cases.");
    println!("Native allocator/insert was replaced with a checked fixed-capacity stub; no accumulation, normalization, smoothing, palette or live editor is established.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
