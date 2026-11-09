//! Compare six fixed synthetic E5AE70 observations against the bounded model.
//! Immediate FSIN C2 was not captured; this is not a live actor observation.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_angle_vector_v1::{self as model, Branch},
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};

const RECEIPT_SHA: &str = "9f7198a32655d88e2f022668f1236f59b7a80b265900c19f4e7a88835df5c256";
const ANGLES: [u32; 6] = [
    0, 0x80000000, 0x3fc90fdb, 0xbfc90fdb, 0x40100000, 0xc0100000,
];
const BRANCHES: [Branch; 6] = [
    Branch::Middle,
    Branch::Middle,
    Branch::Middle,
    Branch::Middle,
    Branch::PositiveOuter,
    Branch::NegativeOuter,
];
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{field}: invalid hexadecimal word").into());
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
        let word = item.as_str().ok_or("word is not text")?;
        if word.len() != 8 || !word.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("{field}: invalid word").into());
        }
        *dst = u32::from_str_radix(word, 16)?;
    }
    Ok(out)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: cell_input_angle_vector_compare_v1 <receipt.json> <reviewed-receipt-sha256>"
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
        ("schema", "cell-angle-vector-native-fixed-v1"),
        (
            "host_sha256",
            "e394329d7d2faf0d60eb0f9ee086ba94821d650264ec2e19a6cc3c7d036c5d58",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "instruction_sha256",
            "21e084b5060f54f84721b0fb0c5b78621900d480f4d5c386fb3fa2a7f7234b21",
        ),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    let baseline = words(&value, "baseline_bits")?;
    if baseline != [0x40000000, 0x40400000, 0xc0e00000]
        || value["synthetic"].as_bool() != Some(true)
        || value["native_exit_code"].as_i64() != Some(0)
    {
        return Err("baseline or synthetic status mismatch".into());
    }
    let cases = value["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != ANGLES.len() {
        return Err("unexpected case count".into());
    }
    for (index, row) in cases.iter().enumerate() {
        if row["id"].as_u64() != Some(index as u64) || hex(row, "angle_bits", 8)? != ANGLES[index] {
            return Err(format!("case {index}: identity/input mismatch").into());
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
        let expected = words(row, "vector_bits")?;
        let actual = model::angle_vector(
            f32::from_bits(ANGLES[index]),
            baseline.map(f32::from_bits),
            Policy::Mxcsr1f80,
            Context::from_control_word(cw as u16)?,
        )?;
        if actual.branch != BRANCHES[index] || actual.vector.map(f32::to_bits) != expected {
            return Err(format!(
                "case {index}: branch/vector mismatch: model {:?} {:08x?}, receipt {:08x?}",
                actual.branch,
                actual.vector.map(f32::to_bits),
                expected
            )
            .into());
        }
    }
    println!("MATCH: 18 vector words and six model branch identities on fixed synthetic cases.");
    println!("Immediate FSIN C2 and native branch identity were not captured; no live actor or gameplay was tested. Hardware FSIN and excluded FP sticky state limit the result.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
