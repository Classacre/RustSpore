//! Verify source UI typed streams over every package, without a renderer/window.
use spore_core::spui::{self, SpuiProperty, SpuiValue};
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
fn count(properties: &[SpuiProperty]) -> usize {
    properties
        .iter()
        .map(|p| {
            1 + match &p.value {
                SpuiValue::Structures { values, .. } => values.iter().map(|v| count(v)).sum(),
                _ => 0,
            }
        })
        .sum()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(2..=5).contains(&args.len()) {
        return Err(
            "usage: spui_layout_verify <install> <new digest TSV> [new SimpleLayout TSV] [new ImageSizing TSV] [new ImageRegion TSV]".into(),
        );
    }
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    let mut layouts = args
        .get(2)
        .map(|path| {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
        })
        .transpose()?;
    let (mut layout_count, mut missing_flags) = (0, 0);
    let mut images = args
        .get(3)
        .map(|path| {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
        })
        .transpose()?;
    let mut image_count = 0;
    let mut regions = args
        .get(4)
        .map(|path| {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
        })
        .transpose()?;
    let mut region_count = 0;
    let mut paths = Vec::new();
    collect(Path::new(&args[0]), &mut paths)?;
    paths.sort();
    let (mut packages, mut resources, mut objects, mut properties) = (0, 0, 0, 0);
    for path in paths {
        let index = dbpf::PackageIndex::open(&path)?;
        packages += 1;
        for entry in index
            .entries()
            .iter()
            .filter(|e| e.key.type_id == 0x250FE9A2)
        {
            let data = index.read(entry)?;
            let file = spui::parse(&data)?;
            let body = spui::parse_elements(&file)
                .map_err(|e| format!("{} {:016X}: {e}", path.display(), entry.key.instance_id))?;
            if let Some(layouts) = layouts.as_mut() {
                use spore_core::spui_layout::{
                    clamp_inverted_edges, LayoutError, SimpleLayout, SIMPLE_LAYOUT_CLASS,
                };
                for element in body
                    .elements
                    .iter()
                    .filter(|e| e.proxy_id == SIMPLE_LAYOUT_CLASS)
                {
                    let prefix = format!(
                        "{}\t{:016X}\t{}",
                        path.file_name().unwrap().to_string_lossy(),
                        entry.key.instance_id,
                        element.index
                    );
                    match SimpleLayout::from_element(element) {
                        Ok(layout) => {
                            let adjusted =
                                layout.apply([10., 20., 810., 620.], [6., -45., -3., -13.])?;
                            let clamped = clamp_inverted_edges(adjusted)?;
                            let words = adjusted
                                .into_iter()
                                .chain(clamped)
                                .map(|v| format!("{:08X}", v.to_bits()))
                                .collect::<Vec<_>>()
                                .join(",");
                            writeln!(layouts, "{prefix}\t{:02X}\t{words}", layout.flags)?;
                            layout_count += 1;
                        }
                        Err(LayoutError::MissingFlags) => {
                            writeln!(layouts, "{prefix}\tmissing-flags")?;
                            missing_flags += 1;
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
            if let Some(images) = images.as_mut() {
                use spore_core::spui_layout::{ImageSizing, IMAGE_DRAWABLE_CLASS};
                for element in body
                    .elements
                    .iter()
                    .filter(|e| e.proxy_id == IMAGE_DRAWABLE_CLASS)
                {
                    let image = ImageSizing::from_element(element)?;
                    let rect = image.rectangle([5., 90., 799., 213.], [420, 166])?;
                    let words = rect
                        .iter()
                        .map(|v| format!("{:08X}", v.to_bits()))
                        .collect::<Vec<_>>()
                        .join(",");
                    writeln!(
                        images,
                        "{}\t{:016X}\t{}\t{:08X}\t{:08X}\t{:08X}\t{:08X}\t{words}",
                        path.file_name().unwrap().to_string_lossy(),
                        entry.key.instance_id,
                        element.index,
                        image.flags,
                        image.scale.to_bits(),
                        image.horizontal_alignment,
                        image.vertical_alignment
                    )?;
                    image_count += 1;
                }
            }
            if let Some(regions) = regions.as_mut() {
                use spore_core::spui_layout::{ImageRegion, IMAGE_CLASS};
                for element in body.elements.iter().filter(|e| e.proxy_id == IMAGE_CLASS) {
                    let image = ImageRegion::from_element(element)?;
                    let mapped = image.map_uv([-0.25, 0.125, 1.25, 0.875])?;
                    let words = image
                        .saved_uv
                        .iter()
                        .chain(&mapped)
                        .map(|v| format!("{:08X}", v.to_bits()))
                        .collect::<Vec<_>>()
                        .join(",");
                    writeln!(
                        regions,
                        "{}\t{:016X}\t{}\t{:04X}\t{:08X}\t{:08X},{:08X}\t{words}",
                        path.file_name().unwrap().to_string_lossy(),
                        entry.key.instance_id,
                        element.index,
                        image.resource_reference,
                        image.flags.0,
                        image.intrinsic[0] as u32,
                        image.intrinsic[1] as u32
                    )?;
                    region_count += 1;
                }
            }
            let encoded = body.to_bytes();
            if encoded != file.element_data {
                return Err("typed serialization differs from source bytes".into());
            }
            let n = body
                .elements
                .iter()
                .map(|e| count(&e.properties))
                .sum::<usize>();
            let digest = encoded.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
            writeln!(
                output,
                "{}\t{:016X}\t{}\t{}\t{digest:016X}",
                path.file_name().unwrap().to_string_lossy(),
                entry.key.instance_id,
                body.elements.len(),
                n
            )?;
            resources += 1;
            objects += body.elements.len();
            properties += n;
        }
    }
    println!("SPUI_VERIFY packages={packages} resources={resources} elements={objects} properties={properties} exact=true");
    if layouts.is_some() {
        println!("SPUI_SIMPLE_LAYOUT explicit_flags={layout_count} missing_flags={missing_flags} global_ui_claim=false");
    }
    if images.is_some() {
        println!("SPUI_IMAGE_SIZING instances={image_count} fixture=true final_ui_claim=false");
    }
    if regions.is_some() {
        println!("SPUI_IMAGE_REGION instances={region_count} fixture=true final_ui_claim=false");
    }
    Ok(())
}
