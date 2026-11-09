//! Compare a pinned synthetic relocated full legacy CIasin path receipt.
//! This does not establish live CRT dispatch, SSE2, or game behavior.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_legacy_ciasin_interior_v1::{self as model, InteriorAdmission},
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "15997af18e4f174e70980a08e81dae4ea3cb57815fef446159d250d8ae954baa";
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
            "usage: cell_legacy_ciasin_full_compare_v1 <receipt.json> <reviewed-sha256>".into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or hash mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (key, expected) in [
        (
            "schema",
            "cell-legacy-ciasin-full-relocated-native-fixed-v1",
        ),
        (
            "dll_sha256",
            "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185",
        ),
        (
            "host_sha256",
            "0a2a5d3d0f53ae6818523a76516764f79b2fb95b22ff7ba33628bb4fa577cf30",
        ),
        (
            "manifest_sha256",
            "1d55093f025769bac7c172538267f91f5e64f4eed6423b028a3157cd42f0f13f",
        ),
        (
            "source_manifest_sha256",
            "2056806bdc0c5f015fbacf2e15fb50cf90f5ee50b446381652870fea9421133a",
        ),
    ] {
        if v[key].as_str() != Some(expected) {
            return Err(format!("receipt {key} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("invalid synthetic status or exit".into());
    }
    let fixture: Vec<u8> = INPUTS.iter().flat_map(|x| x.to_le_bytes()).collect();
    if format!("{:x}", Sha256::digest(&fixture))
        != "455b68baf858d9f05196327e2f21f7a201c5e0422d3bba0833aac479754ac713"
    {
        return Err("fixed fixture digest mismatch".into());
    }
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != INPUTS.len() * 2 {
        return Err("case count mismatch".into());
    }
    for (i, row) in cases.iter().enumerate() {
        if row["return_global"].as_u64() != Some((i / INPUTS.len()) as u64)
            || hex(row, "input_bits", 8)? != INPUTS[i % INPUTS.len()]
        {
            return Err(format!("case {i}: input mismatch").into());
        }
        let cw = hex(row, "cw", 4)?;
        let sw = hex(row, "sw", 4)?;
        let mx = hex(row, "mxcsr", 8)?;
        let flags = hex(row, "eflags", 8)?;
        if cw != 0x27f || sw & 0x38cd != 0 || mx != 0x1f80 || flags & 0x400 != 0 {
            return Err(format!("case {i}: FP/DF metadata mismatch").into());
        }
        let expected = hex(row, "output_bits", 8)?;
        let actual = model::calculate(
            f32::from_bits(INPUTS[i % INPUTS.len()]),
            Context::from_control_word(cw as u16)?,
            InteriorAdmission,
        )?
        .to_bits();
        if actual != expected {
            return Err(format!("case {i}: model {actual:08x}, receipt {expected:08x}").into());
        }
    }
    println!("MATCH: all 14 binary32 outputs across both synthetic return-global values match the interior model.");
    println!("Controlled legacy path with disabled SSE2 dispatch and CW 027F only; no live DLL selection, SSE2 path, endpoints or gameplay. Branch identities are inferred; FPATAN is hardware-dependent and sticky status is not compared to the model.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
