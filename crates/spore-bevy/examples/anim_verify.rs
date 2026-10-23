//! Verify ANIM rows and generic sampling over user-supplied packages.
//! Usage: tools\build.bat run -p spore-bevy --example anim_verify -- [--digest FILE] <package-or-directory>...
//! This checks typed extraction and safe generic sampling, not authored curves,
//! selector-to-skeleton mapping, deformation, or faithful playback.

use spore_core::anim::{Anim, InterpolationPolicy};
use std::io::Write;
use std::path::{Path, PathBuf};

// Independent Python probe uses the same documented little-endian FNV-1a feed.
fn digest(anim: &Anim) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    fn bytes(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
        }
    }
    fn word(hash: &mut u64, value: u32) {
        bytes(hash, &value.to_le_bytes());
    }
    word(&mut hash, anim.bones.len() as u32);
    for bone in &anim.bones {
        word(&mut hash, bone.name.len() as u32);
        bytes(&mut hash, bone.name.as_bytes());
        word(&mut hash, bone.keyframe_count);
        for value in bone.selector_words {
            word(&mut hash, value);
        }
        word(&mut hash, bone.components.len() as u32);
        for c in &bone.components {
            word(&mut hash, c.flags);
            bytes(&mut hash, &c.tag);
            for value in [
                c.offset,
                c.stride,
                c.slot,
                c.reserved[0],
                c.reserved[1],
                c.reserved[2],
            ] {
                word(&mut hash, value);
            }
        }
        word(&mut hash, bone.keyframes.len() as u32);
        for key in &bone.keyframes {
            word(&mut hash, key.frame);
            for value in key.info {
                word(&mut hash, value);
            }
            word(&mut hash, u32::from(key.position.is_some()));
            if let Some(values) = key.position {
                for value in values {
                    word(&mut hash, value.to_bits());
                }
            }
            word(&mut hash, u32::from(key.rotation.is_some()));
            if let Some(values) = key.rotation {
                for value in values {
                    word(&mut hash, value.to_bits());
                }
            }
        }
        word(&mut hash, bone.raw_keyframes.len() as u32);
        bytes(&mut hash, &bone.raw_keyframes);
    }
    hash
}

fn collect(path: &Path, packages: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_symlink() {
                collect(&entry.path(), packages)?;
            }
        }
    } else if path.is_file() {
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("package"))
        {
            packages.push(path.to_path_buf());
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            path.display().to_string(),
        ));
    }
    Ok(())
}

fn main() {
    let mut args = Vec::new();
    let mut digest_path = None;
    let mut input = std::env::args_os().skip(1);
    while let Some(arg) = input.next() {
        if arg == "--digest" {
            digest_path = input.next().map(PathBuf::from);
            if digest_path.is_none() {
                eprintln!("--digest requires a path");
                std::process::exit(2);
            }
        } else {
            args.push(arg);
        }
    }
    if args.is_empty() {
        eprintln!("usage: anim_verify [--digest FILE] <package-or-directory>...");
        std::process::exit(2);
    }
    let mut packages = Vec::new();
    for path in &args {
        if let Err(err) = collect(Path::new(path), &mut packages) {
            eprintln!("{}: {err}", Path::new(path).display());
            std::process::exit(1);
        }
    }
    packages.sort();
    packages.dedup();
    if packages.is_empty() {
        eprintln!("no .package files found");
        std::process::exit(1);
    }
    let (mut opened, mut total, mut parsed, mut channels, mut problems) = (0, 0, 0, 0, 0);
    let (mut keys, mut positions, mut rotations, mut empty, mut unsupported) = (0, 0, 0, 0, 0);
    let (mut nonfinite_positions, mut nonunit_rotations, mut sample_errors) = (0, 0, 0);
    let mut digest_rows = Vec::new();
    for path in &packages {
        let index = match dbpf::PackageIndex::open(path) {
            Ok(index) => index,
            Err(err) => {
                eprintln!("{}: {err}", path.display());
                problems += 1;
                continue;
            }
        };
        opened += 1;
        let (mut local_total, mut local_parsed) = (0, 0);
        for entry in index
            .entries()
            .iter()
            .filter(|entry| entry.key.type_id == 0xEE17_C6AD)
        {
            total += 1;
            local_total += 1;
            let result = index
                .read(entry)
                .map_err(|err| err.to_string())
                .and_then(|bytes| spore_core::anim::parse(&bytes).map_err(|err| err.to_string()));
            match result {
                Ok(anim) => {
                    parsed += 1;
                    local_parsed += 1;
                    channels += anim.bones.len();
                    for bone in &anim.bones {
                        empty += usize::from(bone.keyframes.is_empty());
                        unsupported += bone.components.iter().filter(|c| !c.is_supported()).count();
                        keys += bone.keyframes.len();
                        for key in &bone.keyframes {
                            if let Some(pos) = key.position {
                                positions += 1;
                                nonfinite_positions +=
                                    usize::from(!pos.iter().all(|v| v.is_finite()));
                            }
                            if let Some(rot) = key.rotation {
                                rotations += 1;
                                let norm = rot.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
                                nonunit_rotations +=
                                    usize::from(!norm.is_finite() || (norm - 1.0).abs() >= 0.01);
                            }
                            sample_errors += usize::from(
                                bone.sample(key.frame as f32, InterpolationPolicy::Step)
                                    .is_err(),
                            );
                        }
                        for pair in bone.keyframes.windows(2) {
                            let midpoint =
                                (pair[0].frame as f64 + pair[1].frame as f64) as f32 * 0.5;
                            sample_errors += usize::from(
                                bone.sample(midpoint, InterpolationPolicy::LinearSlerp)
                                    .is_err(),
                            );
                        }
                    }
                    if digest_path.is_some() {
                        digest_rows.push(format!(
                            "{} {} {:016X}",
                            path.file_name().unwrap().to_string_lossy(),
                            entry.key,
                            digest(&anim)
                        ));
                    }
                }
                Err(err) => {
                    problems += 1;
                    eprintln!("{} {}: {err}", path.display(), entry.key);
                }
            }
        }
        if local_total > 0 {
            println!("{}: {local_parsed}/{local_total} parsed", path.display());
        }
    }
    println!(
        "packages={opened}/{} animations={parsed}/{total} channels={channels} problems={problems}",
        packages.len()
    );
    println!("keys={keys} positions={positions} rotations={rotations} empty_channels={empty} unsupported_components={unsupported}");
    println!("nonfinite_positions={nonfinite_positions} nonunit_rotations_01={nonunit_rotations} generic_sample_errors={sample_errors}");
    if let Some(path) = digest_path {
        digest_rows.sort();
        let result = std::fs::File::create(&path).and_then(|mut file| {
            for row in &digest_rows {
                writeln!(file, "{row}")?;
            }
            Ok(())
        });
        if let Err(err) = result {
            eprintln!("{}: {err}", path.display());
            problems += 1;
        }
    }
    if total == 0 || problems > 0 {
        std::process::exit(1);
    }
}
