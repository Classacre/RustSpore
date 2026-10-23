//! Fresh canonical source topology intake; no renderer or native fixture call.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    anim_body_topology::source_topology_inputs,
    creature_data,
    native_x87::{Context, Precision, Rounding},
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
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: anim_body_topology_source_verify install new-digest.txt".into());
    }
    let install = Path::new(&args[0]);
    let mut packages = Vec::new();
    collect(install, &mut packages)?;
    packages.sort();
    packages.dedup();
    let mut dictionary: Option<PartCapabilities> = None;
    let mut dictionaries = 0;
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
                    return Err("source aggregate tables differ; select a dictionary".into());
                }
            } else {
                dictionary = Some(value);
            }
            dictionaries += 1;
        }
    }
    let dictionary = dictionary.ok_or("no source aggregate dictionary")?;
    let context = Context {
        precision: Precision::Bits53,
        rounding: Rounding::NearestEven,
    };
    let (mut resources, mut rows, mut optional, mut missing_spin) =
        (0usize, 0usize, 0usize, 0usize);
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
                return Err("source CreatureData unconsumed bytes".into());
            }
            let input = source_topology_inputs(&source, &dictionary, context)?;
            let mut digest = Digest(0xCBF29CE484222325);
            digest.bytes(b"native-body-topology-input-v1\0");
            digest.word(input.blocks.len() as u32);
            digest.word(input.root_nomination as u32);
            digest.word(u32::from(input.optional_foot_construction_required));
            for map in [&input.source_to_initial, &input.initial_to_source] {
                for &ordinal in map {
                    digest.word(ordinal as u32);
                }
            }
            digest.bytes(&source.data0);
            for block in &input.blocks {
                digest.word(block.parent.map_or(u32::MAX, |p| p as u32));
                digest.word(block.presence_mask);
                digest.word(block.nonzero_mask);
                for component in block.center {
                    digest.word(component.to_bits());
                }
                digest.word(block.source_palette_id);
                digest.word(block.symmetric.map_or(u32::MAX, |s| s as u32));
            }
            rows += input.blocks.len();
            resources += 1;
            optional += usize::from(input.optional_foot_construction_required);
            missing_spin += usize::from(input.blocks[0].nonzero_mask & 8 == 0);
            let key = entry.key;
            let relative = package
                .strip_prefix(install)?
                .to_string_lossy()
                .replace('\\', "/");
            lines.push(format!(
                "{relative} {:08X}:{:08X}:{:08X} {:016X}",
                key.type_id, key.group_id, key.instance_id, digest.0
            ));
        }
    }
    lines.sort();
    let mut out = std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    for line in lines {
        writeln!(out, "{line}")?;
    }
    out.flush()?;
    println!("indexes={} resources={resources} rows={rows} dictionaries={dictionaries} optional_foot_driver={optional} initial_root_missing_nonzero_spine={missing_spin}",packages.len());
    Ok(())
}
