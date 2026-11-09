//! Compare a pinned seven-case synthetic native receipt with bounded angle wrap.
//! Difference delta is not captured. This is not live input, CRT or gameplay.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_angle_wrap_v1::{self as model, X87CopyAdmission},
    native_sse::Policy,
};
use std::{error::Error, fs::File, io::Read};

const RECEIPT_SHA: &str = "dd8b76071f0a506c2a9ce6a21ff3749b56f07bbc8c269473a7bfb5b974fa40a4";
const CASES: [(&str, u32, Option<u32>); 7] = [
    ("wrap", 0xc0490fdb, None),
    ("wrap", 0x40490fdb, None),
    ("wrap", 0x80000000, None),
    ("wrap", 0x40800000, None),
    ("difference", 0x3e800000, Some(0x3fa00000)),
    ("difference", 0x3fa00000, Some(0x3e800000)),
    ("difference", 0, Some(0x80000000)),
];
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{field}: invalid hexadecimal word").into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err("usage: cell_input_angle_wrap_compare_v1 <reviewed-receipt.json> <reviewed-receipt-sha256>".into());
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (field, expected) in [
        ("schema", "cell-angle-wrap-native-fixed-v1"),
        (
            "host_sha256",
            "94678e154a9fe11ada6c8ec29a41ca07f23ecb439703c916bd251354d6fa1acd",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        ("period_bits", "40c90fdb"),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    for (field, expected) in [
        (
            "wrap",
            "5b74a49ba0fed53195a4136d571b6ce0862cc5dd7fd643c491e737bdc084e039",
        ),
        (
            "difference",
            "db78a765d24afe40e89d9eb50078ac619a3058398ff84327e2328f880748c7b3",
        ),
    ] {
        if value["source_sha256"][field].as_str() != Some(expected) {
            return Err(format!("source {field} hash mismatch").into());
        }
    }
    if value["synthetic"].as_bool() != Some(true) || value["native_exit_code"].as_i64() != Some(0) {
        return Err("invalid synthetic status or native exit code".into());
    }
    let cases = value["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != CASES.len() {
        return Err("unexpected case count".into());
    }
    let period = f32::from_bits(0x40c90fdb);
    for (index, (row, &(kind, a, b))) in cases.iter().zip(CASES.iter()).enumerate() {
        if row["id"].as_u64() != Some(index as u64)
            || row["kind"].as_str() != Some(kind)
            || hex(row, "a_bits", 8)? != a
        {
            return Err(format!("case {index}: identity/input mismatch").into());
        }
        match b {
            Some(bits) if hex(row, "b_bits", 8)? != bits => {
                return Err(format!("case {index}: B mismatch").into())
            }
            None if !row["b_bits"].is_null() => {
                return Err(format!("case {index}: unexpected B").into())
            }
            _ => {}
        }
        let cw = hex(row, "cw", 4)?;
        let sw = hex(row, "sw", 4)?;
        let mxcsr = hex(row, "mxcsr", 8)?;
        let eflags = hex(row, "eflags", 8)?;
        if cw != 0x027f
            || sw & 0x38df != 0
            || mxcsr & !0x3f != 0x1f80
            || mxcsr & 0x1f != 0
            || eflags & 0x400 != 0
        {
            return Err(format!("case {index}: FP/stack/direction metadata mismatch").into());
        }
        let observed = hex(row, "return_bits", 8)?;
        if let Some(b) = b {
            let magnitude = hex(row, "magnitude_bits", 8)?;
            let actual = model::difference(
                f32::from_bits(a),
                f32::from_bits(b),
                period,
                8,
                Policy::Mxcsr1f80,
                X87CopyAdmission,
            )?;
            if (actual.sign.to_bits(), actual.magnitude.to_bits()) != (observed, magnitude) {
                return Err(format!("case {index}: sign/magnitude mismatch: model {:08x}/{:08x}, receipt {observed:08x}/{magnitude:08x}", actual.sign.to_bits(), actual.magnitude.to_bits()).into());
            }
        } else {
            if !row["magnitude_bits"].is_null() {
                return Err(format!("case {index}: unexpected magnitude").into());
            }
            let actual = model::wrap(
                f32::from_bits(a),
                period,
                8,
                Policy::Mxcsr1f80,
                X87CopyAdmission,
            )?;
            if actual.value.to_bits() != observed {
                return Err(format!(
                    "case {index}: wrap mismatch: model {:08x}, receipt {observed:08x}",
                    actual.value.to_bits()
                )
                .into());
            }
        }
    }
    println!("MATCH: four wrapped words and three sign/magnitude pairs for seven pinned synthetic cases.");
    println!("Difference delta was not observed; no live input, period, actor, CRT or gameplay is established. FP sticky status is not compared against the model.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
