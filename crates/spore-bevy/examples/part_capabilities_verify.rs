//! Verify all PartCapabilities dictionaries through the Rust package reader.
//! tools\build.bat run -p spore-bevy --example part_capabilities_verify -- [--digest FILE] <package-or-directory>...
use spore_core::part_capabilities::{self, PartCapabilities};
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

fn digest(p: &PartCapabilities) -> u64 {
    let mut d = Digest(0xcbf29ce484222325);
    d.bytes(b"pctp");
    d.word(p.version);
    if let Some(bits) = p.priority_bits {
        d.word(bits);
    }
    d.word(p.names.len() as u32);
    for name in &p.names {
        d.word(name.name_bytes.len() as u32);
        d.bytes(&name.name_bytes);
        d.word(name.capability);
    }
    d.word(p.remaps.len() as u32);
    for r in &p.remaps {
        d.word(r.key);
        d.word(r.capability);
        d.word(r.index);
    }
    d.word(p.aggregates.len() as u32);
    for a in &p.aggregates {
        d.word(a.capability);
        d.word(a.members.len() as u32);
        for value in &a.members {
            d.word(*value);
        }
    }
    d.word(p.deforms.len() as u32);
    for a in &p.deforms {
        d.word(a.capability);
        d.word(a.channels.len() as u32);
        for channel in &a.channels {
            d.word(channel.id);
            for value in channel.raw_parameters {
                d.word(value);
            }
        }
    }
    d.0
}

fn main() {
    let mut input = std::env::args_os().skip(1);
    let mut paths = Vec::new();
    let mut digest_path = None;
    while let Some(arg) = input.next() {
        if arg == "--digest" {
            digest_path = input.next().map(PathBuf::from);
            if digest_path.is_none() {
                eprintln!("--digest requires a path");
                std::process::exit(2);
            }
        } else {
            paths.push(PathBuf::from(arg));
        }
    }
    if paths.is_empty() {
        eprintln!("usage: part_capabilities_verify [--digest FILE] <package-or-directory>...");
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
    let mut rows = Vec::new();
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
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x7C19AA7A)
        {
            total += 1;
            let result = index
                .read(entry)
                .map_err(|err| err.to_string())
                .and_then(|bytes| part_capabilities::parse(&bytes).map_err(|err| err.to_string()));
            match result {
                Ok(p) => {
                    parsed += 1;
                    let hash = digest(&p);
                    let channels: usize = p.deforms.iter().map(|d| d.channels.len()).sum();
                    println!("{} {}: version={} names={} remaps={} aggregates={} deforms={} channels={} digest={hash:016X}",path.display(),entry.key,p.version,p.names.len(),p.remaps.len(),p.aggregates.len(),p.deforms.len(),channels);
                    rows.push(format!(
                        "{} {} {hash:016X}",
                        path.file_name().unwrap().to_string_lossy(),
                        entry.key
                    ));
                }
                Err(err) => {
                    eprintln!("{} {}: {err}", path.display(), entry.key);
                    problems += 1;
                }
            }
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
    println!(
        "packages={opened}/{} dictionaries={parsed}/{total} problems={problems}",
        packages.len()
    );
    if total == 0 || problems > 0 {
        std::process::exit(1);
    }
}
