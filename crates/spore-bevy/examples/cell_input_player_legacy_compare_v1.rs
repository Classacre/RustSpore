//! Compare the fixed synthetic W+D native integration receipt with the Rust fixture.
//! Locale callbacks are stubs and the CRT path is explicitly admitted legacy x87.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_player_target_fixture_v1::{
        self as fixture, Branch, LegacyPathInputs, StableSourceAdmission, State,
    },
    cell_input_target_angle_v1::{LegacyClosedAdmission, Selection},
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "cf0d418053c6099e80cef9b46ec7c16e70c0a2c1da7957487ae4b9791f011a32";
fn hex(v: &serde_json::Value, key: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let s = v[key].as_str().ok_or("missing hex field")?;
    if s.len() != width || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {key}").into());
    }
    Ok(u32::from_str_radix(s, 16)?)
}
fn words(v: &serde_json::Value, key: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let a = v[key].as_array().ok_or("missing words")?;
    if a.len() != 3 {
        return Err(format!("{key} length").into());
    }
    let mut w = [0; 3];
    for (i, x) in a.iter().enumerate() {
        let s = x.as_str().ok_or("bad word")?;
        if s.len() != 8 || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("invalid word".into());
        }
        w[i] = u32::from_str_radix(s, 16)?;
    }
    Ok(w)
}
fn region(v: &serde_json::Value, key: &str, digest: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let s = v[key].as_str().ok_or("missing region")?;
    if s.len() != 512 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("invalid {key}").into());
    }
    let bytes = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()?;
    if format!("{:x}", Sha256::digest(&bytes)) != v[digest].as_str().ok_or("missing region hash")? {
        return Err(format!("{key} hash mismatch").into());
    }
    Ok(bytes)
}
fn put(buf: &mut [u8], at: usize, w: u32) {
    buf[at..at + 4].copy_from_slice(&w.to_le_bytes());
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || !args[1].eq_ignore_ascii_case(RECEIPT_SHA) {
        return Err(
            "usage: cell_input_player_legacy_compare_v1 <receipt.json> <reviewed-receipt-sha256>"
                .into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(&args[0])?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 || format!("{:x}", Sha256::digest(&bytes)) != RECEIPT_SHA {
        return Err("receipt too large or SHA mismatch".into());
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    for (field, expected) in [
        ("schema", "cell-player-target-legacy-full-native-fixed-v1"),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        (
            "source_dll_sha256",
            "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185",
        ),
        (
            "host_sha256",
            "8bfcc3fc288a02058b3885efb73ab07a9138fc8f5e43843e90945ad65f2b5c3d",
        ),
        (
            "asm_sha256",
            "e6d2d5abf253801f030e675b00b63b1a75b86548b07644b9baa133346d3b17d3",
        ),
        (
            "generator_sha256",
            "0f009691fca834cf310f2360a25e6b5ef2a4942f24758eff35bd8cd17513426f",
        ),
        (
            "manifest_sha256",
            "a466a5c300f7841d6727abe27fe3079788a1c2b8d0bd4fd14573711cf3eb4681",
        ),
        (
            "crt_manifest_sha256",
            "2056806bdc0c5f015fbacf2e15fb50cf90f5ee50b446381652870fea9421133a",
        ),
        (
            "runner_sha256",
            "c6bba234be548db8e751e6a30bbc8c6b5968a667ecdb11c1bc6f48c74017c706",
        ),
        (
            "stdout_sha256",
            "2152db1c354521201ba0cac07810ef3f2098b9f47f6adb06ee5bce57476e7d15",
        ),
        ("pool_handle", "00010001"),
        ("scalar_bits", "3e000000"),
        ("period_bits", "40c90fdb"),
    ] {
        if v[field].as_str() != Some(expected) {
            return Err(format!("{field} mismatch").into());
        }
    }
    if v["synthetic"].as_bool() != Some(true)
        || v["native_exit_code"].as_i64() != Some(0)
        || v["crt_dispatch_global"].as_u64() != Some(0)
        || v["crt_return_global"].as_u64() != Some(0)
        || v["input_bits"] != serde_json::json!(["57", "44"])
        || v["layout_al"] != serde_json::json!([0, 0, 0, 0])
        || words(&v, "baseline_bits")? != [0; 3]
        || words(&v, "initial_direction_bits")? != [0, 0x3f800000, 0]
        || words(&v, "actor_position_bits")? != [0x41200000, 0x41a00000, 0x41f00000]
    {
        return Err("fixed fixture mismatch".into());
    }
    let m = &v["metadata"];
    let cw = hex(m, "cw", 4)?;
    let sw = hex(m, "sw", 4)?;
    let mx = hex(m, "mxcsr", 8)?;
    let flags = hex(m, "eflags", 8)?;
    if cw != 0x27f
        || sw & 0x38cd != 0
        || mx & !0x3f != 0x1f80
        || mx & 0x0d != 0
        || flags & 0x400 != 0
    {
        return Err("FP/DF metadata mismatch".into());
    }
    let poison = f32::from_bits(0xa5a5a5a5);
    let state = State {
        direction_cc: [0., 1., 0.],
        selected_c4: 0xa5a5a5a5,
        active_c8: 0,
        indicator_e0: 0,
        actor_position_4c: [10., 20., 30.],
        actor_target_08: [poison; 3],
        actor_flag_04: 0xa5,
    };
    let mut bits = [0u32; 8];
    for code in [0x57usize, 0x44] {
        bits[code >> 5] |= 1 << (code & 31)
    }
    let result = fixture::update_with_legacy_closed(
        state,
        bits,
        [0; 4],
        [0.; 3],
        0.125,
        LegacyPathInputs {
            period: f32::from_bits(0x40c90fdb),
            max_wrap_iterations: 8,
            admission: LegacyClosedAdmission,
        },
        Policy::Mxcsr1f80,
        Context::from_control_word(cw as u16)?,
        StableSourceAdmission,
    )?;
    if result.branch != Branch::ActiveAngle(Selection::Stepped)
        || result.side_effect_requests != [true, false, false, true]
        || result.state.direction_cc.map(f32::to_bits) != words(&v, "direction_bits")?
        || result.state.actor_target_08.map(f32::to_bits) != words(&v, "actor_target_bits")?
    {
        return Err("model branch or dynamic words mismatch".into());
    }
    let actor = region(&v, "actor_hex", "actor_sha256")?;
    let input = region(&v, "state_hex", "state_sha256")?;
    let indicator = region(&v, "indicator_hex", "indicator_sha256")?;
    let mut ea = vec![0xa5; 256];
    put(&mut ea, 0, 0x10001);
    ea[4] = result.state.actor_flag_04;
    for (i, w) in result
        .state
        .actor_target_08
        .map(f32::to_bits)
        .iter()
        .enumerate()
    {
        put(&mut ea, 8 + i * 4, *w)
    }
    for (i, w) in result
        .state
        .actor_position_4c
        .map(f32::to_bits)
        .iter()
        .enumerate()
    {
        put(&mut ea, 0x4c + i * 4, *w)
    }
    let mut es = vec![0xa5; 256];
    for (i, w) in bits.iter().enumerate() {
        put(&mut es, i * 4, *w)
    }
    put(&mut es, 0xc4, result.state.selected_c4);
    es[0xc8] = result.state.active_c8;
    for (i, w) in result
        .state
        .direction_cc
        .map(f32::to_bits)
        .iter()
        .enumerate()
    {
        put(&mut es, 0xcc + i * 4, *w)
    }
    let mut ei = vec![0xa5; 256];
    ei[0xe0] = result.state.indicator_e0;
    if actor != ea || input != es || indicator != ei {
        return Err("actor, state or indicator full snapshot mismatch".into());
    }
    let mut raw = Vec::new();
    raw.extend(actor);
    raw.extend(input);
    raw.extend(indicator);
    raw.extend((cw as u16).to_le_bytes());
    raw.extend((sw as u16).to_le_bytes());
    raw.extend(mx.to_le_bytes());
    raw.extend(flags.to_le_bytes());
    raw.extend(0u32.to_le_bytes());
    if format!("{:x}", Sha256::digest(&raw))
        != v["stdout_sha256"].as_str().ok_or("missing stdout hash")?
    {
        return Err("stdout hash mismatch".into());
    }
    println!("MATCH: full actor/state/indicator regions and modeled target, direction, and flags for one fixed synthetic W+D legacy fixture.");
    println!("The host executes a checked synthetic pool lookup; the model receives the resolved actor. Callback stubs and inferred branch do not establish live routing or branch instrumentation. CRT path is admitted, not an observed game dispatch; FPATAN/FSIN are hardware-dependent.");
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comparison failed: {e}");
        std::process::exit(1);
    }
}
