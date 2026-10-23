//! Fresh canonical archive intake and independent locale byte-grammar digests.
use dbpf::native_index::NativePackageIndex;
use spore_core::locale_table::{self, LocaleTable};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

fn feed(state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *state = (*state ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
}
fn word(state: &mut u64, value: usize) -> Result<(), Box<dyn std::error::Error>> {
    feed(state, &u32::try_from(value)?.to_le_bytes());
    Ok(())
}
fn digest(table: &LocaleTable) -> Result<u64, Box<dyn std::error::Error>> {
    let mut state = 0xcbf29ce484222325;
    feed(&mut state, b"locale-table-grammar-v1\0");
    word(&mut state, table.entries.len())?;
    word(&mut state, table.stopped_at)?;
    for entry in &table.entries {
        feed(&mut state, &entry.id.to_le_bytes());
        word(&mut state, entry.source_text_range.start)?;
        word(&mut state, entry.source_text_range.len())?;
        word(&mut state, entry.text.len())?;
        for unit in &entry.text {
            feed(&mut state, &unit.to_le_bytes());
        }
    }
    Ok(state)
}
fn packages(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            packages(&path, paths)?;
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
    if args.len() != 2 {
        return Err("usage: locale_table_verify INSTALL NEW-DIGESTS.tsv".into());
    }
    let install = Path::new(&args[0]);
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let mut paths = Vec::new();
    packages(install, &mut paths)?;
    paths.sort();
    let (mut resources, mut entries, mut units) = (0, 0, 0);
    let mut lines = Vec::new();
    for path in &paths {
        let package = NativePackageIndex::open(path)?;
        for entry in package
            .entries()
            .iter()
            .filter(|entry| entry.key.type_id == locale_table::RESOURCE_TYPE)
        {
            if entry.key.group_id != locale_table::RESOURCE_SCOPE {
                return Err("unexpected source locale scope".into());
            }
            let bytes = package.read(entry)?;
            let table = locale_table::parse(&bytes)
                .map_err(|error| format!("{} {}: {error}", path.display(), entry.key))?;
            let sum = digest(&table)?;
            let relative = path
                .strip_prefix(install)?
                .to_string_lossy()
                .replace('\\', "/");
            lines.push(format!(
                "{relative}\t{:08X}\t{:08X}\t{sum:016X}",
                entry.key.instance_id, entry.key.group_id
            ));
            resources += 1;
            entries += table.entries.len();
            units += table
                .entries
                .iter()
                .map(|entry| entry.text.len())
                .sum::<usize>();
        }
    }
    lines.sort();
    let mut output = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?,
    );
    for line in lines {
        writeln!(output, "{line}")?;
    }
    output.flush()?;
    println!("LOCALE_TABLE_VERIFY packages={} resources={resources} stored_entries={entries} UTF16_units={units} scope=checked_byte_grammar original_host_executed=false", paths.len());
    Ok(())
}
