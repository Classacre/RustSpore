//! Verify all shipped creature per-part ranges through the Rust package reader.
//! tools\build.bat run -p spore-bevy --example creature_rig_verify -- --digest FILE <install-directory>
use spore_core::creature_data::{self, CreatureData};
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect(path: &Path, packages: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                collect(&entry.path(), packages)?;
            }
        }
    } else if path.is_file() {
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("package"))
        {
            packages.push(path.to_path_buf());
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}

struct Digest(u64);
impl Digest {
    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
    fn word(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
}

/// Canonical typed mapping digest, independently implemented by the Python probe.
fn digest(c: &CreatureData) -> Result<u64, String> {
    c.validate_part_ranges().map_err(|e| e.to_string())?;
    let mut d = Digest(0xcbf29ce484222325);
    d.bytes(b"creature-rig-v1\0");
    for value in [
        c.version as u32,
        c.parts.len() as u32,
        c.capabilities.len() as u32,
        c.morphs.len() as u32,
    ] {
        d.word(value);
    }
    for index in 0..c.parts.len() {
        let view = c
            .part_view(index)
            .ok_or_else(|| format!("part {index} has no checked view"))?;
        let p = view.part;
        for value in [
            u32::from(p.index),
            u32::from(p.link),
            u32::from(p.flags_a),
            u32::from(p.flags_b),
            u32::from(p.group),
            u32::from(p.kind),
            u32::from(p.capability_start),
            u32::from(p.morph_start),
            u32::from(p.capability_count),
            u32::from(p.morph_count),
            u32::from(p.range_reserved),
            (p.part_instance >> 32) as u32,
            p.part_instance as u32,
        ] {
            d.word(value);
        }
        for value in p
            .bounds_min
            .iter()
            .chain(&p.bounds_max)
            .chain(p.rotation.iter().flatten())
            .chain(&p.translation)
            .chain(&p.reserved_floats)
        {
            d.word(value.to_bits());
        }
        d.word(view.capabilities.len() as u32);
        for cap in view.capabilities {
            d.word(cap.packed_id());
            d.word(u32::from(cap.number));
        }
        d.word(view.morphs.len() as u32);
        for morph in view.morphs {
            d.word(morph.id);
            d.word(morph.value1.to_bits());
            d.word(morph.value2.to_bits());
        }
    }
    Ok(d.0)
}

fn header_digest(c: &CreatureData) -> Result<u64, String> {
    if c.uncompressed_buffer.len() != creature_data::HEADER_BUFFER_SIZE {
        return Err("native CreatureData header must retain 128 decoded bytes".into());
    }
    let mut d = Digest(0xcbf29ce484222325);
    d.bytes(b"creature-header-v1\0");
    d.word(c.version as u32);
    d.word(c.compressed_buffer.len() as u32);
    d.bytes(&c.compressed_buffer);
    d.word(c.uncompressed_buffer.len() as u32);
    d.bytes(&c.uncompressed_buffer);
    let count_bytes = if c.version == 9 { 12 } else { 16 };
    d.word((8 + count_bytes + c.compressed_buffer.len()) as u32);
    d.word(c.consumed as u32);
    Ok(d.0)
}

fn main() {
    let mut input = std::env::args_os().skip(1);
    let mut paths = Vec::new();
    let mut digest_path = None;
    let mut header_digest_path = None;
    let mut candidate = "0F43029A:0001FFFF:40626200067C5390".to_owned();
    while let Some(arg) = input.next() {
        if arg == "--digest" {
            digest_path = input.next().map(PathBuf::from);
            if digest_path.is_none() {
                eprintln!("--digest requires a path");
                std::process::exit(2);
            }
        } else if arg == "--header-digest" {
            header_digest_path = input.next().map(PathBuf::from);
            if header_digest_path.is_none() {
                eprintln!("--header-digest requires a path");
                std::process::exit(2);
            }
        } else if arg == "--candidate" {
            candidate = input
                .next()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if candidate.is_empty() {
                eprintln!("--candidate requires a key");
                std::process::exit(2);
            }
        } else {
            paths.push(PathBuf::from(arg));
        }
    }
    if paths.is_empty() {
        eprintln!("usage: creature_rig_verify [--digest FILE] [--header-digest FILE] [--candidate KEY] <package-or-directory>...");
        std::process::exit(2);
    }
    let mut packages = Vec::new();
    for path in paths {
        if let Err(err) = collect(&path, &mut packages) {
            eprintln!("{}: {err}", path.display());
            std::process::exit(1);
        }
    }
    packages.sort();
    packages.dedup();
    let (mut opened, mut total, mut parsed, mut problems) = (0, 0, 0, 0);
    let (mut parts, mut capabilities, mut morphs) = (0, 0, 0);
    let mut rows = Vec::new();
    let mut header_rows = Vec::new();
    let (mut header_bytes, mut nonzero_header_tail_bytes) = (0, 0);
    for path in &packages {
        let index = match dbpf::PackageIndex::open(path) {
            Ok(index) => index,
            Err(err) => {
                eprintln!("{}: {err}", path.display());
                problems += 1;
                continue;
            }
        };
        opened += 1;
        let mut local = 0;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0F43029A)
        {
            total += 1;
            let result = index
                .read(entry)
                .map_err(|err| err.to_string())
                .and_then(|bytes| creature_data::parse(&bytes).map_err(|err| err.to_string()));
            match result {
                Ok(c) => {
                    let hash = match digest(&c) {
                        Ok(hash) => hash,
                        Err(err) => {
                            eprintln!("{} {}: {err}", path.display(), entry.key);
                            problems += 1;
                            continue;
                        }
                    };
                    if !c.exact {
                        eprintln!("{} {}: trailing bytes", path.display(), entry.key);
                        problems += 1;
                    }
                    match header_digest(&c) {
                        Ok(hash) => header_rows.push(format!(
                            "{} {} {hash:016X}",
                            path.file_name().unwrap().to_string_lossy(),
                            entry.key,
                        )),
                        Err(error) => {
                            eprintln!("{} {}: {error}", path.display(), entry.key);
                            problems += 1;
                            continue;
                        }
                    }
                    header_bytes += c.uncompressed_buffer.len();
                    nonzero_header_tail_bytes += c.uncompressed_buffer[68..]
                        .iter()
                        .filter(|byte| **byte != 0)
                        .count();
                    parsed += 1;
                    local += 1;
                    parts += c.parts.len();
                    capabilities += c.capabilities.len();
                    morphs += c.morphs.len();
                    rows.push(format!(
                        "{} {} {hash:016X}",
                        path.file_name().unwrap().to_string_lossy(),
                        entry.key
                    ));
                    if entry.key.to_string().eq_ignore_ascii_case(&candidate) {
                        println!(
                            "CANDIDATE {} {} parts={} digest={hash:016X}",
                            path.display(),
                            entry.key,
                            c.parts.len()
                        );
                        for (i, p) in c.parts.iter().enumerate() {
                            let view = c.part_view(i).unwrap();
                            let caps: Vec<_> = view
                                .capabilities
                                .iter()
                                .map(|cap| format!("{:08X}:{}", cap.packed_id(), cap.number))
                                .collect();
                            let morphs: Vec<_> = view
                                .morphs
                                .iter()
                                .map(|m| format!("{:08X}", m.id))
                                .collect();
                            println!("part={i} instance={:016X} parent={:?} symmetric={:?} capabilities={caps:?} morphs={morphs:?} translation={:?}",p.part_instance,p.parent_index(),p.symmetric_index(),p.translation);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{} {}: {err}", path.display(), entry.key);
                    problems += 1;
                }
            }
        }
        if local > 0 {
            println!("{} creatures={local}", path.display());
        }
    }
    if let Some(path) = digest_path {
        rows.sort();
        let result = std::fs::File::create(&path).and_then(|mut file| {
            for row in rows {
                writeln!(file, "{row}")?;
            }
            Ok(())
        });
        if let Err(err) = result {
            eprintln!("{}: {err}", path.display());
            problems += 1;
        }
    }
    if let Some(path) = header_digest_path {
        header_rows.sort();
        let result = std::fs::File::create(&path).and_then(|mut file| {
            for row in header_rows {
                writeln!(file, "{row}")?;
            }
            Ok(())
        });
        if let Err(error) = result {
            eprintln!("{}: {error}", path.display());
            problems += 1;
        }
    }
    println!("native_header_bytes={header_bytes} nonzero_previously_discarded_tail_bytes={nonzero_header_tail_bytes}");
    println!("packages={opened}/{} creatures={parsed}/{total} parts={parts} capabilities={capabilities} morphs={morphs} problems={problems}", packages.len());
    if total == 0 || problems > 0 {
        std::process::exit(1);
    }
}
