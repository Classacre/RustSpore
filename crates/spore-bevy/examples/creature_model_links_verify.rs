//! Audit direct authored model references at actual source creature instances.
//! tools\build.bat run -p spore-bevy --example creature_model_links_verify -- INSTALL DIGEST
use spore_core::prop::{self, idents, PropTree, PropValue};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect(path: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect(&path, output)?;
        } else if path
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("package"))
        {
            output.push(path);
        }
    }
    Ok(())
}

fn selected(id: u32) -> bool {
    [
        idents::PARENT,
        idents::MODEL_LOD0,
        idents::MODEL_LOD1,
        idents::MODEL_LOD2,
        idents::MODEL_LOD3,
    ]
    .contains(&id)
}

struct Digest(u64);
impl Digest {
    fn bytes(&mut self, values: &[u8]) {
        for byte in values {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
    fn word(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
}

fn digest(data: &[u8], tree: &PropTree) -> Result<(u64, usize), String> {
    let declared = data.get(..4).ok_or("PROP has no declared item count")?;
    let declared = u32::from_be_bytes(declared.try_into().unwrap());
    if !tree.exact
        || tree.consumed != data.len()
        || tree.items.len() != declared as usize
        || tree.items.iter().any(|item| item.anonymous)
    {
        return Err("PROP is not the complete declared named list".into());
    }
    let chosen: Vec<_> = tree.items.iter().filter(|item| selected(item.id)).collect();
    let mut d = Digest(0xCBF29CE484222325);
    d.bytes(b"creature-model-links-v1\0");
    d.word(data.len() as u32);
    d.word(declared);
    d.word(chosen.len() as u32);
    let mut models = 0usize;
    for item in chosen {
        let keys: Vec<_> = item
            .values
            .iter()
            .filter_map(|value| {
                if let PropValue::Key(low, kind, high) = value {
                    Some((*low, *kind, *high))
                } else {
                    None
                }
            })
            .collect();
        for word in [
            item.id,
            u32::from(item.prop_type),
            u32::from(item.specifier),
            keys.len() as u32,
        ] {
            d.word(word);
        }
        if item.id != idents::PARENT {
            models += keys.len();
        }
        for (low, kind, high) in keys {
            d.word(low);
            d.word(kind);
            d.word(high);
        }
    }
    Ok((d.0, models))
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: creature_model_links_verify <install-directory> <digest-out.txt>".into(),
        );
    }
    let install = Path::new(&args[0]);
    let mut paths = Vec::new();
    collect(install, &mut paths).map_err(|e| e.to_string())?;
    paths.sort();
    let mut packages = Vec::new();
    let mut wanted = BTreeSet::new();
    for path in &paths {
        let package =
            dbpf::PackageIndex::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        for entry in package
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0F43_029A)
        {
            wanted.insert(entry.key.instance_id);
        }
        packages.push(package);
    }
    let mut rows = Vec::new();
    let mut models = 0usize;
    let mut linked = BTreeSet::new();
    for (path, package) in paths.iter().zip(&packages) {
        let relative = path
            .strip_prefix(install)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        for entry in package
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x00B1_B104 && wanted.contains(&e.key.instance_id))
        {
            let data = package
                .read(entry)
                .map_err(|e| format!("{}: {e}", entry.key))?;
            let tree = prop::parse(&data).map_err(|e| format!("{}: {e}", entry.key))?;
            let (value, count) = digest(&data, &tree).map_err(|e| format!("{}: {e}", entry.key))?;
            rows.push(format!("{relative}\t{}\t{value:016X}", entry.key));
            models += count;
            linked.insert(entry.key.instance_id);
        }
    }
    rows.sort();
    let mut output =
        std::io::BufWriter::new(std::fs::File::create(&args[1]).map_err(|e| e.to_string())?);
    for row in &rows {
        writeln!(output, "{row}").map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())?;
    println!("indexes_opened={} source_instances={} exact_model_prop_copies={} directly_linked_source_instances={} model_references={models}",
             packages.len(), wanted.len(), rows.len(), linked.len());
    if rows.is_empty() {
        return Err("no direct model PROP records verified".into());
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
