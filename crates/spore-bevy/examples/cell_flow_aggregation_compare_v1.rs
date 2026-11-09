//! Compare one explicit fixed scratch snapshot and supplied host output.
//! Neither input proves original execution, a source environment, or gameplay.
use sha2::{Digest, Sha256};
use spore_core::cell_flow_aggregation_v1::{self as aggregation, Bin, Record, BIN_COUNT, MAX_RECORDS};
use spore_core::native_sse::Policy;
use spore_core::native_x87::Context;
use std::{fs::File, io::Read, path::Path};

const SCRATCH_LEN: usize = 0x130b0;
const HOST_LEN: usize = 6016;
const SCRATCH_SHA: &str = "36a411d608e427b1cbfccd3bb94bc0bf2ae81574f6eb735c9c1afc478c6b0ae6";

fn read_exact_bounded(path: &Path, size: usize) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut bytes = Vec::with_capacity(size + 1);
    file.take((size + 1) as u64).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() != size { return Err(format!("{}: expected {size} bytes, got {}", path.display(), bytes.len())); }
    Ok(bytes)
}

fn word(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().expect("validated fixed extent"))
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: cell_flow_aggregation_compare_v1 <fixed-scratch.bin> <host-output.bin>".into()); }
    let scratch = read_exact_bounded(Path::new(&args[1]), SCRATCH_LEN)?;
    let host = read_exact_bounded(Path::new(&args[2]), HOST_LEN)?;
    let scratch_hash = format!("{:x}", Sha256::digest(&scratch));
    if scratch_hash != SCRATCH_SHA { return Err(format!("fixed scratch SHA-256 mismatch: {scratch_hash}")); }
    let host_hash = format!("{:x}", Sha256::digest(&host));
    let cw = u16::from_le_bytes(host[6000..6002].try_into().unwrap());
    let sw = u16::from_le_bytes(host[6002..6004].try_into().unwrap());
    let mxcsr = word(&host, 6004);
    let eflags = word(&host, 6008);
    let reserved = word(&host, 6012);
    // Sticky status bits are reported, while the control fields must match.
    if cw != 0x027f || mxcsr & !0x3f != 0x1f80 || reserved != 0 {
        return Err(format!("unsupported host metadata cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} reserved={reserved:08X}"));
    }
    let mut records = Vec::<Vec<Record>>::with_capacity(BIN_COUNT);
    for bin in 0..BIN_COUNT {
        let count = word(&scratch, bin * 4) as i32;
        if !(0..=MAX_RECORDS as i32).contains(&count) { return Err(format!("bin {bin}: invalid count {count}")); }
        let mut row = Vec::with_capacity(count as usize);
        for j in 0..count as usize {
            let offset = 0x4b0 + bin * 0x100 + j * 0x20;
            let vector = [0x0c, 0x10, 0x14].map(|n| f32::from_bits(word(&scratch, offset + n)));
            let scalar = f32::from_bits(word(&scratch, offset + 0x18));
            if !scalar.is_finite() || scalar == 0.0 || !vector.iter().all(|v| v.is_finite()) {
                return Err(format!("bin {bin} record {j}: unsupported vector or denominator"));
            }
            row.push(Record { vector, scalar });
        }
        records.push(row);
    }
    let bins: Vec<_> = records.iter().map(|row| Bin { count: row.len() as i32, records: row }).collect();
    let actual = aggregation::aggregate(&bins, Policy::Mxcsr1f80, Context::from_control_word(cw).map_err(|e| e.to_string())?)
        .map_err(|e| format!("model rejected fixed snapshot: {e}"))?;
    let mut mismatches = 0usize;
    let mut compare = |bin: usize, name: &str, got: f32, offset: usize| -> Result<(), String> {
        let expected = word(&host, offset);
        if !f32::from_bits(expected).is_finite() { return Err(format!("host bin {bin} {name}: nonfinite output")); }
        if got.to_bits() != expected {
            mismatches += 1;
            if mismatches <= 12 { eprintln!("mismatch bin={bin} field={name} model={:08X} host={expected:08X}", got.to_bits()); }
        }
        Ok(())
    };
    for (i, item) in actual.iter().enumerate() {
        for (j, name) in ["x", "y", "z"].iter().enumerate() { compare(i, name, item.vector[j], i * 12 + j * 4)?; }
        compare(i, "mean_reciprocal", item.mean_reciprocal, 3600 + i * 4)?;
        compare(i, "magnitude", item.magnitude, 4800 + i * 4)?;
    }
    if mismatches != 0 { return Err(format!("{mismatches} of 1500 output words differ")); }
    println!("FLOW_AGGREGATION_COMPARE matched_words=1500 bins=300 scratch_sha256={scratch_hash} supplied_host_sha256={host_hash} cw={cw:04X} sw={sw:04X} mxcsr={mxcsr:08X} eflags={eflags:08X}");
    println!("fixed synthetic snapshot comparison only; host provenance, live source state, gameplay and original runtime are not established by this tool");
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("FLOW_AGGREGATION_COMPARE error: {error}"); std::process::exit(1); }
}
