//! Compare a fixed synthetic row-edge native receipt, not a live editor.
use sha2::{Digest, Sha256};
use spore_core::creature_generated_skin_edges_v1::{
    build_edges, FrozenPostMutationAdmission, ROW_SIZE,
};
use std::{error::Error, fs::File, io::Read};
const PIN: &str = "838741684932b3c141d887727314c764af56e4f6eddb5b65fc27e20bbd3f8d5c";
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
        return Err("usage: creature_skin_row_edge_compare_v1 <receipt> <reviewed sha256>".into());
    }
    let mut raw = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut raw)?;
    if raw.len() > 65536 || format!("{:x}", Sha256::digest(&raw)) != PIN {
        return Err("receipt hash/size".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&raw)?;
    for (k, w) in [
        ("schema", "creature-skin-row-edge-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "host_sha256",
            "4066b2bbf521f9bb739177dbe79ef0a4ef62a323b64f9175ab71b186b94f30af",
        ),
        (
            "manifest_sha256",
            "54a15e6829b83949da9e2fc25fc1521ae4a704c9443eae39948ba98bb717a31f",
        ),
        (
            "runner_sha256",
            "a364f155c1ca4facba84d06467244c6fae6f0775f7a0059d78b30751b2556f23",
        ),
        (
            "fixture_sha256",
            "8e80403a1733262531035884d6c33581f5bc079981dc184a6118619827d6e522",
        ),
        (
            "stdout_sha256",
            "7b370fac1bc9fb34485eeda163fa7550fdac588b5d2be2e1ac4d08674f1aff1d",
        ),
    ] {
        if v[k].as_str() != Some(w) {
            return Err(format!("{k} mismatch").into());
        }
    }
    for (k, w) in [
        (
            "Prelude",
            "62febf6e8727630560ad4e5c0608a6c8a0844a7a0dc9df6c305136b6d1ecc912",
        ),
        (
            "Writes",
            "9c4ca7a9e0139cf14a34519d709a1fcd89444bfd740e6b3819a1625fffa15306",
        ),
    ] {
        if v["source_ranges"][k].as_str() != Some(w) {
            return Err(format!("{k} source mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true) || v["native_exit_code"].as_i64() != Some(0) {
        return Err("status mismatch".into());
    }
    let cases: [(&[Spec], u64, u64, u64, u64); 4] = [
        (
            &[(5, 0, 0, 0), (5, 1, 0, 0), (3, 3, 0, 2), (4, 4, 1, 0)],
            2,
            3,
            5,
            5,
        ),
        (
            &[(5, 0, 0, 0), (5, 1, 0, 0), (4, 2, 3, 0), (4, 3, 0, 0)],
            2,
            2,
            4,
            4,
        ),
        (&[(3, 0, 0, 0), (3, 1, 0, 2), (4, 2, 1, 0)], 0, 2, 3, 3),
        (&[(4, 0, 0, 0), (4, 0, 0, 0)], 0, 0, 1, 2),
    ];
    let observed = v["cases"].as_array().ok_or("missing cases")?;
    if observed.len() != cases.len() {
        return Err("case count".into());
    }
    let mut fixture = Vec::new();
    let mut stdout = Vec::new();
    let mut compared = 0;
    for (i, (r, (spec, spine, class3, ep, count))) in observed.iter().zip(cases).enumerate() {
        if r["id"].as_u64() != Some(i as u64)
            || r["spine"].as_u64() != Some(spine)
            || r["class3_end"].as_u64() != Some(class3)
            || r["endpoint_count"].as_u64() != Some(ep)
            || r["edge_count"].as_u64() != Some(count)
            || r["rows"] != serde_json::json!(spec)
        {
            return Err(format!("case {i} inputs/counts").into());
        }
        let rows: Vec<_> = spec.iter().copied().map(row).collect();
        for r in &rows {
            fixture.extend_from_slice(r);
        }
        let model = build_edges(&rows, FrozenPostMutationAdmission)?;
        if model.endpoint_count != ep as usize || model.pairs.len() != count as usize {
            return Err(format!("case {i} model counts").into());
        }
        let words = r["edge_words"].as_array().ok_or("missing words")?;
        if words.len() != 2 * count as usize {
            return Err("word count".into());
        }
        for (j, w) in words.iter().enumerate() {
            let x = hex(w, 8)?;
            if model.pairs[j / 2][j % 2] != x {
                return Err(format!("case {i} word {j} mismatch").into());
            }
            stdout.extend(x.to_le_bytes());
            compared += 1;
        }
        for _ in count as usize..5 {
            stdout.extend([0u8; 8]);
        }
        stdout.extend((count as u32).to_le_bytes());
        stdout.extend((ep as u32).to_le_bytes());
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
    if stdout.len() != 256
        || format!("{:x}", Sha256::digest(&stdout))
            != v["stdout_sha256"].as_str().ok_or("stdout hash")?
    {
        return Err("stdout reconstruction hash".into());
    }
    println!("MATCH: {compared} native edge words in four fixed synthetic fixtures; counts and FP metadata validated.");
    println!("Host substituted intervening row-map/coordinate preparation, vector erase/resize/zero-fill and orchestration; no guard pages. No source mutation, palette, generated mesh or live editor claim. Hashes require stable trusted files.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1)
    }
}
