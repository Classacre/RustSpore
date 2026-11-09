//! Compare a pinned synthetic normalization receipt with the bounded model.
//! One active vector only; this is not a live player or input observation.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_candidate_normalize_v1::{self, Outcome},
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "91e4c735ae750d53f80c52c37c10263e3cbfefb6d446c97136036973b99c9909";
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{field}: invalid hexadecimal word").into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn words(value: &serde_json::Value, field: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let arr = value[field].as_array().ok_or("missing word array")?;
    if arr.len() != 3 {
        return Err(format!("{field}: expected three words").into());
    }
    let mut result = [0; 3];
    for (i, item) in arr.iter().enumerate() {
        let text = item.as_str().ok_or("word is not text")?;
        if text.len() != 8 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("{field}: invalid word").into());
        }
        result[i] = u32::from_str_radix(text, 16)?;
    }
    Ok(result)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: cell_input_candidate_normalize_compare_v1 <reviewed-receipt.json> <receipt-sha256>".into());
    }
    if args[1].len() != 64
        || !args[1].bytes().all(|b| b.is_ascii_hexdigit())
        || args[1].to_ascii_lowercase() != RECEIPT_SHA
    {
        return Err("expected SHA-256 must identify the reviewed receipt".into());
    }
    let mut receipt = Vec::new();
    File::open(&args[0])?
        .take(65537)
        .read_to_end(&mut receipt)?;
    if receipt.len() > 65536 || format!("{:x}", Sha256::digest(&receipt)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&receipt)?;
    for (field, expected) in [
        ("schema", "cell-candidate-normalize-native-fixed-v1"),
        (
            "host_sha256",
            "d7318daf31c09577b65af2a7e43ec04b0761a55a4cf1d7dd3301c2b21b6c721e",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "instruction_sha256",
            "8d9580a44f8e9a88c891f34979fdd2d8bfc92cb8345ee21e0d539c2df7c856d4",
        ),
        (
            "threshold_sha256",
            "54e347282ecf0fd46f002970fa012534725e0bf3b2ad2f4ca0efc0134d7e1d28",
        ),
        (
            "epsilon_sha256",
            "b2c6a8c83803ff33154c42757d93d1a39e827f2745b79e92eb09bd4db3aa8e7a",
        ),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    let input = words(&value, "input_bits")?;
    if input != [0x40400000, 0x40800000, 0x80000000]
        || value["synthetic"].as_bool() != Some(true)
        || value["native_exit_code"].as_i64() != Some(0)
    {
        return Err("receipt input or synthetic status mismatch".into());
    }
    let cw = hex(&value, "cw", 4)?;
    let sw = hex(&value, "sw", 4)?;
    let mxcsr = hex(&value, "mxcsr", 8)?;
    let eflags = hex(&value, "eflags", 8)?;
    if cw != 0x027f
        || sw & 0x38df != 0
        || mxcsr & !0x3f != 0x1f80
        || mxcsr & 0x1f != 0
        || eflags & 0x400 != 0
    {
        return Err("receipt FP controls, stack or direction flag outside reviewed policy".into());
    }
    let candidate = words(&value, "candidate_bits")?;
    let inverse = hex(&value, "reciprocal_bits", 8)?;
    match cell_input_candidate_normalize_v1::normalize(
        input.map(f32::from_bits),
        Policy::Mxcsr1f80,
        Context::from_control_word(cw as u16)?,
    )? {
        Outcome::Active {
            reciprocal,
            candidate: actual,
            ..
        } => {
            if reciprocal.to_bits() != inverse || actual.map(f32::to_bits) != candidate {
                return Err(format!("normalization mismatch: Rust reciprocal={:08x} candidate={:08x?}; receipt reciprocal={inverse:08x} candidate={candidate:08x?}", reciprocal.to_bits(), actual.map(f32::to_bits)).into());
            }
        }
        Outcome::Inactive { .. } => return Err("model did not take the active branch".into()),
    }
    println!("MATCH: reciprocal and three candidate words for one pinned synthetic active vector; x87 CW 027F and MXCSR 1F80 controls.");
    println!("No live input, player, target helper, scheduling or gameplay is established; FP sticky status is not compared against the model.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
