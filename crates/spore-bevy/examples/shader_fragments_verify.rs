//! Read every SMT fragment library from installed indexes without a renderer.
use spore_core::shader_fragments;
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
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: shader_fragments_verify <install> <new digest TSV>".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut smt, mut libraries, mut named_vertex, mut named_pixel) = (0, 0, 0, 0, 0);
    let mut headers = BTreeMap::<u32, usize>::new();
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0469A3F7)
        {
            smt += 1;
            let data = index.read(entry)?;
            let header = u32::from_be_bytes(data.get(..4).ok_or("short SMT")?.try_into().unwrap());
            *headers.entry(header).or_default() += 1;
            if header != 1 {
                continue;
            }
            let lib = shader_fragments::parse(&data)
                .map_err(|e| format!("{} {:016X}: {e}", path.display(), entry.key.instance_id))?;
            let encoded = lib.to_bytes();
            if encoded != data {
                return Err("source shader roundtrip mismatch".into());
            }
            let vertex = lib.vertex.iter().filter(|f| !f.name.is_empty()).count();
            let pixel = lib.pixel.iter().filter(|f| !f.name.is_empty()).count();
            let uniforms = lib
                .vertex
                .iter()
                .chain(&lib.pixel)
                .map(|f| f.uniforms.len())
                .sum::<usize>();
            let hash = encoded.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
            writeln!(
                output,
                "{}\t{:016X}\t{}\t{}\t{}\t{hash:016X}",
                path.file_name().unwrap().to_string_lossy(),
                entry.key.instance_id,
                vertex,
                pixel,
                uniforms
            )?;
            libraries += 1;
            named_vertex += vertex;
            named_pixel += pixel;
        }
    }
    println!("SHADER_FRAGMENT_VERIFY packages={packages} smt={smt} fragment_libraries={libraries} named_vertex={named_vertex} named_pixel={named_pixel} headers={headers:?} exact=true");
    Ok(())
}
