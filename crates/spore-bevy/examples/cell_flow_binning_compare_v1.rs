//! Compare the fixed synthetic E4DC40 fixture with an explicitly supplied host output.
//! This does not establish host provenance, live game state, or game conformance.
use sha2::{Digest, Sha256};
use spore_core::cell_flow_binning_v1::{self as binning, Slot, X87CopyAdmission, MAX_BIN_RECORDS};
use spore_core::cell_flow_field::{CellFlowFrame, CELL_FLOW_SAMPLES};
use spore_core::native_sse::Policy;
use std::{fs::File, io::Read, path::Path};

const SLOTS: usize = 1500;
const SCRATCH: usize = 0x130b0;
const HOST_LEN: usize = SCRATCH + 16;
const POS_SHA: &str = "8c73602f4cd0f53391108ea0e6f0534b986186006b9e575eb7284464be9a16cd";
const VEC_SHA: &str = "be0ba040ce8e731c21bd156e7ea370dc532e6c625297572e01c2f86739a09f16";
const SCALAR_SHA: &str = "9956e3d45faf930ce5c3e14841a49445202a97ba3fbffd419d3dc6422ac99371";

fn read_fixed(path: &Path, size: usize, expected_sha: Option<&str>) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut data = Vec::with_capacity(size + 1);
    file.take((size + 1) as u64).read_to_end(&mut data).map_err(|e| format!("{}: {e}", path.display()))?;
    if data.len() != size { return Err(format!("{}: expected {size} bytes, got {}", path.display(), data.len())); }
    let sha = format!("{:x}", Sha256::digest(&data));
    if let Some(expected) = expected_sha {
        if sha != expected { return Err(format!("{}: SHA-256 mismatch: {sha}", path.display())); }
    }
    Ok(data)
}
fn word(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().expect("fixed validated extent"))
}
fn triple(data: &[u8], index: usize) -> [f32; 3] {
    [0, 4, 8].map(|offset| f32::from_bits(word(data, index * 12 + offset)))
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 5 {
        return Err("usage: cell_flow_binning_compare_v1 <positions.bin> <vectors.bin> <scalars.bin> <host-output.bin>".into());
    }
    let positions = read_fixed(Path::new(&args[1]), SLOTS * 12, Some(POS_SHA))?;
    let vectors = read_fixed(Path::new(&args[2]), SLOTS * 12, Some(VEC_SHA))?;
    let scalars = read_fixed(Path::new(&args[3]), SLOTS * 4, Some(SCALAR_SHA))?;
    let host = read_fixed(Path::new(&args[4]), HOST_LEN, None)?;
    let host_sha = format!("{:x}", Sha256::digest(&host));
    let cw = u16::from_le_bytes(host[SCRATCH..SCRATCH + 2].try_into().unwrap());
    let sw = u16::from_le_bytes(host[SCRATCH + 2..SCRATCH + 4].try_into().unwrap());
    let mxcsr = word(&host, SCRATCH + 4);
    let eflags = word(&host, SCRATCH + 8);
    let reserved = word(&host, SCRATCH + 12);
    // Sticky flags are not modeled; reject unexpected controls or x87 stack fault/top.
    if cw != 0x027f || mxcsr & !0x3f != 0x1f80 || sw & 0x3840 != 0 || eflags & 0x400 != 0 || reserved != 0 {
        return Err(format!("unsupported metadata cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} eflags={eflags:08X} reserved={reserved:08X}"));
    }
    let slots: Vec<_> = (0..SLOTS).map(|i| Slot {
        position: triple(&positions, i), vector: triple(&vectors, i), scalar: f32::from_bits(word(&scalars, i * 4)),
    }).collect();
    let model = binning::build(&slots, CellFlowFrame { origin: [0.0; 2], scale: [1.0; 2] }, Policy::Mxcsr1f80, X87CopyAdmission)
        .map_err(|e| format!("model rejected fixture: {e}"))?;
    let mut mismatches = 0usize;
    let mut host_accepted = 0usize;
    let mut populated = 0usize;
    let mut compared_words = 0usize;
    for bin in 0..CELL_FLOW_SAMPLES {
        let host_count = word(&host, bin * 4) as usize;
        if host_count > MAX_BIN_RECORDS { return Err(format!("host bin {bin}: invalid count {host_count}")); }
        host_accepted += host_count;
        populated += usize::from(host_count != 0);
        let row = &model.bins[bin];
        if host_count != row.len() {
            mismatches += 1;
            if mismatches <= 12 { eprintln!("mismatch bin={bin} count model={} host={host_count}", row.len()); }
        }
        // Compare only present records; an unmatched count is already a mismatch.
        // Unused native slots are poisoned and are never source outputs.
        for (j, record) in row.iter().take(host_count).enumerate() {
            let off = 0x4b0 + bin * 0x100 + j * 0x20;
            let expected = [record.position[0].to_bits(), record.position[1].to_bits(), record.position[2].to_bits(),
                record.vector[0].to_bits(), record.vector[1].to_bits(), record.vector[2].to_bits(), record.scalar.to_bits(), record.index];
            for (k, value) in expected.into_iter().enumerate() {
                let observed = word(&host, off + k * 4);
                compared_words += 1;
                if value != observed {
                    mismatches += 1;
                    if mismatches <= 12 { eprintln!("mismatch bin={bin} record={j} word={k} model={value:08X} host={observed:08X}"); }
                }
            }
        }
    }
    println!("FLOW_BINNING_COMPARE bins={CELL_FLOW_SAMPLES} host_populated={populated} host_accepted={host_accepted} compared_record_words={compared_words} mismatches={mismatches} outside={} full_bin={} supplied_host_sha256={host_sha} cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} eflags={eflags:08X}", model.outside, model.full_bin);
    if mismatches != 0 { return Err(format!("{mismatches} count or record-word mismatches")); }
    println!("Fixed synthetic fixture comparison only; supplied host provenance, live source state and game conformance are not established.");
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("FLOW_BINNING_COMPARE error: {error}"); std::process::exit(1); }
}
