//! Compare original SFNT design data with an independent FontTools report.
//! No glyph shaping, native face-selection or rendering is performed.
use spore_core::spui_font::{self, FontError};
use std::{io::Write, path::Path};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];

fn escaped(text: String) -> String {
    text.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: spui_font_verify <Spore_Graphics.package> <new font TSV>".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[1])?;
    let index = dbpf::PackageIndex::open(Path::new(&args[0]))?;
    let mut rows = Vec::new();
    let mut faces = 0;
    let mut glyphs = 0usize;
    let mut mapped = 0usize;
    for entry in index.entries().iter().filter(|e| {
        e.key.type_id == spui_font::FONT_RESOURCE_TYPE
            && (e.key.instance_id >> 32) as u32 == spui_font::FONT_NAMESPACE
            && FACES.contains(&(e.key.instance_id as u32))
    }) {
        let data = index.read(entry)?;
        let font = spui_font::parse(&data)?;
        let key = format!("{:08X}:{:016X}", entry.key.group_id, entry.key.instance_id);
        let m = font.metrics();
        rows.push(format!(
            "{key}\thead\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            m.units_per_em,
            m.glyph_count,
            m.ascender,
            m.descender,
            m.line_gap,
            m.bounds.x_min,
            m.bounds.y_min,
            m.bounds.x_max,
            m.bounds.y_max
        ));
        for name in font.name_records()?.into_iter().filter(|n| {
            n.platform_id == 3
                && n.encoding_id == 1
                && n.language_id == 0x409
                && matches!(n.name_id, 1 | 2 | 4 | 6)
        }) {
            rows.push(format!(
                "{key}\tname\t{}\t{}\t{:04X}\t{}\t{}",
                name.platform_id,
                name.encoding_id,
                name.language_id,
                name.name_id,
                escaped(name.unicode_text()?)
            ));
        }
        for record in font.cmap_records()? {
            rows.push(format!(
                "{key}\tcmap-record\t{}\t{}\t{}",
                record.platform_id, record.encoding_id, record.format
            ));
            let cmap = match font.cmap_format4(record.platform_id, record.encoding_id) {
                Ok(cmap) => cmap,
                Err(FontError::UnsupportedCmapFormat(format)) if format == record.format => {
                    rows.push(format!(
                        "{key}\tcmap-unsupported\t{}\t{}\t{format}",
                        record.platform_id, record.encoding_id
                    ));
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            for code in 0..=u16::MAX {
                let glyph = cmap.glyph_index(code);
                if glyph != 0 {
                    rows.push(format!(
                        "{key}\tcmap\t{}\t{}\t{code:04X}\t{glyph}",
                        record.platform_id, record.encoding_id
                    ));
                    mapped += 1;
                }
            }
        }
        for glyph in 0..m.glyph_count {
            let h = font.horizontal_metrics(glyph)?;
            let bbox = match font.glyph_bounds(glyph)? {
                Some(b) => format!("{}\t{}\t{}\t{}", b.x_min, b.y_min, b.x_max, b.y_max),
                None => "empty".into(),
            };
            rows.push(format!(
                "{key}\tglyph\t{glyph}\t{}\t{}\t{bbox}",
                h.advance_width, h.left_side_bearing
            ));
            glyphs += 1;
        }
        faces += 1;
    }
    if faces != FACES.len() {
        return Err(format!("expected five source faces, found {faces}").into());
    }
    rows.sort();
    for row in &rows {
        writeln!(output, "{row}")?;
    }
    println!("source SFNT faces={faces} glyphs={glyphs} format4_mapped={mapped} rows={}; design data only, no rasterization",rows.len());
    Ok(())
}
