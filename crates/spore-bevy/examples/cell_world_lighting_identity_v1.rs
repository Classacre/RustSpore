//! Conditional fresh-manager, four-band, identity-frame Cell lighting probe.
//! The supplied archive is explicit; this does not choose retail overlay or
//! establish the original frame, draw-time register count, or shader output.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{cell_world_lighting_v1, prop, shader_binding};
use std::collections::BTreeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let package = std::env::args_os().nth(1)
        .ok_or("usage: cell_world_lighting_identity_v1 <explicit package>")?;
    let archive = NativePackageIndex::open(package)?;
    let key = NativeResourceKey { instance_id: 0xA426_730B, type_id: 0x00B1_B104, group_id: 0x4020_0100 };
    let entry = archive.index().find_unique(key)?.ok_or("preset not found in selected package")?;
    let bytes = archive.read(entry)?;
    let preset = cell_world_lighting_v1::parse(&bytes)?;
    // Exact observed property set rules out parent and additional combinations;
    // the typed parser above additionally verifies count, framing and finiteness.
    let tree = prop::parse(&bytes)?;
    let ids: BTreeSet<u32> = tree.items.iter().map(|item| item.id).collect();
    let expected: BTreeSet<u32> = [0x0100_EAB6, 0x0100_EAB7, 0x0100_EAB8,
        0x0100_EAB9, 0x0100_EABA, 0x0100_EAC4, 0x0100_EAC5].into_iter().collect();
    if ids != expected || tree.items.len() != expected.len() {
        return Err("unsupported preset property set for the plain SH route".into());
    }
    let vectors = shader_binding::identity_sh_prefix(&preset.coefficients, preset.exposure, 16)?;
    println!("conditional identity-frame vectors; package={} key={} count={}",
        archive.path().display(), key, vectors.len());
    for (index, vector) in vectors.iter().enumerate() {
        println!("{index:02}: {:08X} {:08X} {:08X} {:08X}",
            vector[0].to_bits(), vector[1].to_bits(), vector[2].to_bits(), vector[3].to_bits());
    }
    Ok(())
}
