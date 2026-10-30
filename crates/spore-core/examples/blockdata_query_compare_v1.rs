//! Production blockdata query adapter for a bounded model-to-model comparison.
//! No original-game execution or floating-point environment parity is implied.
use spore_core::{blockdata::NativeSamples, blockdata_query_v1::query};
use std::{error::Error, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: blockdata_query_compare_v1 input new-output".into()); }
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
        let view = NativeSamples::new(&raw).map_err(|e| format!("shape: {e:?}"))?;
        let (first, second) = query(&view, &coords).map_err(|e| format!("query: {e:?}"))?;
        for v in first.into_iter().chain(second) { output.write_all(&v.to_bits().to_le_bytes())?; }
        cases += 1;
    }
    let mut trailing = [0u8];
    if input.read(&mut trailing)? != 0 { return Err("trailing input".into()); }
    output.flush()?;
    println!("cases={cases} output=f32_bits model_only=true");
    Ok(())
}
