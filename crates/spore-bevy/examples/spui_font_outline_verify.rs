//! Export original ordinary glyph packets for an independent FontTools check.
//! No glyph hints, component flattening or rasterization are performed.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    spui_font,
    spui_font_outline::{self, GlyphKind},
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];

fn hexadecimal(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: spui_font_outline_verify <Spore_Graphics.package> <new outline TSV>".into(),
        );
    }
    let index = NativePackageIndex::open(Path::new(&args[0]))?;
    let mut entries: Vec<_> = index
        .entries()
        .iter()
        .filter(|e| {
            e.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && e.key.group_id == spui_font::FONT_NAMESPACE
                && FACES.contains(&e.key.instance_id)
        })
        .collect();
    entries.sort_by_key(|e| (e.key.type_id, e.key.group_id, e.key.instance_id));
    if entries.len() != FACES.len() {
        return Err(format!("expected five source faces, found {}", entries.len()).into());
    }
    let output_path = std::path::absolute(&args[1])?;
    output_path.strip_prefix(std::path::absolute("target")?)?;
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output_path)?,
    );
    let mut glyphs = 0usize;
    let mut points_count = 0usize;
    let mut component_count = 0usize;
    for entry in entries {
        let data = index.read(entry)?;
        let font = spui_font::parse(&data)?;
        let key = format!(
            "{:08X}:{:08X}:{:08X}",
            entry.key.type_id, entry.key.group_id, entry.key.instance_id
        );
        for glyph in 0..font.metrics().glyph_count {
            let packet = spui_font_outline::parse_glyph(&font, glyph)?;
            let bounds = packet.bounds.map_or_else(
                || "-".into(),
                |b| format!("{},{},{},{}", b.x_min, b.y_min, b.x_max, b.y_max),
            );
            let (kind, ends, instructions, points, components) = match &packet.kind {
                GlyphKind::Empty => ("E".into(), "-".into(), "-".into(), "-".into(), "-".into()),
                GlyphKind::ZeroContours => {
                    ("Z".into(), "-".into(), "-".into(), "-".into(), "-".into())
                }
                GlyphKind::Simple(g) => {
                    points_count += g.points.len();
                    (
                        "S".into(),
                        g.end_points
                            .iter()
                            .map(u16::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                        hexadecimal(g.instructions),
                        g.points
                            .iter()
                            .map(|p| format!("{},{},{}", p.x, p.y, p.flag))
                            .collect::<Vec<_>>()
                            .join(";"),
                        "-".into(),
                    )
                }
                GlyphKind::Composite(g) => {
                    component_count += g.components.len();
                    let components = g
                        .components
                        .iter()
                        .map(|c| {
                            let (kind, a, b) = if let Some([a, b]) = c.xy_offset() {
                                ("xy", i32::from(a), i32::from(b))
                            } else {
                                let [a, b] = c.matched_points().unwrap();
                                ("match", i32::from(a), i32::from(b))
                            };
                            let [xx, xy, yx, yy] = c.transform.matrix();
                            format!(
                                "{},{},{kind},{a},{b},{xx},{xy},{yx},{yy}",
                                c.flags, c.glyph_index
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(";");
                    (
                        format!("C:{}", g.contour_marker),
                        "-".into(),
                        hexadecimal(g.instructions),
                        "-".into(),
                        components,
                    )
                }
            };
            writeln!(
                output,
                "{key}\tglyph\t{glyph}\t{kind}\t{bounds}\t{ends}\t{instructions}\t{points}\t{components}\t{}",
                hexadecimal(packet.remainder)
            )?;
            glyphs += 1;
        }
    }
    output.flush()?;
    println!(
        "source glyph packets faces=5 glyphs={glyphs} points={points_count} components={component_count}; unscaled data only, no hint execution or rasterization"
    );
    Ok(())
}
