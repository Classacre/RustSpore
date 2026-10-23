//! Fresh CreatureData intake comparison with the original-instruction oracle.
//! tools\build.bat run -p spore-bevy --example anim_body_source_verify -- INSTALL DIGEST
use spore_core::anim_body_source::source_rest_blocks;
use spore_core::creature_data;
use spore_core::native_x87::Context;
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: anim_body_source_verify <install-directory> <digest-out.txt>".into());
    }
    let mut packages = Vec::new();
    collect(Path::new(&args[0]), &mut packages).map_err(|e| e.to_string())?;
    packages.sort();
    packages.dedup();
    let mut lines = Vec::new();
    let (mut indexes, mut resources, mut rows, mut evaluated) = (0usize, 0usize, 0usize, 0usize);
    for package in packages {
        let index = dbpf::PackageIndex::open(&package).map_err(|e| e.to_string())?;
        indexes += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0F43029A)
        {
            let bytes = index.read(entry).map_err(|e| e.to_string())?;
            let source = creature_data::parse(&bytes).map_err(|e| e.to_string())?;
            if !source.exact {
                return Err(format!(
                    "{} {} has unconsumed bytes",
                    package.display(),
                    entry.key
                ));
            }
            resources += 1;
            rows += source.parts.len();
            for control in [0x007F, 0x027F, 0x037F] {
                let context = Context::from_control_word(control).map_err(|e| e.to_string())?;
                let blocks = source_rest_blocks(&source, context).map_err(|e| {
                    format!("{} {} {control:04X}: {e}", package.display(), entry.key)
                })?;
                let mut digest = Digest(0xCBF29CE484222325);
                digest.bytes(b"native-source-rest-v1\0");
                digest.word(u32::from(control));
                digest.word(blocks.len() as u32);
                for (row, block) in blocks.iter().enumerate() {
                    if block.palette_id != row as u32 {
                        return Err("strict intake changed source palette identity".into());
                    }
                    digest.word(row as u32);
                    digest.bytes(&source.data0[row * 140..(row + 1) * 140]);
                    for component in block.center.iter().chain(&block.orientation_xyzw) {
                        digest.word(component.to_bits());
                    }
                    evaluated += 1;
                }
                lines.push(format!(
                    "{} {} {control:04X} {:016X}",
                    package.file_name().unwrap().to_string_lossy(),
                    entry.key,
                    digest.0
                ));
            }
        }
    }
    if resources == 0 {
        return Err("no source CreatureData resources found".into());
    }
    lines.sort();
    let mut output =
        std::io::BufWriter::new(std::fs::File::create_new(&args[1]).map_err(|e| e.to_string())?);
    for line in &lines {
        writeln!(output, "{line}").map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())?;
    println!("indexes_opened={indexes} source_resources={resources} source_rows={rows} resource_contexts={} evaluated_rows={evaluated} retail_control_word_known=false", lines.len());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
