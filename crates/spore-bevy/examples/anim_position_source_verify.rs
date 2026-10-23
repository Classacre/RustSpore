//! Sample original full ANIM channels against selected-neighbor native goldens.
//! The fixture's normalized host rows do not replace source channel selection.
use dbpf::{PackageIndex, ResourceKey};
use spore_core::{anim, anim_curve, anim_deform, native_x87::Context, part_capabilities};
use std::{
    collections::BTreeSet,
    io::{BufRead, Write},
    path::{Component, Path},
};

const HEADER: &str = "package\ttype\tgroup\tinstance\tchannel\tframe_bits\tcw\tx_bits\ty_bits\tz_bits\tweight_bits\tprevious\tleft\tright\tfollowing\tnative_case_ordinal";
fn hex(text: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(text, 16)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let nonfinite_mode = args.get(3).is_some_and(|arg| arg == "--nonfinite");
    if args.len() != 3 && !(args.len() == 4 && nonfinite_mode) {
        return Err(
            "usage: anim_position_source_verify <install> <expected.tsv> <new actual.tsv> [--nonfinite]".into(),
        );
    }
    let install = Path::new(&args[0]);
    let patch = PackageIndex::open(install.join("Data/PatchData.package"))?;
    let dictionaries: Vec<_> = patch
        .entries()
        .iter()
        .filter(|e| {
            e.key
                == ResourceKey {
                    type_id: 0x7C19AA7A,
                    group_id: 0x10000,
                    instance_id: 0x82D0142D3338,
                }
        })
        .collect();
    if dictionaries.len() != 1 {
        return Err("patched capability dictionary not unique".into());
    }
    let dictionary = part_capabilities::parse(&patch.read(dictionaries[0])?)?;
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
    let mut rows = std::io::BufReader::new(std::fs::File::open(&args[1])?).lines();
    let header = if nonfinite_mode {
        format!("{HEADER}\treason\tkind\tnonfinite_consumed_source_fields")
    } else {
        HEADER.to_owned()
    };
    if rows.next().transpose()?.as_deref() != Some(&header) {
        return Err("unexpected source fixture header".into());
    }
    let mut output = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[2])?,
    );
    if nonfinite_mode {
        writeln!(output, "{header}\trejection")?;
    } else {
        writeln!(output, "{header}")?;
    }
    let mut package: Option<(String, PackageIndex)> = None;
    let mut loaded: Option<(ResourceKey, anim::Anim, anim_deform::PreparedComponents)> = None;
    let mut unique_clips = BTreeSet::new();
    let mut unique_channels = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut total = 0;
    for line in rows {
        let line = line?;
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != if nonfinite_mode { 19 } else { 16 } {
            return Err(format!("malformed source fixture row{}", total + 2).into());
        }
        let relative = Path::new(row[0]);
        if relative.is_absolute()
            || relative
                .components()
                .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err("fixture package must stay relative to the source install".into());
        }
        let key = ResourceKey {
            type_id: hex(row[1])?,
            group_id: hex(row[2])?,
            instance_id: u64::from_str_radix(row[3], 16)?,
        };
        if key.type_id != 0xEE17C6AD {
            return Err("fixture source is not ANIM".into());
        }
        if package.as_ref().map(|p| p.0.as_str()) != Some(row[0]) {
            package = Some((
                row[0].to_owned(),
                PackageIndex::open(install.join(relative))?,
            ));
            loaded = None;
        }
        if loaded.as_ref().map(|c| c.0) != Some(key) {
            let index = &package.as_ref().unwrap().1;
            let entries: Vec<_> = index.entries().iter().filter(|e| e.key == key).collect();
            if entries.len() != 1 {
                return Err(format!("fixture source not unique: {} {key}", row[0]).into());
            }
            let animation = anim::parse(&index.read(entries[0])?)?;
            let prepared = anim_deform::prepare_native_initial(&animation, &dictionary)?;
            loaded = Some((key, animation, prepared));
        }
        let (_, animation, prepared) = loaded.as_ref().unwrap();
        let channel: usize = row[4].parse()?;
        let frame = f32::from_bits(hex(row[5])?);
        let control = u16::from_str_radix(row[6], 16)?;
        let descriptors = &prepared
            .channels
            .get(channel)
            .ok_or("fixture channel out of bounds")?
            .components;
        let sampled = anim_curve::sample_position(
            animation,
            channel,
            descriptors,
            frame,
            Context::from_control_word(control)?,
        );
        if nonfinite_mode {
            let native = [hex(row[7])?, hex(row[8])?, hex(row[9])?, hex(row[10])?];
            if native.iter().all(|v| f32::from_bits(*v).is_finite())
                || !matches!(sampled, Err(anim_curve::CurveError::NonFiniteInput))
            {
                return Err(format!(
                    "source NaN case{} did not reject with NonFiniteInput: {sampled:?}",
                    row[15]
                )
                .into());
            }
            writeln!(output, "{line}\tNonFiniteInput")?;
            unique_clips.insert((row[0].to_owned(), key));
            unique_channels.insert((row[0].to_owned(), key, channel));
            contexts.insert(control);
            total += 1;
            continue;
        }
        let actual = sampled
            .map_err(|e| {
                format!(
                    "{} {key} channel{channel} frame{frame:?} nativecase{}: {e}",
                    row[0], row[15]
                )
            })?
            .ok_or("fixture's prepared POS is absent")?;
        let bits = [
            actual.position[0],
            actual.position[1],
            actual.position[2],
            actual.weight,
        ]
        .map(f32::to_bits);
        let expected = [hex(row[7])?, hex(row[8])?, hex(row[9])?, hex(row[10])?];
        let neighbors = [
            row[11].parse::<usize>()?,
            row[12].parse()?,
            row[13].parse()?,
            row[14].parse()?,
        ];
        if bits != expected || actual.neighbors != neighbors {
            return Err(format!("{} {key} channel{channel} frame{frame:?} CW{control:04X} nativecase{}: {:08X?}/{:?} != {:08X?}/{neighbors:?}", row[0], row[15], bits, actual.neighbors, expected).into());
        }
        writeln!(
            output,
            "{}\t{:08X}\t{:08X}\t{:08X}\t{:08X}\t{}\t{}\t{}\t{}\t{}",
            row[..7].join("\t"),
            bits[0],
            bits[1],
            bits[2],
            bits[3],
            actual.neighbors[0],
            actual.neighbors[1],
            actual.neighbors[2],
            actual.neighbors[3],
            row[15]
        )?;
        unique_clips.insert((row[0].to_owned(), key));
        unique_channels.insert((row[0].to_owned(), key, channel));
        contexts.insert(control);
        total += 1;
    }
    output.flush()?;
    if total == 0 {
        return Err("empty source position corpus".into());
    }
    println!("ANIM_SOURCE_POSITION_VERIFY cases={total} clips={} channels={} explicit_contexts={} source_nonfinite_rejection={nonfinite_mode} exact_finite_bits={} full_source_selection=true retarget_rotation_deform=UNIMPLEMENTED", unique_clips.len(), unique_channels.len(), contexts.len(), !nonfinite_mode);
    Ok(())
}
