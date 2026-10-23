//! Compose typed source Parent classification, original geometry and cache prefix.
use spore_core::{
    cell_capability_geometry::{derive_cell_capability_geometry, CellCapabilityPart},
    cell_part_classification::{classify_cell_part, CellPartPropertyList},
    cell_part_geometry::build_cell_part,
    creature_data,
    native_x87::Context,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::{Component, Path},
};
#[path = "support/cell_classification_fixture.rs"]
mod fixture;
use fixture::{key, source_chains, source_properties};

fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn float(data: &[u8], at: usize) -> f32 {
    f32::from_bits(word(data, at))
}
fn output(path: &Path) -> std::io::Result<BufWriter<File>> {
    Ok(BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?,
    ))
}
fn compare(actual: &[u8], native: &[u8], label: &str) -> Result<(), Box<dyn std::error::Error>> {
    if actual != native {
        let at = actual.iter().zip(native).position(|(a, b)| a != b).unwrap();
        let aligned = at / 4 * 4;
        return Err(format!(
            "{label} differs at byte{at}: Rust{:08X}, native{:08X}",
            word(actual, aligned),
            word(native, aligned)
        )
        .into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: cell_capability_source_verify <classification-fixture> <composition-fixture> <new geometry-rust.bin> <new cache-rust.bin>".into());
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut control = 0u32;
        // SAFETY: bounded local, on the SSE2 fixture target.
        unsafe {
            std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut control,
                options(nostack, preserves_flags));
        }
        if control & !0x3f != 0x1f80 {
            return Err("source composition requires explicit MXCSR control1F80".into());
        }
    }
    let classification = Path::new(&args[0]);
    let composition = Path::new(&args[1]);
    let properties = source_properties(classification)?;
    let chains = source_chains(classification)?;
    let mut class_native = BufReader::new(File::open(classification.join("output.bin"))?);
    let mut classes = BTreeMap::new();
    let mut property_copies = BTreeSet::new();
    for (ordinal, chain) in chains.iter().enumerate() {
        let first = properties
            .get(chain.records[0])
            .ok_or("source root out of bounds")?;
        if (first.low, first.scope) != (chain.low, chain.scope) {
            return Err("source classification root differs".into());
        }
        let mut lists = Vec::new();
        for (index, &record) in chain.records.iter().enumerate() {
            let property = properties
                .get(record)
                .ok_or("source property out of bounds")?;
            let parent = chain.records.get(index + 1).map(|&next| {
                let property = &properties[next];
                key(property.low, property.scope)
            });
            if parent != property.parent || chain.control != 0x27f {
                return Err("source Parent key/context differs".into());
            }
            property_copies.insert(record);
            lists.push(CellPartPropertyList {
                identifiers: &property.identifiers,
                parent: (index + 1 < chain.records.len()).then_some(index + 1),
            });
        }
        let class = classify_cell_part(&lists, Some(0))?;
        let mut expected = [0; 16];
        class_native.read_exact(&mut expected)?;
        if word(&expected, 0) != class || word(&expected, 4) != 1 {
            return Err(format!("source classification differs at path{ordinal}").into());
        }
        let identity = key(chain.low, chain.scope);
        if classes
            .insert(identity, class)
            .is_some_and(|previous| previous != class)
        {
            return Err("candidate source Parent paths have different classes".into());
        }
    }
    if classes.len() != 719 || property_copies.len() != 1411 {
        return Err("source classification coverage differs".into());
    }
    let table = std::fs::read_to_string(composition.join("creatures.tsv"))?;
    let mut lines = table.lines();
    if lines.next() != Some("case\tpackage\tlow\ttype\tscope\traw_file\tsha256\tsource_rows\tnative_rows\tgeometry_start") {
        return Err("unexpected source Creature table header".into());
    }
    let mut geometry_input = BufReader::new(File::open(composition.join("geometry-input.bin"))?);
    let mut geometry_native = BufReader::new(File::open(composition.join("geometry-output.bin"))?);
    let mut cache_input = BufReader::new(File::open(composition.join("cache-input.bin"))?);
    let mut cache_native = BufReader::new(File::open(composition.join("cache-output.bin"))?);
    let mut geometry_output = output(Path::new(&args[2]))?;
    let mut cache_output = output(Path::new(&args[3]))?;
    let (mut creatures, mut all_rows, mut native_rows, mut clamped) = (0, 0, 0, 0);
    let mut used_parts = BTreeSet::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 10
            || fields[0].parse::<usize>()? != creatures
            || fields[9].parse::<usize>()? != native_rows
        {
            return Err("noncontiguous source Creature mapping".into());
        }
        let creation_low = u32::from_str_radix(fields[2], 16)?;
        let creation_type = u32::from_str_radix(fields[3], 16)?;
        let creation_scope = u32::from_str_radix(fields[4], 16)?;
        let relative = Path::new(fields[5]);
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err("source Creature path leaves fixture".into());
        }
        let raw = std::fs::read(composition.join(relative))?;
        let creature = creature_data::parse(&raw)?;
        if !creature.exact
            || creature.consumed != raw.len()
            || creature.parts.len() != fields[7].parse::<usize>()?
            || creature.parts.len().min(64) != fields[8].parse::<usize>()?
            || creation_type != 0x0f43_029a
        {
            return Err(format!("source Creature {creatures} framing differs").into());
        }
        creature.validate_part_ranges()?;
        all_rows += creature.parts.len();
        clamped += usize::from(creature.parts.len() > 64);
        let mut derived_parts = Vec::new();
        let mut derived_native_rows = Vec::new();
        for index in 0..creature.parts.len().min(64) {
            let view = creature
                .part_view(index)
                .ok_or("source part ranges unavailable")?;
            let source: &[u8; 140] = view.raw_row.try_into()?;
            let part_scope = word(source, 132);
            let part_low = word(source, 136);
            let identity = key(part_low, part_scope);
            if view.part.part_instance != identity {
                return Err("typed source part identity differs from raw row".into());
            }
            let class = *classes
                .get(&identity)
                .ok_or("source part has no resolved Parent class")?;
            used_parts.insert(identity);
            let mut request = [0u8; 224];
            let mut expected = [0u8; 160];
            geometry_input.read_exact(&mut request)?;
            geometry_native.read_exact(&mut expected)?;
            if request[32..172] != source[..]
                || word(&request, 0) != 1
                || word(&request, 4) != class
                || word(&request, 8) != creation_low
                || word(&request, 12) != creation_type
                || word(&request, 16) != creation_scope
                || u16::from_le_bytes(request[20..22].try_into()?) != 0x27f
            {
                return Err(format!(
                    "native geometry source request differs at {creatures}/{index}"
                )
                .into());
            }
            let part = build_cell_part(source, class, creation_low)?;
            let mut native_part = [0xa5; 72];
            part.write_native(&mut native_part);
            let mut actual = [0u8; 160];
            for offset in [0, 4, 8, 12] {
                actual[offset..offset + 4].copy_from_slice(&1u32.to_le_bytes());
            }
            actual[16..88].copy_from_slice(&native_part);
            actual[88..160].copy_from_slice(&native_part);
            compare(
                &actual,
                &expected,
                &format!("source geometry {creatures}/{index}"),
            )?;
            geometry_output.write_all(&actual)?;
            derived_parts.push(CellCapabilityPart::from(part));
            derived_native_rows.push(native_part);
            native_rows += 1;
        }
        let mut request = [0u8; 4784];
        let mut expected = [0u8; 144];
        cache_input.read_exact(&mut request)?;
        cache_native.read_exact(&mut expected)?;
        let control = u16::from_le_bytes(request[4..6].try_into()?);
        if word(&request, 0) as usize != derived_parts.len() || control != 0x27f {
            return Err("source cache count/context differs".into());
        }
        for (index, row) in derived_native_rows.iter().enumerate() {
            compare(
                row,
                &request[32 + index * 72..104 + index * 72],
                "cache source geometry row",
            )?;
        }
        let origin = std::array::from_fn(|axis| float(&request, 8 + axis * 4));
        let value = derive_cell_capability_geometry(
            &derived_parts,
            origin,
            Context::from_control_word(control)?,
        )?;
        let mut actual = [0u8; 144];
        actual[..4].copy_from_slice(&1u32.to_le_bytes());
        actual[4..].copy_from_slice(&request[4640..4780]);
        value.write_native_tail((&mut actual[4..]).try_into()?);
        compare(&actual, &expected, &format!("source cache {creatures}"))?;
        cache_output.write_all(&actual)?;
        creatures += 1;
    }
    for stream in [
        &mut geometry_input,
        &mut geometry_native,
        &mut cache_input,
        &mut cache_native,
    ] {
        if stream.read(&mut [0u8; 1])? != 0 {
            return Err("source native streams contain unaccounted records".into());
        }
    }
    geometry_output.flush()?;
    cache_output.flush()?;
    if (creatures, all_rows, native_rows, clamped, used_parts.len())
        != (1365, 46418, 45664, 73, 654)
    {
        return Err("complete source composition denominators differ".into());
    }
    println!("CELL_CAPABILITY_SOURCE_VERIFY creatures={creatures} source_rows={all_rows} native_rows={native_rows} omitted_rows={} parts={} parent_paths={} property_copies={} class_keys={} exact_geometry_cache=true x87_CW=027F mxcsr_control=1F80", all_rows-native_rows, used_parts.len(), chains.len(), property_copies.len(), classes.len());
    Ok(())
}
