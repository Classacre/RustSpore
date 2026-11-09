use std::{env, fs, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: snr_xas_verify INPUT.snr OUTPUT.pcm".into());
    }
    let source = fs::read(&args[1])?;
    let decoded = match source.first().map(|b| b & 15) {
        Some(2) => spore_core::snr_pcm16::decode(&source)?,
        Some(4) => spore_core::snr_xas::decode(&source)?,
        _ => return Err("unsupported source SNR codec".into()),
    };
    let mut bytes = Vec::with_capacity(decoded.pcm.len() * 2);
    for s in &decoded.pcm {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.write_all(&bytes)?;
    println!(
        "{} Hz, {} channels, {} samples/channel",
        decoded.sample_rate,
        decoded.channels,
        decoded.pcm.len() / decoded.channels
    );
    if let Some(start) = decoded.loop_start {
        println!("source loop start: {start} samples/channel");
    }
    Ok(())
}
