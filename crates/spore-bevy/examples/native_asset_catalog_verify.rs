//! Compare the actual viewer catalog with the independent canonical archive census.
#[allow(dead_code)]
#[path = "../src/assets.rs"]
mod assets;

use assets::{AssetSource, GMDL, GMSH, PNG, RASTER, RW4};
use dbpf::native_index::NativeResourceKey;
use std::{io::Write, path::{Component, Path}};

fn hex(text: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(u32::from_str_radix(text, 16)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: native_asset_catalog_verify INSTALL PYTHON-RECORDS.tsv NEW-OUTPUT.tsv".into());
    }
    let install = Path::new(&args[0]);
    let table = std::fs::read_to_string(&args[1])?;
    let output_path = std::path::absolute(&args[2])?;
    let target = std::path::absolute("target")?;
    output_path.strip_prefix(target)?;
    let mut groups: Vec<(String, usize, Vec<NativeResourceKey>)> = Vec::new();
    for line in table.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        match fields.first().copied() {
            Some("P") if fields.len() == 5 => {
                let path = Path::new(fields[1]);
                if path.is_absolute() || path.components().any(|part| !matches!(part, Component::Normal(_))) {
                    return Err("census package path leaves install".into());
                }
                groups.push((fields[1].to_owned(), fields[3].parse()?, Vec::new()));
            }
            Some("R") if fields.len() == 12 => {
                let (package, _, keys) = groups.last_mut().ok_or("resource precedes package")?;
                if package != fields[1] || fields[2].parse::<usize>()? != keys.len() {
                    return Err("noncontiguous census resource".into());
                }
                keys.push(NativeResourceKey { instance_id: hex(fields[3])?, type_id: hex(fields[4])?, group_id: hex(fields[5])? });
            }
            _ => return Err("unexpected canonical census row".into()),
        }
    }
    let mut output = std::io::BufWriter::new(std::fs::OpenOptions::new().write(true).create_new(true).open(output_path)?);
    let (mut records, mut meshes, mut textures) = (0, 0, 0);
    for (package, declared, keys) in &groups {
        if keys.len() != *declared { return Err("census package count differs".into()); }
        let source = AssetSource::open(install.join(package))?;
        if source.resource_keys().collect::<Vec<_>>() != *keys {
            return Err(format!("canonical catalog differs for {package}").into());
        }
        let mut expected_meshes: Vec<_> = keys.iter().copied().filter(|key| matches!(key.type_id, GMSH | GMDL | RW4)).collect();
        let mut expected_textures: Vec<_> = keys.iter().copied().filter(|key| matches!(key.type_id, RASTER | PNG)).collect();
        for values in [&mut expected_meshes, &mut expected_textures] {
            values.sort_by_key(|key| (key.type_id, key.group_id, key.instance_id));
        }
        if source.mesh_count() != expected_meshes.len() || source.texture_count() != expected_textures.len()
            || expected_meshes.iter().enumerate().any(|(index, key)| source.mesh_key(index) != Some(*key))
            || expected_textures.iter().enumerate().any(|(index, key)| source.texture_key(index) != Some(*key)) {
            return Err(format!("viewer mesh/texture selection differs for {package}").into());
        }
        for (ordinal, key) in source.resource_keys().enumerate() {
            writeln!(output, "{package}\t{ordinal}\t{:08X}\t{:08X}\t{:08X}", key.instance_id, key.type_id, key.group_id)?;
        }
        records += keys.len(); meshes += expected_meshes.len(); textures += expected_textures.len();
    }
    output.flush()?;
    println!("NATIVE_ASSET_CATALOG_VERIFY packages={} resources={records} mesh_keys={meshes} texture_keys={textures} exact=true index_only=true", groups.len());
    Ok(())
}
