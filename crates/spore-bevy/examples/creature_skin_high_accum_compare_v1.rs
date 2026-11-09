//! Compare the retained synthetic high-bit accumulation receipt, not a live editor.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_high_accum_v1::{accumulate, AccumulationAdmission, Query},
    native_sse::Policy,
    native_x87::{Context, Precision, Rounding},
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "313699fc3c228145db76e37d48072d886955a676dee0298d067399b239c620fc";
fn hex(v: &serde_json::Value, n: usize) -> Result<u32, Box<dyn Error>> {
    let s = v.as_str().ok_or("missing hex")?;
    if s.len() != n || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("bad hex".into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(PIN) {
        return Err("usage: creature_skin_high_accum_compare_v1 <receipt> <reviewed sha256>".into());
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-high-accum-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "2092458608a1fe4a3df97df11cd2fa605b6e6be06a102580d7ac56f569c46fe7",
        ),
        (
            "manifest_sha256",
            "65776e9207db5d14adffe8f39c6996660afec3b88b998c46375c940ee4103b74",
        ),
        (
            "instruction_sha256",
            "1ec3f11f1b0dfb79fd2f55f3cf460afdf5556fe6cebefbcd22918c707ace32e5",
        ),
        (
            "runner_sha256",
            "4c25a6022e87a6b6b5a8cfe8f092c1a922b9a9103fd85921a479334abbb239d5",
        ),
        (
            "stdout_sha256",
            "19d97a7f43e44538135daff8d648be0984168107794425343b03a89f0f840058",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    let fixtures: [(&[u32], u32, Option<&[u32]>); 3] = [
        (&[0, 1, 1, 2], 0x40800000, None),
        (&[0, 1, 1, 2], 0x40800000, Some(&[2, 2, 64])),
        (&[0, 1, 1], 0x3f800000, None),
    ];
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 3 {
        return Err("case count".into());
    }
    let mut stdout = Vec::new();
    for (i, (row, (members, weight, remap))) in cases.iter().zip(fixtures).enumerate() {
        if row["id"].as_u64() != Some(i as u64)
            || row["input"]["members"] != serde_json::json!(members)
            || hex(&row["input"]["weight_bits"], 8)? != weight
            || row["input"]["remap"] != serde_json::json!(remap)
        {
            return Err(format!("case {i} fixture mismatch").into());
        }
        let got = accumulate(
            &[0.; 4],
            &[Query {
                key: 0x80000000,
                weight: f32::from_bits(weight),
            }],
            &[members],
            remap,
            Policy::Mxcsr1f80,
            Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            AccumulationAdmission,
        )?;
        let words = row["weights_bits"].as_array().ok_or("missing weights")?;
        if words.len() != 4 {
            return Err("weight count".into());
        }
        for (j, w) in words.iter().enumerate() {
            let x = hex(w, 8)?;
            if got[j].to_bits() != x {
                return Err(format!("case {i} word {j} mismatch").into());
            }
            stdout.extend(x.to_le_bytes())
        }
        let cw = hex(&row["cw"], 4)?;
        let sw = hex(&row["sw"], 4)?;
        let mx = hex(&row["mxcsr"], 8)?;
        let flags = hex(&row["eflags"], 8)?;
        if cw != 0x27f
            || sw & 0x38cd != 0
            || mx & !0x3f != 0x1f80
            || mx & 0x0d != 0
            || flags & 0x400 != 0
        {
            return Err(format!("case {i} FP/DF").into());
        }
        stdout.extend((cw as u16).to_le_bytes());
        stdout.extend((sw as u16).to_le_bytes());
        stdout.extend(mx.to_le_bytes());
        stdout.extend(flags.to_le_bytes());
        stdout.extend(0u32.to_le_bytes());
    }
    if format!("{:x}", Sha256::digest(&stdout))
        != v["stdout_sha256"].as_str().ok_or("missing stdout")?
    {
        return Err("stdout hash".into());
    }
    println!("MATCH: 12 accumulated words across three fixed synthetic cases; receipt inputs and FP metadata validated.");
    println!("Relocated source branch with appended exit; no native spatial query, map construction, ranking, smoothing, palette or live editor. Endpoint hashes assume stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
