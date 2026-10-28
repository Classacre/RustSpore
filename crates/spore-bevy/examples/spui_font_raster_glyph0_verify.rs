//! Original source glyph0 raster and bounded geometry-fixture comparison.
use spore_core::{
    spui_font, spui_font_hint_glyph0, spui_font_hint_prep,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, DesignCoordinate, PowerOfTwoScale},
};
use spore_core::spui_font_raster_glyph0 as raster;
use dbpf::native_index::NativePackageIndex;
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn rectangle(outer: [i32; 4], hole: [i32; 4]) -> Vec<[i32; 2]> {
    let [x0, y0, x1, y1] = outer;
    let [a, b, c, d] = hole;
    vec![
        [x0, y0],
        [x0, y1],
        [x1, y1],
        [x1, y0],
        [a, b],
        [c, b],
        [c, d],
        [a, d],
    ]
}
fn emit(output: &mut impl Write, key: &str, r: &raster::Bitmap) -> std::io::Result<()> {
    writeln!(
        output,
        "{key}\tmetrics\t{}\t{}\t{}\t{}\t{}",
        r.width, r.height, r.pitch, r.left26dot6, r.top26dot6
    )?;
    for (name, rows) in [("horizontal", &r.horizontal), ("vertical", &r.vertical)] {
        for row in rows {
            let events = if row.events.is_empty() {
                "-".into()
            } else {
                row.events
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            };
            writeln!(output, "{key}\t{name}\t{}\t{events}", row.scan)?;
        }
    }
    for (i, a) in r.alpha.iter().enumerate() {
        writeln!(output, "{key}\talpha\t{i}\t{a}")?;
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
        packet.bounds.ok_or("missing bounds")?,
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
    let hinted = spui_font_hint_glyph0::execute_glyph0(
        glyph.instructions,
        &design,
        &scaled,
        &glyph.end_points,
        &prep,
    )
    .map_err(|e| format!("glyph0: {e:?}"))?;
    let cases = [
        ("source", hinted.points[..8].to_vec()),
        (
            "negative_x",
            rectangle([-128, 320, 256, 704], [-64, 384, 192, 640]),
        ),
        (
            "negative_y",
            rectangle([192, -64, 576, 320], [256, 0, 512, 256]),
        ),
        (
            "narrow",
            rectangle([64, 64, 384, 512], [128, 128, 320, 448]),
        ),
        (
            "thick",
            rectangle([-128, -128, 512, 512], [64, 0, 320, 384]),
        ),
        ("wide", rectangle([0, 0, 768, 384], [64, 128, 704, 256])),
        (
            "tall",
            rectangle([0, -256, 320, 768], [128, -192, 256, 704]),
        ),
        (
            "pixel_hole",
            rectangle([0, 0, 192, 192], [64, 64, 128, 128]),
        ),
    ];
    let flags = [true; 8];
    let ends = [3, 7];
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    for (name, points) in cases {
        for (dropout, poison) in [(false, 0xa5), (true, 0)] {
            let params = raster::Parameters::source_glyph0(dropout);
            let result = raster::rasterize(&points, &ends, &flags, params)
                .map_err(|e| format!("raster {name}: {e:?}"))?;
            emit(
                &mut out,
                &format!("{name}:d{}:p{poison:02X}", u8::from(dropout)),
                &result,
            )?;
        }
    }
    writeln!(
        out,
        "source_advance\t{}",
        (hinted.points[9][0] - hinted.points[8][0]) << 10
    )?;
    //Positive failures are owned API guards, not source malformed recovery.
    let mut fractional = hinted.points[..8].to_vec();
    fractional[0][0] += 1;
    assert_eq!(
        raster::rasterize(
            &fractional,
            &ends,
            &flags,
            raster::Parameters::source_glyph0(false)
        ),
        Err(raster::Error::UnsupportedFractionalPoint)
    );
    let mut params = raster::Parameters::source_glyph0(false);
    params.gray_mode = 0;
    assert_eq!(
        raster::rasterize(&hinted.points[..8], &ends, &flags, params),
        Err(raster::Error::UnsupportedGrayMode(0))
    );
    out.flush()?;
    println!(
        "16 bounded raster cases; real glyph0 uses installed source through accepted hinting; no other native glyph pixel claim"
    );
    Ok(())
}
