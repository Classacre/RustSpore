//! Compare four pinned synthetic 52C2D7 record slices with the checked model.
//! This does not establish the full producer, palette, mesh, or live editor.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_record_v1::{self as model, Candidate, DirectOutputAdmission},
    native_sse::Policy,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "3590936dcb646ba9a8a630bdf44e08b422f75ccf6a9a8798ac239807d03ba309";
const CASES: [[(u32, u32); 4]; 4] = [
    [
        (0, 0x3f800000),
        (255, 0x40000000),
        (256, 0),
        (0x12345678, 0x3f800000),
    ],
    [
        (0x102, 0x60ad78ec),
        (0x203, 0xe0ad78ec),
        (4, 0x40400000),
        (5, 0x3f800000),
    ],
    [(0x11223344, 0x80000000), (1, 0x3f800000), (2, 0), (3, 0)],
    [
        (0xabc00101, 0x40e00000),
        (0x203, 0x40000000),
        (0x304, 0x40a00000),
        (0x405, 0x40400000),
    ],
];
fn hex(value: &serde_json::Value, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = value.as_str().ok_or("missing hexadecimal field")?;
    if s.len() != width || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid hexadecimal field".into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err("usage: creature_generated_skin_record_compare_v1 <receipt.json> <reviewed-receipt-sha256>".into());
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (key, expected) in [
        ("schema", "creature-skin-record-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "af34f5ccec65bfb202c68e5b6915e9e426495325fa03ec9e99ef10bd1c121a9a",
        ),
        (
            "instruction_sha256",
            "9a84439add89cb47c862bce510dcb82d6cf4f5862b1ae53441ee18d4e994b4b4",
        ),
        (
            "fixture_sha256",
            "b11f7587b961dd42e5dc10b517e139df488f636ca8657f4a89f30b2a253be85b",
        ),
    ] {
        if v[key].as_str() != Some(expected) {
            return Err(format!("receipt {key} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("invalid synthetic status/exit".into());
    }
    let rows = v["records"].as_array().ok_or("missing records")?;
    if rows.len() != 4 {
        return Err("record count mismatch".into());
    }
    for (i, row) in rows.iter().enumerate() {
        if row["id"].as_u64() != Some(i as u64) {
            return Err(format!("case {i}: identity mismatch").into());
        }
        let inputs = row["candidate_words"]
            .as_array()
            .ok_or("missing candidates")?;
        if inputs.len() != 4 {
            return Err(format!("case {i}: candidate count mismatch").into());
        }
        for (j, input) in inputs.iter().enumerate() {
            let pair = input.as_array().ok_or("candidate is not a pair")?;
            if pair.len() != 2 || (hex(&pair[0], 8)?, hex(&pair[1], 8)?) != CASES[i][j] {
                return Err(format!("case {i}: candidate {j} mismatch").into());
            }
        }
        let cw = hex(&row["cw"], 4)?;
        let sw = hex(&row["sw"], 4)?;
        let mxcsr = hex(&row["mxcsr"], 8)?;
        let flags = hex(&row["eflags"], 8)?;
        if cw != 0x027f
            || sw & 0x38df != 0
            || mxcsr & !0x3f != 0x1f80
            || mxcsr & 0x1f != 0
            || flags & 0x400 != 0
        {
            return Err(format!("case {i}: FP/DF metadata mismatch").into());
        }
        let text = row["record_hex"].as_str().ok_or("missing record hex")?;
        if text.len() != 40 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("case {i}: invalid record bytes").into());
        }
        let mut expected = [0u8; 20];
        for (j, out) in expected.iter_mut().enumerate() {
            *out = u8::from_str_radix(&text[j * 2..j * 2 + 2], 16)?;
        }
        let candidates = CASES[i].map(|(index_word, word)| Candidate {
            index_word,
            weight: f32::from_bits(word),
        });
        let actual = model::record(candidates, Policy::Mxcsr1f80, DirectOutputAdmission)?;
        if actual != expected {
            return Err(
                format!("case {i}: mismatch model {actual:02x?}, receipt {expected:02x?}").into(),
            );
        }
    }
    println!("MATCH: all four 20-byte records for fixed synthetic staged candidates.");
    println!("This excludes candidate generation, smoothing, live creature/palette and baked mesh/editor behavior. FP sticky state is not compared with the model.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
