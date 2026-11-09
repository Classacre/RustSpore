//! Controlled inspection of a single candidate Cell's kind-0 source art.
//! This does not resolve live Cell selection, runtime pointers, or pose.
use crate::{assets::{TextureData, TextureMipChain}, source_cell::{CellRecord, CellStructure}};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use spore_core::{material_links, mesh::MeshData, prop, raster, rw4};
use std::{fs::File, io::Read, path::Path};

pub(super) const CELL_ID: u32 = 0xfd24_574b;
const MODEL_ID: u32 = 0xdae8_ce17;

pub(super) struct Kind0Art {
    pub mesh: MeshData,
    pub texture: TextureData,
    pub mips: TextureMipChain,
    pub sampler: ImageSamplerDescriptor,
    pub skin_lanes: Option<SkinLanes>,
}
pub(super) struct SkinLanes { pub indices: Vec<[f32; 4]>, pub weights: Vec<[f32; 4]> }

fn decode_skin_rows(data: &[u8]) -> Result<SkinLanes, String> {
    if data.len() != 454 * 36 { return Err("unexpected candidate skin vertex extent".into()); }
    let mut indices = Vec::with_capacity(454);
    let mut weights = Vec::with_capacity(454);
    for v in data.chunks_exact(36) {
        if v[28..32].iter().any(|&x| x > 24 || x % 3 != 0)
            || v[35] != 0 || v[32..35].iter().map(|&x| u32::from(x)).sum::<u32>() != 255 {
            return Err("unsupported candidate skin indices or weights".into());
        }
        indices.push([v[28] as f32, v[29] as f32, v[30] as f32, v[31] as f32]);
        weights.push([v[32] as f32 / 255.0, v[33] as f32 / 255.0, v[34] as f32 / 255.0, v[35] as f32 / 255.0]);
    }
    Ok(SkinLanes { indices, weights })
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
// RW4 BASE_RESOURCE stores the DXT5 levels consecutively, without the
// per-level length prefixes used by standalone RASTER resources.
fn decode_mips(raster: &rw4::Raster, blocks: &[u8]) -> Result<TextureMipChain, String> {
    if raster.width != 128 || raster.height != 128 || raster.mipmap_levels != 8
        || raster.format != raster::FORMAT_DXT5 {
        return Err("unsupported candidate mip metadata".into());
    }
    let mut levels = Vec::with_capacity(raster.mipmap_levels as usize);
    let mut offset = 0usize;
    for level in 0..raster.mipmap_levels {
        let width = (raster.width >> level).max(1);
        let height = (raster.height >> level).max(1);
        let size = (width as usize).div_ceil(4) * (height as usize).div_ceil(4) * 16;
        let end = offset.checked_add(size).ok_or("candidate mip extent overflow")?;
        let source = blocks.get(offset..end).ok_or("truncated candidate mip chain")?;
        let rgba = raster::decode_pixels(raster.format, width, height, source).map_err(|e| e.to_string())?;
        if rgba.len() != width as usize * height as usize * 4 {
            return Err("candidate mip decoded extent mismatch".into());
        }
        levels.push(TextureData { width, height, rgba });
        offset = end;
    }
    if offset != blocks.len() { return Err("trailing candidate mip bytes".into()); }
    Ok(TextureMipChain { levels })
}
pub(super) fn load(install: &Path, require_skin: bool) -> Result<Kind0Art, String> {
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
    let mut sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat, address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest, lod_min_clamp: 0.0, ..Default::default()
    };
    let raster = model.raster(&bytes, 17).ok_or("invalid saved stage-zero raster")?;
    if raster.width != 128 || raster.height != 128 || raster.mipmap_levels != 8 || raster.format != raster::FORMAT_DXT5
        || raster.data != 16 || kind(16) != Some(rw4::object_types::BASE_RESOURCE) {
        return Err("unsupported candidate stage-zero raster".into());
    }
    let blocks = model.object(&bytes, raster.data).ok_or("missing raster data")?;
    let rgba = raster::decode_pixels(raster.format, raster.width, raster.height, blocks).map_err(|e| e.to_string())?;
    let mips = decode_mips(&raster, blocks)?;
    sampler.lod_max_clamp = mips.levels.len().saturating_sub(1) as f32;
    let base = mips.levels.first().ok_or("candidate mip chain empty")?;
    if base.width != raster.width || base.height != raster.height || base.rgba != rgba {
        return Err("candidate base mip differs from source texture".into());
    }
    let geometry = spore_core::mesh::from_rw4(&bytes, &model).map_err(|e| e.to_string())?;
    geometry.validate().map_err(|e| e.to_string())?;
    let skin_lanes = if require_skin {
        let m = model.mesh(&bytes, 13).ok_or("candidate mesh missing")?;
        let vb = model.vertex_buffer(&bytes, 4).ok_or("candidate vertex buffer missing")?;
        let ib = model.index_buffer(&bytes, 12).ok_or("candidate index buffer missing")?;
        let desc = model.vertex_description(&bytes, 7).ok_or("candidate declaration missing")?;
        let expected = [(0,0,2,0,0,0,0),(0,12,5,0,3,0,2),(0,16,5,0,6,0,19),(0,20,1,0,5,0,6),(0,28,5,0,2,0,14),(0,32,8,0,1,0,15)];
        if m.primitive_type != 4 || m.index_buffer != 12 || m.triangle_count != 896 || m.first_index != 0 || m.primitive_count != 2688 || m.first_vertex != 0 || m.vertex_count != 454 || m.vertex_buffers != [4]
            || vb.description != 7 || vb.data != 3 || vb.base_vertex_index != 0 || vb.vertex_count != 454 || vb.vertex_size != 36
            || ib.start_index != 0 || ib.primitive_count != 2688 || ib.format != 101 || ib.primitive_type != 4 || ib.data != 11
            || desc.vertex_size != 36 || desc.elements.len() != expected.len()
            || desc.elements.iter().zip(expected).any(|(e,x)| (e.stream,e.offset,e.element_type,e.method,e.usage,e.usage_index,e.rw_decl) != x) {
            return Err("unsupported candidate skin declaration or mesh".into());
        }
        let raw = model.object(&bytes, 3).ok_or("candidate vertices missing")?;
        let lanes = decode_skin_rows(raw)?;
        let raw_indices = model.object(&bytes, 11).ok_or("candidate indices missing")?;
        if geometry.positions.len() != 454 || geometry.uvs.len() != 454 || geometry.indices.len() != 2688 || geometry.sanitised != 0 || raw_indices.len() != 2688 * 2 {
            return Err("candidate skin geometry extent differs".into());
        }
        for (i, row) in raw.chunks_exact(36).enumerate() {
            let f = |at| f32::from_le_bytes(row[at..at+4].try_into().unwrap());
            let p = geometry.positions[i]; let uv = geometry.uvs[i];
            if [p.x.to_bits(),p.y.to_bits(),p.z.to_bits(),uv[0].to_bits(),uv[1].to_bits()] != [f(0).to_bits(),f(4).to_bits(),f(8).to_bits(),f(20).to_bits(),f(24).to_bits()] {
                return Err("candidate skin vertex projection differs".into());
            }
        }
        if raw_indices.chunks_exact(2).map(|b| u16::from_le_bytes([b[0],b[1]]) as u32).ne(geometry.indices.iter().copied()) {
            return Err("candidate skin index projection differs".into());
        }
        Some(lanes)
    } else { None };
    println!("CELL_SOURCE_KIND0 cell=FD24574B model=DAE8CE17 assignment=controlled_prototype_player renderer=caller_selected static=true runtime_fixup=unverified pose=unverified topology_completeness=unverified saved_stage0=17 inspection_base_color=17 saved_stage1=19 stage1_rendered=false kind1_effect_rendered=false camera_scale=inspection mip_chain=declared levels=8");
    Ok(Kind0Art { mesh: geometry, texture: TextureData { width: raster.width, height: raster.height, rgba }, mips, sampler, skin_lanes })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skin_rows_reject_unadmitted_indices_and_weights() {
        let mut raw = vec![0u8; 454 * 36];
        for row in raw.chunks_exact_mut(36) { row[32] = 255; }
        assert_eq!(decode_skin_rows(&raw).unwrap().weights[0], [1.0,0.0,0.0,0.0]);
        raw[28] = 1; assert!(decode_skin_rows(&raw).is_err());
        raw[28] = 0; raw[35] = 1; assert!(decode_skin_rows(&raw).is_err());
    }
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
    #[test]
    fn rw4_mips_decode_separate_levels_and_require_exact_extent() {
        let raster = rw4::Raster { format: raster::FORMAT_DXT5, width: 128, height: 128, mipmap_levels: 8, data: 16 };
        let mut blocks = Vec::new();
        for level in 0..8 {
            let dimension = (128usize >> level).max(1);
            let count = dimension.div_ceil(4).pow(2);
            // A valid DXT5 block with a distinct constant alpha per level.
            for _ in 0..count {
                let mut block = [0u8; 16];
                block[0] = level as u8 * 30;
                block[1] = block[0];
                blocks.extend_from_slice(&block);
            }
        }
        let chain = decode_mips(&raster, &blocks).unwrap();
        assert_eq!(chain.levels.len(), 8);
        for (level, image) in chain.levels.iter().enumerate() {
            assert_eq!(image.width, (128u32 >> level).max(1));
            assert_eq!(image.height, image.width);
            assert!(image.rgba.chunks_exact(4).all(|pixel| pixel[3] == level as u8 * 30));
        }
        assert!(decode_mips(&raster, &blocks[..blocks.len() - 1]).is_err());
        blocks.push(0);
        assert!(decode_mips(&raster, &blocks).is_err());
        let mut bad = raster;
        bad.mipmap_levels = 7;
        assert!(decode_mips(&bad, &blocks).is_err());
    }

}
