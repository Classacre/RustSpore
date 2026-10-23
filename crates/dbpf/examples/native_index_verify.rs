//! Canonical native archive-index records, with optional complete payload reads.
use dbpf::native_index::NativePackageIndex;
use std::{
    collections::HashSet,
    io::Write,
    path::{Path, PathBuf},
};

fn collect(folder: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, paths)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("package"))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let payloads = args.get(2).is_some_and(|arg| arg == "--verify-payloads");
    if args.len() != 2 && !(args.len() == 3 && payloads) {
        return Err(
            "usage: native_index_verify <install> <new-records.tsv> [--verify-payloads]".into(),
        );
    }
    let root = Path::new(&args[0]);
    let mut paths = Vec::new();
    collect(root, &mut paths)?;
    paths.sort_by_cached_key(|path| {
        path.strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase()
    });
    let mut output = std::io::BufWriter::new(std::fs::File::create_new(&args[1])?);
    let (mut records, mut coded, mut decoded_bytes) = (0u64, 0u64, 0u64);
    for (package_number, path) in paths.iter().enumerate() {
        let archive = NativePackageIndex::open(path)?;
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        writeln!(
            output,
            "P\t{relative}\t{:08X}\t{}\t{}",
            archive.index().shared_flags,
            archive.entries().len(),
            archive.header().index_size
        )?;
        let mut keys = HashSet::new();
        for (ordinal, entry) in archive.entries().iter().enumerate() {
            if !keys.insert(entry.key) {
                return Err(format!("duplicate native key{} in{relative}", entry.key).into());
            }
            let meta = entry
                .metadata_word
                .map(|word| format!("{word:08X}"))
                .unwrap_or_else(|| "-".into());
            writeln!(
                output,
                "R\t{relative}\t{ordinal}\t{:08X}\t{:08X}\t{:08X}\t{}\t{}\t{}\t{:04X}\t{}\t{meta}",
                entry.key.instance_id,
                entry.key.type_id,
                entry.key.group_id,
                entry.offset,
                entry.stored_size,
                entry.decompressed_size,
                entry.compression_type,
                u8::from(entry.disk_committed)
            )?;
            records += 1;
            coded += u64::from(entry.compression_type == 0xffff);
            if payloads {
                let bytes = archive.read(entry)?;
                if bytes.len() != entry.decompressed_size as usize {
                    return Err(format!("payload size mismatch{} in{relative}", entry.key).into());
                }
                decoded_bytes += bytes.len() as u64;
            }
        }
        if payloads && (package_number + 1) % 16 == 0 {
            println!(
                "verified_packages={} records={records} decoded_bytes={decoded_bytes}",
                package_number + 1
            );
        }
    }
    output.flush()?;
    println!("NATIVE_INDEX_VERIFY packages={} records={records} coded={coded} payloads={payloads} decoded_bytes={decoded_bytes}", paths.len());
    Ok(())
}
