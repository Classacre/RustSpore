//! Synthetic adapter for the source sweep model, not gameplay.
use spore_core::cell_source_sweep_pairs_v1 as sweep;
use std::{error::Error, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}};

fn word(reader: &mut impl Read) -> Result<u32, Box<dyn Error>> {
    let mut bytes = [0; 4]; reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: sweep-adapter input new-output".into()); }
    let mut input = BufReader::new(std::fs::File::open(&args[1])?);
    let mut magic = [0u8; 8]; input.read_exact(&mut magic)?;
    if &magic != b"SWPIN001" { return Err("bad input magic".into()); }
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&args[2])?);
    output.write_all(b"SWPOUT01")?;
    let mut cases = 0usize;
    loop {
        let n = word(&mut input)?; let capacity = word(&mut input)?;
        if n == u32::MAX && capacity == 0 { break; }
        if n > 64 || capacity > 128 || cases >= 10_000 { return Err("protocol bound".into()); }
        let mut slots = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let id = word(&mut input)? as i32;
            let tag = word(&mut input)?;
            let x0 = f32::from_bits(word(&mut input)?);
            let x1 = f32::from_bits(word(&mut input)?);
            let y0 = f32::from_bits(word(&mut input)?);
            let y1 = f32::from_bits(word(&mut input)?);
            if ![x0,x1,y0,y1].into_iter().all(|v| v == 0.0 || v.is_normal()) {
                return Err("subnormal or nonfinite endpoint outside comparison domain".into());
            }
            slots.push(sweep::SweepSlot { id, tag, min:[x0,y0], max:[x1,y1] });
        }
        let result = sweep::rebuild(&slots,capacity as usize).map_err(|e| format!("sweep: {e:?}"))?;
        output.write_all(&u32::try_from(result.stored_order().len())?.to_le_bytes())?;
        output.write_all(&u32::try_from(result.suppressed)?.to_le_bytes())?;
        for (a,b) in result.stored_order().iter().copied().chain(result.dispatch_order()) {
            output.write_all(&a.to_le_bytes())?; output.write_all(&b.to_le_bytes())?;
        }
        cases += 1;
    }
    let mut trailing = [0]; if input.read(&mut trailing)? != 0 { return Err("trailing input".into()); }
    output.flush()?;
    println!("cases={cases} model_only=true source_execution=false");
    Ok(())
}
