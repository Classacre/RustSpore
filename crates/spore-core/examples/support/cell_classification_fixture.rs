//! Exact named source-PROP and Parent-chain loaders for native conformance.
use spore_core::prop::{self, PropValue};
use std::path::{Component, Path};

fn hex(value: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(u32::from_str_radix(value, 16)?)
}
pub(crate) fn key(low: u32, scope: u32) -> u64 {
    u64::from(low) | (u64::from(scope) << 32)
}
#[derive(Debug)]
pub(crate) struct SourceProperty {
    pub(crate) low: u32,
    pub(crate) scope: u32,
    pub(crate) identifiers: Vec<u32>,
    pub(crate) parent: Option<u64>,
}
#[derive(Debug)]
pub(crate) struct SourceChain {
    pub(crate) low: u32,
    pub(crate) scope: u32,
    pub(crate) control: u16,
    pub(crate) records: Vec<usize>,
}

pub(crate) fn source_properties(
    root: &Path,
) -> Result<Vec<SourceProperty>, Box<dyn std::error::Error>> {
    let table = std::fs::read_to_string(root.join("properties.tsv"))?;
    let mut lines = table.lines();
    if lines.next()
        != Some("record\tlow\tscope\tpackage\traw_file\tsha256\tparent_key\tfield_count")
    {
        return Err("unexpected source property table header".into());
    }
    let mut records = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 8 || fields[0].parse::<usize>()? != records.len() {
            return Err("noncontiguous source property table".into());
        }
        let (low, scope) = (hex(fields[1])?, hex(fields[2])?);
        let relative = Path::new(fields[4]);
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err("source property path leaves fixture".into());
        }
        let raw = std::fs::read(root.join(relative))?;
        if raw.len() < 4 {
            return Err("short original property list".into());
        }
        let declared = u32::from_be_bytes(raw[..4].try_into()?) as usize;
        let tree = prop::parse(&raw)?;
        if !tree.exact
            || tree.consumed != raw.len()
            || tree.items.len() != declared
            || tree.items.iter().any(|item| item.anonymous)
            || tree.items.len() != fields[7].parse::<usize>()?
        {
            return Err(format!(
                "source property {} is not exact named framing: declared{declared},items{},consumed{}/{},exact{}",
                records.len(), tree.items.len(), tree.consumed, raw.len(), tree.exact
            )
            .into());
        }
        let identifiers: Vec<_> = tree.items.iter().map(|item| item.id).collect();
        if !identifiers.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(
                format!("source property {} is not strictly ordered", records.len()).into(),
            );
        }
        // Ordinary lookup uses unsigned lower_bound and then the first key.
        // The native Parent binder ignores the middle resource-type word and
        // substitutes the current scope when the third raw word is zero.
        let at = identifiers.partition_point(|id| *id < prop::idents::PARENT);
        let parent = if identifiers.get(at) == Some(&prop::idents::PARENT) {
            let item = &tree.items[at];
            if item.prop_type != prop::types::KEY {
                return Err(format!("source property {} Parent is not KEY", records.len()).into());
            }
            let Some(PropValue::Key(parent_low, _ignored_type, parent_scope)) = item.values.first()
            else {
                return Err(format!("source property {} Parent has no key", records.len()).into());
            };
            Some(key(
                *parent_low,
                if *parent_scope == 0 {
                    scope
                } else {
                    *parent_scope
                },
            ))
        } else {
            None
        };
        let expected_parent = if fields[6] == "-" {
            None
        } else {
            Some(u64::from_str_radix(fields[6], 16)?)
        };
        if parent != expected_parent {
            return Err(format!("source property {} Parent key differs", records.len()).into());
        }
        records.push(SourceProperty {
            low,
            scope,
            identifiers,
            parent,
        });
    }
    if records.len() != 1411 {
        return Err("expected all 1,411 source property copies".into());
    }
    Ok(records)
}

pub(crate) fn source_chains(root: &Path) -> Result<Vec<SourceChain>, Box<dyn std::error::Error>> {
    let table = std::fs::read_to_string(root.join("source-chains.tsv"))?;
    let mut lines = table.lines();
    if lines.next() != Some("case\tlow\tscope\tcw\tnodes\trecord0\trecord1\trecord2\trecord3\trecord4\trecord5\trecord6\trecord7") {
        return Err("unexpected source chain table header".into());
    }
    let mut result = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 13 || fields[0].parse::<usize>()? != result.len() {
            return Err("noncontiguous source chain cases".into());
        }
        let count = fields[4].parse::<usize>()?;
        if !(1..=8).contains(&count) {
            return Err("source parent chain exceeds fixture bound".into());
        }
        let mut records = Vec::new();
        for (index, field) in fields[5..].iter().enumerate() {
            if index < count {
                records.push(field.parse::<usize>()?);
            } else if *field != "-1" {
                return Err("source chain has nonempty unused slot".into());
            }
        }
        result.push(SourceChain {
            low: hex(fields[1])?,
            scope: hex(fields[2])?,
            control: u16::from_str_radix(fields[3], 16)?,
            records,
        });
    }
    if result.len() != 23306 {
        return Err("expected all 23,306 candidate source parent paths".into());
    }
    Ok(result)
}
