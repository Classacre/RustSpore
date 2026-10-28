//! Export source-texel bounds and an indexed contact sheet for inspection, not a HUD.
//! Usage: cell_ui_atlas_regions_v1 <layout.package> <image.package> <new-directory-under-target>
//! Explicit package inputs do not imply runtime package precedence. Objects whose
//! UVs are inverted or out of bounds are recorded as skipped. Bounding boxes
//! include surrounding texels and are not native filtered draws.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{spui, spui_layout::{ImageRegion, IMAGE_CLASS}};
use std::{error::Error, fs, io::{BufWriter, Write}, path::Path};

const LAYOUT: NativeResourceKey = NativeResourceKey {
    instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200,
};
const ATLAS: NativeResourceKey = NativeResourceKey {
    instance_id: 0xb8b5_0133, type_id: 0x2f7d_0004, group_id: 0x31a4_4893,
};

fn resource(package: &NativePackageIndex, key: NativeResourceKey) -> Result<Vec<u8>, Box<dyn Error>> {
    let entry = package.index().find_unique(key)?
        .ok_or_else(|| format!("missing exact resource {key} in {}", package.path().display()))?;
    Ok(package.read(entry)?)
}

/// Bounding texels of an in-bounds source UV rectangle. This is not a sampled
/// rendering of the fractional edges or a native texture filtering policy.
fn source_pixel_bounds(uv: [f32; 4], width: u32, height: u32) -> Option<[u32; 4]> {
    if uv[0] >= uv[2] || uv[1] >= uv[3] { return None; }
    let scales = [width as f32, height as f32, width as f32, height as f32];
    let mut edges = [0u32; 4];
    for i in 0..4 {
        let edge = uv[i] * scales[i];
        if !edge.is_finite() || edge < 0.0 || edge > scales[i] {
            return None;
        }
        edges[i] = if i < 2 { edge.floor() as u32 } else { edge.ceil() as u32 };
    }
    if edges[0] >= edges[2] || edges[1] >= edges[3] { return None; }
    Some(edges)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: cell_ui_atlas_regions_v1 <layout.package> <image.package> <new-directory-under-target>".into());
    }
    // The parent must already exist under this repository's target directory.
    // create_dir then refuses an existing destination, including a symlink.
    let output = Path::new(&args[2]);
    let parent = output.parent().ok_or("output needs a parent directory")?.canonicalize()?;
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("target").canonicalize()?;
    if !parent.starts_with(&target) || output.file_name().is_none() {
        return Err("output parent must be inside this repository's target directory".into());
    }
    let layout_package = NativePackageIndex::open(Path::new(&args[0]))?;
    let image_package = NativePackageIndex::open(Path::new(&args[1]))?;
    let file = spui::parse(&resource(&layout_package, LAYOUT)?)?;
    let body = spui::parse_elements(&file)?;
    let image_bytes = resource(&image_package, ATLAS)?;
    let atlas = image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Png)?.to_rgba8();
    if atlas.dimensions() != (512, 512) { return Err("unexpected atlas dimensions".into()); }
    let resources: Vec<_> = file.image_resources.iter().chain(&file.atlas_resources).collect();
    let mut regions = Vec::new();
    for element in &body.elements {
        if element.proxy_id != IMAGE_CLASS { continue; }
        let region = ImageRegion::from_element(element)?;
        let key = resources.get(usize::from(region.resource_reference)).ok_or("image reference outside resource table")?;
        if key.instance != ATLAS.instance_id || key.type_id != ATLAS.type_id || key.group != ATLAS.group_id {
            return Err(format!("unexpected image resource at object {}", element.index).into());
        }
        let uv = region.map_uv([0., 0., 1., 1.])?;
        regions.push((element.index, region, uv, source_pixel_bounds(uv, atlas.width(), atlas.height())));
    }
    if regions.is_empty() { return Err("no image regions".into()); }
    fs::create_dir(output)?;
    // Synthetic grid positions are diagnostic only. Every copied pixel comes
    // from the selected atlas; there are no inferred screen positions.
    let cell_width = regions.iter().filter_map(|r| r.3.map(|v| v[2] - v[0])).max().unwrap_or(1);
    let cell_height = regions.iter().filter_map(|r| r.3.map(|v| v[3] - v[1])).max().unwrap_or(1);
    let count = regions.iter().filter(|r| r.3.is_some()).count() as u32;
    let columns = count.min(8).max(1);
    let rows = ((count + columns - 1) / columns).max(1);
    let mut sheet = image::RgbaImage::new(cell_width * columns, cell_height * rows);
    let mut manifest = BufWriter::new(fs::File::create_new(output.join("regions.tsv"))?);
    writeln!(manifest, "# Diagnostic texel bounds and synthetic contact sheet only; no screen hierarchy, visibility, ordering, sampling, or active package selection")?;
    writeln!(manifest, "# layout={} key={LAYOUT}", layout_package.path().display())?;
    writeln!(manifest, "# image={} key={ATLAS} size=512x512", image_package.path().display())?;
    writeln!(manifest, "object\tresource_ref\tintrinsic_w\tintrinsic_h\tflags\tuv_bits\tpixel_rect\tsheet_xy\tfile")?;
    let mut exported = 0;
    for (index, region, uv, rect) in regions {
        let (pixels, sheet_xy, name) = if let Some([left, top, right, bottom]) = rect {
            let name = format!("object-{index:04X}.png");
            let crop = image::imageops::crop_imm(&atlas, left, top, right-left, bottom-top).to_image();
            let sheet_x = (exported % columns) * cell_width;
            let sheet_y = (exported / columns) * cell_height;
            image::imageops::replace(&mut sheet, &crop, i64::from(sheet_x), i64::from(sheet_y));
            let out = fs::File::create_new(output.join(&name))?;
            crop.write_to(&mut BufWriter::new(out), image::ImageFormat::Png)?;
            exported += 1;
            (format!("{left},{top},{right},{bottom}"), format!("{sheet_x},{sheet_y}"), name)
        } else { ("unsupported".to_string(), "-".to_string(), "-".to_string()) };
        writeln!(manifest, "{index:04X}\t{}\t{}\t{}\t{:08X}\t{:08X},{:08X},{:08X},{:08X}\t{pixels}\t{sheet_xy}\t{name}",
            region.resource_reference, region.intrinsic[0], region.intrinsic[1], region.flags.0,
            uv[0].to_bits(), uv[1].to_bits(), uv[2].to_bits(), uv[3].to_bits())?;
    }
    let sheet_file = fs::File::create_new(output.join("contact-sheet.png"))?;
    sheet.write_to(&mut BufWriter::new(sheet_file), image::ImageFormat::Png)?;
    manifest.flush()?;
    println!("Exported {exported} source-texel bounds and diagnostic grid; no HUD rendered");
    Ok(())
}
