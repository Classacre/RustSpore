//! Pending root review. Independent fixed-record decoder; no source install needed.
use spore_core::anim_root_binding::{construct, Descriptor, Input};
use std::io::Write;
use std::{env, fs, path::PathBuf};

fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn run(b: &[u8]) -> Result<Vec<u8>, String> {
    if b.len() != 364 { return Err("length".into()); }
    let h: Vec<u32> = (0..16).map(|i| word(b, i * 4)).collect();
    if h[10] > 24 || h[3] > 255 || h[4] > 255 || h[7] > 1 || h[8] > 255
        || (h[11] != u32::MAX && h[11] > 255) { return Err("range".into()); }
    let n = h[10] as usize;
    if h[12..].iter().any(|v| *v != 0) || b[64+n*12..352].iter().any(|v| *v != 0)
        || b[361..].iter().any(|v| *v != 0) { return Err("padding".into()); }
    let input = Input {
        primary_flags: h[0], selected_root: h[1], selected_ordinal: h[2],
        channel_byte: h[3] as u8, channel_index: h[4] as u8,
        words: [h[5], h[6]], mirror_argument: h[7] != 0,
        owner_byte: h[8] as u8, metadata_ac: h[9],
        secondary_override: (h[11] != u32::MAX).then_some(h[11] as u8),
        descriptors: (0..n).map(|i| Descriptor {
            flags: word(b, 64+i*12), tag: word(b, 68+i*12), slot: word(b, 72+i*12),
        }).collect(), claims: b[352..361].try_into().unwrap(),
    };
    let p = construct(&input).map_err(|e| format!("{e:?}"))?;
    let mut out = Vec::with_capacity(48);
    for v in [u32::from(p.record.is_some()), 0, p.root_count_delta, p.secondary_count_delta] {
        out.extend(v.to_le_bytes());
    }
    out.extend(p.claims);
    out.extend([0; 3]);
    for v in p.record.unwrap_or([0; 4]) { out.extend(v.to_le_bytes()); }
    out.extend([0; 4]);
    Ok(out)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: INPUT_DIR OUTPUT_BIN".into()); }
    let dir = PathBuf::from(&args[0]);
    let input = fs::read(dir.join("inputs.bin"))?;
    let reference = fs::read(dir.join("reference.bin"))?;
    if input.len() % 364 != 0 || reference.len() != input.len()/364*48 {
        return Err("cohort framing".into());
    }
    let mut output = Vec::with_capacity(reference.len());
    for (i, b) in input.chunks_exact(364).enumerate() {
        let p = run(b).map_err(|e| format!("case {i}: {e}"))?;
        if p != reference[i*48..(i+1)*48] { return Err(format!("mismatch {i}").into()); }
        output.extend(p);
    }
    let rejected = fs::read(dir.join("rejected.bin"))?;
    if rejected.len() % 364 != 0 { return Err("negative framing".into()); }
    for (i, b) in rejected.chunks_exact(364).enumerate() {
        if run(b).is_ok() { return Err(format!("negative {i} accepted").into()); }
    }
    fs::OpenOptions::new().create_new(true).write(true)
        .open(PathBuf::from(&args[1]))?.write_all(&output)?;
    println!("{} reference cases agree; {} explicit errors", input.len()/364, rejected.len()/364);
    Ok(())
}
