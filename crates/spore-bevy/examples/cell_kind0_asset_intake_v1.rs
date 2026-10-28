//! Inspect one explicit candidate kind-0 Cell asset chain without runtime selection.
//! Usage: cell_kind0_asset_intake_v1 <cell-and-structure.package> <asset.package>
//! Requires trusted stable installed package inputs; index preflight reopens paths.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{cell_definition, material_links, mesh, prop, raster, rw4};
use std::{error::Error, fs::File, io::Read, path::Path};

fn key(instance_id: u32, type_id: u32, group_id: u32) -> NativeResourceKey {
    NativeResourceKey { instance_id, type_id, group_id }
}
fn package(path: &Path) -> Result<NativePackageIndex, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut raw = [0; dbpf::HEADER_SIZE];
    file.read_exact(&mut raw)?;
    let header = dbpf::DbpfHeader::parse(&raw)?;
    if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
        return Err("package index exceeds preflight bounds".into());
    }
    // The index reader reopens the path; this is not atomic against replacement.
    Ok(NativePackageIndex::open(path)?)
}
fn read(index: &NativePackageIndex, key: NativeResourceKey) -> Result<Vec<u8>, Box<dyn Error>> {
    let entry = index.index().find_unique(key)?.ok_or_else(|| format!("missing exact resource {key}"))?;
    if entry.stored_size > 16 * 1024 * 1024 || entry.decompressed_size > 16 * 1024 * 1024 {
        return Err("resource exceeds bounds".into());
    }
    Ok(index.read(entry)?)
}
fn word(b: &[u8], offset: usize) -> Result<u32, Box<dyn Error>> {
    let bytes: [u8; 4] = b.get(offset..offset.checked_add(4).ok_or("offset overflow")?)
        .ok_or("truncated word")?.try_into()?;
    Ok(u32::from_le_bytes(bytes))
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: cell_kind0_asset_intake_v1 <cell-and-structure.package> <asset.package>".into()); }
    let world = package(Path::new(&args[0]))?;
    let assets = package(Path::new(&args[1]))?;
    let cell_key = key(0xfd24_574b, 0xdfad_9f51, 0);
    let structure_key = key(0xfd24_574b, 0x4b9e_f6dc, 0);
    let prop_key = key(0xdae8_ce17, 0x00b1_b104, 0);
    let rw_key = key(0xdae8_ce17, 0x2f4e_681b, 0);
    let cell = read(&world, cell_key)?;
    if cell_definition::parse(&cell)?.structure_id() != structure_key.instance_id {
        return Err("unexpected raw Cell structure word".into());
    }
    let structure = read(&world, structure_key)?;
    let offset = word(&structure, 0x14)? as usize;
    let count = word(&structure, 0x18)? as usize;
    if count != 2 || offset < 28 || offset.checked_add(count * 40) != Some(structure.len()) {
        return Err("unsupported candidate structure layout".into());
    }
    let a = &structure[offset..offset + 40];
    let b = &structure[offset + 40..offset + 80];
    if word(a, 4)? != 0 || word(a, 0x10)? != prop_key.instance_id || word(b, 4)? != 1 {
        return Err("unexpected candidate structure records".into());
    }
    let prop_bytes = read(&assets, prop_key)?;
    let tree = prop::parse(&prop_bytes)?;
    if !tree.exact || tree.consumed != prop_bytes.len() || tree.key(prop::idents::MODEL_LOD0) != Some((u64::from(rw_key.instance_id), rw_key.type_id)) {
        return Err("unexpected or unsupported candidate PROP LOD0".into());
    }
    // The PROP's explicit model type and instance match the chosen RW4 key.
    // Provider resolution and the runtime choice remain unverified.
    let model_bytes = read(&assets, rw_key)?;
    let model = rw4::parse(&model_bytes)?;
    let section_meshes = model.sections.iter().filter(|s| s.info.type_code == rw4::object_types::MESH).count();
    let meshes = model.meshes(&model_bytes);
    if section_meshes == 0 || section_meshes != meshes.len() { return Err("missing or undecodable mesh section".into()); }
    let geometry = mesh::from_rw4(&model_bytes, &model)?;
    geometry.validate()?;
    println!("CELL_KIND0 package={} cell_key={} structure_key={} asset_package={} prop_key={} rw4_key={} source_runtime_fixup=unverified provider_precedence=unverified", world.path().display(), cell_key, structure_key, assets.path().display(), prop_key, rw_key);
    println!("CELL_KIND0 projected_row=0 kind=0 raw_id=DAE8CE17 other_row_kind=1 unsupported=true prop_lod0_type=2F4E681B prop_lod0_instance=00000000DAE8CE17 runtime_resolution=unverified");
    println!("CELL_KIND0 rw4_sections={} mesh_sections={} projected_vertices={} projected_triangles={} normals_from_file={} sanitised={} mesh_projection_not_native_render=true mesh_topology_completeness=unverified", model.sections.len(), section_meshes, geometry.positions.len(), geometry.triangle_count(), geometry.normals_from_file, geometry.sanitised);
    println!("CELL_KIND0 material_binding=unresolved prop_texture_keys={:?} prop_parent={:?}", tree.textures(), tree.parent());
    let mut raster_count = 0;
    for (index, section) in model.sections.iter().enumerate() {
        match section.info.type_code {
            rw4::object_types::RASTER => {
                let r = model.raster(&model_bytes, index as u32).ok_or("invalid raster section")?;
                if r.width == 0 || r.height == 0 || r.width > 4096 || r.height > 4096 {
                    return Err("raster extent outside inspection bounds".into());
                }
                let payload = model.object(&model_bytes, r.data).ok_or("unresolved raster data")?;
                let rgba = raster::decode_pixels(r.format, r.width, r.height, payload)?;
                println!("CELL_KIND0 raster_section={index} data_section={} extent={}x{} format={:08X} mips={} payload_bytes={} rgba_bytes={} shader_binding=unverified", r.data, r.width, r.height, r.format, r.mipmap_levels, payload.len(), rgba.len());
                raster_count += 1;
            }
            rw4::object_types::COMPILED_STATE => {
                let state = material_links::compiled_state(&model, &model_bytes, index)?;
                println!("CELL_KIND0 state_section={index} shader={:08X} primitive={} soft_dirty={:08X} soft_delta={:08X} hard_dirty={:08X} hard_delta={:08X} binding=unresolved", state.shader_id, state.primitive_type, state.soft_dirty, state.soft_delta, state.hard_dirty, state.hard_delta);
                let bindings = state.bindings()?;
                if let Some(group) = &bindings.groups {
                    for entry in group {
                        println!("CELL_KIND0 state_section={index} group={} state_pairs={:?}", entry.id, entry.values);
                    }
                }
                if bindings.textures.len() != 2 || bindings.textures.iter().filter(|s| s.stage == 0).count() != 1 || bindings.textures.iter().filter(|s| s.stage == 1).count() != 1 {
                    return Err("unsupported candidate texture stages".into());
                }
                for stage in &bindings.textures {
                    let expected = match stage.stage { 0 => 17, 1 => 19, _ => return Err("unsupported candidate texture stage".into()) };
                    if stage.raster_reference != expected || model.sections.get(expected as usize).map(|s| s.info.type_code) != Some(rw4::object_types::RASTER) {
                        return Err("candidate saved raster reference does not match raster section".into());
                    }
                    println!("CELL_KIND0 state_section={index} stage={} saved_raster_word={:08X} candidate_section={} list_a_flags={:08X} list_a={:?} list_b_flags={:08X} list_b={:?} runtime_pointer_fixup=unverified", stage.stage, stage.raster_reference, expected, stage.list_a_flags, stage.list_a, stage.list_b_flags, stage.list_b);
                }
            }
            rw4::object_types::MESH_COMPILED_STATE_LINK => {
                let raw = model.object(&model_bytes, index as u32).ok_or("invalid link section")?;
                if raw.len() % 4 != 0 { return Err("unaligned mesh/state link".into()); }
                let words: Vec<_> = raw.chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();
                println!("CELL_KIND0 mesh_state_link_section={index} raw_words={words:08X?} interpretation=unverified");
            }
            _ => {}
        }
    }
    println!("CELL_KIND0 raster_sections={raster_count} runtime_texture_binding=unverified");
    Ok(())
}
