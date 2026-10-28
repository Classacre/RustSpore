//! Explicit-source Cell SPUI intake, without package priority or HUD rendering.
//!
//! Usage: cell_ui_explicit_source_v1 <layout.package> <image.package> <new.tsv>
//! The two input paths are deliberately independent. The program never chooses
//! a package or substitutes another candidate when an exact key is absent.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{
    spui,
    spui_layout::{ImageRegion, IMAGE_CLASS},
};
use std::{error::Error, io::Write, path::Path};

const LAYOUT: NativeResourceKey = NativeResourceKey {
    instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200,
};
const ATLAS: NativeResourceKey = NativeResourceKey {
    instance_id: 0xb8b5_0133, type_id: 0x2f7d_0004, group_id: 0x31a4_4893,
};
const CONTROL_IDS: [u32; 6] = [
    0x05af_4af0, 0x05af_4bf0, 0x05af_4af1,
    0x05af_4bf1, 0x05af_4af2, 0x05af_4bf2,
];

fn read_exact_resource(package: &NativePackageIndex, key: NativeResourceKey)
    -> Result<Vec<u8>, Box<dyn Error>>
{
    let entry = package.index().find_unique(key)?
        .ok_or_else(|| format!("missing exact resource {key} in {}", package.path().display()))?;
    Ok(package.read(entry)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: cell_ui_explicit_source_v1 <layout.package> <image.package> <new.tsv>".into());
    }
    let layout_package = NativePackageIndex::open(Path::new(&args[0]))?;
    let image_package = NativePackageIndex::open(Path::new(&args[1]))?;
    let file = spui::parse(&read_exact_resource(&layout_package, LAYOUT)?)?;
    let body = spui::parse_elements(&file)?;
    let image_bytes = read_exact_resource(&image_package, ATLAS)?;
    let decoded = image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Png)?;
    if decoded.width() != 512 || decoded.height() != 512 {
        return Err("unexpected atlas dimensions".into());
    }

    let resources: Vec<_> = file.image_resources.iter().chain(&file.atlas_resources).collect();
    let mut rows = Vec::new();
    let mut found = [0u32; 6];
    let mut image_count = 0usize;
    for element in &body.elements {
        if let Some(id) = element.property(0xeec1_b001).and_then(|p| p.u32_value()) {
            if let Some(slot) = CONTROL_IDS.iter().position(|candidate| *candidate == id) {
                found[slot] += 1;
                rows.push(format!("control\t{:04X}\t{:08X}\t{:08X}", element.index, element.proxy_id, id));
            }
        }
        if element.proxy_id != IMAGE_CLASS { continue; }
        let region = ImageRegion::from_element(element)?;
        let key = resources.get(usize::from(region.resource_reference))
            .ok_or("image reference outside resource table")?;
        if key.instance != ATLAS.instance_id || key.type_id != ATLAS.type_id || key.group != ATLAS.group_id {
            return Err(format!("unexpected image resource at object {}", element.index).into());
        }
        image_count += 1;
        let uv = region.map_uv([0.0, 0.0, 1.0, 1.0])?;
        rows.push(format!("image\t{:04X}\t{}\t{}\t{}\t{:08X}\t{:08X},{:08X},{:08X},{:08X}",
            element.index, region.resource_reference, region.intrinsic[0], region.intrinsic[1],
            region.flags.0, uv[0].to_bits(), uv[1].to_bits(), uv[2].to_bits(), uv[3].to_bits()));
    }
    if found != [1; 6] || image_count == 0 {
        return Err(format!("unexpected control counts {found:?} or image count {image_count}").into());
    }
    // Do not create a report until all inputs are decoded and validated.
    let mut out = std::io::BufWriter::new(std::fs::File::create_new(&args[2])?);
    writeln!(out, "# Explicit copies only; not native package precedence or a rendered HUD")?;
    writeln!(out, "# layout={} key={LAYOUT}", layout_package.path().display())?;
    writeln!(out, "# image={} key={ATLAS} size={}x{}", image_package.path().display(), decoded.width(), decoded.height())?;
    writeln!(out, "# objects={} image_objects={image_count}", body.elements.len())?;
    for row in rows { writeln!(out, "{row}")?; }
    out.flush()?;
    println!("Explicit Cell SPUI: {} objects, {image_count} image regions; no screen rendered", body.elements.len());
    Ok(())
}
