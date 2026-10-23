//! Survey initial source blocks, never a final runtime rig or bone pose.
//! tools\build.bat run -p spore-bevy --example anim_binding_verify -- --blocks FILE --selection-digest FILE <install-directory>
use spore_core::anim_binding::{BindingError, SourceRig, SourceSelector};
use spore_core::{anim, creature_data, part_capabilities};
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
    } else if path.is_file() {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
        {
            out.push(path.to_path_buf());
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}
struct Digest(u64);
impl Digest {
    fn bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x100000001b3);
        }
    }
    fn word(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
}
fn digest(rig: &SourceRig, clip: &anim::Anim) -> Result<u64, String> {
    let mut d = Digest(0xcbf29ce484222325);
    d.bytes(b"initial-source-selectors-v1\0");
    d.word(clip.bones.len() as u32);
    for channel in &clip.bones {
        let selector = SourceSelector::primary(channel);
        for v in [
            selector.flags,
            selector.capability,
            selector.alternate,
            selector.reserved,
        ] {
            d.word(v);
        }
        match rig.candidate_parts(selector) {
            Ok(candidates) => {
                d.word(0);
                d.word(candidates.source_rows.len() as u32);
                for i in candidates.source_rows {
                    d.word(i as u32);
                }
            }
            Err(BindingError::UnsupportedSelector { .. }) => d.word(1),
            Err(err) => return Err(err.to_string()),
        }
    }
    Ok(d.0)
}
fn run() -> Result<(), String> {
    let mut input = std::env::args_os().skip(1);
    let mut paths = Vec::new();
    let mut blocks_path = None;
    let mut digest_path = None;
    while let Some(arg) = input.next() {
        if arg == "--blocks" {
            blocks_path = Some(PathBuf::from(
                input.next().ok_or("--blocks requires a path")?,
            ));
        } else if arg == "--selection-digest" {
            digest_path = Some(PathBuf::from(
                input.next().ok_or("--selection-digest requires a path")?,
            ));
        } else {
            paths.push(PathBuf::from(arg));
        }
    }
    if paths.is_empty() {
        return Err("usage: anim_binding_verify [--blocks FILE] [--selection-digest FILE] <install-directory>".into());
    }
    let mut packages = Vec::new();
    for p in paths {
        collect(&p, &mut packages).map_err(|e| e.to_string())?;
    }
    packages.sort();
    packages.dedup();
    let patches: Vec<_> = packages
        .iter()
        .filter(|p| p.file_name().is_some_and(|v| v == "PatchData.package"))
        .collect();
    if patches.len() != 1 {
        return Err("explicit PatchData package source is not unique".into());
    }
    let patch = dbpf::PackageIndex::open(patches[0]).map_err(|e| e.to_string())?;
    let dictionaries: Vec<_> = patch
        .entries()
        .iter()
        .filter(|e| {
            e.key.type_id == 0x7C19AA7A
                && e.key.group_id == 0x10000
                && e.key.instance_id == 0x000082D0142D3338
        })
        .collect();
    let clips: Vec<_> = patch
        .entries()
        .iter()
        .filter(|e| {
            e.key.type_id == 0xEE17C6AD
                && e.key.group_id == 0x1FFFF
                && e.key.instance_id == 0xDAC89265
        })
        .collect();
    if dictionaries.len() != 1 || clips.len() != 1 {
        return Err("explicit v4 dictionary / BP1_cycle_walk resource is not unique".into());
    }
    let dictionary =
        part_capabilities::parse(&patch.read(dictionaries[0]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let clip = anim::parse(&patch.read(clips[0]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    println!(
        "dictionary={} clip={} candidate_stage=InitialSourceBlocks finalization=UNSUPPORTED",
        dictionaries[0].key, clips[0].key
    );
    let mut blocks_file = blocks_path
        .map(std::fs::File::create)
        .transpose()
        .map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    let mut rejected = std::collections::BTreeMap::new();
    let (mut opened, mut total, mut prepared, mut parts, mut problems) = (0, 0, 0, 0, 0);
    for p in &packages {
        let index = match dbpf::PackageIndex::open(p) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}: {e}", p.display());
                problems += 1;
                continue;
            }
        };
        opened += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x0F43029A)
        {
            total += 1;
            let bytes = index.read(entry).map_err(|e| e.to_string())?;
            let c = creature_data::parse(&bytes).map_err(|e| e.to_string())?;
            match SourceRig::from_creature(&c, &dictionary) {
                Ok(rig) => {
                    prepared += 1;
                    parts += rig.blocks().len();
                    let hash = digest(&rig, &clip)?;
                    rows.push(format!(
                        "{} {} {hash:016X}",
                        p.file_name().unwrap().to_string_lossy(),
                        entry.key
                    ));
                    if let Some(file) = &mut blocks_file {
                        for b in rig.blocks() {
                            write!(
                                file,
                                "{}\t{}\t{}",
                                p.file_name().unwrap().to_string_lossy(),
                                entry.key,
                                b.source_row
                            )
                            .map_err(|e| e.to_string())?;
                            for value in b
                                .selection_position
                                .iter()
                                .chain(&b.half_extents)
                                .chain(&b.body_origin_local)
                                .chain(&b.effects_origin_local)
                            {
                                write!(file, "\t{value:.9e}").map_err(|e| e.to_string())?;
                            }
                            writeln!(file).map_err(|e| e.to_string())?;
                        }
                    }
                    if entry.key.instance_id == 0x40626200067C5390
                        && p.file_name().is_some_and(|v| v == "Spore_Content.package")
                    {
                        println!("CANDIDATE {} source_parts={} selector_digest={hash:016X} finalization={:?}",entry.key,rig.blocks().len(),rig.require_finalized());
                        for channel in &clip.bones {
                            println!(
                                "channel={} selector={:?} result={:?}",
                                channel.name,
                                SourceSelector::primary(channel),
                                rig.candidate_parts(SourceSelector::primary(channel))
                            );
                        }
                    }
                }
                Err(err) => {
                    *rejected.entry(err.to_string()).or_insert(0usize) += 1;
                }
            }
        }
    }
    if let Some(p) = digest_path {
        rows.sort();
        let mut f = std::fs::File::create(p).map_err(|e| e.to_string())?;
        for row in rows {
            writeln!(f, "{row}").map_err(|e| e.to_string())?;
        }
    }
    println!("packages={opened}/{} creatures={prepared}/{total} source_parts={parts} initial_unsupported={} problems={problems} final_runtime_bindings=0",packages.len(),total-prepared);
    println!("initial_unsupported={rejected:?}");
    if total == 0 || problems != 0 {
        return Err("source binding census incomplete".into());
    }
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
