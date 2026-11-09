//! Compare one pinned original-instruction receipt with the bounded input model.
//! This fixed synthetic fixture uses callback stubs, not a live controller.
use sha2::{Digest, Sha256};
use spore_core::{cell_indexed_input_vector_v1, native_sse::Policy};
use std::{error::Error, fs::File, io::Read};

const RECEIPT_SHA: &str = "eea707b4e6d581d05e0bd5d26eebb10654ecd9f980c571442394bdff9f28c26d";

fn hex(text: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    if text.len() != width || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid hexadecimal word".into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn words<const N: usize>(
    value: &serde_json::Value,
    field: &str,
) -> Result<[u32; N], Box<dyn Error>> {
    let values = value[field].as_array().ok_or("missing word array")?;
    if values.len() != N {
        return Err(format!("{field}: expected {N} words").into());
    }
    let mut result = [0; N];
    for (index, value) in values.iter().enumerate() {
        result[index] = hex(value.as_str().ok_or("word must be hexadecimal text")?, 8)?;
    }
    Ok(result)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_indexed_input_compare_v1 <reviewed-receipt.json> <receipt-sha256>".into(),
        );
    }
    if args[1].len() != 64
        || !args[1].bytes().all(|c| c.is_ascii_hexdigit())
        || args[1].to_ascii_lowercase() != RECEIPT_SHA
    {
        return Err("expected SHA-256 must identify the reviewed fixed receipt".into());
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
        ("schema", "cell-indexed-input-native-fixed-v1"),
        (
            "host_sha256",
            "f7d69119b55726061a1f2f1c12852c578faefd063eb7f3dd985018713f240c74",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "instruction_sha256",
            "621939edf13bcaa056bdfef8963a0fcd95558b76d84e4f0539759c50dea8792e",
        ),
        (
            "constant_sha256",
            "ab43fbd0c2e97ddd3c5c378b47c725170aea81b0b572a646a0d083a55b5068ad",
        ),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    let snapshot = words::<8>(&value, "snapshot_words")?;
    let baseline = words::<3>(&value, "baseline_bits")?;
    if snapshot != [0, 0, 0x04080012, 0, 0, 0, 0, 0]
        || baseline != [0x40000000, 0x80000000, 0x40800000]
        || value["layout_al"]
            .as_array()
            .map(|a| a.iter().map(|v| v.as_u64()).collect::<Vec<_>>())
            != Some(vec![Some(1), Some(0), Some(0), Some(0)])
        || value["synthetic"].as_bool() != Some(true)
        || value["native_exit_code"].as_i64() != Some(0)
        || value["callback_count"].as_u64() != Some(4)
    {
        return Err("receipt fixed fixture inputs or callback count mismatch".into());
    }
    let sw = hex(value["sw"].as_str().ok_or("missing sw")?, 4)?;
    let mxcsr = hex(value["mxcsr"].as_str().ok_or("missing mxcsr")?, 8)?;
    let eflags = hex(value["eflags"].as_str().ok_or("missing eflags")?, 8)?;
    if sw & 0x38df != 0 || mxcsr & !0x3f != 0x1f80 || mxcsr & 0x1f != 0 || eflags & 0x400 != 0 {
        return Err("receipt FP controls, stack or direction flag outside reviewed policy".into());
    }
    let actual = cell_indexed_input_vector_v1::assemble(
        snapshot,
        [1, 0, 0, 0],
        baseline.map(f32::from_bits),
        Policy::Mxcsr1f80,
    )?;
    let native = words::<3>(&value, "vector_bits")?;
    if actual.vector.map(f32::to_bits) != native {
        return Err(format!(
            "vector mismatch: Rust {:08x?}, receipt {native:08x?}",
            actual.vector.map(f32::to_bits)
        )
        .into());
    }
    let requested = actual.side_effect_requests.iter().any(|&v| v);
    if !requested
        || value["state_c4"].as_u64() != Some(0)
        || value["side_effect_e0"].as_u64() != Some(1)
    {
        return Err("aggregate side-effect request differs from final receipt fields".into());
    }
    println!("MATCH: three vector words and aggregate C4/E0 request for one fixed synthetic snapshot; four callback stubs, MXCSR 1F80 controls.");
    println!("Final memory does not independently establish which of the four groups performed a store. No live input, controller, normalization, actor target or gameplay is established.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
