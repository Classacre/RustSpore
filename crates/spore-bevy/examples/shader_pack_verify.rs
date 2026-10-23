//! Verify every installed header-7 pack without creating a renderer/window.
use spore_core::shader_pack;
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};
fn collect(path: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect(&entry.path(), paths)?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: shader_pack_verify <install> <new digest TSV>".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut packs, mut standard, mut builders, mut vertex, mut pixel) =
        (0, 0, 0, 0, 0, 0);
    let mut headers = BTreeMap::new();
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0469A3F7)
        {
            let data = index.read(entry)?;
            let header = u32::from_be_bytes(data.get(..4).ok_or("short SMT")?.try_into().unwrap());
            *headers.entry(header).or_insert(0usize) += 1;
            if header != 7 {
                continue;
            }
            let pack = shader_pack::parse(&data)
                .map_err(|e| format!("{} {:016X}: {e}", path.display(), entry.key.instance_id))?;
            let encoded = pack.to_bytes();
            if encoded != data {
                return Err("source pack roundtrip mismatch".into());
            }
            for table in [&pack.compiled_vertex, &pack.compiled_pixel] {
                for program in &table.programs {
                    let recipe = program.fragment_recipe()?;
                    let resolved = table
                        .lookup(recipe)?
                        .ok_or("missing original compiled permutation")?;
                    if !std::ptr::eq(program, resolved) {
                        return Err("wrong compiled permutation selected".into());
                    }
                    if program.uniform_dependency_mask() != program.flags_word {
                        return Err("uniform dependency mask mismatch".into());
                    }
                }
            }
            let hash = encoded.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
            writeln!(
                output,
                "{}\t{:016X}\t{}\t{}\t{}\t{}\t{hash:016X}",
                path.file_name().unwrap().to_string_lossy(),
                entry.key.instance_id,
                pack.standard.len(),
                pack.builders.len(),
                pack.compiled_vertex.programs.len(),
                pack.compiled_pixel.programs.len()
            )?;
            packs += 1;
            standard += pack.standard.len();
            builders += pack.builders.len();
            vertex += pack.compiled_vertex.programs.len();
            pixel += pack.compiled_pixel.programs.len();
        }
    }
    println!("SHADER_PACK_VERIFY packages={packages} packs={packs} standard={standard} builders={builders} vertex={vertex} pixel={pixel} headers={headers:?} exact=true");
    Ok(())
}
