//! Local, experimental command-line edits to an extracted source payload.
//! This is not the original editor and does not regenerate a playable model.
use dbpf::{
    native_index::{NativePackageIndex, NativeResourceEntry, NativeResourceKey},
    DbpfHeader, HEADER_SIZE,
};
use sha2::{Digest, Sha256};
use spore_core::creature_data_session::{CreatureDataSession, MAX_PAYLOAD_BYTES};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const HISTORY: usize = 128;
const MAX_INDEX_BYTES: u32 = 64 * 1024 * 1024;
const LIST_PAGE_SIZE: usize = 32;

fn scoped_key(text: &str) -> Result<NativeResourceKey, String> {
    if text.len() != 16 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("key must be exactly 16 hex digits (scope followed by instance)".into());
    }
    let value = u64::from_str_radix(text, 16).map_err(|e| e.to_string())?;
    Ok(NativeResourceKey::from_scoped_instance(0x0f43029a, value))
}

fn expected_sha(text: &str) -> Result<String, String> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("source SHA-256 must be exactly 64 hex digits".into());
    }
    Ok(text.to_ascii_lowercase())
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut input = File::open(path).map_err(|e| format!("package: {e}"))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = input
            .read(&mut buffer)
            .map_err(|e| format!("package hash: {e}"))?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[derive(Debug, PartialEq)]
enum Command {
    Empty,
    List(usize),
    Show(usize),
    Set(usize, [f32; 3]),
    Move(usize, [f32; 3]),
    Transform(usize, [[f32; 3]; 3], [f32; 3]),
    Undo,
    Redo,
    Status,
    Help,
    Save,
    SaveAs(String),
    Quit,
}

fn parse_command(line: &str) -> Result<Command, String> {
    if let Some(path) = line.trim().strip_prefix("save-as ") {
        let path = path.trim();
        return if path.is_empty() {
            Err("save-as requires a path".into())
        } else {
            Ok(Command::SaveAs(path.to_string()))
        };
    }
    let fields: Vec<_> = line.split_whitespace().collect();
    let ordinal = |text: &str| {
        text.parse::<usize>()
            .map_err(|_| "row must be a non-negative ordinal".to_string())
    };
    let numbers = |input: &[&str]| -> Result<Vec<f32>, String> {
        input
            .iter()
            .map(|word| {
                let value = word
                    .parse::<f32>()
                    .map_err(|_| format!("invalid number {word:?}"))?;
                if !value.is_finite() {
                    return Err("transform values must be finite".into());
                }
                Ok(value)
            })
            .collect()
    };
    match fields.as_slice() {
        [] => Ok(Command::Empty),
        ["list"] => Ok(Command::List(0)),
        ["list", start] => Ok(Command::List(ordinal(start)?)),
        ["show", index] => Ok(Command::Show(ordinal(index)?)),
        ["set", index, x, y, z] => {
            let v = numbers(&[x, y, z])?;
            Ok(Command::Set(ordinal(index)?, [v[0], v[1], v[2]]))
        }
        ["move", index, x, y, z] => {
            let v = numbers(&[x, y, z])?;
            Ok(Command::Move(ordinal(index)?, [v[0], v[1], v[2]]))
        }
        ["transform", index, rest @ ..] if rest.len() == 12 => {
            let v = numbers(rest)?;
            Ok(Command::Transform(
                ordinal(index)?,
                [[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]],
                [v[9], v[10], v[11]],
            ))
        }
        ["undo"] => Ok(Command::Undo),
        ["redo"] => Ok(Command::Redo),
        ["status"] => Ok(Command::Status),
        ["help"] => Ok(Command::Help),
        ["save"] => Ok(Command::Save),
        ["quit"] => Ok(Command::Quit),
        _ => Err("unknown command or wrong number of arguments; enter help".into()),
    }
}

// List serialized rows, not rig-index or baked-mesh ownership. Parsing the
// current bytes makes edits and undo/redo visible without a stale side cache.
fn list_page(session: &CreatureDataSession, start: usize) -> Result<String, String> {
    let parsed = spore_core::creature_data::parse(session.bytes()).map_err(|e| e.to_string())?;
    let count = parsed.parts.len();
    if start > count {
        return Err(format!("start row {start} is outside {count} source rows"));
    }
    let end = start.saturating_add(LIST_PAGE_SIZE).min(count);
    let mut text = format!("source rows {start}..{end} of {count} (end exclusive)");
    for (ordinal, part) in parsed.parts.iter().enumerate().take(end).skip(start) {
        text.push_str(&format!(
            "\nrow={ordinal} stored_index={} prop={:016X} parent={:?} symmetric={:?} translation={:?}",
            part.index, part.part_instance, part.parent_index(), part.symmetric_index(), part.translation
        ));
    }
    if end < count {
        text.push_str(&format!("\nnext: list {end}"));
    }
    Ok(text)
}

fn output_path(path: &Path) -> Result<PathBuf, String> {
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("creaturedata"))
    {
        return Err("output must have a .creaturedata extension".into());
    }
    let target = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .canonicalize()
        .map_err(|e| format!("workspace target: {e}"))?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| format!("output parent: {e}"))?;
    if !parent.starts_with(target) {
        return Err("output parent must resolve under workspace target".into());
    }
    Ok(parent.join(path.file_name().ok_or("output filename required")?))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<String, String> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("cannot create new output {}: {e}", path.display()))?;
    let result = (|| -> io::Result<Vec<u8>> {
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        let mut actual = Vec::with_capacity(bytes.len());
        (&mut file)
            .take(bytes.len() as u64 + 1)
            .read_to_end(&mut actual)?;
        Ok(actual)
    })();
    let actual = result.map_err(|e| format!("output may be partial at {}: {e}", path.display()))?;
    if actual != bytes {
        return Err(format!(
            "output readback differs; inspect {}",
            path.display()
        ));
    }
    Ok(format!("{:x}", Sha256::digest(&actual)))
}

// Read at most 4096 content bytes; discard the remainder of an oversized line.
fn read_command<R: BufRead>(input: &mut R) -> io::Result<Option<Result<String, String>>> {
    let mut bytes = Vec::new();
    let read = (&mut *input).take(4098).read_until(b'\n', &mut bytes)?;
    if read == 0 {
        return Ok(None);
    }
    let ended = bytes.last() == Some(&b'\n');
    if ended {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    let too_long = bytes.len() > 4096;
    if too_long && !ended {
        loop {
            let available = input.fill_buf()?;
            if available.is_empty() {
                break;
            }
            let newline = available.iter().position(|&byte| byte == b'\n');
            let consumed = newline.map_or(available.len(), |index| index + 1);
            input.consume(consumed);
            if newline.is_some() {
                break;
            }
        }
    }
    if too_long {
        return Ok(Some(Err("command exceeds 4096 bytes".into())));
    }
    Ok(Some(
        String::from_utf8(bytes).map_err(|_| "command must be UTF-8".into()),
    ))
}

fn commands<R: BufRead, W: Write>(
    session: &mut CreatureDataSession,
    output: &Path,
    mut input: R,
    out: &mut W,
) -> Result<(), String> {
    writeln!(
        out,
        "{} parts; type help for commands. Changes do not regenerate a playable model.",
        session.part_count()
    )
    .map_err(|e| e.to_string())?;
    out.flush().map_err(|e| e.to_string())?;
    while let Some(line) = read_command(&mut input).map_err(|e| format!("input: {e}"))? {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                writeln!(out, "error: {error}").map_err(|e| e.to_string())?;
                continue;
            }
        };
        let command = match parse_command(&line) {
            Ok(command) => command,
            Err(error) => {
                writeln!(out, "error: {error}").map_err(|e| e.to_string())?;
                continue;
            }
        };
        let response = match command {
            Command::Empty => continue,
            Command::Help => "list [start-row] | show <row> | set <row> <x> <y> <z> | move <row> <dx> <dy> <dz> | transform <row> <9 row-major matrix values> <x> <y> <z> | undo | redo | status | save | save-as <new path> | quit".to_string(),
            Command::Status => format!("revision={} undo={} redo={} parts={}", session.revision(), session.undo_len(), session.redo_len(), session.part_count()),
            Command::List(start) => match list_page(session, start) {
                Ok(page) => page,
                Err(error) => format!("error: {error}"),
            },
            Command::Show(row) => match session.transform(row) {
                Ok(value) => format!("row={row} rotation={:?} translation={:?}", value.rotation, value.translation),
                Err(error) => format!("error: {error}"),
            },
            Command::Set(row, translation) => match session.transform(row) {
                Ok(mut value) => { value.translation = translation; match session.set_transform(row, value) {
                    Ok(changed) => format!("changed={changed} revision={}", session.revision()), Err(error) => format!("error: {error}")
                }},
                Err(error) => format!("error: {error}"),
            },
            Command::Move(row, delta) => match session.transform(row) {
                Ok(mut value) => {
                    for (component, offset) in value.translation.iter_mut().zip(delta) {
                        *component += offset;
                    }
                    match session.set_transform(row, value) {
                        Ok(changed) => format!("changed={changed} revision={}", session.revision()),
                        Err(error) => format!("error: {error}"),
                    }
                },
                Err(error) => format!("error: {error}"),
            },
            Command::Transform(row, rotation, translation) => match session.set_transform(row, spore_core::creature_data_edit::RestTransform { rotation, translation }) {
                Ok(changed) => format!("changed={changed} revision={}", session.revision()), Err(error) => format!("error: {error}"),
            },
            Command::Undo => match session.undo() { Ok(changed) => format!("undo={changed} revision={}", session.revision()), Err(error) => format!("error: {error}") },
            Command::Redo => match session.redo() { Ok(changed) => format!("redo={changed} revision={}", session.revision()), Err(error) => format!("error: {error}") },
            Command::Save | Command::SaveAs(_) => {
                let destination = match command {
                    Command::Save => Ok(output.to_path_buf()),
                    Command::SaveAs(path) => output_path(Path::new(&path)),
                    _ => unreachable!(),
                };
                match destination.and_then(|path| write_new(&path, session.bytes()).map(|hash| (path, hash))) {
                    Ok((path, hash)) => {
                        writeln!(out, "saved {} bytes to {}; readback_sha256={hash}", session.bytes().len(), path.display()).map_err(|e| e.to_string())?;
                        return Ok(());
                    },
                    Err(error) => format!("error: {error}; session retained, use save-as <new path>"),
                }
            }
            Command::Quit => { writeln!(out, "quit without saving").map_err(|e| e.to_string())?; return Ok(()); }
        };
        writeln!(out, "{response}").map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
    }
    Err("end of input; no file saved".into())
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("usage: --experimental --source-creature-edit <source.creaturedata> <sha256> <new-target-output.creaturedata>".into());
    }
    let expected = expected_sha(&args[1])?;
    let output = output_path(Path::new(&args[2]))?;
    let file = File::open(&args[0]).map_err(|e| format!("source: {e}"))?;
    let mut source = Vec::new();
    file.take((MAX_PAYLOAD_BYTES + 1) as u64)
        .read_to_end(&mut source)
        .map_err(|e| format!("source: {e}"))?;
    if source.len() > MAX_PAYLOAD_BYTES || format!("{:x}", Sha256::digest(&source)) != expected {
        return Err("source payload exceeds limit or SHA-256 does not match".into());
    }
    let mut session = CreatureDataSession::new(source, HISTORY).map_err(|e| e.to_string())?;
    eprintln!("Experimental source-byte editing only; no baked-model regeneration or game integration. Keep the target directory stable while saving.");
    commands(
        &mut session,
        &output,
        io::stdin().lock(),
        &mut io::stdout().lock(),
    )
}

fn check_entry_sizes(entry: &NativeResourceEntry) -> Result<(), String> {
    if u64::from(entry.stored_size) > MAX_PAYLOAD_BYTES as u64
        || u64::from(entry.decompressed_size) > MAX_PAYLOAD_BYTES as u64
    {
        return Err("stored or decoded CreatureData exceeds 64 MiB bound".into());
    }
    Ok(())
}

fn load_package_session(
    package_path: &Path,
    key: NativeResourceKey,
    expected: &str,
) -> Result<(CreatureDataSession, String), String> {
    let before = hash_file(package_path)?;
    let mut file = File::open(package_path).map_err(|e| format!("package: {e}"))?;
    let mut header = [0u8; HEADER_SIZE];
    file.read_exact(&mut header)
        .map_err(|e| format!("package header: {e}"))?;
    if DbpfHeader::parse(&header)
        .map_err(|e| format!("package header: {e}"))?
        .index_size
        > MAX_INDEX_BYTES
    {
        return Err("package index exceeds 64 MiB bound".into());
    }
    let package =
        NativePackageIndex::open(package_path).map_err(|e| format!("package index: {e}"))?;
    let entry = package
        .index()
        .find_unique(key)
        .map_err(|e| format!("package key: {e}"))?
        .ok_or_else(|| format!("CreatureData key {key} not found"))?;
    check_entry_sizes(entry)?;
    let source = package
        .read(entry)
        .map_err(|e| format!("CreatureData read: {e}"))?;
    if source.len() > MAX_PAYLOAD_BYTES || format!("{:x}", Sha256::digest(&source)) != expected {
        return Err("decoded CreatureData exceeds limit or payload SHA-256 does not match".into());
    }
    let after = hash_file(package_path)?;
    if before != after {
        return Err("package endpoint hashes changed during read".into());
    }
    let session = CreatureDataSession::new(source, HISTORY).map_err(|e| e.to_string())?;
    Ok((session, before))
}

/// Start the same recoverable editing session from a uniquely indexed package
/// resource. Multiple file opens require a caller-admitted stable package path.
pub(crate) fn run_package(args: &[String]) -> Result<(), String> {
    if args.len() != 4 {
        return Err("usage: --experimental --source-creature-edit-package <package> <16-hex scope+instance> <payload-sha256> <new-target-output.creaturedata>".into());
    }
    let key = scoped_key(&args[1])?;
    let expected = expected_sha(&args[2])?;
    let output = output_path(Path::new(&args[3]))?;
    let package_path = Path::new(&args[0])
        .canonicalize()
        .map_err(|e| format!("package: {e}"))?;
    eprintln!("Keep the package file and path stable during all reads, and the target directory stable while saving; endpoint hashes cannot rule out transient replacement.");
    let (mut session, before) = load_package_session(&package_path, key, &expected)?;
    eprintln!("Package {} key {key} SHA-256 before/after {before}; experimental source-byte editing only, no baked-model regeneration or game integration.", package_path.display());
    commands(
        &mut session,
        &output,
        io::stdin().lock(),
        &mut io::stdout().lock(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_creature() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&spore_core::creature_data::HEADER.to_le_bytes());
        bytes.extend_from_slice(&9u32.to_le_bytes());
        for value in [0u32, 0, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0xfe, 127]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes
    }

    // A small canonical index with common type/scope/high and raw or RefPack
    // payload. All data is synthetic and no installed package is touched.
    fn package_fixture(payload: &[u8], compressed: bool, copies: u32, decoded: u32) -> Vec<u8> {
        let stored = if compressed {
            // 29-byte literal-only RefPack stream: 28-byte block and 1-byte tail.
            assert_eq!(payload.len(), 29);
            let mut v = vec![0x10, 0xfb, 0, 0, 29, 0xe6];
            v.extend_from_slice(&payload[..28]);
            v.push(0xfd);
            v.push(payload[28]);
            v
        } else {
            payload.to_vec()
        };
        let mut out = vec![0u8; HEADER_SIZE];
        out[..4].copy_from_slice(b"DBPF");
        out[4..8].copy_from_slice(&3u32.to_le_bytes());
        out[0x24..0x28].copy_from_slice(&copies.to_le_bytes());
        let index_offset = (HEADER_SIZE + stored.len()) as u32;
        let index_size = 16 + 16 * copies;
        out[0x2c..0x30].copy_from_slice(&index_size.to_le_bytes());
        out[0x40..0x44].copy_from_slice(&index_offset.to_le_bytes());
        out.extend_from_slice(&stored);
        for value in [7u32, 0x0f43029a, 0x01234567, 0] {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for _ in 0..copies {
            for value in [
                0x89abcdefu32,
                HEADER_SIZE as u32,
                stored.len() as u32,
                decoded,
            ] {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        out
    }

    struct TestPackage(PathBuf);
    impl TestPackage {
        fn new(bytes: &[u8]) -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            loop {
                let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "rustspore-package-intake-{}-{timestamp}-{sequence}",
                    std::process::id()
                ));
                match OpenOptions::new().write(true).create_new(true).open(&path) {
                    Ok(mut file) => {
                        file.write_all(bytes).unwrap();
                        return Self(path);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("could not create package fixture: {error}"),
                }
            }
        }
    }
    impl Drop for TestPackage {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn package_intake_reads_unique_raw_and_refpack_payloads() {
        let payload = minimal_creature();
        let expected = format!("{:x}", Sha256::digest(&payload));
        let key = scoped_key("0123456789abcdef").unwrap();
        for compressed in [false, true] {
            let fixture = TestPackage::new(&package_fixture(
                &payload,
                compressed,
                1,
                payload.len() as u32,
            ));
            let (session, _) = load_package_session(&fixture.0, key, &expected).unwrap();
            assert_eq!(session.bytes(), payload);
            assert_eq!(session.part_count(), 0);
        }
    }

    #[test]
    fn package_intake_rejects_duplicate_wrong_hash_and_truncated_extents() {
        let payload = minimal_creature();
        let expected = format!("{:x}", Sha256::digest(&payload));
        let key = scoped_key("0123456789abcdef").unwrap();
        let duplicate =
            TestPackage::new(&package_fixture(&payload, false, 2, payload.len() as u32));
        assert!(load_package_session(&duplicate.0, key, &expected)
            .unwrap_err()
            .contains("more than one"));
        let valid = TestPackage::new(&package_fixture(&payload, false, 1, payload.len() as u32));
        assert!(load_package_session(&valid.0, key, &"0".repeat(64))
            .unwrap_err()
            .contains("SHA-256"));
        let mut truncated = package_fixture(&payload, false, 1, payload.len() as u32);
        truncated.pop();
        let truncated = TestPackage::new(&truncated);
        assert!(load_package_session(&truncated.0, key, &expected)
            .unwrap_err()
            .contains("package index"));
        let mut bad_entry = package_fixture(&payload, false, 1, payload.len() as u32);
        let index = HEADER_SIZE + payload.len();
        bad_entry[index + 20..index + 24].copy_from_slice(&u32::MAX.to_le_bytes());
        let bad_entry = TestPackage::new(&bad_entry);
        assert!(load_package_session(&bad_entry.0, key, &expected)
            .unwrap_err()
            .contains("outside"));
    }

    #[test]
    fn package_intake_bounds_index_and_both_resource_sizes() {
        let payload = minimal_creature();
        let expected = format!("{:x}", Sha256::digest(&payload));
        let key = scoped_key("0123456789abcdef").unwrap();
        let mut oversized_index = package_fixture(&payload, false, 1, payload.len() as u32);
        oversized_index[0x2c..0x30].copy_from_slice(&(MAX_INDEX_BYTES + 1).to_le_bytes());
        let fixture = TestPackage::new(&oversized_index);
        assert!(load_package_session(&fixture.0, key, &expected)
            .unwrap_err()
            .contains("index exceeds"));
        let oversized_decoded = TestPackage::new(&package_fixture(
            &payload,
            false,
            1,
            MAX_PAYLOAD_BYTES as u32 + 1,
        ));
        assert!(load_package_session(&oversized_decoded.0, key, &expected)
            .unwrap_err()
            .contains("decoded CreatureData exceeds"));
        let entry = NativeResourceEntry {
            key,
            offset: 0,
            stored_size: MAX_PAYLOAD_BYTES as u32 + 1,
            decompressed_size: 1,
            compression_type: 0,
            disk_committed: false,
            metadata_word: None,
        };
        assert!(check_entry_sizes(&entry).is_err());
    }

    #[test]
    fn package_key_and_hash_admission_are_strict() {
        let key = scoped_key("0123456789abcdef").unwrap();
        assert_eq!(
            (key.group_id, key.instance_id, key.type_id),
            (0x01234567, 0x89abcdef, 0x0f43029a)
        );
        assert!(scoped_key("0x0123456789abcdef").is_err());
        assert!(scoped_key("0123456789abcdeg").is_err());
        assert_eq!(expected_sha(&"A".repeat(64)).unwrap(), "a".repeat(64));
        assert!(expected_sha(&"z".repeat(64)).is_err());
        assert!(expected_sha("ab").is_err());
    }

    #[test]
    fn commands_are_strict_and_reject_nonfinite_values() {
        assert_eq!(parse_command("list").unwrap(), Command::List(0));
        assert_eq!(parse_command("list 32").unwrap(), Command::List(32));
        assert!(parse_command("list -1").is_err());
        assert_eq!(
            parse_command("set 4 1 -2 3.5").unwrap(),
            Command::Set(4, [1., -2., 3.5])
        );
        assert_eq!(
            parse_command("move 4 1 -2 3.5").unwrap(),
            Command::Move(4, [1., -2., 3.5])
        );
        assert!(parse_command("set -1 1 2 3").is_err());
        assert!(parse_command("set 0 NaN 2 3").is_err());
        assert!(parse_command("move 0 inf 2 3").is_err());
        assert!(parse_command("undo extra").is_err());
        assert!(parse_command("transform 0 1 2").is_err());
        assert_eq!(parse_command("  quit \r\n").unwrap(), Command::Quit);
        assert_eq!(
            parse_command("save-as target/my file.creaturedata").unwrap(),
            Command::SaveAs("target/my file.creaturedata".into())
        );
    }

    #[test]
    fn listing_tracks_edits_and_pages_by_serialized_row() {
        use spore_core::creature_data::{HEADER, PART_ROW_SIZE};
        let mut payload = Vec::new();
        payload.extend_from_slice(&HEADER.to_le_bytes());
        payload.extend_from_slice(&9u32.to_le_bytes());
        for count in [34u32, 0, 0] {
            payload.extend_from_slice(&count.to_le_bytes());
        }
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&[0, 0xfe, 127]);
        for ordinal in 0..34u16 {
            let mut row = [0u8; PART_ROW_SIZE];
            row[..2].copy_from_slice(&(100 + ordinal).to_le_bytes());
            row[4..8].copy_from_slice(&[0xff; 4]);
            row[132..136].copy_from_slice(&0x1234_5678u32.to_le_bytes());
            row[136..140].copy_from_slice(&(0xab00_0000u32 + u32::from(ordinal)).to_le_bytes());
            payload.extend_from_slice(&row);
        }
        payload.extend_from_slice(&0u32.to_le_bytes());
        let mut session = CreatureDataSession::new(payload, 8).unwrap();
        let first = list_page(&session, 0).unwrap();
        assert!(first.contains("row=1 stored_index=101 prop=12345678AB000001"));
        assert!(first.contains("next: list 32"));
        assert!(!first.contains("row=32 "));
        let last = list_page(&session, 32).unwrap();
        assert!(last.contains("row=32 stored_index=132"));
        assert!(last.contains("row=33 stored_index=133"));
        assert!(!last.contains("next:"));
        assert!(list_page(&session, 35).is_err());

        let mut transcript = Vec::new();
        commands(
            &mut session,
            Path::new("unused.creaturedata"),
            io::Cursor::new("move 1 1 -2 3\nlist 1\nundo\nlist 1\nredo\nquit\n"),
            &mut transcript,
        )
        .unwrap();
        let transcript = String::from_utf8(transcript).unwrap();
        assert!(transcript.contains("translation=[1.0, -2.0, 3.0]"));
        assert!(transcript.contains("translation=[0.0, 0.0, 0.0]"));
        assert_eq!(session.transform(1).unwrap().translation, [1.0, -2.0, 3.0]);
    }

    #[test]
    fn relative_move_overflow_does_not_change_session() {
        use spore_core::creature_data::{HEADER, PART_ROW_SIZE};
        let mut payload = Vec::new();
        payload.extend_from_slice(&HEADER.to_le_bytes());
        payload.extend_from_slice(&9u32.to_le_bytes());
        for count in [1u32, 0, 0] {
            payload.extend_from_slice(&count.to_le_bytes());
        }
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&[0, 0xfe, 127]);
        let mut row = [0u8; PART_ROW_SIZE];
        row[4..8].copy_from_slice(&[0xff; 4]);
        row[84..88].copy_from_slice(&f32::MAX.to_le_bytes());
        payload.extend_from_slice(&row);
        payload.extend_from_slice(&0u32.to_le_bytes());
        let original = payload.clone();
        let mut session = CreatureDataSession::new(payload, 8).unwrap();
        let mut transcript = Vec::new();
        commands(
            &mut session,
            Path::new("unused.creaturedata"),
            io::Cursor::new("move 0 3.4028235e38 0 0\nquit\n"),
            &mut transcript,
        )
        .unwrap();
        assert!(String::from_utf8(transcript).unwrap().contains("nonfinite"));
        assert_eq!(session.bytes(), original);
        assert_eq!((session.revision(), session.undo_len()), (0, 0));
    }

    #[test]
    fn oversized_and_invalid_utf8_lines_are_drained_before_next_command() {
        let mut bytes = vec![b'x'; 5000];
        bytes.extend_from_slice(b"\n\xff\nquit\r\n");
        let mut input = io::Cursor::new(bytes);
        assert!(read_command(&mut input).unwrap().unwrap().is_err());
        assert!(read_command(&mut input).unwrap().unwrap().is_err());
        assert_eq!(read_command(&mut input).unwrap().unwrap().unwrap(), "quit");
        assert!(read_command(&mut input).unwrap().is_none());
        let mut exact = io::Cursor::new([vec![b'a'; 4096], b"\r\n".to_vec()].concat());
        assert_eq!(
            read_command(&mut exact).unwrap().unwrap().unwrap().len(),
            4096
        );
    }

    #[test]
    fn save_does_not_replace_existing_files() {
        let path = std::env::temp_dir().join(format!(
            "rustspore-edit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, b"existing").unwrap();
        assert!(write_new(&path, b"different").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"existing");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn failed_save_retains_edits_and_allows_a_new_destination() {
        use spore_core::creature_data::{HEADER, PART_ROW_SIZE};
        let mut payload = Vec::new();
        payload.extend_from_slice(&HEADER.to_le_bytes());
        payload.extend_from_slice(&9u32.to_le_bytes());
        for count in [1u32, 0, 0] {
            payload.extend_from_slice(&count.to_le_bytes());
        }
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&[0, 0xfe, 127]);
        let mut row = [0u8; PART_ROW_SIZE];
        row[4..8].copy_from_slice(&[0xff; 4]);
        payload.extend_from_slice(&row);
        payload.extend_from_slice(&0u32.to_le_bytes());
        let mut session = CreatureDataSession::new(payload, 8).unwrap();
        let target = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .canonicalize()
            .unwrap();
        let unique = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let existing = target.join(format!("editor-test-{unique}-existing.creaturedata"));
        let destination = target.join(format!("editor-test-{unique} new.creaturedata"));
        std::fs::write(&existing, b"existing").unwrap();
        let input = format!(
            "set 0 3 4 5\nsave\nundo\nredo\nsave-as {}\n",
            destination.display()
        );
        let mut transcript = Vec::new();
        commands(
            &mut session,
            &existing,
            io::Cursor::new(input),
            &mut transcript,
        )
        .unwrap();
        assert!(String::from_utf8(transcript)
            .unwrap()
            .contains("session retained"));
        assert_eq!(std::fs::read(&existing).unwrap(), b"existing");
        assert_eq!(std::fs::read(&destination).unwrap(), session.bytes());
        assert_eq!(session.transform(0).unwrap().translation, [3., 4., 5.]);
        std::fs::remove_file(existing).unwrap();
        std::fs::remove_file(destination).unwrap();
    }
}
