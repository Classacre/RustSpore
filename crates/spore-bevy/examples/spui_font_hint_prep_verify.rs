//! Whole bounded font0 PREP comparison with source-instruction reference.
use spore_core::{spui_font, spui_font_hint_prep as prep};
use dbpf::native_index::NativePackageIndex;
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{x:02x}")).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: verifier <Spore_Graphics.package> <new target TSV>".into());
    }
    let output = std::path::absolute(&args[1])?;
    output.strip_prefix(std::path::absolute("target")?)?;
    let package = NativePackageIndex::open(Path::new(&args[0]))?;
    let entry = package
        .entries()
        .iter()
        .find(|x| {
            x.key.type_id == spui_font::FONT_RESOURCE_TYPE
                && x.key.group_id == spui_font::FONT_NAMESPACE
                && x.key.instance_id == 0x511b49ca
        })
        .ok_or("missing font0")?;
    let data = package.read(entry)?;
    let font = spui_font::parse(&data)?;
    let head = font.table(*b"head")?;
    let units = u16::from_be_bytes([head[18], head[19]]);
    let r = prep::execute_font0_prep(
        font.table(*b"fpgm")?,
        font.table(*b"prep")?,
        font.table(*b"maxp")?,
        font.table(*b"cvt ")?,
        units,
    )
    .map_err(|e| format!("bounded PREP: {e:?}"))?;
    // Source-table identity is supplied by the installed package selection;
    // these cloned owned mutations prove the API guards, not native recovery.
    let mut recursive = font.table(*b"fpgm")?.to_vec();
    recursive[231..234].copy_from_slice(&[0xb0, 8, 0x2b]);
    assert_eq!(
        prep::execute_font0_prep(
            &recursive,
            font.table(*b"prep")?,
            font.table(*b"maxp")?,
            font.table(*b"cvt ")?,
            units
        ),
        Err(prep::Error::CallDepth)
    );
    let mut reserved = font.table(*b"fpgm")?.to_vec();
    let at = reserved
        .windows(3)
        .position(|bytes| bytes == [0xb0, 70, 0x76])
        .ok_or("missing source super-round input")?;
    reserved[at + 1] = 192;
    assert_eq!(
        prep::execute_font0_prep(
            &reserved,
            font.table(*b"prep")?,
            font.table(*b"maxp")?,
            font.table(*b"cvt ")?,
            units
        ),
        Err(prep::Error::UnsupportedSuperRoundPeriod(192))
    );
    let mut out = BufWriter::new(
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?,
    );
    for (n, x) in r.dispatches.iter().enumerate() {
        writeln!(
            out,
            "dispatch\t{n}\t{}\t{}\t{:02X}\t{}",
            x.selector, x.pc, x.opcode, x.stack
        )?;
    }
    for (n, x) in r.cvt.iter().enumerate() {
        writeln!(out, "cvt\t{n}\t{x}")?;
    }
    for (n, x) in r.storage.iter().enumerate() {
        writeln!(out, "storage\t{n}\t{x}")?;
    }
    writeln!(out, "graphics\t{}", hex(&r.graphics))?;
    writeln!(out, "twilight\t{}", hex(&r.twilight))?;
    writeln!(
        out,
        "final\t{}\t{}\t{}\t{}\t{}\t{}",
        r.calls,
        r.stack.len(),
        r.axis,
        r.reference_points[0],
        r.reference_points[1],
        r.reference_points[2]
    )?;
    out.flush()?;
    println!(
        "font0 PREP dispatches={} actual CALLs={} remaining stack={}; source15ppem only",
        r.dispatches.len(),
        r.calls,
        r.stack.len()
    );
    Ok(())
}
