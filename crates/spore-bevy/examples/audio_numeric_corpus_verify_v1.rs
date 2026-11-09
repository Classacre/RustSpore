//! Canonical-index installed corpus inspection for typed audio numeric values.
//! Framing, selected native loader and live provider are distinct claims.
use dbpf::native_index::NativePackageIndex;
use serde_json::json;
use sha2::{Digest, Sha256};
use spore_core::audio_event::{self, AudioValue};
use std::{error::Error, fs::File, io::{BufReader, Read}, path::Path};

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn numeric_word(kind: u16, value: &AudioValue) -> Result<u32, Box<dyn Error>> {
    Ok(match (kind, value) {
        (0x0009, AudioValue::Int32(v)) => *v as u32,
        (0x000a, AudioValue::Uint32(v)) => *v,
        (0x000d, AudioValue::Float(v)) => v.to_bits(),
        _ => return Err(format!("numeric type {kind:04X} has unexpected parsed value").into()),
    })
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: audio_numeric_corpus_verify_v1 <PatchData.package> <Spore_Audio2.package>".into()); }
    let paths = [Path::new(&args[0]), Path::new(&args[1])];
    let before = [hash_file(paths[0])?, hash_file(paths[1])?];
    let mut reports = Vec::new();
    for (i, path) in paths.iter().enumerate() {
        let package = NativePackageIndex::open(path)?;
        let mut digest = Sha256::new();
        let (mut events, mut singles, mut arrays, mut raw_unsupported) = (0u64, 0u64, 0u64, 0u64);
        let (mut array_values, mut raw_bytes) = (0u64, 0u64);
        for entry in package.entries().iter().filter(|e| e.key.type_id == 0x02b9f662) {
            let bytes = package.read(entry)?;
            let event = audio_event::parse(&bytes)?;
            if !event.exact { return Err(format!("{} event {:08X}:{:08X}:{:08X}: inspection framing not exact at {}", path.display(), entry.key.instance_id, entry.key.type_id, entry.key.group_id, event.consumed).into()); }
            events += 1;
            digest.update(entry.key.instance_id.to_le_bytes());
            digest.update(entry.key.type_id.to_le_bytes());
            digest.update(entry.key.group_id.to_le_bytes());
            for field in event.entries.iter().filter(|f| matches!(f.val_type, 0x0009 | 0x000a | 0x000d)) {
                digest.update(field.hash.to_le_bytes());
                digest.update(field.val_type.to_le_bytes());
                digest.update(field.flags.to_le_bytes());
                match &field.value {
                    AudioValue::Int32(_) | AudioValue::Uint32(_) | AudioValue::Float(_) => {
                        singles += 1;
                        digest.update([0]);
                        digest.update(1u32.to_le_bytes());
                        digest.update(numeric_word(field.val_type, &field.value)?.to_le_bytes());
                    }
                    AudioValue::Array(values) => {
                        arrays += 1;
                        array_values += values.len() as u64;
                        digest.update([1]);
                        digest.update(u32::try_from(values.len())?.to_le_bytes());
                        for value in values { digest.update(numeric_word(field.val_type, value)?.to_le_bytes()); }
                    }
                    AudioValue::Raw(bytes) => {
                        raw_unsupported += 1;
                        raw_bytes += bytes.len() as u64;
                        digest.update([2]);
                        digest.update(u32::try_from(bytes.len())?.to_le_bytes());
                        digest.update(bytes);
                    }
                    _ => return Err(format!("{}: numeric field {:08X} has unexpected value", path.display(), field.hash).into()),
                }
            }
        }
        let role = ["PatchData", "Spore_Audio2"][i];
        reports.push(json!({"role": role, "path": path.to_string_lossy(),
            "sha256_before": before[i], "events": events, "numeric_single_fields": singles,
            "numeric_supported_array_fields": arrays, "numeric_supported_array_values": array_values,
            "numeric_raw_unsupported_fields": raw_unsupported, "numeric_raw_unsupported_bytes": raw_bytes,
            "numeric_stream_sha256": format!("{:x}", digest.finalize())}));
    }
    let after = [hash_file(paths[0])?, hash_file(paths[1])?];
    if before != after { return Err("package endpoint hashes changed during inspection".into()); }
    for i in 0..2 { reports[i]["sha256_after"] = json!(after[i]); }
    println!("{}", json!({"schema":"audio-numeric-corpus-verify-v1", "packages": reports,
        "scope":"Canonical native-index order; inspection framing exact for iterated type 02B9F662 entries only. Numeric byte order is conditional on the dedicated native loader and complete unchanged stream reads. Provider selection, live game behavior and playback are not established. Matching endpoint hashes do not prove continuous path or file stability; trusted stable inputs are required."}));
    Ok(())
}
