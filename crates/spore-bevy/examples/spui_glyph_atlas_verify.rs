//! Run synthetic source-instruction inputs through the bounded atlas primitives.
//! Expected bytes are generated independently and are not read by this example.
use spore_core::spui_glyph_atlas::{clear_dxt3, write_alpha8_dxt3, Dxt3Layout, GlyphRectangle};
use std::io::Write;

fn unhex(text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if text == "-" {
        return Ok(Vec::new());
    }
    if text.len() % 2 != 0 {
        return Err("odd hex input".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| Ok(u8::from_str_radix(&text[i..i + 2], 16)?))
        .collect()
}
fn hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: spui_glyph_atlas_verify <input TSV> <new result TSV>".into());
    }
    let inputs = std::fs::read_to_string(&args[0])?;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    let mut count = 0;
    for line in inputs
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        if f.len() != 11 {
            return Err(format!("expected 11 fields, got {}", f.len()).into());
        }
        let layout = Dxt3Layout {
            side: f[2].parse()?,
            row_pitch: f[3].parse()?,
        };
        let mut destination = unhex(f[4])?;
        let rectangle = GlyphRectangle {
            x: f[5].parse()?,
            y: f[6].parse()?,
            width: f[7].parse()?,
            height: f[8].parse()?,
        };
        let source_pitch = f[9].parse()?;
        let source = unhex(f[10])?;
        let result = match f[1] {
            "C" => clear_dxt3(&mut destination, layout),
            "W" => write_alpha8_dxt3(&mut destination, layout, rectangle, &source, source_pitch),
            "CW" => clear_dxt3(&mut destination, layout).and_then(|()| {
                write_alpha8_dxt3(&mut destination, layout, rectangle, &source, source_pitch)
            }),
            _ => return Err(format!("unknown operation {}", f[1]).into()),
        };
        let status = match result {
            Ok(()) => "ok".into(),
            Err(error) => format!("{error:?}"),
        };
        writeln!(output, "{}\t{status}\t{}", f[0], hex(&destination))?;
        count += 1;
    }
    println!("source glyph-atlas synthetic cases={count}; no original glyph raster output");
    Ok(())
}
