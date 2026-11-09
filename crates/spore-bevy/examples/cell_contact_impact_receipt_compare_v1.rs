//! Compare the Rust flag with a specific retained synthetic native-host receipt.
use sha2::{Digest, Sha256};
use spore_core::{cell_contact_impact_flag_v1::impact_flag, native_sse::Policy};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).ok_or("supply receipt path")?;
    let bytes = fs::read(path)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    if digest != "07187383b290234bb71e046cf6038a98f7e78534d3e81a49e43986341a71e21d" {
        return Err("receipt SHA-256 mismatch".into());
    }
    let receipt: serde_json::Value = serde_json::from_slice(&bytes)?;
    let distances = receipt["distances"].as_array().ok_or("missing distances")?;
    let rows = receipt["rows_result_cw_sw_mxcsr_post_guard_eflags_threshold"]
        .as_array().ok_or("missing rows")?;
    if distances.len() != 6 || rows.len() != 6 { return Err("unexpected row count".into()); }
    for (distance, row) in distances.iter().zip(rows) {
        let row = row.as_array().ok_or("invalid row")?;
        if row.len() != 5 { return Err("invalid row width".into()); }
        let parse = |value: &serde_json::Value| -> Result<u32, Box<dyn Error>> {
            let s = value.as_str().ok_or("not a hex string")?;
            if s.len() != 8 { return Err("invalid hex width".into()); }
            Ok(u32::from_str_radix(s, 16)?)
        };
        let result = parse(&row[0])?;
        if result & !0xff != 0xcccccc00 { return Err("result sentinel mismatch".into()); }
        let actual = impact_flag(parse(distance)?, parse(&row[4])?, Policy::Mxcsr1f80)?;
        if u32::from(actual) != result & 0xff { return Err("model and host flag differ".into()); }
    }
    println!("6/6 synthetic native-host result bytes match the Rust model; MXCSR status and EFLAGS are not compared");
    Ok(())
}
