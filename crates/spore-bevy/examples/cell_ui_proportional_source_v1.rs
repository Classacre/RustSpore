//! Explicit Cell candidate intake for the bounded ProportionalLayout decoder.
//! Reports source coefficients and drawable slot words; does not select a package or render UI.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{spui, spui_proportional_layout::{ProportionalLayout, PROPORTIONAL_LAYOUT_CLASS},
    spui_std_drawable::{StdDrawableSlots, STD_DRAWABLE_CLASS}};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 { return Err("usage: cell_ui_proportional_source_v1 <explicit-layout.package>".into()); }
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let key = NativeResourceKey { instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200 };
    let entry = package.index().find_unique(key)?.ok_or("candidate key missing")?;
    let file = spui::parse(&package.read(entry)?)?;
    let body = spui::parse_elements(&file)?;
    let mut count = 0;
    for e in body.elements.iter().filter(|e| e.proxy_id == PROPORTIONAL_LAYOUT_CLASS) {
        let value = ProportionalLayout::from_element(e)?;
        println!("{}\t{:08X}\t{:08X}\t{:08X}\t{:08X}", e.index,
            value.edges[0].to_bits(), value.edges[1].to_bits(), value.edges[2].to_bits(), value.edges[3].to_bits());
        count += 1;
    }
    eprintln!("{}: {count} decoded proportional procedures; source intake only", package.path().display());
    let mut drawable_count = 0;
    for e in body.elements.iter().filter(|e| e.proxy_id == STD_DRAWABLE_CLASS) {
        let slots = StdDrawableSlots::from_element(e)?;
        print!("drawable\t{}", e.index);
        for word in slots.0 { print!("\t{word:04X}"); }
        println!();
        drawable_count += 1;
    }
    eprintln!("{}: {drawable_count} decoded drawable slot arrays; resolution not inferred", package.path().display());
    Ok(())
}
