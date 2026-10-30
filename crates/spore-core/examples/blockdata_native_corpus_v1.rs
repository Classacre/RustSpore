//! Real-data verification adapter for NativeSamples. See tools/blockdata_native_corpus_v1.py.
//! Input is a bounded stream of length-prefixed raw resources; output contains
//! exact f32 bits, all samples, and checked mixed-radix index round trips.
use spore_core::blockdata::NativeSamples;
use std::{error::Error, fs::OpenOptions, io::{BufReader, BufWriter, Read, Write}};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { return Err("usage: blockdata_native_corpus_v1 <input> <new-output>".into()); }
    let mut input = BufReader::new(std::fs::File::open(&args[1])?);
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&args[2])?);
    let mut magic = [0u8; 8]; input.read_exact(&mut magic)?;
    if &magic != b"BDRAW001" { return Err("bad input magic".into()); }
    output.write_all(b"BDOUT001")?;
    let mut records = 0usize;
    let mut samples_total = 0usize;
    loop {
        let mut word = [0u8; 4]; input.read_exact(&mut word)?;
        let len = u32::from_le_bytes(word) as usize;
        if len == 0 { break; }
        if !(36..100_000).contains(&len) || records >= 100_000 { return Err("input bounds exceeded".into()); }
        let mut bytes = vec![0; len]; input.read_exact(&mut bytes)?;
        let view = NativeSamples::new(&bytes).map_err(|e| format!("record {records}: {e:?}"))?;
        let dims = view.dimensions();
        output.write_all(&[u8::try_from(dims.len())?])?;
        output.write_all(dims)?;
        output.write_all(&(u32::try_from(view.sample_count())?).to_le_bytes())?;
        let (a,b) = view.header_triples();
        for value in a.into_iter().chain(b) { output.write_all(&value.to_bits().to_le_bytes())?; }
        for i in 0..view.sample_count() {
            let mut coords = vec![0u8; dims.len()];
            let mut n = i;
            for j in (0..dims.len()).rev() { coords[j] = (n % usize::from(dims[j])) as u8; n /= usize::from(dims[j]); }
            if n != 0 || view.index(&coords).map_err(|e| format!("index: {e:?}"))? != i {
                return Err(format!("index mismatch record {records} sample {i}").into());
            }
            let (a,b) = view.sample(i).map_err(|e| format!("sample: {e:?}"))?;
            for value in a.into_iter().chain(b) { output.write_all(&value.to_bits().to_le_bytes())?; }
            samples_total += 1;
        }
        records += 1;
    }
    let mut extra = [0]; if input.read(&mut extra)? != 0 { return Err("trailing input".into()); }
    output.flush()?;
    println!("records={records} samples={samples_total} output=f32_bits indexing=round_trip");
    Ok(())
}
