//! Verify two installed, hash-pinned event payloads and their parsed gain.
//! Static package inspection only; no native provider selection or playback.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::audio_event::{self, AudioValue};
use std::{error::Error, fs::File, io::{BufReader, Read}, path::Path};

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: collision_audio_gain_verify_v1 <PatchData.package> <Spore_Audio2.package>".into()); }
    let paths = [Path::new(&args[0]), Path::new(&args[1])];
    let expected = [
        "359d9b975aca87c70e0d4414199893c9819c8a4b4390dc43ff4666838f0a0c12",
        "54b30dab634ccc52c40035cce4da31572fd0fbc1c536c323b1481867cbd1a2db",
    ];
    let before = [hash_file(paths[0])?, hash_file(paths[1])?];
    let key = NativeResourceKey { instance_id: 0x34c2ff0b, type_id: 0x02b9f662, group_id: 0x021407ee };
    for (i, path) in paths.iter().enumerate() {
        let package = NativePackageIndex::open(path)?;
        let entry = package.index().find_unique(key)?.ok_or("event key missing")?;
        let bytes = package.read(entry)?;
        let payload_sha = format!("{:x}", Sha256::digest(&bytes));
        if bytes.len() != 296 || payload_sha != expected[i] {
            return Err(format!("{}: unexpected event length {} or SHA-256 {payload_sha}", path.display(), bytes.len()).into());
        }
        // Independently check the first property header and bytes, not just
        // the parser's interpretation of them. Type and ID are big-endian.
        if bytes[4..8] != 0x25df0108u32.to_be_bytes() || bytes[8..10] != 0x000du16.to_be_bytes()
            || bytes[12..16] != [0x3f, 0x00, 0x00, 0x00] {
            return Err(format!("{}: unexpected gain header or raw value", path.display()).into());
        }
        let event = audio_event::parse(&bytes)?;
        if !event.exact { return Err(format!("{}: event scan was not exact", path.display()).into()); }
        let gains: Vec<_> = event.entries.iter().filter(|e| e.hash == 0x25df0108).collect();
        if gains.len() != 1 || gains[0].val_type != 0x000d {
            return Err(format!("{}: expected one float gain", path.display()).into());
        }
        let AudioValue::Float(value) = &gains[0].value else { return Err("gain is not a scalar float".into()); };
        if value.to_bits() != 0x3f000000 { return Err(format!("{}: gain bits {:08X}, expected 3F000000", path.display(), value.to_bits()).into()); }
        println!("event={} key=34C2FF0B:02B9F662:021407EE decoded_bytes=296 payload_sha256={payload_sha} gain_raw=3f000000 gain_bits={:08X} gain={value}", path.display(), value.to_bits());
    }
    let after = [hash_file(paths[0])?, hash_file(paths[1])?];
    if before != after { return Err("input package endpoint hashes changed during verification".into()); }
    for i in 0..2 {
        println!("package={} sha256_before={} sha256_after={}", paths[i].display(), before[i], after[i]);
    }
    println!("Pinned decoded source bytes and parser value checked. Native big-endian interpretation is conditional on the dedicated loader and untransformed complete stream reads; provider selection, live execution and playback are not established. Equal endpoint hashes do not prove continuous file/path stability; trusted stable inputs are required.");
    Ok(())
}
