//! Inspect CellGame lighting from the package explicitly supplied on the CLI.
//! Does not select an overlay or install a renderer.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::cell_world_lighting_v1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = std::env::args_os().nth(1)
        .ok_or("usage: cell_world_lighting_intake_v1 <explicit package>")?;
    let archive = NativePackageIndex::open(package)?;
    let key = NativeResourceKey { instance_id: 0xA426_730B, type_id: 0x00B1_B104, group_id: 0x4020_0100 };
    let entry = archive.index().find_unique(key)?.ok_or("preset not found in selected package")?;
    let bytes = archive.read(entry)?;
    let preset = cell_world_lighting_v1::parse(&bytes)?;
    println!("package={} key={} SH={} exposure_bits={:08X}",
        archive.path().display(), key, preset.coefficients.len(), preset.exposure.to_bits());
    for (index, rgb) in preset.coefficients.iter().enumerate() {
        println!("{index:02}: {:08X} {:08X} {:08X}", rgb[0].to_bits(), rgb[1].to_bits(), rgb[2].to_bits());
    }
    Ok(())
}
