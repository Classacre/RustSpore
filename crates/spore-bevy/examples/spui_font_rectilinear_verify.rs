//! Actual E/F/H/L/T through the bounded rectilinear source path.
use spore_core::{
    spui_font, spui_font_hint_prep,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, DesignCoordinate, PowerOfTwoScale},
};
use spore_core::spui_font_hint_rectilinear as axis;
use spore_core::spui_font_raster_rectilinear as raster;
use dbpf::native_index::NativePackageIndex;
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
    let cmap = font.cmap_format4(3, 1)?;
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
    for (gid, code) in [(38, 69), (39, 70), (41, 72), (45, 76), (53, 84)] {
        assert_eq!(cmap.glyph_index(code), gid);
        let key = format!("gid{gid}");
        let packet = spui_font_outline::parse_glyph(&font, gid)?;
        let GlyphKind::Simple(glyph) = &packet.kind else {
            return Err("visible glyph not simple".into());
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
        let r = axis::execute_rectilinear(
            glyph.instructions,
            &design,
            &scaled,
            &glyph.end_points,
            &prep,
            fpgm,
            maxp,
        )
        .map_err(|e| format!("axis {gid}: {e:?}"))?;
        writeln!(out, "{key}\tmapping\t{code:04X}\t{gid}")?;
        writeln!(
            out,
            "{key}\tshape\t{}\t{}\t{}",
            glyph.points.len(),
            glyph.end_points.len(),
            glyph.instructions.len()
        )?;
        for (i, d) in r.dispatches.iter().enumerate() {
            writeln!(
                out,
                "{key}\tdispatch\t{i}\t{}\t{}\t{:02X}\t{}",
                d.selector, d.pc, d.opcode, d.stack
            )?;
        }
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
        for dropout in [false, true] {
            let bitmap = raster::rasterize(
                &r.points[..glyph.points.len()],
                &glyph.end_points,
                &on_curve,
                raster::Parameters::source_glyph0(dropout),
            )
            .map_err(|e| format!("raster {gid}: {e:?}"))?;
            emit_raster(&mut out, &format!("{key}:d{}", u8::from(dropout)), &bitmap)?;
        }
        let mut unsupported = glyph.instructions.to_vec();
        unsupported[0] = 0x0f;
        assert!(matches!(
            axis::execute_rectilinear(
                &unsupported,
                &design,
                &scaled,
                &glyph.end_points,
                &prep,
                fpgm,
                maxp
            ),
            Err(axis::Error::Unsupported(0x0f))
        ));
        let mut fractional = r.points[..glyph.points.len()].to_vec();
        fractional[0][0] += 1;
        assert!(matches!(
            raster::rasterize(
                &fractional,
                &glyph.end_points,
                &on_curve,
                raster::Parameters::source_glyph0(false)
            ),
            Err(raster::Error::UnsupportedFractionalPoint)
        ));
        if gid == 53 {
            let mut wrong_branch = prep.clone();
            // Source F75 starts PUSH2/RS/EQ; the caller supplies comparison6.
            wrong_branch.storage[2] = 6;
            assert!(matches!(
                axis::execute_rectilinear(
                    glyph.instructions,
                    &design,
                    &scaled,
                    &glyph.end_points,
                    &wrong_branch,
                    fpgm,
                    maxp
                ),
                Err(axis::Error::UnsupportedFunctionBranch(75))
            ));
            let mut recursive = fpgm.to_vec();
            recursive[1347..1350].copy_from_slice(&[0xb0, 75, 0x2b]);
            assert!(matches!(
                axis::execute_rectilinear(
                    glyph.instructions,
                    &design,
                    &scaled,
                    &glyph.end_points,
                    &prep,
                    &recursive,
                    maxp
                ),
                Err(axis::Error::CallDepth)
            ));
        }
    }
    out.flush()?;
    println!(
        "five actual rectilinear characters; fresh15ppem source-axis path, complete hint/raster projection; fulltext/UI remain open"
    );
    Ok(())
}
