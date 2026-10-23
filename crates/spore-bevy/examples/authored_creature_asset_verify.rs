//! Check the inspection viewer's exact authored creation -> PROP -> GMDL -> map.
//! tools\build.bat run -p spore-bevy --example authored_creature_asset_verify -- INSTALL OUT [--keep-going] [HEX ...]
#[allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;

use assets::{AssetSource, PackageSet};
use std::io::Write;
use std::path::{Path, PathBuf};

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF29CE484222325u64, |digest, byte| {
        (digest ^ u64::from(*byte)).wrapping_mul(0x100000001B3)
    })
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 2 {
        return Err("usage: authored_creature_asset_verify <install> <new-out.tsv> [--keep-going] [creature-hex ...]".into());
    }
    let install = Path::new(&args[0]);
    let out = PathBuf::from(&args[1]);
    if !out.components().any(|component| component.as_os_str() == "target") {
        return Err("source-derived results must remain under ignored target/".into());
    }
    if let Some(parent) = out.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(out)
        .map_err(|e| e.to_string())?;
    // Every archive, including the primary package, retains only its index in RAM.
    let source = AssetSource::open(install.join("Data/PatchData.package"))?
        .with_set(PackageSet::discover(install));
    let keep_going = args[2..].iter().any(|value| value == "--keep-going");
    let requested: Vec<_> = args[2..].iter().filter(|value| *value != "--keep-going").collect();
    let instances = if requested.is_empty() {
        vec![0x4062_6200_067C_5390, 0x4066_6200_D1B4_BB56, 0x406B_6200_0660_EACA]
    } else {
        requested.iter().map(|value| {
            let text = value.to_str().ok_or("invalid instance text")?;
            u64::from_str_radix(text.trim_start_matches("0x"), 16).map_err(|e| e.to_string())
        }).collect::<Result<Vec<_>, String>>()?
    };
    let mut count = 0;
    let mut failures = 0;
    for instance in instances {
        let asset = match source.authored_creature_asset(instance, 0) {
            Ok(asset) => asset,
            Err(error) if keep_going => {
                writeln!(output, "ERROR\t{instance:016X}\t{}", error.to_string().replace(['\t', '\r', '\n'], " "))
                    .map_err(|error| error.to_string())?;
                failures += 1;
                continue;
            }
            Err(error) => return Err(format!("{instance:016X}: {error}")),
        };
        let settings = asset.texture_binding.settings.iter().map(|byte| format!("{byte:02X}"))
            .collect::<String>();
        writeln!(output, "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:016X}",
            asset.reference.creature, asset.reference.prop, asset.reference.model, asset.texture_key,
            asset.mesh.positions.len(), asset.mesh.triangle_count(), asset.texture.width,
            asset.texture.height, settings, fnv(&asset.texture.rgba)).map_err(|e| e.to_string())?;
        count += 1;
    }
    output.flush().map_err(|e| e.to_string())?;
    println!("AUTHORED_CREATURE_ASSET_VERIFY attempted={} passed={count} failed={failures} scope=CPU_saved_mesh_and_source_map", count + failures);
    if failures != 0 {
        return Err(format!("{failures} authored creation intake failures; every result is retained in the output table"));
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("authored creature inspection validation: {error}");
        std::process::exit(1);
    }
}
