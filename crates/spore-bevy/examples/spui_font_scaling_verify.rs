//! Compare unhinted simple-glyph preparation with a pinned instruction oracle.
//! Explicitly excludes component assembly, empty glyph construction and the
//! native coefficient selectors not implemented by the reviewed helper.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    spui_font,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, DesignCoordinate, PowerOfTwoScale, ScaleError},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: spui_font_scaling_verify <Spore_Graphics.package> <new scaling TSV>".into(),
        );
    }
    let output_path = std::path::absolute(&args[1])?;
    output_path.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let mut faces: Vec<_> = package
        .entries()
        .iter()
        .filter(|e| {
            e.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && e.key.group_id == spui_font::FONT_NAMESPACE
                && FACES.contains(&e.key.instance_id)
        })
        .collect();
    faces.sort_by_key(|e| (e.key.type_id, e.key.group_id, e.key.instance_id));
    if faces.len() != 5 {
        return Err("expected exactly five canonical source faces".into());
    }
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output_path)?,
    );
    let (mut simple_phantoms, mut scaled_glyphs, mut scaled_points, mut unsupported) =
        (0usize, 0usize, 0usize, 0usize);
    for entry in faces {
        let data = package.read(entry)?;
        let font = spui_font::parse(&data)?;
        if font.tables().iter().any(|t| t.tag == *b"vmtx") {
            return Err("vmtx input is outside this no-vmtx fallback corpus".into());
        }
        let key = format!(
            "{:08X}:{:08X}:{:08X}",
            entry.key.type_id, entry.key.group_id, entry.key.instance_id
        );
        let metrics = font.metrics();
        let vertical = spui_font_scaling::vertical_fallback(
            metrics.ascender,
            metrics.descender,
            metrics.line_gap,
        );
        let scale = PowerOfTwoScale::for_ppem(15, metrics.units_per_em);
        match scale {
            Ok(s) => {
                let p = s.parameters();
                writeln!(
                    output,
                    "{key}\tparams\t15\t{}\t0\t{}\t{}\t{}\t{}\t{}",
                    metrics.units_per_em,
                    p.multiplier,
                    p.shift,
                    p.denominator,
                    p.rounding_addend,
                    p.fixed_multiplier
                )?;
            }
            Err(ScaleError::UnsupportedSelector(id)) => {
                writeln!(
                    output,
                    "{key}\tparams\t15\t{}\tunsupported\t{id}",
                    metrics.units_per_em
                )?;
            }
            Err(error) => return Err(error.into()),
        }
        for gid in 0..metrics.glyph_count {
            let packet = spui_font_outline::parse_glyph(&font, gid)?;
            let glyph = match &packet.kind {
                GlyphKind::Simple(g) => g,
                GlyphKind::Composite(_) => {
                    writeln!(output, "{key}\tglyph\t{gid}\texcluded:composite")?;
                    continue;
                }
                GlyphKind::Empty | GlyphKind::ZeroContours => {
                    writeln!(output, "{key}\tglyph\t{gid}\texcluded:empty")?;
                    continue;
                }
            };
            let phantoms = spui_font_scaling::simple_phantom_coordinates(
                packet.bounds.unwrap(),
                font.horizontal_metrics(gid)?,
                vertical,
            );
            let encoded_phantoms = phantoms
                .iter()
                .map(|p| format!("{},{}", p.x, p.y))
                .collect::<Vec<_>>()
                .join(";");
            simple_phantoms += 1;
            let encoded_scaled = match scale {
                Ok(s) => {
                    let mut points: Vec<_> = glyph
                        .points
                        .iter()
                        .map(|p| DesignCoordinate { x: p.x, y: p.y })
                        .collect();
                    points.extend_from_slice(&phantoms);
                    scaled_glyphs += 1;
                    scaled_points += points.len();
                    spui_font_scaling::scale_coordinates(&points, s, s)
                        .iter()
                        .map(|p| format!("{},{}", p.x, p.y))
                        .collect::<Vec<_>>()
                        .join(";")
                }
                Err(ScaleError::UnsupportedSelector(id)) => {
                    unsupported += 1;
                    format!("unsupported:{id}")
                }
                Err(error) => return Err(error.into()),
            };
            writeln!(
                output,
                "{key}\tglyph\t{gid}\tsimple\t{},{}\t{encoded_phantoms}\t{encoded_scaled}",
                vertical.advance, vertical.top_side_bearing
            )?;
        }
    }
    output.flush()?;
    println!(
        "simple phantom glyphs={simple_phantoms} selector0_scaled_glyphs={scaled_glyphs} selector0_points_including_phantoms={scaled_points} unsupported_simple_glyphs={unsupported}; no hint or raster execution"
    );
    Ok(())
}
