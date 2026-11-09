//! Compare the retained synthetic clear-edge accumulation receipt, not a live editor.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_edge_accum_v1::{accumulate, Endpoint, Query, SnapshotAdmission},
    native_sse::Policy,
    native_x87::{Context, Precision, Rounding},
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "93ee442109458f9522a44d73df3b701da01f6ba437c387f0c33a144a6643d12a";
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
        return Err(
            "usage: creature_skin_edge_accum_compare_v1 <receipt> <reviewed sha256>".into(),
        );
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-edge-accum-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "fdccba7482dfd583bf186841046541944f62cad70edd647954c31ce141a28023",
        ),
        (
            "manifest_sha256",
            "87f8f0630b5d73ffa4d1ffb334fe8c18be493805981c0ab3c79d764b575f5a61",
        ),
        (
            "runner_sha256",
            "b620c491622407cbf37536757239b0c588cbc1f5cd8268f568eb6a875faac9fb",
        ),
        (
            "stdout_sha256",
            "d6db3a113e73adc7e806b7c7c2f852b3a9c07d2e7e18a0112b54275a51f67fb7",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    if v["source_ranges"]["Helper"].as_str()
        != Some("abbd724a1f8b947a405031e3f84c6d302459d7fd67fbb61845d4fd31b969f1b3")
        || v["source_ranges"]["Edge"].as_str()
            != Some("80466b426eb620c5d938f7fd7c30716c40b909f918fd64bc02c449ea4bd091ad")
    {
        return Err("source range mismatch".into());
    }
    // These fixed source inputs are bound by the host and manifest hashes;
    // the receipt itself stores only the point and optional remap.
    let fixtures: [(u32, Option<&[u32]>); 5] = [
        (0, None),
        (0x3f800000, None),
        (0x3fc00000, None),
        (0x3fc00000, Some(&[3, 2, 2])),
        (0x40400000, None),
    ];
    let endpoints = [
        Endpoint {
            position: [0.; 3],
            members: &[0, 1],
        },
        Endpoint {
            position: [2., 0., 0.],
            members: &[1, 2],
        },
    ];
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 5 {
        return Err("case count".into());
    }
    let mut stdout = Vec::new();
    for (i, (row, (point, remap))) in cases.iter().zip(fixtures).enumerate() {
        if row["id"].as_u64() != Some(i as u64)
            || hex(&row["point_x_bits"], 8)? != point
            || row["remap"] != serde_json::json!(remap)
        {
            return Err(format!("case {i} fixture mismatch").into());
        }
        let got = accumulate(
            &[0.; 4],
            [f32::from_bits(point), 0., 0.],
            &[Query { key: 0, weight: 1. }],
            &[[0, 1]],
            &endpoints,
            remap,
            Policy::Mxcsr1f80,
            Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            SnapshotAdmission,
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
    println!("MATCH: 20 accumulated words across five fixed synthetic cases; receipt inputs and FP metadata validated.");
    println!("Relocated source branch and original subtraction helper; six constant operands patched and exit appended. Synthetic fixture, no guard pages or native spatial query, map construction, correction, smoothing, palette, mesh or live editor. Hashes assume stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
