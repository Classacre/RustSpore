//! Check one explicit installed Cell Editor Setup copy and its template keys.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{cell_editor_template_array_v3 as templates, prop};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 { return Err("usage: cell_editor_template_verify_v2 <package>".into()); }
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let setup = NativeResourceKey::from_scoped_instance(0x00b1_b104, 0x4060_0100_3615_a30b);
    let entry = package.index().find_unique(setup)?.ok_or("Cell Editor Setup is absent")?;
    let data = package.read(entry)?;
    let tree = prop::parse(&data)?;
    if !tree.exact || tree.consumed != data.len() { return Err("setup PROP did not parse exactly".into()); }
    let field = tree.items.iter().find(|item| !item.anonymous && item.id == templates::TEMPLATE_PROPERTY_ID)
        .ok_or("template property disappeared")?;
    let mut header = Vec::new();
    header.extend_from_slice(&templates::TEMPLATE_PROPERTY_ID.to_be_bytes());
    header.extend_from_slice(&field.prop_type.to_be_bytes());
    header.extend_from_slice(&field.specifier.to_be_bytes());
    let locations: Vec<_> = data.windows(header.len()).enumerate()
        .filter_map(|(at, bytes)| (bytes == header).then_some(at)).collect();
    if locations.len() != 1 { return Err("template property byte header is ambiguous".into()); }
    let start = locations[0] + header.len();
    let end = start.checked_add(8 + field.values.len() * 12).ok_or("template array length overflow")?;
    let raw = data.get(start..end).ok_or("truncated template array")?;
    let keys = templates::templates_from_setup(&tree, raw).map_err(|e| format!("template property: {e:?}"))?;
    let raw_keys = templates::parse_template_array(raw).map_err(|e| format!("raw array: {e:?}"))?;
    if raw_keys != keys { return Err("raw and PROP template readers disagree".into()); }
    for (index, key) in keys.iter().enumerate() {
        let resource = NativeResourceKey { instance_id: key.low, type_id: key.type_id, group_id: key.scope };
        let entry = package.index().find_unique(resource)?.ok_or("template resource missing from this package")?;
        let payload = package.read(entry)?;
        if payload.is_empty() { return Err("empty template resource".into()); }
        let (next, _) = templates::next_template(&keys, index).map_err(|e| format!("next template: {e:?}"))?;
        println!("TEMPLATE\t{index}\t{:08X}\t{:08X}\t{:08X}\t{next}", key.low, key.type_id, key.scope);
    }
    println!("CELL_EDITOR_TEMPLATES count={} package_copy_only=true catalog_claim=false", keys.len());
    Ok(())
}
