//! Fresh native-index CreatureData capability intake and aggregate identity.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    anim_body_capabilities::{prepare_source_capabilities, CapabilitySlots},
    creature_data,
    part_capabilities::{self, PartCapabilities},
};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                collect(&entry.path(), out)?;
            }
        }
    } else if path.is_file()
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
    {
        out.push(path.to_path_buf());
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
    fn word(&mut self, word: u32) {
        self.bytes(&word.to_le_bytes());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: anim_body_capabilities_source_verify <install-directory> <new-digest.txt>"
                .into(),
        );
    }
    let install = Path::new(&args[0]);
    let mut packages = Vec::new();
    collect(install, &mut packages)?;
    packages.sort();
    packages.dedup();
    let mut dictionary: Option<PartCapabilities> = None;
    let mut dictionary_resources = 0;
    for package in &packages {
        let index = NativePackageIndex::open(package)?;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x7C19AA7A)
        {
            let value = part_capabilities::parse(&index.read(entry)?)?;
            if let Some(first) = &dictionary {
                if first.aggregates != value.aggregates {
                    return Err("installed aggregate tables differ; select an explicit source dictionary context".into());
                }
            } else {
                dictionary = Some(value);
            }
            dictionary_resources += 1;
        }
    }
    let dictionary = dictionary.ok_or("no source aggregate dictionary found")?;
    let (mut resources, mut rows) = (0usize, 0usize);
    let mut lines = Vec::new();
    for package in &packages {
        let index = NativePackageIndex::open(package)?;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0F43029A)
        {
            let source = creature_data::parse(&index.read(entry)?)?;
            if !source.exact {
                return Err("source CreatureData has unconsumed bytes".into());
            }
            let mut digest = Digest(0xCBF29CE484222325);
            digest.bytes(b"native-body-capabilities-v1\0");
            digest.word(source.parts.len() as u32);
            for part in 0..source.parts.len() {
                let view = source.part_view(part).ok_or("source part view missing")?;
                let mut initial = CapabilitySlots::default();
                for (i, capability) in view.capabilities.iter().enumerate() {
                    if i >= 32 {
                        return Err("source capability array exceeds32slots".into());
                    }
                    initial.ids[i] = capability.packed_id();
                    initial.numbers[i] = capability.number;
                }
                let prepared = prepare_source_capabilities(&source, part, &dictionary)?;
                digest.word(part as u32);
                digest.bytes(view.raw_row);
                for id in initial.ids {
                    digest.word(id);
                }
                digest.bytes(&initial.numbers);
                digest.word(prepared.presence_mask());
                digest.word(prepared.nonzero_mask());
                for id in prepared.slots().ids {
                    digest.word(id);
                }
                digest.bytes(&prepared.slots().numbers);
                rows += 1;
            }
            let key = entry.key;
            let relative = package
                .strip_prefix(install)?
                .to_string_lossy()
                .replace('\\', "/");
            lines.push(format!(
                "{relative} {:08X}:{:08X}:{:08X} {:016X}",
                key.type_id, key.group_id, key.instance_id, digest.0
            ));
            resources += 1;
        }
    }
    if resources == 0 {
        return Err("no source CreatureData resources found".into());
    }
    lines.sort();
    let mut out = std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for line in &lines {
        writeln!(out, "{line}")?;
    }
    out.flush()?;
    println!("indexes_opened={} source_resources={resources} source_rows={rows} dictionary_resources={dictionary_resources} aggregate_tables_identical=true", packages.len());
    Ok(())
}
