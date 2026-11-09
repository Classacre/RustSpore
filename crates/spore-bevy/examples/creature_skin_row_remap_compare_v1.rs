//! Compare a fixed synthetic row-remap native receipt, not a live editor.
use sha2::{Digest, Sha256};
use spore_core::{
    creature_generated_skin_edges_v1::ROW_SIZE,
    creature_generated_skin_remap_v1::{build_remap, FrozenPreparedAdmission},
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "9340a005826551b0d1256312d9a5d1dddacb631fdc4ab3ac10dfd7ed5597e7a0";
type Spec = (u8, i16, i16, u16);
fn row((kind, link, parent, flags): Spec) -> [u8; ROW_SIZE] {
    let mut r = [0; ROW_SIZE];
    r[2..4].copy_from_slice(&link.to_le_bytes());
    r[4..6].copy_from_slice(&parent.to_le_bytes());
    r[8..10].copy_from_slice(&flags.to_le_bytes());
    r[11] = kind;
    for (i, b) in r.iter_mut().enumerate().take(96).skip(48) {
        *b = (i as i32 + kind as i32 * 17 + link as i32) as u8;
    }
    r
}
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
        return Err("usage: creature_skin_row_remap_compare_v1 <receipt> <reviewed sha256>".into());
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-row-remap-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "d3e9eaf25a7ed496512d9b1c72e106faa81e014d3c6b6aefae262b72b04fb5b1",
        ),
        (
            "manifest_sha256",
            "8640668528c727fe9f197b8da657d28af2265434e74a29ebe58ba1d08367b879",
        ),
        (
            "runner_sha256",
            "5ca713a8e1d5602cb8eaa6fa623aea3b0d9a4fc6bb137ab545ee58eab3d1a247",
        ),
        (
            "fixture_sha256",
            "43715eeb228ecf4e7556a5642c7dfbbfba32313bae802e19ef4e5496b9cca369",
        ),
        (
            "stdout_sha256",
            "4aca64e7a8d0ce80632bf299c43bd76b4da424b52dc49425236e4b31c0c02e34",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    for (k, w) in [
        (
            "EndpointMap",
            "baa3484aeea51bc2e7d3094689cc887cc1b461724dab966e6dd03b48a2f2eb96",
        ),
        (
            "EdgeRemap",
            "532308d785bd092366b8bfc894ceffb593ef8bbe4998c46f6ca94c1271623796",
        ),
    ] {
        if v["source_ranges"][k].as_str() != Some(w) {
            return Err(format!("{k} source mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    let cases: [(&[Spec], &[[u32; 2]], u64); 3] = [
        (
            &[(4, 0, 0, 0), (1, 1, 0, 2), (3, 3, 1, 0)],
            &[[0, 0], [0, 1], [1, 3], [0, 0]],
            4,
        ),
        (&[(4, -1, 0, 2), (4, 1, 0, 0)], &[[0, 0], [0, 1]], 2),
        (
            &[(5, 0, 0, 0), (5, 1, 0, 0), (3, 3, 0, 2), (4, 4, 1, 0)],
            &[[0, 1], [1, 2], [2, 3], [2, 4], [1, 2]],
            5,
        ),
    ];
    let observed = v["cases"].as_array().ok_or("missing cases")?;
    if observed.len() != cases.len() {
        return Err("case count".into());
    }
    let mut fixture = Vec::new();
    let mut stdout = Vec::new();
    let mut compared = 0;
    for (i, (r, (spec, edges, ep))) in observed.iter().zip(cases).enumerate() {
        if r["id"].as_u64() != Some(i as u64)
            || r["endpoint_count"].as_u64() != Some(ep)
            || r["edge_count"].as_u64() != Some(edges.len() as u64)
            || r["rows"] != serde_json::json!(spec)
            || r["edges"] != serde_json::json!(edges)
        {
            return Err(format!("case {i} inputs/counts").into());
        }
        let rows: Vec<_> = spec.iter().copied().map(row).collect();
        for row in &rows {
            fixture.extend_from_slice(row);
        }
        for pair in edges {
            for w in pair {
                fixture.extend(w.to_le_bytes());
            }
        }
        let model = build_remap(&rows, edges, FrozenPreparedAdmission)?;
        for (key, expected, pad) in [
            ("endpoint_map", &model.endpoint_to_row, 0xffffffffu32),
            ("edge_remap", &model.edge_to_row, 0u32),
        ] {
            let words = r[key].as_array().ok_or("missing words")?;
            if words.len() != expected.len() {
                return Err(format!("case {i} {key} count").into());
            }
            for (j, (w, expected)) in words.iter().zip(expected).enumerate() {
                let x = hex(w, 8)?;
                if x != *expected {
                    return Err(format!("case {i} {key} word {j} mismatch").into());
                }
                stdout.extend(x.to_le_bytes());
                compared += 1;
            }
            for _ in words.len()..5 {
                stdout.extend(pad.to_le_bytes());
            }
        }
        stdout.extend((ep as u32).to_le_bytes());
        stdout.extend((edges.len() as u32).to_le_bytes());
        let cw = hex(&r["cw"], 4)?;
        let sw = hex(&r["sw"], 4)?;
        let mx = hex(&r["mxcsr"], 8)?;
        let flags = hex(&r["eflags"], 8)?;
        if cw != 0x27f || sw & 0x38cd != 0 || mx != 0x1f80 || flags & 0x400 != 0 {
            return Err(format!("case {i} FP/DF").into());
        }
        stdout.extend((cw as u16).to_le_bytes());
        stdout.extend((sw as u16).to_le_bytes());
        stdout.extend(mx.to_le_bytes());
        stdout.extend(flags.to_le_bytes());
        stdout.extend(0u32.to_le_bytes());
    }
    if format!("{:x}", Sha256::digest(&fixture))
        != v["fixture_sha256"].as_str().ok_or("fixture hash")?
    {
        return Err("fixture reconstruction hash".into());
    }
    if stdout.len() != 192
        || format!("{:x}", Sha256::digest(&stdout))
            != v["stdout_sha256"].as_str().ok_or("stdout hash")?
    {
        return Err("stdout reconstruction hash".into());
    }
    println!("MATCH: {compared} endpoint-map and edge-remap words in three fixed synthetic fixtures; counts and FP metadata validated.");
    println!("Host substituted vector creation/resize with an initialized FFFFFFFF endpoint map and zeroed remap; no guard pages. No source mutation, palette, generated mesh or live editor claim. Hashes require stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
