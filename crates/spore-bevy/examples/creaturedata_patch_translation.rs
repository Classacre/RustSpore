//! Explicit, local-only extraction and translation patch; see docs/creaturedata-translation-example.md.
use dbpf::{native_index::{NativePackageIndex, NativeResourceKey}, DbpfHeader, HEADER_SIZE};
use sha2::{Digest, Sha256};
use spore_core::{creature_data, creature_data_edit::{patch_rest_transform, RestTransform}};
use std::{error::Error, fs::{self, File, OpenOptions}, io::{Read, Write}, path::Path};
type R<T> = Result<T, Box<dyn Error>>;
const LIMIT: u32 = 64 * 1024 * 1024;
fn hash_file(path: &Path) -> R<String> {
    let mut f = File::open(path)?; let mut h = Sha256::new(); let mut buf = [0; 65536];
    loop { let n = f.read(&mut buf)?; if n == 0 { break; } h.update(&buf[..n]); }
    Ok(format!("{:x}", h.finalize()))
}
fn run() -> R<()> {
    let a: Vec<_> = std::env::args_os().skip(1).collect();
    if a.len() != 8 { return Err("usage: creaturedata_patch_translation <package> <16-hex scope+instance> <payload-sha256> <row-ordinal> <x> <y> <z> <new-output.creaturedata>".into()); }
    let text = |i: usize| -> R<&str> { a[i].to_str().ok_or_else(|| "arguments must be UTF-8 except paths".into()) };
    let keystr = text(1)?;
    if keystr.len() != 16 || !keystr.bytes().all(|c| c.is_ascii_hexdigit()) { return Err("key must be exactly 16 hex digits".into()); }
    let expected = text(2)?;
    if expected.len() != 64 || !expected.bytes().all(|c| c.is_ascii_hexdigit()) { return Err("SHA-256 must be exactly 64 hex digits".into()); }
    let ordinal: usize = text(3)?.parse()?;
    let translation: [f32; 3] = [text(4)?.parse()?, text(5)?.parse()?, text(6)?.parse()?];
    if translation.iter().any(|v| !v.is_finite()) { return Err("translation must be finite".into()); }
    let package = fs::canonicalize(&a[0])?;
    let output = Path::new(&a[7]);
    if !output.extension().is_some_and(|x| x.eq_ignore_ascii_case("creaturedata")) { return Err("output must have .creaturedata extension".into()); }
    let target = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"))?;
    let parent = fs::canonicalize(output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")))?;
    if !parent.starts_with(&target) { return Err("output parent must resolve under workspace target".into()); }
    let destination = parent.join(output.file_name().ok_or("missing output filename")?);
    if destination == package { return Err("output must not be source".into()); }
    // The caller admits the selected package path is stable, including during the
    // separate opens inside NativePackageIndex; hashes cannot detect transient changes.
    let before = hash_file(&package)?;
    let mut f = File::open(&package)?; let mut header = [0u8; HEADER_SIZE]; f.read_exact(&mut header)?;
    if DbpfHeader::parse(&header)?.index_size > LIMIT { return Err("index exceeds 64 MiB bound".into()); }
    let index = NativePackageIndex::open(&package)?;
    let key = NativeResourceKey::from_scoped_instance(0x0f43029a, u64::from_str_radix(keystr, 16)?);
    let entry = index.index().find_unique(key)?.ok_or("resource key not found")?;
    if entry.stored_size > LIMIT || entry.decompressed_size > LIMIT { return Err("resource exceeds 64 MiB bound".into()); }
    let raw = index.read(entry)?;
    if raw.len() > LIMIT as usize || format!("{:x}", Sha256::digest(&raw)) != expected.to_ascii_lowercase() { return Err("payload hash mismatch or size exceeds bound".into()); }
    let parsed = creature_data::parse(&raw)?;
    let row = parsed.parts.get(ordinal).ok_or("row ordinal out of range")?;
    let patched = patch_rest_transform(&raw, ordinal, RestTransform { rotation: row.rotation, translation })?;
    let start = 8 + if parsed.version == 9 { 12 } else { 16 } + parsed.compressed_buffer.len() + ordinal * creature_data::PART_ROW_SIZE + 84;
    if patched.len() != raw.len() || patched[..start] != raw[..start] || patched[start+12..] != raw[start+12..] { return Err("unexpected non-translation change".into()); }
    if hash_file(&package)? != before { return Err("package changed during read".into()); }
    let mut out = OpenOptions::new().write(true).create_new(true).open(&destination)?;
    if let Err(error) = out.write_all(&patched).and_then(|_| out.flush()).and_then(|_| out.sync_all()) {
        return Err(format!("write or sync failed; partial output may remain at {}: {error}", destination.display()).into());
    }
    eprintln!("Warning: output is not a complete playable creation; baked GMDL/model is not regenerated. Package path stability was admitted; before/after hashes cannot detect transient replacement.");
    println!("wrote {} bytes to {} (source package SHA-256 {}; intended output buffer SHA-256 {:x})", patched.len(), destination.display(), before, Sha256::digest(&patched));
    Ok(())
}
fn main() { if let Err(e) = run() { eprintln!("error: {e}"); std::process::exit(1); } }

