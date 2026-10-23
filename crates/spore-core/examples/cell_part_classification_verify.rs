//! Compare original class lookup, including independently parsed source chains.
use spore_core::cell_part_classification::{
    classify_cell_part, native_local_property_present, CellPartPropertyList, CELL_CLASS_PREDICATES,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Read, Write},
    path::Path,
};

#[path = "support/cell_classification_fixture.rs"]
mod fixture;
use fixture::{key, source_chains, source_properties};

fn word(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: cell_part_classification_verify <fixture-directory> <new rust-output.bin>"
                .into(),
        );
    }
    let fixture = Path::new(&args[0]);
    let properties = source_properties(fixture)?;
    let chains = source_chains(fixture)?;
    let input = std::fs::File::open(fixture.join("input.bin"))?;
    let native = std::fs::File::open(fixture.join("output.bin"))?;
    let extent = input.metadata()?.len();
    if extent == 0 || extent % 1120 != 0 || native.metadata()?.len() != extent / 1120 * 16 {
        return Err("classification fixture record extents differ".into());
    }
    let cases = extent / 1120;
    if cases != 75938 {
        return Err("expected the complete source and all-mask classification cohort".into());
    }
    let mut input = std::io::BufReader::new(input);
    let mut native = std::io::BufReader::new(native);
    let mut output = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&args[1])?,
    );
    let mut contexts = BTreeSet::new();
    let mut used_copies = BTreeSet::new();
    let mut source_key_classes = BTreeMap::<u64, BTreeSet<u32>>::new();
    for ordinal in 0..cases as usize {
        let mut request = [0u8; 1120];
        let mut expected = [0u8; 16];
        input.read_exact(&mut request)?;
        native.read_exact(&mut expected)?;
        let count = word(&request, 0) as usize;
        let control = u16::from_le_bytes(request[12..14].try_into()?);
        if count > 8
            || !matches!(control & 0x300, 0 | 0x200 | 0x300)
            || control & !0xf00 != 0x7f
            || request[14..32].iter().any(|byte| *byte != 0)
        {
            return Err(format!("invalid classification request {ordinal}").into());
        }
        contexts.insert(control);
        let mut ids = Vec::new();
        let mut parents = Vec::new();
        for index in 0..count {
            let at = 32 + 136 * index;
            let fields = word(&request, at) as usize;
            let parent = word(&request, at + 4);
            if fields > 32
                || (parent != u32::MAX && !(index + 1..count).contains(&(parent as usize)))
            {
                return Err(format!("invalid classification node {ordinal}/{index}").into());
            }
            ids.push(
                (0..fields)
                    .map(|field| word(&request, at + 8 + 4 * field))
                    .collect::<Vec<_>>(),
            );
            parents.push((parent != u32::MAX).then_some(parent as usize));
        }
        let lists: Vec<_> = (0..count)
            .map(|index| CellPartPropertyList {
                identifiers: &ids[index],
                parent: parents[index],
            })
            .collect();
        let class = classify_cell_part(&lists, (count != 0).then_some(0))?;
        let (mut queries, mut visits) = (0u32, 0u32);
        if count != 0 {
            for (query, _) in CELL_CLASS_PREDICATES {
                queries += 1;
                let mut current = Some(0);
                let mut found = false;
                while let Some(index) = current {
                    visits += 1;
                    if native_local_property_present(lists[index].identifiers, query)? {
                        found = true;
                        break;
                    }
                    current = lists[index].parent;
                }
                if found {
                    break;
                }
            }
        }
        if let Some(chain) = chains.get(ordinal) {
            if word(&request, 4) != chain.low
                || word(&request, 8) != chain.scope
                || control != chain.control
                || count != chain.records.len()
            {
                return Err(format!("source chain header differs at {ordinal}").into());
            }
            for (index, &record) in chain.records.iter().enumerate() {
                let source = properties
                    .get(record)
                    .ok_or("source record index out of bounds")?;
                if lists[index].identifiers != source.identifiers
                    || lists[index].parent != (index + 1 < count).then_some(index + 1)
                {
                    return Err(format!(
                        "source named IDs/parent index differ at {ordinal}/{index}"
                    )
                    .into());
                }
                let expected_parent = chain.records.get(index + 1).map(|&next| {
                    let property = &properties[next];
                    key(property.low, property.scope)
                });
                if source.parent != expected_parent {
                    return Err(
                        format!("source Parent binding differs at {ordinal}/{index}").into(),
                    );
                }
                used_copies.insert(record);
            }
            let first = &properties[chain.records[0]];
            if (first.low, first.scope) != (chain.low, chain.scope) {
                return Err(format!("source root identity differs at {ordinal}").into());
            }
            source_key_classes
                .entry(key(chain.low, chain.scope))
                .or_default()
                .insert(class);
        }
        let mut actual = [0u8; 16];
        for (index, value) in [class, 1, queries, visits].into_iter().enumerate() {
            actual[4 * index..4 * index + 4].copy_from_slice(&value.to_le_bytes());
        }
        if actual != expected {
            return Err(format!(
                "classification case {ordinal} differs: Rust {:?}, native {:?}",
                [class, 1, queries, visits],
                std::array::from_fn::<_, 4, _>(|index| word(&expected, index * 4))
            )
            .into());
        }
        output.write_all(&actual)?;
    }
    output.flush()?;
    if used_copies.len() != 1411 || source_key_classes.len() != 719 || contexts.len() != 12 {
        return Err("classification corpus source/context coverage differs".into());
    }
    let ambiguous_keys = source_key_classes
        .values()
        .filter(|values| values.len() != 1)
        .count();
    println!("CELL_PART_CLASSIFICATION_VERIFY cases={cases} source_chains={} source_properties={} source_keys={} ambiguous_keys={ambiguous_keys} contexts={} exact_bytes=true", chains.len(), used_copies.len(), source_key_classes.len(), contexts.len());
    Ok(())
}
