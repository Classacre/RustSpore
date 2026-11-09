//! Inspect collision-event candidate joins without asserting native sample semantics.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spore_core::audio_event::{self, AudioValue};
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: collision_audio_candidates_v1 <Audio1.package> <PatchData.package> <Audio2.package>".into());
    }
    let paths: Vec<_> = args.iter().map(Path::new).collect();
    let before: Vec<_> = paths
        .iter()
        .map(|p| hash_file(p))
        .collect::<Result<_, _>>()?;
    let samples = NativePackageIndex::open(paths[0])?;
    let mut types = BTreeMap::<String, usize>::new();
    let mut keys = HashSet::new();
    for e in samples.entries() {
        *types.entry(format!("{:08X}", e.key.type_id)).or_default() += 1;
        keys.insert(e.key);
    }
    let mut variants: Vec<Value> = Vec::new();
    for path in &paths[1..] {
        let package = NativePackageIndex::open(path)?;
        let event_key = NativeResourceKey {
            instance_id: 0x34c2ff0b,
            type_id: 0x02b9f662,
            group_id: 0x021407ee,
        };
        let entry = package
            .index()
            .find_unique(event_key)?
            .ok_or("event missing")?;
        let bytes = package.read(entry)?;
        let event = audio_event::parse(&bytes)?;
        if !event.exact {
            return Err("event walk was not exact".into());
        }
        let fields: Vec<_> = event
            .entries
            .iter()
            .filter(|e| e.hash == 0x701ed91e)
            .collect();
        if fields.len() != 1 {
            return Err("expected one samples field".into());
        }
        let AudioValue::Array(values) = &fields[0].value else {
            return Err("samples is not an array".into());
        };
        let mut rows = Vec::new();
        for value in values {
            let AudioValue::Raw(raw) = value else {
                return Err("sample entry was not retained raw".into());
            };
            if raw.len() != 12 {
                return Err("sample entry is not 12 bytes".into());
            }
            let instance = u32::from_le_bytes(raw[0..4].try_into()?);
            let matches: Vec<_> = samples
                .entries()
                .iter()
                .filter(|e| e.key.instance_id == instance)
                .collect();
            let mut candidates = Vec::new();
            for e in matches {
                let payload = samples.read(e)?;
                candidates.push(json!({"key_words": [format!("{:08X}",e.key.instance_id),format!("{:08X}",e.key.type_id),format!("{:08X}",e.key.group_id)], "decoded_bytes": payload.len(), "sha256": hash_bytes(&payload), "metadata_word": e.metadata_word.map(|v| format!("{v:08X}"))}));
            }
            rows.push(json!({"raw_hex": raw.iter().map(|b|format!("{b:02x}")).collect::<String>(), "candidate_instance_le":format!("{instance:08X}"), "matches":candidates}));
        }
        variants.push(json!({"package":path.to_string_lossy(), "event_key_words":["34C2FF0B","02B9F662","021407EE"], "event_length":bytes.len(), "event_sha256":hash_bytes(&bytes), "samples_type":format!("{:04X}",fields[0].val_type), "samples_flags":format!("{:04X}",fields[0].flags), "rows":rows}));
    }
    let after: Vec<_> = paths
        .iter()
        .map(|p| hash_file(p))
        .collect::<Result<_, _>>()?;
    if before != after {
        return Err("one or more input package hashes changed during inspection".into());
    }
    let identities: Vec<_> = paths.iter().enumerate().map(|(i, path)| json!({"role": (["audio1", "patch_event", "audio2_event"][i]), "path": path.to_string_lossy(), "sha256_before": before[i], "sha256_after": after[i]})).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"scope":"exploratory candidate join; no sample-reference semantics, mount winner, selection, or playback inferred; candidate uniqueness, expected count and type are reported, not acceptance gates", "inputs":identities, "audio1":{"flags":format!("{:08X}",samples.index().shared_flags), "common_group":samples.index().common_group.map(|v|format!("{v:08X}")), "records":samples.entries().len(), "unique_native_keys":keys.len(), "type_counts":types}, "variants":variants})
        )?
    );
    Ok(())
}
