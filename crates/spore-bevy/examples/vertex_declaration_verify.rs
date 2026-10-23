//! Derive native source selector features from every installed GMDL declaration.
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
        return Err("usage: vertex_declaration_verify <install> <new TSV>".into());
    }
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    let mut output = std::io::BufWriter::new(file);
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut models, mut declarations, mut elements) = (0, 0, 0, 0);
    let mut semantic_counts = BTreeMap::<i32, usize>::new();
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x00E6_BCE5)
        {
            let data = index.read(entry)?;
            let model = spore_core::gmdl::parse(&data)?;
            models += 1;
            for (i, format) in model.vertex_formats.iter().enumerate() {
                let flags = format.native_shader_features()?;
                writeln!(
                    output,
                    "{}\t{}\t{i}\t{}\t{flags:08X}",
                    path.display(),
                    entry.key,
                    format.elements.len()
                )?;
                declarations += 1;
                elements += format.elements.len();
                for element in &format.elements {
                    *semantic_counts.entry(element.rw_decl).or_default() += 1;
                }
            }
        }
    }
    output.flush()?;
    if declarations == 0 {
        return Err("no source declarations".into());
    }
    println!("VERTEX_DECLARATION_VERIFY packages={packages} models={models} declarations={declarations} elements={elements} semantics={semantic_counts:?}");
    Ok(())
}
