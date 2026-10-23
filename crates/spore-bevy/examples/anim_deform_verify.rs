//! Verify native descriptor preparation with an explicit patched dictionary and
//! native initial empty fallback. This does not claim generated-rig or authored-curve playback.
//! tools\build.bat run -p spore-bevy --example anim_deform_verify -- --native-initial --digest FILE <install-directory>

use spore_core::{anim, anim_deform, part_capabilities};
use std::io::Write;
use std::path::{Path, PathBuf};

struct Digest(u64);
impl Digest {
    fn bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x100000001B3);
        }
    }
    fn word(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }
}

fn digest(prepared: &anim_deform::PreparedComponents) -> u64 {
    let mut d = Digest(0xCBF29CE484222325);
    d.bytes(b"native-anim-descriptors-v1\0");
    d.word(prepared.channels.len() as u32);
    for c in &prepared.channels {
        for value in [
            c.selector.flags,
            c.selector.capability,
            c.selector.alternate,
            c.selector.reserved,
        ] {
            d.word(value);
        }
        match c.layout_source {
            anim_deform::LayoutSource::Capability(key) => {
                d.word(1);
                d.word(key);
            }
            anim_deform::LayoutSource::Fallback => {
                d.word(0);
                d.word(0);
            }
        }
        d.word(c.deforms.len() as u32);
        for deform in &c.deforms {
            d.word(deform.id);
            for value in deform.raw_parameters {
                d.word(value);
            }
        }
        d.word(c.components.len() as u32);
        for component in &c.components {
            d.word(component.flags);
            d.bytes(&component.tag);
            for value in [
                component.offset,
                component.stride,
                component.slot,
                component.reserved[0],
                component.reserved[1],
                component.reserved[2],
            ] {
                d.word(value);
            }
        }
    }
    d.0
}

fn collect(path: &Path, packages: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                collect(&entry.path(), packages)?;
            }
        }
    } else if path.is_file()
        && path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("package"))
    {
        packages.push(path.to_path_buf());
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let mut root = None;
    let mut output = None;
    let mut fallback_empty = false;
    while let Some(arg) = args.next() {
        if arg == "--native-initial" || arg == "--default-empty" {
            fallback_empty = true;
        } else if arg == "--digest" {
            output = Some(PathBuf::from(
                args.next().ok_or("--digest requires a path")?,
            ));
        } else if root.replace(PathBuf::from(arg)).is_some() {
            return Err("only one installation directory is supported".into());
        }
    }
    if !fallback_empty {
        return Err("--native-initial must select this initial fallback context".into());
    }
    let root = root
        .ok_or("usage: anim_deform_verify --native-initial [--digest FILE] <install-directory>")?;
    let patch = root.join("Data/PatchData.package");
    let patch_index = dbpf::PackageIndex::open(&patch).map_err(|e| e.to_string())?;
    let entries: Vec<_> = patch_index
        .entries()
        .iter()
        .filter(|e| {
            e.key.type_id == 0x7C19AA7A
                && e.key.group_id == 0x10000
                && e.key.instance_id == 0x82D0142D3338
        })
        .collect();
    if entries.len() != 1 {
        return Err("explicit patched capability dictionary is not unique".into());
    }
    let dictionary =
        part_capabilities::parse(&patch_index.read(entries[0]).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    println!(
        "dictionary={} fallback=NATIVE_INITIAL_EMPTY curves=UNIMPLEMENTED finalization=UNIMPLEMENTED",
        entries[0].key
    );
    let mut packages = Vec::new();
    collect(&root, &mut packages).map_err(|e| e.to_string())?;
    packages.sort();
    packages.dedup();
    let (mut opened, mut total, mut problems, mut channels, mut fallback) = (0, 0, 0, 0, 0);
    let (
        mut components,
        mut matched,
        mut deforms,
        mut changed_flags,
        mut changed_slots,
        mut enabled,
    ) = (0, 0, 0, 0, 0, 0);
    let mut digests = Vec::new();
    for package in packages {
        let index = match dbpf::PackageIndex::open(&package) {
            Ok(index) => index,
            Err(e) => {
                problems += 1;
                eprintln!("{}: {e}", package.display());
                continue;
            }
        };
        opened += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0xEE17C6AD)
        {
            total += 1;
            let result = index
                .read(entry)
                .map_err(|e| e.to_string())
                .and_then(|bytes| anim::parse(&bytes).map_err(|e| e.to_string()))
                .and_then(|animation| {
                    let prepared = anim_deform::prepare_native_initial(&animation, &dictionary)
                        .map_err(|e| e.to_string())?;
                    Ok((animation, prepared))
                });
            match result {
                Ok((animation, prepared)) => {
                    channels += prepared.channels.len();
                    for (original, prepared) in animation.bones.iter().zip(&prepared.channels) {
                        fallback += usize::from(
                            prepared.layout_source == anim_deform::LayoutSource::Fallback,
                        );
                        components += prepared.components.len();
                        for (original, prepared) in
                            original.components.iter().zip(&prepared.components)
                        {
                            matched += usize::from(prepared.slot != u32::MAX);
                            deforms += usize::from((3..9).contains(&prepared.slot));
                            changed_flags += usize::from(prepared.flags != original.flags);
                            changed_slots += usize::from(prepared.slot != original.slot);
                            enabled += usize::from(prepared.flags & 0x10 != 0);
                        }
                    }
                    digests.push(format!(
                        "{} {} {:016X}",
                        package.file_name().unwrap().to_string_lossy(),
                        entry.key,
                        digest(&prepared)
                    ));
                }
                Err(e) => {
                    problems += 1;
                    eprintln!("{} {}: {e}", package.display(), entry.key);
                }
            }
        }
    }
    digests.sort();
    if let Some(output) = output {
        let mut file = std::fs::File::create(output).map_err(|e| e.to_string())?;
        for row in &digests {
            writeln!(file, "{row}").map_err(|e| e.to_string())?;
        }
    }
    println!("packages={opened} resources={total} prepared={} channels={channels} fallback_channels={fallback}",
             digests.len());
    println!("components={components} matched={matched} unmatched={} deforms={deforms} flags_changed={changed_flags} slots_changed={changed_slots} enabled={enabled} problems={problems}",
             components - matched);
    if problems != 0 || total == 0 {
        return Err("descriptor-binding corpus verification failed".into());
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
