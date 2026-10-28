//! Controlled inspection of a single candidate Cell's kind-0 source art.
//! This does not resolve live Cell selection, runtime pointers, or pose.
use crate::{assets::TextureData, source_cell::{CellRecord, CellStructure}};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{material_links, mesh::MeshData, prop, raster, rw4};
use std::{fs::File, io::Read, path::Path};

pub(super) const CELL_ID: u32 = 0xfd24_574b;
const MODEL_ID: u32 = 0xdae8_ce17;

pub(super) struct Kind0Art {
    pub mesh: MeshData,
    pub texture: TextureData,
    pub sampler: ImageSamplerDescriptor,
}
fn key(instance_id: u32, type_id: u32) -> NativeResourceKey {
    NativeResourceKey { instance_id, type_id, group_id: 0 }
}
fn package(path: &Path) -> Result<NativePackageIndex, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut raw = [0; dbpf::HEADER_SIZE];
    file.read_exact(&mut raw).map_err(|e| e.to_string())?;
    let header = dbpf::DbpfHeader::parse(&raw).map_err(|e| e.to_string())?;
    if header.index_size > 64 * 1024 * 1024 || header.index_entry_count > 2_000_000 {
        return Err("kind-0 package index exceeds limit".into());
    }
    // Reader reopens the path; installed files must remain trusted and stable.
    NativePackageIndex::open(path).map_err(|e| e.to_string())
}
fn read(package: &NativePackageIndex, key: NativeResourceKey) -> Result<Vec<u8>, String> {
    let entry = package.index().find_unique(key).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("missing explicit source key {key}"))?;
    if entry.stored_size > 16 * 1024 * 1024 || entry.decompressed_size > 16 * 1024 * 1024 {
        return Err("kind-0 source resource exceeds limit".into());
    }
    let data = package.read(entry).map_err(|e| e.to_string())?;
    if data.len() != entry.decompressed_size as usize { return Err("source decoded extent mismatch".into()); }
    println!("CELL_SOURCE_RESOURCE package={:?} key={key}", package.path());
    Ok(data)
}
fn check_structure(data: &[u8]) -> Result<(), String> {
    let structure = CellStructure::parse(data)?;
    if structure.entries.len() != 2 || structure.entries[0].kind != 0
        || structure.entries[0].reference_id != MODEL_ID || structure.entries[1].kind != 1 {
        return Err("unsupported candidate kind-0 structure".into());
    }
    println!("CELL_SOURCE_PENDING kind=1 raw_reference={:08X} resolved=false", structure.entries[1].reference_id);
    Ok(())
}
pub(super) fn load(install: &Path) -> Result<Kind0Art, String> {
    let cell_package = package(&install.join("Data/PatchData.package"))?;
    let asset_package = package(&install.join("Data/Spore_Graphics.package"))?;
    let cell = read(&cell_package, key(CELL_ID, 0xdfad_9f51))?;
    if CellRecord::parse(&cell)?.structure_id != CELL_ID { return Err("unexpected Cell structure reference".into()); }
    check_structure(&read(&cell_package, key(CELL_ID, 0x4b9e_f6dc))?)?;
    let bytes = read(&asset_package, key(MODEL_ID, 0x00b1_b104))?;
    let tree = prop::parse(&bytes).map_err(|e| e.to_string())?;
    if !tree.exact || tree.consumed != bytes.len()
        || tree.key(prop::idents::MODEL_LOD0) != Some((u64::from(MODEL_ID), 0x2f4e_681b)) {
        return Err("unsupported candidate PROP LOD0".into());
    }
    let bytes = read(&asset_package, key(MODEL_ID, 0x2f4e_681b))?;
    let model = rw4::parse(&bytes).map_err(|e| e.to_string())?;
    let kind = |index: usize| model.sections.get(index).map(|s| s.info.type_code);
    if kind(13) != Some(rw4::object_types::MESH) || kind(14) != Some(rw4::object_types::COMPILED_STATE)
        || kind(15) != Some(rw4::object_types::MESH_COMPILED_STATE_LINK)
        || kind(17) != Some(rw4::object_types::RASTER) || kind(19) != Some(rw4::object_types::RASTER) {
        return Err("unsupported candidate RW4 sections".into());
    }
    let link = model.object(&bytes, 15).ok_or("missing mesh-state link")?;
    if link.len() != 12 || link.chunks_exact(4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect::<Vec<_>>() != [13, 1, 14] {
        return Err("unsupported candidate mesh-state link".into());
    }
    if model.sections.iter().filter(|s| s.info.type_code == rw4::object_types::MESH).count() != 1
        || model.meshes(&bytes).len() != 1 { return Err("unsupported candidate mesh count".into()); }
    let state = material_links::compiled_state(&model, &bytes, 14).map_err(|e| e.to_string())?;
    if state.shader_id != 0x8000_0004 { return Err("unexpected candidate shader identity".into()); }
    let bindings = state.bindings().map_err(|e| e.to_string())?;
    if bindings.textures.len() != 2 { return Err("unsupported candidate saved texture stages".into()); }
    let stage = |id| -> Result<&material_links::TextureStage, String> {
        let mut entries = bindings.textures.iter().filter(|s| s.stage == id);
        let one = entries.next().ok_or("candidate stage absent")?;
        if entries.next().is_some() { return Err("duplicate candidate stage".into()); }
        Ok(one)
    };
    let stage0 = stage(0)?;
    if stage0.raster_reference != 17 || stage(1)?.raster_reference != 19 {
        return Err("candidate saved raster references differ".into());
    }
    // Translate only the measured saved stage-zero sampler state. PBR remains
    // an inspection approximation; effective runtime sampler may differ.
    let pairs: Vec<_> = stage0.list_b.iter().map(|p| (p.state, p.value)).collect();
    if pairs != [(1, 1), (2, 1), (5, 2), (6, 2), (7, 1)] {
        return Err("unsupported candidate saved sampler".into());
    }
    let sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest, ..Default::default()
    };
    let raster = model.raster(&bytes, 17).ok_or("invalid saved stage-zero raster")?;
    if raster.width != 128 || raster.height != 128 || raster.mipmap_levels != 8 || raster.format != raster::FORMAT_DXT5
        || raster.data != 16 || kind(16) != Some(rw4::object_types::BASE_RESOURCE) {
        return Err("unsupported candidate stage-zero raster".into());
    }
    let blocks = model.object(&bytes, raster.data).ok_or("missing raster data")?;
    let rgba = raster::decode_pixels(raster.format, raster.width, raster.height, blocks).map_err(|e| e.to_string())?;
    let geometry = spore_core::mesh::from_rw4(&bytes, &model).map_err(|e| e.to_string())?;
    geometry.validate().map_err(|e| e.to_string())?;
    println!("CELL_SOURCE_KIND0 cell=FD24574B model=DAE8CE17 assignment=controlled_prototype_player renderer=pbr static=true runtime_fixup=unverified pose=unverified topology_completeness=unverified saved_stage0=17 inspection_base_color=17 saved_stage1=19 stage1_rendered=false kind1_effect_rendered=false camera_scale=inspection alpha=inspection_opaque color_space=inspection_srgb mip_chain=base_only");
    Ok(Kind0Art { mesh: geometry, texture: TextureData { width: raster.width, height: raster.height, rgba }, sampler })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structure_rejects_missing_or_changed_candidate_rows() {
        let mut data = vec![0u8; 108];
        data[20..24].copy_from_slice(&28u32.to_le_bytes());
        data[24..28].copy_from_slice(&2u32.to_le_bytes());
        data[44..48].copy_from_slice(&MODEL_ID.to_le_bytes());
        data[72..76].copy_from_slice(&1u32.to_le_bytes());
        assert!(check_structure(&data).is_ok());
        data[44] ^= 1;
        assert!(check_structure(&data).is_err());
        assert!(check_structure(&data[..107]).is_err());
    }
}
