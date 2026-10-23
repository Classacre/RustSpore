//! Compare bounded native curve outputs, then decode original PROP arrays anew.
use spore_core::{
    cell_capability_curve::sample_capability_curve,
    native_x87::Context,
    prop::{self, PropValue},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, Read, Seek, SeekFrom, Write},
    path::Path,
};

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn float(bytes: &[u8], at: usize) -> f32 {
    f32::from_bits(word(bytes, at))
}
fn hex(text: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(text, 16)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: cell_capability_curve_verify <fixture-dir> <new source.tsv> <new values.bin>"
                .into(),
        );
    }
    let root = Path::new(&args[0]);
    #[cfg(target_arch = "x86_64")]
    {
        let mut mxcsr = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{address}]", address = in(reg) &mut mxcsr, options(nostack, preserves_flags));
        }
        if mxcsr & 0xffc0 != 0x1f80 {
            return Err("fixture requires MXCSR control1F80".into());
        }
    }
    let count = std::fs::metadata(root.join("input.bin"))?.len();
    if count == 0 || count % 160 != 0 {
        return Err("native input has an invalid record extent".into());
    }
    let count = count / 160;
    if std::fs::metadata(root.join("output.bin"))?.len() != count * 16 {
        return Err("native reply count differs".into());
    }
    let mut supplied = std::io::BufReader::new(std::fs::File::open(root.join("input.bin"))?);
    let mut native = std::io::BufReader::new(std::fs::File::open(root.join("output.bin"))?);
    let mut values = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    let mut contexts = BTreeSet::new();
    let mut default_cases = 0;
    for ordinal in 0..count {
        let mut request = [0; 160];
        let mut reply = [0; 16];
        supplied.read_exact(&mut request)?;
        native.read_exact(&mut reply)?;
        let size = word(&request, 8) as usize;
        let mode = word(&request, 16);
        if size > 16 || mode > 2 || (mode == 0 && size != 0) || (mode == 2 && size > 1) {
            return Err(format!("invalid native geometry at {ordinal}").into());
        }
        let points: Vec<_> = (0..size)
            .map(|index| {
                [
                    float(&request, 32 + 8 * index),
                    float(&request, 36 + 8 * index),
                ]
            })
            .collect();
        let control = u16::from_le_bytes(request[12..14].try_into().unwrap());
        let expected = word(&reply, 0);
        let actual = sample_capability_curve(
            (mode != 0).then_some(points.as_slice()),
            float(&request, 4),
            Context::from_control_word(control)?,
        )
        .map_err(|e| format!("native case{ordinal} CW{control:04X}: {e}"))?;
        if actual.to_bits() != expected {
            return Err(format!(
                "native case{ordinal} CW{control:04X}: {:08X} != {expected:08X}",
                actual.to_bits()
            )
            .into());
        }
        if [word(&reply, 4), word(&reply, 8), word(&reply, 12)]
            != [1, u32::from(mode != 0), u32::from(mode != 0)]
        {
            return Err(format!("provider counts differ at {ordinal}").into());
        }
        default_cases += usize::from(size == 0);
        contexts.insert(control);
        values.write_all(&actual.to_le_bytes())?;
    }
    values.flush()?;

    // These are byte-identical copies read independently from all82 package
    // indexes by the producer. This second pass uses Rust's source PROP parser,
    // rather than the already normalized native request rows.
    let mut sources = BTreeMap::new();
    for filename in ["source-0.prop", "source-1.prop"] {
        let tree = prop::parse(&std::fs::read(root.join(filename))?)?;
        if tree.items.len() != 18 {
            return Err("source curve property count differs".into());
        }
        let mut curves = BTreeMap::new();
        let mut points = 0;
        for item in tree.items {
            if item.anonymous || item.prop_type != 0x30 || item.specifier != 0x9c {
                return Err("source curve is not a named Vector2 array".into());
            }
            let mut pairs = vec![];
            for value in item.values {
                let PropValue::Vector2(x, y) = value else {
                    return Err("source curve element is not Vector2".into());
                };
                pairs.push([x, y]);
            }
            points += pairs.len();
            if curves.insert(item.id, pairs).is_some() {
                return Err("duplicate source curve identifier".into());
            }
        }
        if points != 104 {
            return Err("source point count differs".into());
        }
        sources.insert(filename, curves);
    }
    let mut source_log =
        std::io::BufReader::new(std::fs::File::open(root.join("source-cases.tsv"))?).lines();
    const HEADER: &str =
        "native_case_ordinal\tproperty_file\tproperty_id\tquery_bits\tcw\texpected_bits";
    if source_log.next().transpose()?.as_deref() != Some(HEADER) {
        return Err("unexpected source metadata header".into());
    }
    let mut actual_source = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[1])?,
    );
    writeln!(actual_source, "{HEADER}")?;
    let mut source_cases = 0;
    let mut used = BTreeSet::new();
    for line in source_log {
        let line = line?;
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 6 {
            return Err("malformed source metadata".into());
        }
        let ordinal: u64 = row[0].parse()?;
        if ordinal >= count {
            return Err("source native ordinal out of bounds".into());
        }
        let ident = hex(row[2])?;
        let query = hex(row[3])?;
        let control = u16::from_str_radix(row[4], 16)?;
        let points = sources
            .get(row[1])
            .and_then(|curves| curves.get(&ident))
            .ok_or("source property absent")?;
        let mut request = [0; 160];
        supplied.seek(SeekFrom::Start(ordinal * 160))?;
        supplied.read_exact(&mut request)?;
        if word(&request, 0) != ident
            || word(&request, 4) != query
            || word(&request, 8) as usize != points.len()
            || word(&request, 16) != 1
            || u16::from_le_bytes(request[12..14].try_into().unwrap()) != control
            || points.iter().enumerate().any(|(i, p)| {
                p[0].to_bits() != word(&request, 32 + 8 * i)
                    || p[1].to_bits() != word(&request, 36 + 8 * i)
            })
        {
            return Err(format!(
                "source rows differ from independently decoded PROP at case{ordinal}"
            )
            .into());
        }
        let actual = sample_capability_curve(
            Some(points),
            f32::from_bits(query),
            Context::from_control_word(control)?,
        )?;
        if actual.to_bits() != hex(row[5])? {
            return Err(format!("source curve {ident:08X} case{ordinal} differs at bits").into());
        }
        writeln!(
            actual_source,
            "{}\t{:08X}",
            row[..5].join("\t"),
            actual.to_bits()
        )?;
        source_cases += 1;
        used.insert((row[1].to_owned(), ident));
    }
    actual_source.flush()?;
    if used.len() != 36 || contexts.len() != 12 {
        return Err("source curve/context coverage differs".into());
    }
    println!("CELL_CAPABILITY_CURVE_VERIFY cases={count} source_cases={source_cases} source_copies=2 curve_copies={} unique_curves=18 unique_points=104 explicit_contexts={} default_cases={default_cases} exact_bits=true independent_PROP_decode=true manager_cache_actor=UNIMPLEMENTED", used.len(), contexts.len());
    Ok(())
}
