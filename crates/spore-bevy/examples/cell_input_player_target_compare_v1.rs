//! Compare one pinned synthetic E5B160 integration receipt with the checked fixture.
//! No live actor/input/provider, original scheduling or long path is established.
use sha2::{Digest, Sha256};
use spore_core::{
    cell_input_player_target_fixture_v1::{self as fixture, Branch, StableSourceAdmission, State},
    cell_input_target_early_v1::EarlyBranch,
    native_sse::Policy,
    native_x87::Context,
};
use std::{error::Error, fs::File, io::Read};
const RECEIPT_SHA: &str = "dc20425ac472b8389592b8fd38d135fc0af5b6bff8c9d79df6771fe597e48be6";
fn hex(value: &serde_json::Value, field: &str, width: usize) -> Result<u32, Box<dyn Error>> {
    let text = value[field].as_str().ok_or("missing hexadecimal field")?;
    if text.len() != width || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("{field}: invalid hexadecimal word").into());
    }
    Ok(u32::from_str_radix(text, 16)?)
}
fn words(value: &serde_json::Value, field: &str) -> Result<[u32; 3], Box<dyn Error>> {
    let array = value[field].as_array().ok_or("missing word array")?;
    if array.len() != 3 {
        return Err(format!("{field}: expected three words").into());
    }
    let mut result = [0; 3];
    for (i, item) in array.iter().enumerate() {
        let text = item.as_str().ok_or("word is not text")?;
        if text.len() != 8 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("{field}: invalid word").into());
        }
        result[i] = u32::from_str_radix(text, 16)?;
    }
    Ok(result)
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_input_player_target_compare_v1 <reviewed-receipt.json> <receipt-sha256>"
                .into(),
        );
    }
    if args[1].len() != 64
        || !args[1].bytes().all(|b| b.is_ascii_hexdigit())
        || args[1].to_ascii_lowercase() != RECEIPT_SHA
    {
        return Err("expected SHA-256 must identify the reviewed receipt".into());
    }
    let mut receipt = Vec::new();
    File::open(&args[0])?
        .take(65537)
        .read_to_end(&mut receipt)?;
    if receipt.len() > 65536 || format!("{:x}", Sha256::digest(&receipt)) != RECEIPT_SHA {
        return Err("receipt too large or SHA-256 mismatch".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&receipt)?;
    for (field, expected) in [
        ("schema", "cell-player-target-integration-native-fixed-v1"),
        (
            "host_sha256",
            "2899e6a922cca3e8e4bf859cef5afa5ad0c998a5b9507fad6e3444112c587461",
        ),
        (
            "source_pe_sha256",
            "17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a",
        ),
        ("pool_handle", "00010001"),
        ("bit_code", "57"),
        ("scalar_bits", "3f800000"),
    ] {
        if value[field].as_str() != Some(expected) {
            return Err(format!("receipt {field} mismatch").into());
        }
    }
    for (field, expected) in [
        (
            "update",
            "0075dd5d9404e7c18d43d7e08925b69946c30438537950d0909ec92a40c81ce8",
        ),
        (
            "pool",
            "bb658d79bc3b228d4254c6d694a7a7363f711aa6ea387f43957ff57938fb9a65",
        ),
        (
            "helper",
            "1858ca70a26f1d28f1951918c904761892830e95b2dae36b598fbd15dd7feff3",
        ),
    ] {
        if value["source_sha256"][field].as_str() != Some(expected) {
            return Err(format!("source {field} mismatch").into());
        }
    }
    if value["synthetic"].as_bool() != Some(true)
        || value["native_exit_code"].as_i64() != Some(0)
        || value["pool_index"].as_u64() != Some(1)
        || value["layout_al"]
            .as_array()
            .map(|v| v.iter().map(|x| x.as_u64()).collect::<Vec<_>>())
            != Some(vec![Some(0); 4])
        || words(&value, "baseline_bits")? != [0; 3]
        || words(&value, "current_direction_bits")? != [0; 3]
        || words(&value, "actor_position_bits")? != [0x41200000, 0x41a00000, 0x41f00000]
    {
        return Err("receipt fixed inputs or synthetic status mismatch".into());
    }
    let cw = hex(&value, "cw", 4)?;
    let sw = hex(&value, "sw", 4)?;
    let mxcsr = hex(&value, "mxcsr", 8)?;
    let eflags = hex(&value, "eflags", 8)?;
    if cw != 0x027f
        || sw & 0x38df != 0
        || mxcsr & !0x3f != 0x1f80
        || mxcsr & 0x1f != 0
        || eflags & 0x400 != 0
    {
        return Err("receipt FP controls, stack or direction flag outside reviewed policy".into());
    }
    // The host's A5A5A5A5 initial target words are finite binary32 values;
    // they are overwritten by this admitted active branch, not zeroed here.
    let poison = f32::from_bits(0xa5a5a5a5);
    if !poison.is_finite() {
        return Err("host initial target words are outside the model domain".into());
    }
    let state = State {
        direction_cc: [0.; 3],
        selected_c4: 0xa5a5a5a5,
        active_c8: 0,
        indicator_e0: 0,
        actor_position_4c: [10., 20., 30.],
        actor_target_08: [poison; 3],
        actor_flag_04: 0xa5,
    };
    let mut bits = [0u32; 8];
    bits[0x57 >> 5] |= 1u32 << (0x57 & 31);
    let result = fixture::update(
        state,
        bits,
        [0; 4],
        [0.; 3],
        1.0,
        Policy::Mxcsr1f80,
        Context::from_control_word(cw as u16)?,
        StableSourceAdmission,
    )?;
    if result.branch != Branch::Active(EarlyBranch::SmallCurrent)
        || result.side_effect_requests != [true, false, false, false]
    {
        return Err("model branch or per-group requests differ from fixed input".into());
    }
    if result.state.actor_position_4c.map(f32::to_bits) != words(&value, "actor_position_bits")?
        || result.state.actor_target_08.map(f32::to_bits) != words(&value, "actor_target_bits")?
        || result.state.direction_cc.map(f32::to_bits) != words(&value, "direction_bits")?
        || value["actor_flag_04"].as_u64() != Some(u64::from(result.state.actor_flag_04))
        || value["selected_c4"].as_u64() != Some(u64::from(result.state.selected_c4))
        || value["active_c8"].as_u64() != Some(u64::from(result.state.active_c8))
        || value["indicator_e0"].as_u64() != Some(u64::from(result.state.indicator_e0))
    {
        return Err("modeled target, direction, flags or aggregate side effects differ".into());
    }
    println!("MATCH: one fixed synthetic actor-pool/input fixture; target, direction, actor flag and aggregate C4/C8/E0 fields.");
    println!("The native host executes the checked pool lookup; the Rust fixture receives the resolved actor state.");
    println!("Early branch and one group request are inferred from fixed inputs and source; repeated native stores are not independently instrumented. No live player, provider, scheduler or long angle path is established.");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("comparison failed: {error}");
        std::process::exit(1);
    }
}
