//! Bounded synthetic model adapter; no original instructions or actors run.
use spore_core::{cell_actor_geometry_projection_v1::Output,
    cell_broadphase_gate_v1::{evaluate, Error as GateError}, native_sse::Policy};
use std::{error::Error, fs::OpenOptions, io::{Read, Write}};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: gate-adapter input new-output".into()); }
    let input = std::fs::File::open(&args[1])?;
    if input.metadata()?.len() > 1024 * 1024 { return Err("input too large".into()); }
    let mut bytes = Vec::new();
    input.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() < 12 || bytes.len() > 1024 * 1024 || &bytes[..8] != b"GATEIN01" {
        return Err("invalid header".into());
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    if count == 0 || count > 10_000 || bytes.len() != 12 + count * 32 {
        return Err("invalid extent".into());
    }
    let mut output = b"GATEOUT1".to_vec();
    output.extend_from_slice(&(count as u32).to_le_bytes());
    for row in bytes[12..].chunks_exact(32) {
        let mut f = [0u32; 8];
        for (i, c) in row.chunks_exact(4).enumerate() { f[i] = u32::from_le_bytes(c.try_into()?); }
        let a = Output { center: [f[0], f[1], f[2]], scalar: f[3] };
        let b = Output { center: [f[4], f[5], f[6]], scalar: f[7] };
        let (status, values) = match evaluate(a, b, Policy::Mxcsr1f80) {
            Ok(v) => (0u32, [v.delta[0], v.delta[1], v.delta[2], v.squared_distance, v.squared_sum, u32::from(v.passes)]),
            Err(GateError::NonFiniteInput) => (1, [0; 6]),
            Err(GateError::NonFiniteIntermediate) => (2, [0; 6]),
            Err(e) => return Err(format!("unsupported model error: {e:?}").into()),
        };
        output.extend_from_slice(&status.to_le_bytes());
        for value in values { output.extend_from_slice(&value.to_le_bytes()); }
    }
    let mut out = OpenOptions::new().write(true).create_new(true).open(&args[2])?;
    out.write_all(&output)?;
    println!("cases={count} model_only=true original_execution=false");
    Ok(())
}
