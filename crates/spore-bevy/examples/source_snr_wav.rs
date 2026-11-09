//! Export a specifically identified supported source SNR to a new PCM WAV.
//! Installed assets remain on the user's machine; this does not map game events.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use std::{
    env,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 3 && args.len() != 4 {
        return Err(
            "usage: source_snr_wav <package> <16-hex-scope-instance> <new-target-file.wav> [explicit-sns-package]".into(),
        );
    }
    let text = args[1].to_str().ok_or("key must be ASCII hex")?;
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if digits.len() != 16 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("key must be exactly 16 hexadecimal digits, optionally prefixed by 0x".into());
    }
    let scoped = u64::from_str_radix(digits, 16)?;
    let output = Path::new(&args[2]);
    if !output
        .extension()
        .is_some_and(|v| v.eq_ignore_ascii_case("wav"))
    {
        return Err("output must end in .wav".into());
    }
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .canonicalize()?;
    let parent = output
        .parent()
        .ok_or("output needs a parent directory")?
        .canonicalize()?;
    if !parent.starts_with(&target) {
        return Err("output parent must be inside target".into());
    }
    let package = Path::new(&args[0]);
    let mut file = fs::File::open(package)?;
    let mut header = [0; dbpf::HEADER_SIZE];
    file.read_exact(&mut header)?;
    let header = dbpf::DbpfHeader::parse(&header)?;
    if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
        return Err("index exceeds bounds".into());
    }
    let archive = NativePackageIndex::open(package)?;
    let key = NativeResourceKey::from_scoped_instance(0x01a527db, scoped);
    let entry = archive
        .index()
        .find_unique(key)?
        .ok_or("source SNR key not found")?;
    if entry.stored_size.max(entry.decompressed_size) > 32 * 1024 * 1024 {
        return Err("resource exceeds 32 MiB".into());
    }
    let source = archive.read(entry)?;
    let audio = if args.len() == 4 {
        let sns_path = Path::new(&args[3]);
        let mut sns_file = fs::File::open(sns_path)?;
        let mut sns_header = [0; dbpf::HEADER_SIZE];
        sns_file.read_exact(&mut sns_header)?;
        let sns_header = dbpf::DbpfHeader::parse(&sns_header)?;
        if sns_header.index_size > 64 * 1024 * 1024 || sns_header.index_entry_count > 2_000_000 {
            return Err("SNS index exceeds bounds".into());
        }
        let sns_archive = NativePackageIndex::open(sns_path)?;
        let sns_key = NativeResourceKey::from_scoped_instance(0x01eef63a, scoped);
        let sns_entry = sns_archive
            .index()
            .find_unique(sns_key)?
            .ok_or("source SNS key not found")?;
        if sns_entry.stored_size.max(sns_entry.decompressed_size) > 32 * 1024 * 1024 {
            return Err("SNS resource exceeds 32 MiB".into());
        }
        let sns = sns_archive.read(sns_entry)?;
        spore_core::snr_xas::decode_gigasample(&source, &sns)?
    } else {
        match source.first().map(|b| b & 15) {
            Some(2) => spore_core::snr_pcm16::decode(&source)?,
            Some(4) => spore_core::snr_xas::decode(&source)?,
            _ => return Err("unsupported source SNR codec".into()),
        }
    };
    let pcm_bytes: u32 = audio
        .pcm
        .len()
        .checked_mul(2)
        .ok_or("PCM size overflow")?
        .try_into()?;
    let riff_size = pcm_bytes.checked_add(36).ok_or("WAV size overflow")?;
    let channels: u16 = audio.channels.try_into()?;
    let block_align = channels * 2;
    let byte_rate = audio
        .sample_rate
        .checked_mul(u32::from(block_align))
        .ok_or("byte rate overflow")?;
    let mut wav = Vec::with_capacity(pcm_bytes as usize + 44);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_size.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&audio.sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&pcm_bytes.to_le_bytes());
    for sample in &audio.pcm {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(parent.join(output.file_name().ok_or("output filename required")?))?;
    file.write_all(&wav)?;
    println!(
        "Exported {key}: {} Hz, {} channels, {} samples/channel",
        audio.sample_rate,
        channels,
        audio.pcm.len() / audio.channels
    );
    if let Some(start) = audio.loop_start {
        println!(
            "Source loop starts at sample {start}; WAV contains one pass without loop metadata."
        );
    }
    Ok(())
}
