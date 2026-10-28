//! Source-derived Cell candidate visitation and local-layout fixture.
//! Usage: cell_ui_ordered_local_v2 <layout.package> <new.tsv-under-target>
//! Fresh empty child lists, successful attachment, no runtime mutation, zero
//! private flags, and authored root rectangle are explicit fixture assumptions.
//! No global coordinates, active package, screen frame, or complete UI claimed.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{spui::{self, SpuiElement}, spui_layout::{simple_window_layout, SimpleLayout, SIMPLE_LAYOUT_CLASS}};
use std::{collections::{HashMap, HashSet}, error::Error, io::{BufWriter, Write}, path::Path};
const KEY: NativeResourceKey = NativeResourceKey { instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200 };

fn refs(e: &SpuiElement, id: u32) -> Result<Vec<u16>, Box<dyn Error>> {
    let Some(p) = e.property(id) else { return Ok(Vec::new()) };
    if p.type_code != 19 { return Err(format!("invalid ref type {id:08X}").into()); }
    let b = p.scalar_bytes().ok_or("non-scalar ref array")?;
    if b.len() != 2 * usize::from(p.count) { return Err("invalid ref count".into()); }
    Ok(b.chunks_exact(2).map(|v| u16::from_le_bytes([v[0], v[1]])).collect())
}
fn flags(e: &SpuiElement) -> Result<u32, Box<dyn Error>> {
    e.property(0xeec1_b000).and_then(|p| p.u32_value()).ok_or_else(|| "missing authored flags".into())
}
fn rectangle(e: &SpuiElement) -> Result<[f32; 4], Box<dyn Error>> {
    let rect = e.property(0xeec1_b005).and_then(|p| p.rectangle()).ok_or("missing authored rectangle")?;
    if rect.iter().any(|v| !v.is_finite()) { return Err("nonfinite authored rectangle".into()); }
    Ok(rect)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: cell_ui_ordered_local_v2 <layout.package> <new.tsv-under-target>".into()); }
    let out_path = Path::new(&args[1]);
    let parent = out_path.parent().ok_or("output needs a parent directory")?.canonicalize()?;
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("target").canonicalize()?;
    if !parent.starts_with(target) || out_path.file_name().is_none() { return Err("output parent must be inside target".into()); }
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package.index().find_unique(KEY)?.ok_or("missing exact candidate")?;
    let file = spui::parse(&package.read(entry)?)?;
    let body = spui::parse_elements(&file)?;
    let objects: HashMap<_, _> = body.elements.iter().map(|e| (e.index, e)).collect();
    let roots: Vec<_> = body.elements.iter().filter(|e| e.root).collect();
    if roots.len() != 1 { return Err("expected exactly one root".into()); }
    let root = roots[0];
    // A root with procedures would need dispatch and an external parent area.
    if !refs(root, 0xeec1_b00c)?.is_empty() { return Err("unsupported root procedures".into()); }
    let root_area = rectangle(root)?;
    let mut pending = vec![(root.index, None, 0usize, Some(root_area))];
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    while let Some((index, parent, depth, local)) = pending.pop() {
        if !seen.insert(index) { return Err("repeated reachable child or cycle".into()); }
        let e = objects.get(&index).ok_or("reference is not an element")?;
        let authored_flags = flags(e)?;
        if authored_flags & 1 == 0 { continue; } // 95A690 prunes this subtree under zero private flags.
        let children = refs(e, 0xeec1_b00b)?;
        // 962820 handles bit 0x40 with a partitioned insertion. This narrow
        // diagnostic refuses that case instead of guessing its final order.
        for child in &children {
            let obj = objects.get(child).ok_or("child is not an element")?;
            if flags(obj)? & 0x40 != 0 { return Err("unsupported child flag 0x40".into()); }
        }
        rows.push((index, parent, depth, authored_flags, local));
        // 960760 inserts in reverse source order; 962820 inserts ahead of the
        // first non-0x40 child; 95A690 visits tail to previous. Thus the visit
        // order is reverse source order for this restricted fresh-list case.
        for child in children {
            let obj = objects[&child];
            let procedures = refs(obj, 0xeec1_b00c)?;
            let mut unique_procedures = HashSet::new();
            if procedures.iter().any(|p| !unique_procedures.insert(*p)) {
                return Err(format!("duplicate procedure reference on object {child:04X}").into());
            }
            let mut simple = Vec::new();
            let mut supported = true;
            for p in procedures {
                let proc = objects.get(&p).ok_or("procedure is not an element")?;
                if proc.proxy_id != SIMPLE_LAYOUT_CLASS { supported = false; break; }
                simple.push(SimpleLayout::from_element(proc)?);
            }
            let child_local = match (local, supported) {
                (Some(area), true) => Some(simple_window_layout(area, rectangle(obj)?, &simple)?),
                _ => None,
            };
            pending.push((child, Some(index), depth + 1, child_local));
        }
    }
    let mut out = BufWriter::new(std::fs::File::create_new(out_path)?);
    writeln!(out, "# Source candidate {}; key={KEY}", package.path().display())?;
    writeln!(out, "# Fixture: authored root area, fresh empty child lists, successful attachment, no runtime mutation, private flags zero")?;
    writeln!(out, "# Conditional visit order only; local rect bits are not global positions; unsupported procedures propagate unknown")?;
    writeln!(out, "ordinal\tobject\tparent\tdepth\tclass\tflags\tlocal_rect_f32_bits")?;
    for (ordinal, (index, parent, depth, flags, rect)) in rows.iter().enumerate() {
        let parent = parent.map_or_else(|| "-".to_string(), |p| format!("{p:04X}"));
        let rect = rect.map_or_else(|| "unknown".to_string(), |v| format!("{:08X},{:08X},{:08X},{:08X}",v[0].to_bits(),v[1].to_bits(),v[2].to_bits(),v[3].to_bits()));
        writeln!(out,"{ordinal}\t{index:04X}\t{parent}\t{depth}\t{:08X}\t{flags:08X}\t{rect}",objects[index].proxy_id)?;
    }
    out.flush()?;
    println!("{} conditionally eligible objects; no screen rendered", rows.len());
    Ok(())
}
