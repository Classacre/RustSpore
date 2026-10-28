//! Source-instruction hinted outline comparison; no raster/pixel claim.
use spore_core::{
    spui_font, spui_font_hint_prep,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, DesignCoordinate, PowerOfTwoScale},
};
use spore_core::spui_font_hint_glyph0 as glyph0;
use dbpf::native_index::NativePackageIndex;
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{x:02x}")).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: verifier <Spore_Graphics.package> <new target TSV>".into());
    }
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package
        .entries()
        .iter()
        .find(|x| {
            x.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && x.key.group_id == spui_font::FONT_NAMESPACE
                && x.key.instance_id == 0x511b49ca
        })
        .ok_or("missing font0")?;
    let data = package.read(entry)?;
    let font = spui_font::parse(&data)?;
    let metrics = font.metrics();
    let prep = spui_font_hint_prep::execute_font0_prep(
        font.table(*b"fpgm")?,
        font.table(*b"prep")?,
        font.table(*b"maxp")?,
        font.table(*b"cvt ")?,
        metrics.units_per_em,
    )
    .map_err(|e| format!("PREP: {e:?}"))?;
    let packet = spui_font_outline::parse_glyph(&font, 0)?;
    let GlyphKind::Simple(glyph) = &packet.kind else {
        return Err("glyph0 not simple".into());
    };
    let vertical =
        spui_font_scaling::vertical_fallback(metrics.ascender, metrics.descender, metrics.line_gap);
    let phantoms = spui_font_scaling::simple_phantom_coordinates(
        packet.bounds.ok_or("missing glyph bounds")?,
        font.horizontal_metrics(0)?,
        vertical,
    );
    let mut design: Vec<_> = glyph
        .points
        .iter()
        .map(|p| DesignCoordinate { x: p.x, y: p.y })
        .collect();
    design.extend_from_slice(&phantoms);
    let scale = PowerOfTwoScale::for_ppem(15, metrics.units_per_em)?;
    let scaled = spui_font_scaling::scale_coordinates(&design, scale, scale);
    let result = glyph0::execute_glyph0(
        glyph.instructions,
        &design,
        &scaled,
        &glyph.end_points,
        &prep,
    )
    .map_err(|e| format!("glyph0: {e:?}"))?;
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    for (n, d) in result.dispatches.iter().enumerate() {
        writeln!(
            out,
            "dispatch\t{n}\t{}\t{:02X}\t{}",
            d.pc, d.opcode, d.stack
        )?;
    }
    for (i, p) in result.points.iter().enumerate() {
        writeln!(
            out,
            "point\t{i}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            design[i].x,
            design[i].y,
            result.original[i][0],
            result.original[i][1],
            p[0],
            p[1],
            result.touched[i]
        )?;
    }
    for (i, v) in result.cvt.iter().enumerate() {
        writeln!(out, "cvt\t{i}\t{v}")?;
    }
    for (i, v) in result.storage.iter().enumerate() {
        writeln!(out, "storage\t{i}\t{v}")?;
    }
    writeln!(out, "graphics\t{}", hex(&result.graphics))?;
    writeln!(
        out,
        "final\t{}\t{}\t{}\t{}\t{}",
        result.stack.len(),
        result.axis,
        result.reference_points[0],
        result.reference_points[1],
        result.reference_points[2]
    )?;
    out.flush()?;
    println!(
        "glyph0 hinted outline: {} source VM dispatches; no native-point or pixel acceptance",
        result.dispatches.len()
    );
    Ok(())
}
