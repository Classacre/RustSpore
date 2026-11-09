//! Compare sequential synthetic setter snapshots with the reviewed relocated host.
//! The original ignores code 0x100; this checked API returns a transactional error.
use sha2::{Digest, Sha256};
use spore_core::cell_virtual_key_snapshot_v1::{DownGate, Error as ModelError, Event, Snapshot};
use std::{error::Error, fs::File, io::Read};

const RECEIPT_SHA: &str = "be98eaa720fa77a2200f1e98f34fa3a568c8fe616956d0c7fbfffb8e28e61f41";
const ACTIONS: [(&str, u32); 10] = [
    ("Down", 0x10),
    ("Down", 0x11),
    ("Down", 0x12),
    ("Down", 0xe1),
    ("Down", 0x5b),
    ("Up", 0x11),
    ("Up", 0x11),
    ("Down", 0xff),
    ("Down", 0x100),
    ("Up", 0xff),
];
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = value[field].as_str().ok_or("missing hex field")?;
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {field}").into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: cell_virtual_key_compare_v1 <receipt.json> <reviewed-receipt-sha256>".into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (field, expected) in [
        ("schema", "cell-key-bit-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "c514595d3dd642518677a113fe0203d28a6723943e9b571a38cd0b19815f7e30",
        ),
        (
            "asm_sha256",
            "b3b6ab3011802d1aae1ce9a1e5eb5c118a135d33b43f49529ba59b8f1d2050f7",
        ),
        (
            "generator_sha256",
            "748150b57f3207d7a9b5545c94770bfbfaf0cf3fb1f3fcdeeedf34a097b7c7ae",
        ),
        (
            "manifest_sha256",
            "b82a5495033ce302feddd051429e5ff1e8546688baa3ecca351650ca8f767b79",
        ),
        (
            "relocated_manifest_sha256",
            "ea5e6a329963f7a1a8243dc225f1131800588ce8b8bf5a940c2f955808a31fc2",
        ),
        (
            "runner_sha256",
            "abd97801bcdbad0cf440a08be04cdbf192b4c572fb92407159be74f8681173b3",
        ),
        (
            "stdout_sha256",
            "4552225d4346e41a8339507603eb29a11bbbcc54cc1c151593def4ced24caeea",
        ),
        ("initial_summary", "cafebabe"),
    ] {
        if v[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("receipt synthetic/exit mismatch".into());
    }
    let actions = v["actions"].as_array().ok_or("missing actions")?;
    if actions.len() != ACTIONS.len() {
        return Err("unexpected action count".into());
    }
    let mut state = Snapshot::new([0; 8], 0xcafebabe);
    let mut stdout = Vec::new();
    for (index, (row, &(kind, code))) in actions.iter().zip(&ACTIONS).enumerate() {
        if row["kind"].as_str() != Some(kind) || hex(row, "code", 8)? != code {
            return Err(format!("action {index}: identity mismatch").into());
        }
        let event = if kind == "Down" {
            Event::Down {
                code,
                gate: DownGate::Passed,
            }
        } else {
            Event::Up { code }
        };
        let before = state;
        if code == 0x100 {
            if state.apply(event) != Err(ModelError::CodeOutOfRange) || state != before {
                return Err("out-of-range action was not transactional".into());
            }
        } else if !state
            .apply(event)
            .map_err(|e| format!("model error: {e:?}"))?
            .summary_recomputed
        {
            return Err(format!("action {index}: summary not recomputed").into());
        }
        let expected = row["state_words"].as_array().ok_or("missing state words")?;
        if expected.len() != 9 {
            return Err(format!("action {index}: word count").into());
        }
        let mut actual = [0u32; 9];
        actual[..8].copy_from_slice(&state.words());
        actual[8] = state.summary();
        for (j, (item, word)) in expected.iter().zip(actual).enumerate() {
            let s = item.as_str().ok_or("word not text")?;
            if s.len() != 8
                || !s.bytes().all(|b| b.is_ascii_hexdigit())
                || u32::from_str_radix(s, 16)? != word
            {
                return Err(format!("action {index} word {j} mismatch").into());
            }
            stdout.extend(word.to_le_bytes());
        }
    }
    let m = &v["metadata"];
    let cw = hex(m, "cw", 4)?;
    let sw = hex(m, "sw", 4)?;
    let mx = hex(m, "mxcsr", 8)?;
    let flags = hex(m, "eflags", 8)?;
    if cw != 0x27f || sw & 0x38ff != 0 || mx != 0x1f80 || flags & 0x400 != 0 {
        return Err("FP/DF metadata mismatch".into());
    }
    stdout.extend((cw as u16).to_le_bytes());
    stdout.extend((sw as u16).to_le_bytes());
    stdout.extend(mx.to_le_bytes());
    stdout.extend(flags.to_le_bytes());
    stdout.extend(0u32.to_le_bytes());
    if format!("{:x}", Sha256::digest(&stdout))
        != v["stdout_sha256"].as_str().ok_or("missing stdout hash")?
    {
        return Err("stdout digest mismatch".into());
    }
    println!("MATCH: 80 bit words and 10 summary words across ten sequential synthetic actions.");
    println!("Native ignores code 0x100; the Rust API rejects it transactionally. No down-gate, live routing, locale, H toggle or game was tested; final DF follows wrapper CLD.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
