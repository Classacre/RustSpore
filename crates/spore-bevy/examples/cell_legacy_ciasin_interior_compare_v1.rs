//! Compare a pinned synthetic legacy CIasin interior instruction receipt.
//! This does not establish a full CRT call, dispatch, or live game behavior.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_legacy_ciasin_interior_v1::{self as model, InteriorAdmission},
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "c4c3b1cde5a505d5b6240729fab83e1b04061cec7d5dea454a969ef815facae2";
const INPUTS: [u32; 7] = [
    0, 0x80000000, 0x3f000000, 0xbf000000, 1, 0x3f7fffff, 0xbf7fffff,
];
fn hex(v: &serde_json::Value, key: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = v[key].as_str().ok_or("missing hex field")?;
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {key}").into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: cell_legacy_ciasin_interior_compare_v1 <receipt.json> <reviewed-sha256>".into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or hash mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (key, expected) in [
        ("schema", "cell-legacy-ciasin-interior-native-fixed-v1"),
        (
            "source_dll_sha256",
            "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185",
        ),
        (
            "host_sha256",
            "f77e0568888cd15323cef5149f9bcc10d9ef5f307d2e02b94e93602ebd7df5ae",
        ),
        (
            "instruction_sha256",
            "d833289e80ab7f3fa6ba94fa1880577530ae2ebdec718612a09fb86fad4944f1",
        ),
        (
            "fixture_sha256",
            "455b68baf858d9f05196327e2f21f7a201c5e0422d3bba0833aac479754ac713",
        ),
    ] {
        if v[key].as_str() != Some(expected) {
            return Err(format!("receipt {key} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("invalid synthetic status or exit".into());
    }
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != INPUTS.len() {
        return Err("case count mismatch".into());
    }
    for (i, row) in cases.iter().enumerate() {
        if row["id"].as_u64() != Some(i as u64) || hex(row, "input_bits", 8)? != INPUTS[i] {
            return Err(format!("case {i}: input mismatch").into());
        }
        let cw = hex(row, "cw", 4)?;
        let sw = hex(row, "sw", 4)?;
        let mx = hex(row, "mxcsr", 8)?;
        let flags = hex(row, "eflags", 8)?;
        if cw != 0x27f || sw & 0x384d != 0 || mx != 0x1f80 || flags & 0x400 != 0 {
            return Err(format!("case {i}: FP/DF metadata mismatch").into());
        }
        let expected = hex(row, "output_bits", 8)?;
        let actual = model::calculate(
            f32::from_bits(INPUTS[i]),
            Context::from_control_word(cw as u16)?,
            InteriorAdmission,
        )?
        .to_bits();
        if actual != expected {
            return Err(format!("case {i}: model {actual:08x}, receipt {expected:08x}").into());
        }
    }
    println!("MATCH: all seven binary32 outputs from fixed synthetic legacy CIasin interior instruction cases.");
    println!("Not a full CRT call or live game: dispatch, classification, control adjustment, return policy and sticky status are outside the comparison; FPATAN is hardware-dependent.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
