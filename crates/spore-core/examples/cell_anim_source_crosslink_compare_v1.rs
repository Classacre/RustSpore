//! Standalone source-join comparison driver. Inputs are supplied by the local
//! Python audit; it does not load packages or infer mounted resource selection.
use spore_core::{animlist_v7, animlist_v10, cell_anim_state_table_v1, cell_anim_source_crosslink_v1 as crosslink};
use crosslink::{Identity, List, Records};
use std::{error::Error, path::PathBuf};
fn main() -> Result<(), Box<dyn Error>> {
    let dir = PathBuf::from(std::env::args_os().nth(1).ok_or("expected input directory")?);
    let table = cell_anim_state_table_v1::parse_table(&std::fs::read(dir.join("table.bin"))?).ok_or("bad table extent")?;
    let manifest = std::fs::read_to_string(dir.join("manifest.tsv"))?;
    let mut blobs = Vec::new(); let mut ids = Vec::new(); let mut versions = Vec::new();
    for line in manifest.lines() {
        let f: Vec<_> = line.split('\t').collect(); if f.len() != 7 { return Err("bad manifest".into()); }
        versions.push(f[0].parse::<u32>()?);
        ids.push(Identity { package: f[1].into(), index_ordinal: f[2].parse()?, type_id: f[3].parse()?, group: f[4].parse()?, instance: f[5].parse()? });
        blobs.push(std::fs::read(dir.join(f[6]))?);
    }
    let mut v7 = Vec::new(); let mut v10 = Vec::new();
    for (i, blob) in blobs.iter().enumerate() {
        match versions[i] { 7 => { v7.push(Some(animlist_v7::parse(blob, Default::default()).map_err(|e| format!("{e:?}"))?)); v10.push(None); }, 10 => { v10.push(Some(animlist_v10::parse(blob, Default::default()).map_err(|e| format!("{e:?}"))?)); v7.push(None); }, _ => return Err("unsupported version".into()) }
    }
    let lists: Vec<_> = ids.into_iter().enumerate().map(|(i, identity)| List { identity, records: if let Some(ref list) = v7[i] { Records::V7(&list.records) } else { Records::V10(&v10[i].as_ref().unwrap().records) } }).collect();
    let inventory = std::fs::read_to_string(dir.join("anims.tsv"))?;
    let mut anims = Vec::new();
    for line in inventory.lines() {
        let f: Vec<_> = line.split('\t').collect(); if f.len() != 5 { return Err("bad ANIM inventory".into()); }
        anims.push(Identity { package: f[0].into(), index_ordinal: f[1].parse()?, type_id: f[2].parse()?, group: f[3].parse()?, instance: f[4].parse()? });
    }
    // Explicit audit ceilings; these do not imply runtime resource selection.
    let limits = crosslink::Limits { rows: 131, records: 100_000, entries: 1_000_000, anim_matches: 1_000_000 };
    for row in crosslink::crosslink(&table, &lists, &anims, limits).map_err(|e| format!("{e:?}"))? {
        println!("R\t{}", row.row_index);
        for rec in row.records {
            println!("M\t{}\t{}\t{}", row.row_index, rec.list_index, rec.record_index);
            for e in rec.entries {
                let matches = e.anim_indexes.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
                println!("E\t{}\t{}\t{}\t{}\t{}\t{}\t{}", row.row_index, rec.list_index, rec.record_index, e.variant, e.entry, e.anim_instance, matches);
            }
        }
    }
    Ok(())
}
