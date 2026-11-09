//! Inspect the bounded edge and membership models against explicit source bytes.
//! This does not regenerate geometry or establish a runtime resource selection.

use sha2::{Digest, Sha256};
use spore_core::{
    creature_data,
    creature_generated_skin_edges_v1::{self as edges, FrozenPostMutationAdmission},
    creature_generated_skin_membership_v1::{self as membership, SnapshotAdmission},
    creature_generated_skin_remap_v1::{self as remap, FrozenPreparedAdmission},
};
use std::{error::Error, fs::File, io::Read, path::Path};

const MAX_BYTES: u64 = 64 * 1024 * 1024;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let include_remap = args.len() == 4 && args[3] == "--include-remap";
    if !(args.len() == 3 || include_remap) || args[2] != "--assume-frozen-post-mutation" {
        return Err("usage: creature_skin_source_edge_inspect_v1 <decoded-creaturedata> <sha256> --assume-frozen-post-mutation [--include-remap]".into());
    }
    let expected = args[1].to_str().ok_or("SHA-256 must be UTF-8")?;
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SHA-256 must be exactly 64 hex digits".into());
    }
    let path = Path::new(&args[0]);
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("source exceeds 64 MiB".into());
    }
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err("source SHA-256 mismatch".into());
    }
    let creature = creature_data::parse(&bytes)?;
    if !creature.exact || creature.consumed != bytes.len() {
        return Err("source has trailing or unconsumed bytes".into());
    }
    if creature.parts.len() > edges::MAX_ROWS {
        return Err("source exceeds the edge model row limit".into());
    }
    let mut chunks = creature.data0.chunks_exact(edges::ROW_SIZE);
    let mut rows = Vec::new();
    for chunk in &mut chunks {
        let row: [u8; edges::ROW_SIZE] = chunk.try_into()?;
        rows.push(row);
    }
    if !chunks.remainder().is_empty() || rows.len() != creature.parts.len() {
        return Err("inconsistent source row extent".into());
    }
    let generated = edges::build_edges(&rows, FrozenPostMutationAdmission)?;
    let members = membership::build_membership(
        generated.endpoint_count,
        &generated.pairs,
        SnapshotAdmission,
    )?;
    let mapped = if include_remap {
        Some(remap::build_remap(
            &rows,
            &generated.pairs,
            FrozenPreparedAdmission,
        )?)
    } else {
        None
    };
    let mut report = serde_json::json!({
        "schema": "creature-skin-source-edge-inspection-v1",
        "source_path": path.display().to_string(),
        "source_sha256": actual,
        "source_bytes": bytes.len(),
        "version": creature.version,
        "row_count": rows.len(),
        "endpoint_count": generated.endpoint_count,
        "edges": generated.pairs,
        "endpoint_members": members,
        "admissions": [
            "supplied serialized rows represent a stable post-mutation snapshot",
            "earlier native row mapping and endpoint coordinates succeeded",
            "native edge storage was freshly zero-initialized and vector operations succeeded"
        ],
        "not_established": ["runtime resource selection", "mutation behavior", "palette identity", "generated geometry", "gameplay"]
    });
    if let Some(mapped) = mapped {
        report["endpoint_to_row"] = serde_json::json!(mapped.endpoint_to_row);
        report["edge_to_row"] = serde_json::json!(mapped.edge_to_row);
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
