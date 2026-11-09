//! Compare fixed synthetic native grid markers with the bounded address model.
use sha2::{Digest, Sha256};
use spore_core::{creature_generated_skin_grid_query_v1::address, native_sse::Policy};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "fd64ee4c01493ca9afa5358650606392ec79fb76e16905a3606ba0badaa84ee7";
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
        return Err("usage: creature_skin_grid_query_compare_v1 <receipt> <reviewed sha256>".into());
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-grid-query-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "b401cf42ff1ab24e986f141cdaec265ac1f560a8ab4f256caa16728b0219d9ef",
        ),
        (
            "manifest_sha256",
            "a108de2913ca78406cecbd566412c47e9260a002b65d93e19f09dc5d94b826fd",
        ),
        (
            "runner_sha256",
            "e258d827fc29347133596dc95887430a6f686fe0bde3a95bd72eeea1acbc562f",
        ),
        (
            "fixture_sha256",
            "9a484e5f987859b0b02c623b6f92c2a36b06fd54db63127c18124f0a35aa0782",
        ),
        (
            "table_sha256",
            "d4908365f7f80bdb842fa274e4ebce2641d64d91cb2aab23343fb89cf57fa597",
        ),
        (
            "stdout_sha256",
            "fd3aeea19b82c83fb85f3810ee24745668081f15d3166e886e576fd9ca9e8255",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    for (k, w) in [
        (
            "Subtract",
            "abbd724a1f8b947a405031e3f84c6d302459d7fd67fbb61845d4fd31b969f1b3",
        ),
        (
            "Grid",
            "51629276c48624068e9527b47ed634dd18ba2b460cef896ed2186731352e25e8",
        ),
        (
            "Multiply",
            "56047c50f688b4e96094bfdb33550fea8fd5762a693b4036cca04fae29ad8e24",
        ),
    ] {
        if v["source_ranges"][k].as_str() != Some(w) {
            return Err(format!("{k} source mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    let cases: [([f32; 3], [f32; 3], [f32; 3], Option<usize>); 6] = [
        ([1.75, 2.5, 3.75], [0.; 3], [1.; 3], Some(83)),
        ([0., -1., 8.], [0.; 3], [1.; 3], Some(0)),
        ([67108864., 0., 1.], [0.; 3], [1.; 3], Some(1)),
        ([7., 7., 7.], [0.; 3], [1.; 3], Some(511)),
        ([8., 0., 0.], [0.; 3], [1.; 3], None),
        ([12.5, 22., 31.], [10., 20., 30.], [2., 0.5, 4.], Some(332)),
    ];
    let records = v["cases"].as_array().ok_or("missing cases")?;
    if records.len() != cases.len() {
        return Err("case count".into());
    }
    let mut fixture = Vec::new();
    let mut stdout = Vec::new();
    for (i, (r, (point, origin, scale, index))) in records.iter().zip(cases).enumerate() {
        if r["id"].as_u64() != Some(i as u64)
            || r["point"] != serde_json::json!(point)
            || r["origin"] != serde_json::json!(origin)
            || r["scale"] != serde_json::json!(scale)
            || r["index"] != serde_json::json!(index)
        {
            return Err(format!("case {i} fixture").into());
        }
        for f in point.into_iter().chain(origin).chain(scale) {
            fixture.extend(f.to_le_bytes())
        }
        let model = address(point, origin, scale, Policy::Mxcsr1f80)?;
        if model.cell != index {
            return Err(format!("case {i} model address").into());
        }
        let result = hex(&r["marker"], 8)?;
        let expected = index.map_or(0, |n| 0x10000000 + n as u32);
        if result != expected {
            return Err(format!("case {i} native marker").into());
        }
        let cw = hex(&r["cw"], 4)?;
        let sw = hex(&r["sw"], 4)?;
        let mx = hex(&r["mxcsr"], 8)?;
        let flags = hex(&r["eflags"], 8)?;
        if cw != 0x27f
            || sw & 0x38cd != 0
            || mx & !0x3f != 0x1f80
            || mx & 0x0d != 0
            || flags & 0x400 != 0
        {
            return Err(format!("case {i} FP/DF").into());
        }
        stdout.extend(result.to_le_bytes());
        stdout.extend((cw as u16).to_le_bytes());
        stdout.extend((sw as u16).to_le_bytes());
        stdout.extend(mx.to_le_bytes());
        stdout.extend(flags.to_le_bytes());
        stdout.extend([0u8; 8]);
    }
    let mut table = Vec::new();
    for i in 0..512u32 {
        table.extend((0x10000000 + i).to_le_bytes())
    }
    if format!("{:x}", Sha256::digest(&fixture))
        != v["fixture_sha256"].as_str().ok_or("fixture hash")?
        || format!("{:x}", Sha256::digest(&table))
            != v["table_sha256"].as_str().ok_or("table hash")?
    {
        return Err("fixture/table reconstruction hash".into());
    }
    if stdout.len() != 144
        || format!("{:x}", Sha256::digest(&stdout))
            != v["stdout_sha256"].as_str().ok_or("stdout hash")?
    {
        return Err("stdout reconstruction hash".into());
    }
    println!("MATCH: six synthetic native grid-address markers, fixture identities and FP metadata validated.");
    println!("Markers are not dereferenced. Original 4F9C30 dispatch, grid construction, live actor and game are not exercised; host has no guard pages. Hashes require stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
