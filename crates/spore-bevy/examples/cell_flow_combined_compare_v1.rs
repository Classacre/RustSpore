//! Compare a supplied combined fixed-host output with a controlled Rust composition.
//! Does not establish host provenance, native schedule, live state or gameplay.
use sha2::{Digest, Sha256};
use spore_core::cell_flow_binning_v1::{self as binning, Slot, X87CopyAdmission, MAX_BIN_RECORDS};
use spore_core::cell_flow_field::{CellFlowFrame, CELL_FLOW_SAMPLES};
use spore_core::cell_flow_snapshot_v1::{self as snapshot, PreservedScratchAdmission};
use spore_core::native_sse::Policy;
use spore_core::native_x87::Context;
use std::{fs::File, io::Read, path::Path};

const SLOTS: usize = 1500;
const SCRATCH: usize = 78000;
const META: usize = 84000;
const HOST_LEN: usize = 84032;
const POS_SHA: &str = "8c73602f4cd0f53391108ea0e6f0534b986186006b9e575eb7284464be9a16cd";
const VEC_SHA: &str = "be0ba040ce8e731c21bd156e7ea370dc532e6c625297572e01c2f86739a09f16";
const SCALAR_SHA: &str = "0e6f5c53d8dd3c7b939a650fbf26bbf10c9a9b900740d74c18868ef48ac1705d";

fn read_fixed(path: &Path, size: usize, expected: Option<&str>) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut data = Vec::with_capacity(size + 1);
    file.take((size + 1) as u64).read_to_end(&mut data).map_err(|e| format!("{}: {e}", path.display()))?;
    if data.len() != size { return Err(format!("{}: expected {size} bytes, got {}", path.display(), data.len())); }
    let sha = format!("{:x}", Sha256::digest(&data));
    if let Some(expected) = expected {
        if sha != expected { return Err(format!("{}: SHA-256 mismatch: {sha}", path.display())); }
    }
    Ok(data)
}
fn word(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().expect("validated fixed extent"))
}
fn triple(data: &[u8], index: usize) -> [f32; 3] {
    [0, 4, 8].map(|offset| f32::from_bits(word(data, index * 12 + offset)))
}
fn metadata(host: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    let cw = u16::from_le_bytes(host[offset..offset + 2].try_into().unwrap());
    let sw = u16::from_le_bytes(host[offset + 2..offset + 4].try_into().unwrap());
    let mxcsr = word(host, offset + 4);
    let eflags = word(host, offset + 8);
    let reserved = word(host, offset + 12);
    // Sticky FP status is reported, but not modeled as an output.
    if cw != 0x027f || sw & 0x3840 != 0 || mxcsr & !0x3f != 0x1f80 || eflags & 0x400 != 0 || reserved != 0 {
        return Err(format!("{label}: unsupported metadata cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} eflags={eflags:08X} reserved={reserved:08X}"));
    }
    println!("{label} cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} eflags={eflags:08X}");
    Ok(cw)
}
fn compare(mismatches: &mut usize, label: &str, model: u32, host: u32) {
    if model != host {
        *mismatches += 1;
        if *mismatches <= 12 { eprintln!("mismatch {label} model={model:08X} host={host:08X}"); }
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 5 { return Err("usage: cell_flow_combined_compare_v1 <positions.bin> <vectors.bin> <scalars.bin> <host-output.bin>".into()); }
    let positions = read_fixed(Path::new(&args[1]), SLOTS * 12, Some(POS_SHA))?;
    let vectors = read_fixed(Path::new(&args[2]), SLOTS * 12, Some(VEC_SHA))?;
    let scalars = read_fixed(Path::new(&args[3]), SLOTS * 4, Some(SCALAR_SHA))?;
    let host = read_fixed(Path::new(&args[4]), HOST_LEN, None)?;
    let host_sha = format!("{:x}", Sha256::digest(&host));
    metadata(&host, META, "binning")?;
    let aggregate_cw = metadata(&host, META + 16, "aggregation")?;
    let slots: Vec<_> = (0..SLOTS).map(|i| Slot {
        position: triple(&positions, i), vector: triple(&vectors, i), scalar: f32::from_bits(word(&scalars, i * 4)),
    }).collect();
    let frame = CellFlowFrame { origin: [0.0; 2], scale: [1.0; 2] };
    let bins = binning::build(&slots, frame, Policy::Mxcsr1f80, X87CopyAdmission).map_err(|e| format!("binning rejected fixture: {e}"))?;
    let field = snapshot::build(&slots, frame, Policy::Mxcsr1f80,
        Context::from_control_word(aggregate_cw).map_err(|e| e.to_string())?, X87CopyAdmission, PreservedScratchAdmission)
        .map_err(|e| format!("snapshot rejected fixture: {e}"))?;
    let mut mismatches = 0usize;
    let mut accepted = 0usize;
    let mut populated = 0usize;
    let mut record_words = 0usize;
    for bin in 0..CELL_FLOW_SAMPLES {
        let count = word(&host, bin * 4) as usize;
        if count > MAX_BIN_RECORDS { return Err(format!("host bin {bin}: invalid count {count}")); }
        accepted += count;
        populated += usize::from(count != 0);
        compare(&mut mismatches, &format!("bin={bin} count"), field.counts[bin] as u32, count as u32);
        if bins.bins[bin].len() != field.counts[bin] as usize {
            return Err(format!("separate binning and snapshot counts differ for bin {bin}"));
        }
        // Only records with a counterpart are compared; differing counts are
        // already mismatches. Never interpret unused A5-filled slots.
        for (j, r) in bins.bins[bin].iter().take(count).enumerate() {
            let off = 0x4b0 + bin * 0x100 + j * 0x20;
            let words = [r.position[0].to_bits(), r.position[1].to_bits(), r.position[2].to_bits(),
                r.vector[0].to_bits(), r.vector[1].to_bits(), r.vector[2].to_bits(), r.scalar.to_bits(), r.index];
            for (k, value) in words.into_iter().enumerate() {
                compare(&mut mismatches, &format!("bin={bin} record={j} word={k}"), value, word(&host, off + k * 4));
                record_words += 1;
            }
        }
        let out = field.outputs[bin];
        for axis in 0..3 {
            compare(&mut mismatches, &format!("bin={bin} vector={axis}"), out.vector[axis].to_bits(), word(&host, SCRATCH + bin * 12 + axis * 4));
        }
        compare(&mut mismatches, &format!("bin={bin} mean_reciprocal"), out.mean_reciprocal.to_bits(), word(&host, SCRATCH + 3600 + bin * 4));
        compare(&mut mismatches, &format!("bin={bin} magnitude"), out.magnitude.to_bits(), word(&host, SCRATCH + 4800 + bin * 4));
    }
    println!("FLOW_COMBINED_COMPARE bins={CELL_FLOW_SAMPLES} host_populated={populated} host_accepted={accepted} compared_record_words={record_words} field_words=1500 mismatches={mismatches} outside={} full_bin={} supplied_host_sha256={host_sha}", bins.outside, bins.full_bin);
    println!("Fixed synthetic comparison only: host provenance and skipped E4E730 behavior are not established; FP flags are reset between host calls; no live or gameplay claim.");
    if mismatches != 0 { return Err(format!("{mismatches} count or word mismatches")); }
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("FLOW_COMBINED_COMPARE error: {error}"); std::process::exit(1); }
}
