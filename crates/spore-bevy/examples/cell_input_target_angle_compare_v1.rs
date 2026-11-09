//! Compare the reviewed synthetic E5AF20 host receipt with the bounded model.
//! `_CIasin` was replaced by supplied values; native branches and immediate FSIN C2 were not captured.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_angle_wrap_v1::X87CopyAdmission,
    cell_input_target_angle_v1::{self as model, Input, Selection},
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};

const RECEIPT_SHA: &str = "05147d82e102eac006c3617cfcc832052472dfa66436469af06d0a4f6162076e";
const ASIN: [u32; 3] = [0x3f800000, 0x3f000000, 0x3f000000];
const STEPS: [u32; 3] = [0x3e800000, 0x3e800000, 0x3f400000];
const LABELS: [&str; 3] = ["direct_copy", "step", "overshoot"];
const SELECTIONS: [Selection; 3] = [
    Selection::CopyCandidate,
    Selection::Stepped,
    Selection::CandidateAngle,
];
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{field}: invalid hexadecimal field").into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn words(value: &serde_json::Value, field: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let values = value[field].as_array().ok_or("missing word array")?;
    if values.len() != 3 {
        return Err(format!("{field}: expected three words").into());
    }
    let mut out = [0; 3];
    for (dst, item) in out.iter_mut().zip(values) {
        let s = item.as_str().ok_or("word is not text")?;
        if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("{field}: invalid word").into());
        }
        *dst = u32::from_str_radix(s, 16)?;
    }
    Ok(out)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: cell_input_target_angle_compare_v1 <receipt.json> <reviewed-receipt-sha256>"
                .into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (field, expected) in [
        ("schema", "cell-input-target-long-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "d8f56baa9bd5a7b7e2463b2a248cc746d2006f6ffaee554088d404c49814a612",
        ),
        (
            "asm_sha256",
            "3c362bee50556555219f19cffff7ead6f11a9c28d48a196f1d7d0ea4185d19d9",
        ),
        (
            "generator_sha256",
            "fd16702b4524a7c4b9427590936dc818bb92267c599a8ec6363f075449291e5e",
        ),
        (
            "manifest_sha256",
            "6e84e1bef56f52043e1860f6d0d19fb1beb207ff35eaf7a91c8b60227ead09b1",
        ),
        (
            "runner_sha256",
            "1db6d782e762922f9d46ad8621127ee78e15b718ff88cd2a93717112bc8087df",
        ),
        (
            "stdout_sha256",
            "f011bc379d9c8e26ec8e2403e3dacdf4759bb9315c704ad0cc037b3f29babd55",
        ),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    if value["synthetic"].as_bool() != Some(true) || value["native_exit_code"].as_i64() != Some(0) {
        return Err("synthetic status or exit mismatch".into());
    }
    let cases = value["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 3 {
        return Err("unexpected case count".into());
    }
    let mut stdout = Vec::new();
    let mut metadata_bytes = Vec::new();
    for (i, row) in cases.iter().enumerate() {
        if hex(row, "stub_first_bits", 8)? != 0
            || hex(row, "stub_second_bits", 8)? != ASIN[i]
            || hex(row, "scalar_bits", 8)? != STEPS[i]
            || row["expected_branch_unobserved"].as_str() != Some(LABELS[i])
        {
            return Err(format!("case {i}: fixture mismatch").into());
        }
        let metadata = &row["metadata"];
        let cw = hex(metadata, "cw", 4)?;
        let sw = hex(metadata, "sw", 4)?;
        let mxcsr = hex(metadata, "mxcsr", 8)?;
        let eflags = hex(metadata, "eflags", 8)?;
        if cw != 0x027f
            || sw & 0x38cd != 0
            || mxcsr & !0x3f != 0x1f80
            || mxcsr & 0x0d != 0
            || eflags & 0x400 != 0
        {
            return Err(format!("case {i}: FP/stack/direction metadata mismatch").into());
        }
        let expected = words(row, "output_bits")?;
        let actual = model::long_path(
            Input {
                current_y: 1.0,
                candidate: [0.0, 1.0, 0.0],
                current_asin: 0.0,
                candidate_asin: f32::from_bits(ASIN[i]),
                step: f32::from_bits(STEPS[i]),
                period: f32::from_bits(0x40c90fdb),
                baseline: [2.0, 3.0, -7.0],
                max_wrap_iterations: 8,
            },
            Policy::Mxcsr1f80,
            Context::from_control_word(cw as u16)?,
            X87CopyAdmission,
        )?;
        if actual.selection != SELECTIONS[i] || actual.vector.map(f32::to_bits) != expected {
            return Err(format!(
                "case {i}: model {:?} {:08x?}, receipt {:08x?}",
                actual.selection,
                actual.vector.map(f32::to_bits),
                expected
            )
            .into());
        }
        for word in expected {
            stdout.extend(word.to_le_bytes());
        }
        metadata_bytes.extend((cw as u16).to_le_bytes());
        metadata_bytes.extend((sw as u16).to_le_bytes());
        metadata_bytes.extend(mxcsr.to_le_bytes());
        metadata_bytes.extend(eflags.to_le_bytes());
        metadata_bytes.extend(0u32.to_le_bytes());
    }
    stdout.extend(metadata_bytes);
    if format!("{:x}", Sha256::digest(&stdout))
        != value["stdout_sha256"]
            .as_str()
            .ok_or("missing stdout hash")?
    {
        return Err("reconstructed stdout hash mismatch".into());
    }
    println!(
        "MATCH: nine vector words and three model selections for the fixed synthetic fixture."
    );
    println!("Native branch identities and immediate FSIN C2 were not captured. Asin was stubbed; no CRT or live gameplay claim follows. Hardware FSIN and excluded FP status limit this comparison.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
