//! Unexported all-glyph bounded coverage verifier; source provenance required.
use dbpf::native_index::NativePackageIndex;
use spore_core::{spui_font_hint_simple_state as axis,spui_font_raster_simple_contours as raster};
use spore_core::{
    spui_font, spui_font_hint_prep,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, DesignCoordinate, PowerOfTwoScale},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{x:02x}")).collect()
}
fn emit_raster(output: &mut impl Write, key: &str, r: &raster::Bitmap) -> std::io::Result<()> {
    writeln!(
        output,
        "{key}\tmetrics\t{}\t{}\t{}\t{}\t{}",
        r.width, r.height, r.pitch, r.left26dot6, r.top26dot6
    )?;
    for (kind, rows) in [("horizontal", &r.horizontal), ("vertical", &r.vertical)] {
        for row in rows {
            let values = if row.events.is_empty() {
                "-".into()
            } else {
                row.events
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            };
            writeln!(output, "{key}\t{kind}\t{}\t{values}", row.scan)?;
        }
    }
    for (i, v) in r.alpha.iter().enumerate() {
        writeln!(output, "{key}\talpha\t{i}\t{v}")?;
    }
    Ok(())
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
        .find(|e| {
            e.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && e.key.group_id == spui_font::FONT_NAMESPACE
                && e.key.instance_id == 0x511b49ca
        })
        .ok_or("missing font0")?;
    let data = package.read(entry)?;
    let font = spui_font::parse(&data)?;
    let metrics = font.metrics();
    let _cmap = font.cmap_format4(3, 1)?;
    let fpgm = font.table(*b"fpgm")?;
    let maxp = font.table(*b"maxp")?;
    let prep = spui_font_hint_prep::execute_font0_prep(
        fpgm,
        font.table(*b"prep")?,
        maxp,
        font.table(*b"cvt ")?,
        metrics.units_per_em,
    )
    .map_err(|e| format!("PREP: {e:?}"))?;
    let vertical =
        spui_font_scaling::vertical_fallback(metrics.ascender, metrics.descender, metrics.line_gap);
    let scale = PowerOfTwoScale::for_ppem(15, metrics.units_per_em)?;
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    for gid in 0..metrics.glyph_count {
        let attempt = (|| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
            let mut out = Vec::new();
            let key = format!("gid{gid}");
            let packet = spui_font_outline::parse_glyph(&font, gid)?;
            let glyph = match &packet.kind {
                GlyphKind::Simple(glyph) => glyph,
                GlyphKind::Empty => return Err("empty".into()),
                GlyphKind::ZeroContours => return Err("zero contours".into()),
                GlyphKind::Composite(_) => return Err("composite".into()),
            };
            let phantoms = spui_font_scaling::simple_phantom_coordinates(
                packet.bounds.ok_or("missing glyph bounds")?,
                font.horizontal_metrics(gid)?,
                vertical,
            );
            let mut design: Vec<_> = glyph
                .points
                .iter()
                .map(|p| DesignCoordinate { x: p.x, y: p.y })
                .collect();
            design.extend_from_slice(&phantoms);
            let scaled = spui_font_scaling::scale_coordinates(&design, scale, scale);
            let r = axis::execute_simple(
                glyph.instructions,
                &design,
                &scaled,
                &glyph.end_points,
                &prep,
                fpgm,
                maxp,
            )
            .map_err(|e| format!("axis {gid}: {e:?}"))?;
            writeln!(
                out,
                "{key}\tshape\t{}\t{}\t{}",
                glyph.points.len(),
                glyph.end_points.len(),
                glyph.instructions.len()
            )?;
            for (i, p) in r.points.iter().enumerate() {
                writeln!(
                    out,
                    "{key}\tpoint\t{i}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    design[i].x,
                    design[i].y,
                    r.original[i][0],
                    r.original[i][1],
                    p[0],
                    p[1],
                    r.touched[i]
                )?;
            }
            for (kind, values) in [("cvt", &r.cvt), ("storage", &r.storage)] {
                for (i, v) in values.iter().enumerate() {
                    writeln!(out, "{key}\t{kind}\t{i}\t{v}")?;
                }
            }
            writeln!(out, "{key}\tgraphics\t{}", hex(&r.graphics))?;
            writeln!(out, "{key}\tloop_minus_one\t{}", r.loop_minus_one)?;
            writeln!(
                out,
                "{key}\tfinal\t{}\t{}\t{}\t{}\t{}\t{}",
                r.stack.len(),
                r.axis,
                r.reference_points[0],
                r.reference_points[1],
                r.reference_points[2],
                r.calls
            )?;
            writeln!(
                out,
                "{key}\tadvance\t{}",
                (r.points[glyph.points.len() + 1][0] - r.points[glyph.points.len()][0]) << 10
            )?;
            let on_curve: Vec<_> = glyph.points.iter().map(|p| p.flag & 1 != 0).collect();
            for shift in [false] {
                let points: Vec<_> = r.points[..glyph.points.len()]
                    .iter()
                    .map(|p| {
                        [
                            p[0] + if shift { 64 } else { 0 },
                            p[1] + if shift { -128 } else { 0 },
                        ]
                    })
                    .collect();
                for dropout in [false, true] {
                    let bitmap = raster::rasterize(
                        &points,
                        &glyph.end_points,
                        &on_curve,
                        raster::Parameters::source_glyph0(dropout),
                    )
                    .map_err(|e| format!("raster {gid}: {e:?}"))?;
                    let key = if shift {
                        format!("{key}:shift")
                    } else {
                        key.clone()
                    };
                    emit_raster(&mut out, &format!("{key}:d{}", u8::from(dropout)), &bitmap)?;
                    writeln!(
                        out,
                        "{key}:d{}\tfinal_origin\t{}\t{}",
                        u8::from(dropout),
                        (bitmap.left26dot6 - r.points[glyph.points.len()][0]) & !63,
                        (bitmap.top26dot6 - r.points[glyph.points.len()][1]) & !63
                    )?;
                }
            }
            Ok(out)
        })();
        match attempt {
            Ok(bytes) => {
                writeln!(out, "gid{gid}\tstatus\tcomplete")?;
                out.write_all(&bytes)?;
            }
            Err(e) => writeln!(out, "gid{gid}\tstatus\texcluded\t{e}")?,
        }
    }
    out.flush()?;
    Ok(())
}
