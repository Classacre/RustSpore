//! Compare two fixed E5AF20 early-path observations with the bounded model.
//! Branch identity is inferred from source and fixed inputs, not instrumented.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_target_early_v1::{self, EarlyBranch, Outcome, X87CopyAdmission},
    native_sse::Policy,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "1c947a44331584e143067df4e68aac215f11e4caefebd55be18c7ced8c014a34";

fn hex(text: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hexadecimal field".into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn words(value: &serde_json::Value, name: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let array = value[name].as_array().ok_or("missing word array")?;
    if array.len() != 3 {
        return Err(format!("{name}: expected three words").into());
    }
    let mut result = [0; 3];
    for (i, v) in array.iter().enumerate() {
        result[i] = hex(v.as_str().ok_or("word is not text")?, 8)?;
    }
    Ok(result)
}
fn field(value: &serde_json::Value, name: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    hex(
        value[name].as_str().ok_or("missing hexadecimal field")?,
        width,
    )
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_input_target_early_compare_v1 <reviewed-receipt.json> <receipt-sha256>"
                .into(),
        );
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
    for (key, expected) in [
        ("schema", "cell-input-target-early-native-fixed-v1"),
        (
            "host_sha256",
            "b19501f981d06f8b0fbd30cb01447dea080ffd7508c30062eb19e8ad087b001c",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "instruction_sha256",
            "1858ca70a26f1d28f1951918c904761892830e95b2dae36b598fbd15dd7feff3",
        ),
        (
            "threshold_sha256",
            "54e347282ecf0fd46f002970fa012534725e0bf3b2ad2f4ca0efc0134d7e1d28",
        ),
    ] {
        if value[key].as_str() != Some(expected) {
            return Err(format!("receipt {key} mismatch").into());
        }
    }
    if value["synthetic"].as_bool() != Some(true) || value["native_exit_code"].as_i64() != Some(0) {
        return Err("receipt not a successful synthetic fixture".into());
    }
    let cases = value["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 2 {
        return Err("expected two cases".into());
    }
    let expected = [
        (
            "small-current",
            EarlyBranch::SmallCurrent,
            [0, 0, 0],
            [0x80000000, 1, 0x80000001],
        ),
        (
            "small-difference",
            EarlyBranch::SmallDifference,
            [0x3f800000, 0, 0],
            [0x3f800000, 0x80000000, 1],
        ),
    ];
    for (case, (label, branch, a, b)) in cases.iter().zip(expected) {
        if case["branch"].as_str() != Some(label)
            || words(case, "a_bits")? != a
            || words(case, "b_bits")? != b
        {
            return Err(format!("{label}: fixture inputs or inferred branch mismatch").into());
        }
        let meta = &case["metadata"];
        let cw = field(meta, "cw", 4)?;
        let sw = field(meta, "sw", 4)?;
        let mxcsr = field(meta, "mxcsr", 8)?;
        let eflags = field(meta, "eflags", 8)?;
        // As in the reviewed runner, denormal/underflow/precision sticky bits
        // are permitted; invalid, divide-by-zero, overflow and stack fault are not.
        if cw != 0x027f
            || sw & 0x38cd != 0
            || mxcsr & !0x3f != 0x1f80
            || mxcsr & 0x0d != 0
            || eflags & 0x400 != 0
        {
            return Err(format!("{label}: unsupported FP/stack/direction metadata").into());
        }
        let observed = words(case, "output_bits")?;
        match cell_input_target_early_v1::early(
            a.map(f32::from_bits),
            b.map(f32::from_bits),
            Policy::Mxcsr1f80,
            X87CopyAdmission,
        )? {
            Outcome::Early {
                branch: actual_branch,
                candidate,
            } => {
                if actual_branch != branch || candidate.map(f32::to_bits) != observed {
                    return Err(format!("{label}: branch or output words differ").into());
                }
            }
            Outcome::NotEarly => {
                return Err(format!("{label}: model reached unimplemented path").into())
            }
        }
    }
    println!("MATCH: two inferred early branches and six candidate words for one pinned synthetic receipt; denormal/underflow/precision status permitted.");
    println!("No native actor, live input, scheduling, or long angle path is established.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
