//! Compare a reviewed fixed native-instruction receipt with the bounded Rust model.
//! This fixture begins after source sampling; it does not represent a game tick.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_movement_short_part_flow_v1, cell_movement_short_part_step_v1::State, native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs};

const RECEIPT_SHA: &str = "cb1d54892fbefd60d2516ba10db918b1a2e07877e4d52ed4eb794c0accb61fd2";

fn hex_field(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid hexadecimal field".into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}

fn words(value: &serde_json::Value, field: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let values = value[field].as_array().ok_or("missing word array")?;
    if values.len() != 3 {
        return Err("word array must have three entries".into());
    }
    let mut result = [0; 3];
    for (index, word) in values.iter().enumerate() {
        let text = word.as_str().ok_or("word must be hexadecimal text")?;
        if text.len() != 8 || !text.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("invalid word".into());
        }
        result[index] = u32::from_str_radix(text, 16)?;
    }
    Ok(result)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_short_flow_compare_v1 <reviewed-receipt.json> <receipt-sha256>".into(),
        );
    }
    if args[1].len() != 64 || !args[1].bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("expected receipt SHA-256 must be 64 hex digits".into());
    }
    if args[1].to_ascii_lowercase() != RECEIPT_SHA {
        return Err("this comparison admits only the reviewed fixed receipt".into());
    }
    let receipt = fs::read(&args[0])?;
    if format!("{:x}", Sha256::digest(&receipt)) != args[1].to_ascii_lowercase() {
        return Err("receipt SHA-256 mismatch".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&receipt)?;
    for (field, expected) in [
        ("schema", "cell-short-flow-native-fixed-v1"),
        (
            "host_sha256",
            "942a14c645216ce110535466036ba54e5b9f22c2a0d13d73ed6052f2f346bc06",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "instruction_sha256",
            "7fb95e9bb9a4ca90cff48af8e555c3db9c7b6761bc41a77d6025d000d87eb42f",
        ),
        ("scalar_bits", "3f800000"),
        ("cw", "027f"),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    if value["synthetic"].as_bool() != Some(true)
        || value["native_exit_code"].as_i64() != Some(0)
        || words(&value, "sample_bits")? != [0x4100_0000, 0x4110_0000, 0]
    {
        return Err("receipt synthetic input mismatch".into());
    }
    let sw = hex_field(&value, "sw", 4)?;
    let mxcsr = hex_field(&value, "mxcsr", 8)?;
    let eflags = hex_field(&value, "eflags", 8)?;
    if sw & 0x38df != 0 || mxcsr & !0x3f != 0x1f80 || mxcsr & 0x1f != 0 || eflags & 0x400 != 0 {
        return Err("receipt FP controls, stack or direction flag outside reviewed policy".into());
    }
    let state = State {
        position_4c: [10., 20., 30.],
        previous_84: [10., 20., 30.],
        accumulator_90: [2., -4., 0.],
        vector_1c4: [4., 6., 0.],
        flags_48: 0x8001,
        counter_4a: 0xffff,
    };
    let result = cell_movement_short_part_flow_v1::step(
        state,
        1,
        1,
        1,
        1.,
        [8., 9., 0.],
        Policy::Mxcsr1f80,
        Context::from_control_word(0x027f)?,
    )?;
    for (field, actual) in [
        ("position_bits", result.position_4c.map(f32::to_bits)),
        ("previous_bits", result.previous_84.map(f32::to_bits)),
        ("accumulator_bits", result.accumulator_90.map(f32::to_bits)),
        ("vector_bits", result.vector_1c4.map(f32::to_bits)),
    ] {
        let native = words(&value, field)?;
        if actual != native {
            return Err(
                format!("{field} differs: native {native:08x?}, Rust {actual:08x?}").into(),
            );
        }
    }
    if value["flags"].as_u64() != Some(u64::from(result.flags_48))
        || value["counter"].as_u64() != Some(u64::from(result.counter_4a))
    {
        return Err("flags or counter differ".into());
    }
    println!("MATCH: 12 actor float words and two flag/counter words, one fixed synthetic post-sampler fixture; x87 CW 027F, MXCSR control 1F80");
    println!("Previous-position and vector words were initialized by the host; their earlier source writes were not executed.");
    println!("No live actor, field sampler, source branch selection, runtime scheduling, FP sticky-status, or gameplay claim.");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
