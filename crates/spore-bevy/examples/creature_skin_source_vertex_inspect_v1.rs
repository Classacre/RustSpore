//! Explicit source-row and caller-supplied grid snapshot inspection.
//! This does not establish native grid selection or generate a creature.
use sha2::{Digest, Sha256};
use spore_core::{creature_data, creature_generated_skin_edges_v1 as edges,
    creature_generated_skin_grid_query_v1 as grid,
    creature_generated_skin_kernel_v1::Lane,
    creature_generated_skin_vertex_fixture_v1::{self as fixture, PackedInput, SnapshotAdmission},
    native_sse::Policy, native_x87::{Context, Precision, Rounding}};
use std::{error::Error, fs::File, io::Read};

fn read_bounded(path: &str, max: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max { return Err("input exceeds size limit".into()); }
    Ok(bytes)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 || args[3] != "--assume-frozen-and-supplied-snapshot" {
        return Err("usage: creature_skin_source_vertex_inspect_v1 <decoded-creaturedata> <sha256> <snapshot.json> --assume-frozen-and-supplied-snapshot".into());
    }
    if args[1].len() != 64 || !args[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("SHA-256 must be exactly 64 hex digits".into());
    }
    let bytes = read_bounded(&args[0], 64 * 1024 * 1024)?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    if !hash.eq_ignore_ascii_case(&args[1]) { return Err("source SHA-256 mismatch".into()); }
    let parsed = creature_data::parse(&bytes)?;
    if !parsed.exact || parsed.consumed != bytes.len() { return Err("source is not exact".into()); }
    if parsed.parts.len() > edges::MAX_ROWS || parsed.data0.len() > edges::MAX_ROWS * edges::ROW_SIZE {
        return Err("source exceeds row limit".into());
    }
    let rows: Vec<[u8; edges::ROW_SIZE]> = parsed.data0.chunks_exact(edges::ROW_SIZE)
        .map(|s| s.try_into().unwrap()).collect();
    if rows.len() != parsed.parts.len() || rows.len() * edges::ROW_SIZE != parsed.data0.len() {
        return Err("inconsistent rows".into());
    }
    let snapshot_bytes = read_bounded(&args[2], 16 * 1024 * 1024)?;
    let snapshot: serde_json::Value = serde_json::from_slice(&snapshot_bytes)?;
    let get = |key: &str| snapshot.get(key).cloned().ok_or_else(|| format!("missing {key}"));
    let positions: Vec<[f32; 3]> = serde_json::from_value(get("endpoint_positions")?)?;
    let point = serde_json::from_value(get("point")?)?;
    let origin = serde_json::from_value(get("grid_origin")?)?;
    let scale = serde_json::from_value(get("grid_scale")?)?;
    let lane_values = get("lanes")?;
    let lane_values = lane_values.as_array().ok_or("lanes must be an array")?;
    let mut lanes = Vec::new();
    for item in lane_values {
        let value = |key: &str| item.get(key).cloned().ok_or_else(|| format!("missing lane {key}"));
        lanes.push(Lane { center: serde_json::from_value(value("center")?)?,
            cutoff: serde_json::from_value(value("cutoff")?)?,
            scale: serde_json::from_value(value("scale")?)?,
            coefficient: serde_json::from_value(value("coefficient")?)?,
            key: serde_json::from_value(value("key")?)? });
    }
    let lists: Vec<Vec<usize>> = serde_json::from_value(get("candidates")?)?;
    let candidates: [Vec<usize>; grid::CELL_COUNT] = lists.try_into().map_err(|_| "candidates must have 512 lists")?;
    let result = fixture::evaluate_packed(PackedInput { rows: &rows, endpoint_positions: &positions,
        point, grid_origin: origin, grid_scale: scale, lanes: &lanes, candidates: &candidates,
        sse: Policy::Mxcsr1f80, x87: Context { precision: Precision::Bits53, rounding: Rounding::NearestEven } }, SnapshotAdmission)?;
    println!("{}", serde_json::json!({
        "schema":"creature-skin-source-vertex-inspection-v1", "source_sha256":hash,
        "snapshot_sha256":format!("{:x}", Sha256::digest(&snapshot_bytes)),
        "rows":rows.len(), "endpoints":result.topology.endpoint_count,
        "edges":result.topology.edges.len(), "address":result.query.address.linear_word,
        "cell_present":result.query.cell_present, "requests":result.query.requests.len(),
        "weights":result.initial_record.weights, "record":result.initial_record.record,
        "admissions":["serialized rows represent frozen post-mutation rows", "supplied endpoint positions, grid bounds, lane records and candidate lists correspond", "preparation and vector operations succeeded", "no positive correction or smoothing"],
        "not_established":["runtime resource selection", "native grid construction", "palette identity", "generated creature"]
    }));
    Ok(())
}
