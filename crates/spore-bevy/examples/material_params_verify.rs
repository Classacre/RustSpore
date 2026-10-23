//! Verify source 0x210 conversion without a renderer or desktop window.
use spore_core::{
    gmdl, gmdl_skin,
    shader_binding::{self, MaterialParams},
};
use std::{
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
    if !(2..=3).contains(&args.len()) {
        return Err(
            "usage: material_params_verify <install> <new parameter TSV> [new texture TSV]".into(),
        );
    }
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    let mut output = std::io::BufWriter::new(file);
    let mut textures = args
        .get(2)
        .map(|path| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map(std::io::BufWriter::new)
        })
        .transpose()?;
    let mut texture_count = 0;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut models, mut parameters, mut markers) = (0, 0, 0, 0);
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x00E6_BCE5)
        {
            let data = index.read(entry)?;
            let model = gmdl::parse(&data)?;
            models += 1;
            let Some(skin) = gmdl_skin::parse(&data, &model)? else {
                continue;
            };
            for (i, material) in skin.materials.iter().enumerate() {
                for field in material.shader_data.iter().filter(|f| f.id == 0x20D) {
                    for (j, binding) in shader_binding::texture_bindings(&field.payload)?
                        .iter()
                        .enumerate()
                    {
                        if let Some(textures) = &mut textures {
                            let optional = |v: Option<i32>| {
                                v.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())
                            };
                            writeln!(
                                textures,
                                "{}\t{}\t{i}\t{j}\t{}\t{:016X}\t{}\t{}\t{}\t{}",
                                path.display(),
                                entry.key,
                                binding.sampler,
                                binding.raster_instance,
                                optional(binding.address_mode()),
                                optional(binding.min_mag_filter()),
                                optional(binding.mip_filter()),
                                optional(binding.availability().map(i32::from))
                            )?;
                        }
                        texture_count += 1;
                    }
                }
                for field in material.shader_data.iter().filter(|f| f.id == 0x210) {
                    let value = MaterialParams::decode(&field.payload)?;
                    let b = value.constant_bits;
                    writeln!(
                        output,
                        "{}\t{}\t{i}\t{}\t{}\t{:08X}\t{:08X}\t{:08X}\t{:08X}",
                        path.display(),
                        entry.key,
                        value.mode,
                        u8::from(value.has_fourth_component),
                        b[0],
                        b[1],
                        b[2],
                        b[3]
                    )?;
                    parameters += 1;
                    markers += usize::from(value.has_fourth_component);
                }
            }
        }
    }
    output.flush()?;
    if let Some(textures) = &mut textures {
        textures.flush()?;
    }
    if parameters == 0 {
        return Err("no source material parameters".into());
    }
    println!("MATERIAL_PARAMS_VERIFY packages={packages} models={models} parameters={parameters} four_component_markers={markers} textures={texture_count}");
    Ok(())
}
