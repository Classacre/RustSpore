//! Experimental reconstructed MPEG inspection only, not source sound playback.
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = env::args_os().skip(1).collect();
    if a.len() != 2 {
        return Err("usage: eal31b_inspect <source.snr> <new-target-file.mp3>".into());
    }
    let output = Path::new(&a[1]);
    if !output
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("mp3"))
    {
        return Err("output must end in .mp3".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .canonicalize()?;
    let parent = output.parent().ok_or("missing parent")?.canonicalize()?;
    if !parent.starts_with(root) {
        return Err("output parent must be inside target".into());
    }
    let source = fs::read(&a[0])?;
    if source.len() > 32 * 1024 * 1024 {
        return Err("source exceeds 32 MiB".into());
    }
    let result = spore_core::eal31b::reconstruct(&source)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(parent.join(output.file_name().ok_or("missing file name")?))?;
    file.write_all(&result.mpeg)?;
    println!("Reconstructed MPEG only: {} Hz, {} channels, {} granules, {} PCM insertions; source declares {} samples/channel. PCM inserts and delay are NOT applied.",result.sample_rate,result.channels,result.granule_count,result.insertions.len(),result.declared_samples);
    for i in result.insertions {
        println!(
            "PCM insertion granule={} offset={} samples/channel={} source bytes={}..{} unknown={}",
            i.granule,
            i.offset_samples,
            i.samples_per_channel,
            i.source_start,
            i.source_end,
            i.unknown
        );
    }
    Ok(())
}
