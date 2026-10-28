//! Conditional, serialized-reference diagnostic for the Cell layout candidate.
//! Does not infer runtime child-list order, screen geometry, or paint output.
//! Usage: cell_ui_tree_diagnostic_v1 <layout.package> <new.tsv-under-target>
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::spui::{self, SpuiElement};
use std::{collections::{HashMap, HashSet}, error::Error, io::{BufWriter, Write}, path::Path};

const LAYOUT: NativeResourceKey = NativeResourceKey {
    instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200,
};
const CHILD_REFS: u32 = 0xeec1_b00b;
const AUTHORED_FLAGS: u32 = 0xeec1_b000;

fn refs(element: &SpuiElement) -> Result<Vec<u16>, Box<dyn Error>> {
    let Some(property) = element.property(CHILD_REFS) else { return Ok(Vec::new()) };
    if property.type_code != 19 { return Err("unexpected reference property type".into()); }
    let bytes = property.scalar_bytes().ok_or("reference property is not scalar bytes")?;
    if bytes.len() != usize::from(property.count) * 2 { return Err("bad reference array length".into()); }
    Ok(bytes.chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: cell_ui_tree_diagnostic_v1 <layout.package> <new.tsv-under-target>".into()); }
    let output = Path::new(&args[1]);
    let parent = output.parent().ok_or("output needs a parent directory")?.canonicalize()?;
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("target").canonicalize()?;
    if !parent.starts_with(target) || output.file_name().is_none() { return Err("output parent must be inside target".into()); }
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package.index().find_unique(LAYOUT)?.ok_or("missing exact Cell layout candidate")?;
    let file = spui::parse(&package.read(entry)?)?;
    let body = spui::parse_elements(&file)?;
    let objects: HashMap<_, _> = body.elements.iter().map(|e| (e.index, e)).collect();
    let roots: Vec<_> = body.elements.iter().filter(|e| e.root).collect();
    if roots.len() != 1 { return Err("expected one serialized root".into()); }
    let mut pending = vec![(roots[0].index, None, 0usize, true)];
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    while let Some((index, parent, depth, ancestor_eligible)) = pending.pop() {
        if !seen.insert(index) { return Err(format!("repeated or cyclic object {index}").into()); }
        let object = objects.get(&index).ok_or("reference does not name an element")?;
        let flags = object.property(AUTHORED_FLAGS).and_then(|p| p.u32_value()).ok_or("missing or invalid authored flags")?;
        // 95A690 accepts bit 0 when its private flags byte is zero. This
        // fixture assumes that state for every object and prunes ancestors;
        // game code or class constructors may change it before a frame.
        let eligible = ancestor_eligible && flags & 1 != 0;
        let children = refs(object)?;
        rows.push((index, parent, depth, flags, eligible, children.len()));
        // Preserve serialized array order for the report only; native draw
        // order depends on insertion and linked-list traversal, not this stack.
        for child in children.into_iter().rev() { pending.push((child, Some(index), depth + 1, eligible)); }
    }
    let mut out = BufWriter::new(std::fs::File::create_new(output)?);
    writeln!(out, "# Explicit candidate; EEC1B00B serialized reference order, not established draw order")?;
    writeln!(out, "# Conditional eligibility assumes every Window private flags byte is zero; no runtime frame asserted")?;
    writeln!(out, "# source={} key={LAYOUT}", package.path().display())?;
    writeln!(out, "object\tparent_reference\tdepth\tclass\tauthored_flags\tconditional_eligible\tserialized_ref_count")?;
    for (index, parent, depth, flags, eligible, children) in rows {
        let parent = parent.map_or_else(|| "-".to_string(), |p| format!("{p:04X}"));
        writeln!(out, "{index:04X}\t{parent}\t{depth}\t{:08X}\t{flags:08X}\t{eligible}\t{children}", objects[&index].proxy_id)?;
    }
    out.flush()?;
    println!("{} reachable objects, conditional inventory only", seen.len());
    Ok(())
}
