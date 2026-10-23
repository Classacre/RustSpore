//! Full installed RASTER mip framing and content digest check.
use std::{
    io::Write,
    path::{Path, PathBuf},
};
fn collect(root: &Path, paths: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for e in std::fs::read_dir(root)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            collect(&e.path(), paths)?;
        } else if e
            .path()
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("package"))
        {
            paths.push(e.path());
        }
    }
    Ok(())
}
fn feed(hash: &mut u64, bytes: &[u8]) {
    for b in bytes {
        *hash = (*hash ^ u64::from(*b)).wrapping_mul(0x100000001b3);
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: raster_mip_verify <install> <new TSV>".into());
    }
    let mut out = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[1])?,
    );
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut resources, mut levels, mut pixel_bytes) = (0, 0, 0, 0u64);
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for e in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x2f4e681c)
        {
            let raster = spore_core::raster::parse(&index.read(e)?)?;
            let mips = raster
                .mip_levels()
                .map_err(|error| format!("{} {}: {error}", path.display(), e.key))?;
            let mut hash = 0xcbf29ce484222325;
            for word in [raster.width, raster.height, raster.mipmaps, raster.format] {
                feed(&mut hash, &word.to_le_bytes());
            }
            for (level, mip) in mips.iter().enumerate() {
                for word in [level as u32, mip.width, mip.height, mip.data.len() as u32] {
                    feed(&mut hash, &word.to_le_bytes());
                }
                feed(&mut hash, mip.data);
                pixel_bytes += mip.data.len() as u64;
            }
            writeln!(
                out,
                "{}\t{}\t{}\t{hash:016X}",
                path.display(),
                e.key,
                mips.len()
            )?;
            resources += 1;
            levels += mips.len();
        }
    }
    out.flush()?;
    if resources == 0 {
        return Err("no installed raster resources".into());
    }
    println!("RASTER_MIP_VERIFY packages={packages} resources={resources} levels={levels} pixel_bytes={pixel_bytes} exact=true");
    Ok(())
}
