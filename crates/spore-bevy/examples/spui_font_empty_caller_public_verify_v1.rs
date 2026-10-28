//! Offline verifier for the bounded font0 empty-program caller. Requires original package.
use dbpf::native_index::NativePackageIndex;
use spore_core::{spui_font, spui_font_outline::{self, GlyphKind}, spui_font_scaling::{self, DesignCoordinate}};
use spore_core::spui_font_empty_caller_v1 as caller;
use spore_core::spui_font_raster_simple_contours_maxp_v1 as raster;
use std::{io::{BufWriter, Write}, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 { return Err("usage: verifier <Spore_Graphics.package> <new target TSV>".into()); }
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package.entries().iter().find(|e| e.key.type_id == spui_font::FONT_RESOURCE_TYPE && e.key.group_id == spui_font::FONT_NAMESPACE && e.key.instance_id == 0x511b49ca).ok_or("missing font0")?;
    let data = package.read(entry)?;
    let font = spui_font::parse(&data)?;
    let metrics = font.metrics();
    let vertical = spui_font_scaling::vertical_fallback(metrics.ascender, metrics.descender, metrics.line_gap);
    let mut out = BufWriter::new(std::fs::OpenOptions::new().create_new(true).write(true).open(output)?);
    for gid in [5, 132, 219, 230] {
        let packet = spui_font_outline::parse_glyph(&font, gid)?;
        let glyph = match &packet.kind { GlyphKind::Simple(g) => g, _ => return Err("expected simple glyph".into()) };
        let mut design: Vec<_> = glyph.points.iter().map(|p| DesignCoordinate { x: p.x, y: p.y }).collect();
        design.extend_from_slice(&spui_font_scaling::simple_phantom_coordinates(packet.bounds.ok_or("missing bounds")?, font.horizontal_metrics(gid)?, vertical));
        let result = caller::run(glyph.instructions, &design, glyph.points.len(), metrics.units_per_em).map_err(|e| format!("caller {gid}: {e:?}"))?;
        for (i, p) in result.helper.iter().enumerate() { writeln!(out, "{gid}\thelper\t{i}\t{}\t{}", p[0], p[1])?; }
        for (i, p) in result.points.iter().enumerate() { writeln!(out, "{gid}\tpoint\t{i}\t{}\t{}", p[0], p[1])?; }
        writeln!(out, "{gid}\tadvance\t{}", result.advance16dot16)?;
        let on_curve: Vec<_> = glyph.points.iter().map(|p| p.on_curve()).collect();
        let bitmap = raster::rasterize(&result.points[..glyph.points.len()], &glyph.end_points, &on_curve, raster::Parameters::source_glyph0(true)).map_err(|e| format!("raster {gid}: {e:?}"))?;
        let origin = caller::aligned_origin(bitmap.left26dot6, bitmap.top26dot6, result.points[glyph.points.len()]);
        writeln!(out, "{gid}\tmetrics\t{}\t{}\t{}\t{}\t{}", bitmap.width, bitmap.height, bitmap.pitch, origin[0], origin[1])?;
        for (i, v) in bitmap.alpha.iter().enumerate() { writeln!(out, "{gid}\talpha\t{i}\t{v}")?; }
    }
    Ok(())
}
