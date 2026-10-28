//! Owned source-proven contour geometry regression; no embedded coordinates/pixels.
use spore_core::spui_font_raster_simple_contours as raster;
use std::io::{BufWriter, Write};
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
        return Err("usage geometry.tsv new-target.tsv".into());
    }
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)?,
    );
    for line in std::fs::read_to_string(&args[0])?.lines() {
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 4 {
            return Err("geometry fields".into());
        }
        let points: Vec<[i32; 2]> = f[1]
            .split(';')
            .map(|p| -> Result<_, Box<dyn std::error::Error>> {
                let v: Vec<i32> = p.split(',').map(str::parse).collect::<Result<_, _>>()?;
                Ok(v.try_into().map_err(|_| "point width")?)
            })
            .collect::<Result<_, _>>()?;
        let flags: Vec<bool> = f[2]
            .split(',')
            .map(|v| -> Result<_, Box<dyn std::error::Error>> {
                match v {
                    "0" => Ok(false),
                    "1" => Ok(true),
                    _ => Err("flag".into()),
                }
            })
            .collect::<Result<_, _>>()?;
        let ends: Vec<u16> = f[3].split(',').map(str::parse).collect::<Result<_, _>>()?;
        for d in [false, true] {
            let r = raster::rasterize(&points, &ends, &flags, raster::Parameters::source_glyph0(d))
                .map_err(|e| format!("{}: {e:?}", f[0]))?;
            emit_raster(&mut out, &format!("{}:d{}", f[0], u8::from(d)), &r)?;
        }
    }
    out.flush()?;
    Ok(())
}
