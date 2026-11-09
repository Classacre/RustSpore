//! Standalone adapter for blockdata-query-draft-v1 model comparison.
//! This uses a minimal Vec3 shim; it does not execute or validate Spore code.
use std::{error::Error, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}};

#[derive(Clone, Debug)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }
impl Vec3 { pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } } }

#[path = "../crates/spore-core/src/blockdata.rs"]
mod blockdata;
#[path = "../re/blockdata-query-draft-v1.rs"]
mod query_draft;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: adapter input new-output".into()); }
    let mut input = BufReader::new(std::fs::File::open(&args[1])?);
    let mut magic = [0u8; 8]; input.read_exact(&mut magic)?;
    if &magic != b"BDQIN001" { return Err("invalid input magic".into()); }
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&args[2])?);
    output.write_all(b"BDQOUT01")?;
    let mut cases = 0usize;
    loop {
        let mut framing = [0u8; 8]; input.read_exact(&mut framing)?;
        let len = u32::from_le_bytes(framing[..4].try_into()?) as usize;
        let ncoords = u32::from_le_bytes(framing[4..].try_into()?) as usize;
        if len == 0 && ncoords == 0 { break; }
        if !(36..100_000).contains(&len) || ncoords > 8 || cases >= 20_000 {
            return Err("invalid record bounds".into());
        }
        let mut raw = vec![0; len]; input.read_exact(&mut raw)?;
        let mut coords = Vec::with_capacity(ncoords);
        for _ in 0..ncoords {
            let mut b = [0; 4]; input.read_exact(&mut b)?;
            coords.push(f32::from_le_bytes(b));
        }
        let view = blockdata::NativeSamples::new(&raw).map_err(|e| format!("shape: {e:?}"))?;
        let (first, second) = query_draft::query(&view, &coords).map_err(|e| format!("query: {e:?}"))?;
        for v in first.into_iter().chain(second) { output.write_all(&v.to_bits().to_le_bytes())?; }
        cases += 1;
    }
    let mut trailing = [0u8];
    if input.read(&mut trailing)? != 0 { return Err("trailing input".into()); }
    output.flush()?;
    println!("cases={cases} output=f32_bits model_only=true");
    Ok(())
}
