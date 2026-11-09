//! Compare the retained synthetic mixed ordered accumulation receipt, not a live editor.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_edge_accum_v1::Endpoint,
    creature_generated_skin_mixed_accum_v1::{accumulate_rank_record, Admission, Query},
    native_sse::Policy,
    native_x87::{Context, Precision, Rounding},
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "b84fff5c45162fca7485304b35961b07bd33778dac9936996233fcc574b95df7";
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
            "usage: creature_skin_mixed_accum_compare_v1 <receipt> <reviewed sha256>".into(),
        );
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-mixed-accum-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "21d31efc02b1e981907bc7cae0572de39dace25da046925b63cf1ab55d07d21b",
        ),
        (
            "manifest_sha256",
            "41ea6151a62f281ebe912d97b656de1ef81080175be6b802d136b790c0b89549",
        ),
        (
            "runner_sha256",
            "61b0d8c94aedc912f832bce05b7c2aa8de11ad4e8056d44af40ab368dbfa705d",
        ),
        (
            "stdout_sha256",
            "45fa6c705ffc493fac5d8254d672165f17ab250bf901089bc985e10b70422e1f",
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
        || v["source_ranges"]["HighSlice"].as_str()
            != Some("1ec3f11f1b0dfb79fd2f55f3cf460afdf5556fe6cebefbcd22918c707ace32e5")
        || v["source_ranges"]["Edge"].as_str()
            != Some("80466b426eb620c5d938f7fd7c30716c40b909f918fd64bc02c449ea4bd091ad")
    {
        return Err("source range mismatch".into());
    }
    // The exact fixture beyond the recorded point/order is bound by the host
    // and manifest hashes; the outer query dispatch is host-supplied.
    let endpoints = [
        Endpoint {
            position: [0.; 3],
            members: &[0],
        },
        Endpoint {
            position: [2., 0., 0.],
            members: &[0],
        },
    ];
    let clear = Query { key: 0, weight: 1. };
    let high = Query {
        key: 0x8000_0000,
        weight: 1.,
    };
    let big = Query {
        key: 0x8000_0000,
        weight: 16777216.,
    };
    let queries = [[clear, high, big], [high, big, clear]];
    let orders = [
        ["clear_1", "high_1", "high_16777216"],
        ["high_1", "high_16777216", "clear_1"],
    ];
    let cases = v["cases"].as_array().ok_or("missing cases")?;
    if cases.len() != 2 {
        return Err("case count".into());
    }
    let mut stdout = Vec::new();
    let mut first_words = [0u32; 2];
    for (i, row) in cases.iter().enumerate() {
        if row["id"].as_u64() != Some(i as u64)
            || hex(&row["point_x_bits"], 8)? != 0x3f800000
            || row["query_order"] != serde_json::json!(orders[i])
        {
            return Err(format!("case {i} fixture mismatch").into());
        }
        let got = accumulate_rank_record(
            &[0.; 4],
            [1., 0., 0.],
            &queries[i],
            &[[0, 1]],
            &endpoints,
            None,
            Policy::Mxcsr1f80,
            Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            Admission,
        )?;
        let words = row["weights_bits"].as_array().ok_or("missing weights")?;
        if words.len() != 4 {
            return Err("weight count".into());
        }
        for (j, w) in words.iter().enumerate() {
            let x = hex(w, 8)?;
            if got.weights[j].to_bits() != x {
                return Err(format!("case {i} word {j} mismatch").into());
            }
            if j == 0 {
                first_words[i] = x;
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
    if first_words != [0x4b800001, 0x4b800000] {
        return Err("expected one-bit order difference absent".into());
    }
    if format!("{:x}", Sha256::digest(&stdout))
        != v["stdout_sha256"].as_str().ok_or("missing stdout")?
    {
        return Err("stdout hash".into());
    }
    println!("MATCH: all eight weight words across two synthetic query orders; expected one-bit difference and FP metadata validated.");
    println!("Original branch/helper spans with host-supplied outer dispatch and constant bindings; no native spatial query, correction, smoothing, palette, mesh or live editor. Hashes assume stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
