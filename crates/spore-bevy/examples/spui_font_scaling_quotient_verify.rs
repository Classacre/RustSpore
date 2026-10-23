//! Compare admitted selector 0/1 integer preparation with source instructions.
//! The actual installed fonts remain under the user's package path; no source
//! payload or generated point corpus is part of this example.
use dbpf::native_index::NativePackageIndex;
use spore_core::{
    spui_font,
    spui_font_outline::{self, GlyphKind},
    spui_font_scaling::{self, PowerOfTwoScale, ScaleError, ScaleParameters},
    spui_font_scaling_quotient::IntegerQuotientScale,
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};

const FACES: [u32; 5] = [0x511b49ca, 0x61639dfb, 0x6bbc4e0e, 0xbacaaaa5, 0xffe355df];
const PPEMS: [u16; 14] = [
    1, 2, 3, 15, 16, 31, 511, 512, 1023, 32767, 32768, 32769, 65534, 65535,
];
const UNITS: [u16; 16] = [
    1, 2, 3, 16, 31, 32, 63, 64, 125, 192, 384, 1000, 2048, 32767, 32768, 65535,
];
const WORD_DOMAINS: [(u16, u16); 5] = [(15, 2048), (15, 1000), (1, 384), (32767, 192), (1, 65535)];

#[derive(Clone, Copy)]
enum AdmittedScale {
    PowerOfTwo(PowerOfTwoScale),
    Quotient(IntegerQuotientScale),
}

impl AdmittedScale {
    fn for_ppem(ppem: u16, units: u16) -> Result<Self, ScaleError> {
        match PowerOfTwoScale::for_ppem(ppem, units) {
            Ok(scale) => Ok(Self::PowerOfTwo(scale)),
            Err(ScaleError::UnsupportedSelector(1)) => {
                IntegerQuotientScale::for_ppem(ppem, units).map(Self::Quotient)
            }
            Err(error) => Err(error),
        }
    }

    fn parameters(self) -> ScaleParameters {
        match self {
            Self::PowerOfTwo(scale) => scale.parameters(),
            Self::Quotient(scale) => scale.parameters(),
        }
    }

    fn coordinate(self, word: i16) -> i32 {
        match self {
            Self::PowerOfTwo(scale) => scale.coordinate(word),
            Self::Quotient(scale) => scale.coordinate(word),
        }
    }
}

fn parameter_text(scale: Result<AdmittedScale, ScaleError>) -> Result<String, ScaleError> {
    match scale {
        Ok(scale) => {
            let p = scale.parameters();
            Ok(format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                p.selector,
                p.multiplier,
                p.shift,
                p.denominator,
                p.rounding_addend,
                p.fixed_multiplier
            ))
        }
        Err(ScaleError::UnsupportedSelector(2)) => Ok("unsupported\t2".into()),
        Err(error) => Err(error),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: spui_font_scaling_quotient_verify <Spore_Graphics.package> <new quotient TSV>"
                .into(),
        );
    }
    let output_path = std::path::absolute(&args[1])?;
    output_path.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let mut faces: Vec<_> = package
        .entries()
        .iter()
        .filter(|entry| {
            entry.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && entry.key.group_id == spui_font::FONT_NAMESPACE
                && FACES.contains(&entry.key.instance_id)
        })
        .collect();
    faces.sort_by_key(|entry| (entry.key.type_id, entry.key.group_id, entry.key.instance_id));
    if faces.len() != 5 {
        return Err("expected exactly five canonical source faces".into());
    }
    let mut output = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output_path)?,
    );
    for ppem in PPEMS {
        for units in UNITS {
            writeln!(
                output,
                "domain\t{ppem}\t{units}\t{}",
                parameter_text(AdmittedScale::for_ppem(ppem, units))?
            )?;
        }
    }
    for (ppem, units) in WORD_DOMAINS {
        let scale = AdmittedScale::for_ppem(ppem, units)?;
        for value in i16::MIN..=i16::MAX {
            writeln!(
                output,
                "word\t{ppem}\t{units}\t{value}\t{}",
                scale.coordinate(value)
            )?;
        }
    }
    let (mut scaled_glyphs, mut scaled_points, mut excluded) = (0usize, 0usize, 0usize);
    for entry in faces {
        let data = package.read(entry)?;
        let font = spui_font::parse(&data)?;
        if font.tables().iter().any(|table| table.tag == *b"vmtx") {
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
        let scale = AdmittedScale::for_ppem(15, metrics.units_per_em)?;
        writeln!(
            output,
            "{key}\tparams\t15\t{}\t{}",
            metrics.units_per_em,
            parameter_text(Ok(scale))?
        )?;
        for gid in 0..metrics.glyph_count {
            let packet = spui_font_outline::parse_glyph(&font, gid)?;
            let glyph = match &packet.kind {
                GlyphKind::Simple(glyph) => glyph,
                GlyphKind::Composite(_) => {
                    writeln!(output, "{key}\tglyph\t{gid}\texcluded:composite")?;
                    excluded += 1;
                    continue;
                }
                GlyphKind::Empty | GlyphKind::ZeroContours => {
                    writeln!(output, "{key}\tglyph\t{gid}\texcluded:empty")?;
                    excluded += 1;
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
                .map(|point| format!("{},{}", point.x, point.y))
                .collect::<Vec<_>>()
                .join(";");
            let encoded_scaled = glyph
                .points
                .iter()
                .map(|point| (point.x, point.y))
                .chain(phantoms.iter().map(|point| (point.x, point.y)))
                .map(|(x, y)| format!("{},{}", scale.coordinate(x), scale.coordinate(y)))
                .collect::<Vec<_>>()
                .join(";");
            scaled_glyphs += 1;
            scaled_points += glyph.points.len() + phantoms.len();
            writeln!(
                output,
                "{key}\tglyph\t{gid}\tsimple\t{},{}\t{encoded_phantoms}\t{encoded_scaled}",
                vertical.advance, vertical.top_side_bearing
            )?;
        }
    }
    output.flush()?;
    println!(
        "word inputs=327680 parameter cases=224 scaled simple glyphs={scaled_glyphs} scaled points including phantoms={scaled_points} excluded packets={excluded}; no hint or raster execution"
    );
    Ok(())
}
