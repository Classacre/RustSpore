//! Conditional direct-draw descriptor selection and influence compatibility.
//!
//! This bounded helper is not a renderer or actor pose. The
//! caller must attest to the synchronous 769A80 load, fresh graphics object,
//! unchanged transferred vectors, and execution of the direct 73FD30 path.
//! Other render paths and modified objects do not have this ordinal guarantee.
//! The descriptor's second word is used only for the observed compatibility
//! test; its interpretation by the ultimate state consumer remains unknown.

use crate::gmdl::Gmdl;
use crate::gmdl_skin::{GmdlPalette, GmdlSkin};
use crate::gmdl_skin_raw_influences_v1::{self, RawInfluence, RawInfluenceError};

/// A caller assertion, not a fact established by this helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPath {
    DirectFresh769a80Unmodified,
    OtherOrUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    PathNotAdmitted,
    InvalidMesh,
    InvalidVertexBuffer,
    MissingDescriptor,
    InvalidIndexBuffer,
    OffsetOverflow,
    Influence(RawInfluenceError),
    IncompatibleActiveOffset { vertex: usize, lane: usize, offset: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectDrawInfluences {
    pub mesh_ordinal: usize,
    pub descriptor: GmdlPalette,
    /// Byte offset into the selected native base; it does not identify the base.
    pub base_offset_bytes: u32,
    /// Unique triangle-referenced vertices in ascending vertex-buffer order.
    pub referenced: Vec<RawInfluence>,
}

/// Resolve one direct draw. `data`, `model`, and `skin` must describe the same
/// source asset. Only referenced vertices are checked, including when draws share
/// a vertex buffer. Inactive lanes are returned intact and are not validated.
/// A successful compatibility check does not prove register remapping, a GPU
/// upload, or an actor palette. No matrices or pose are generated.
pub fn resolve(
    data: &[u8], model: &Gmdl, skin: &GmdlSkin, mesh_ordinal: usize, path: DrawPath,
) -> Result<DirectDrawInfluences, ResolveError> {
    use ResolveError::*;
    if path != DrawPath::DirectFresh769a80Unmodified { return Err(PathNotAdmitted); }
    let mesh = model.meshes.get(mesh_ordinal).ok_or(InvalidMesh)?;
    let vb = usize::try_from(mesh.vertex_index).ok()
        .and_then(|i| model.vertex_buffers.get(i)).ok_or(InvalidVertexBuffer)?;
    let vertex_bytes = usize::try_from(vb.vertex_count).ok()
        .and_then(|count| count.checked_mul(vb.stride)).ok_or(InvalidVertexBuffer)?;
    if usize::try_from(vb.buffer_size).ok() != Some(vertex_bytes) {
        return Err(InvalidVertexBuffer);
    }
    let descriptor = *skin.palettes.get(mesh_ordinal).ok_or(MissingDescriptor)?;
    let base_offset_bytes = descriptor.first_word.checked_mul(48).ok_or(OffsetOverflow)?;
    let tri = usize::try_from(mesh.triangle_index).ok()
        .and_then(|i| model.triangle_buffers.get(i)).ok_or(InvalidIndexBuffer)?;
    let width = match tri.indices_bits { 16 => 2usize, 32 => 4, _ => return Err(InvalidIndexBuffer) };
    let count = usize::try_from(tri.indices_count).map_err(|_| InvalidIndexBuffer)?;
    let expected = count.checked_mul(width).ok_or(InvalidIndexBuffer)?;
    if tri.primitive_type != 4 || count != tri.indices.len() || count % 3 != 0
        || usize::try_from(tri.buffer_size).ok() != Some(expected)
        || (width == 2 && tri.indices.iter().any(|&i| i > u16::MAX as u32)) {
        return Err(InvalidIndexBuffer);
    }
    let mut indices = Vec::new();
    indices.try_reserve_exact(count).map_err(|_| Influence(RawInfluenceError::AllocationFailed))?;
    for &index in &tri.indices {
        indices.push(usize::try_from(index).map_err(|_| InvalidIndexBuffer)?);
    }
    indices.sort_unstable();
    indices.dedup();
    let referenced = gmdl_skin_raw_influences_v1::decode_indices(data, model, mesh_ordinal, &indices)
        .map_err(Influence)?;
    for row in &referenced {
        for lane in 0..4 {
            let offset = row.register_offsets[lane];
            if row.weight_bytes[lane] != 0
                && (offset % 3 != 0 || u32::from(offset / 3) >= descriptor.bone_count) {
                return Err(IncompatibleActiveOffset { vertex: row.vertex_index, lane, offset });
            }
        }
    }
    Ok(DirectDrawInfluences { mesh_ordinal, descriptor, base_offset_bytes, referenced })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gmdl::{Mesh, TriangleBuffer, VertexBuffer, VertexElement, VertexFormat};
    use crate::Vec3;

    fn fixture() -> (Vec<u8>, Gmdl, GmdlSkin) {
        let element = |usage, kind, offset, rw_decl| VertexElement {
            stream: 0, offset, element_type: kind, method: 0, usage, usage_index: 0, rw_decl,
        };
        // Two draws share a buffer but reference disjoint vertices. Inactive
        // byte 255 is deliberately not compatible and must remain untouched.
        let data = vec![189, 255, 0, 0, 255, 0, 0, 0, 54, 255, 0, 0, 255, 0, 0, 0];
        let tri = |v| TriangleBuffer { primitive_type: 4, indices_count: 3, indices_bits: 16,
            buffer_size: 6, indices: vec![v, v, v] };
        let model = Gmdl { version: 8, ref_count: 0, mesh_count: 2, bbox_min: Vec3::ZERO,
            bbox_max: Vec3::ZERO, radius: 0., triangle_buffers: vec![tri(0), tri(1)],
            vertex_formats: vec![VertexFormat { elements: vec![element(2, 5, 0, 3), element(1, 8, 4, 4)] }],
            vertex_buffers: vec![VertexBuffer { format_index: 0, vertex_count: 2, buffer_size: 16, stride: 8, data_offset: 0 }],
            meshes: vec![Mesh { triangle_index: 0, vertex_index: 0, material_id: 0 },
                Mesh { triangle_index: 1, vertex_index: 0, material_id: 0 }],
            total_size: 16, consumed: 16, exact: true };
        let skin = GmdlSkin { source_offset: 0, materials: vec![], palettes: vec![
            GmdlPalette { first_word: 0, bone_count: 64 }, GmdlPalette { first_word: 64, bone_count: 19 }],
            parts: vec![], footer: [0; 3] };
        (data, model, skin)
    }
    const ADMITTED: DrawPath = DrawPath::DirectFresh769a80Unmodified;

    #[test]
    fn shared_buffer_uses_only_draw_references_and_preserves_inactive_bytes() {
        let (data, model, skin) = fixture();
        let first = resolve(&data, &model, &skin, 0, ADMITTED).unwrap();
        let second = resolve(&data, &model, &skin, 1, ADMITTED).unwrap();
        assert_eq!(first.referenced.len(), 1);
        assert_eq!(second.referenced.len(), 1);
        assert_eq!(second.base_offset_bytes, 3072);
        assert_eq!(second.referenced[0].vertex_index, 1);
        assert_eq!(second.referenced[0].register_offsets, [54, 255, 0, 0]);
        assert_eq!(second.descriptor.bone_count, 19);
    }
    #[test]
    fn requires_path_and_descriptor_and_rejects_bad_active_offsets() {
        let (mut data, model, mut skin) = fixture();
        assert_eq!(resolve(&data, &model, &skin, 0, DrawPath::OtherOrUnknown), Err(ResolveError::PathNotAdmitted));
        skin.palettes.pop();
        assert_eq!(resolve(&data, &model, &skin, 1, ADMITTED), Err(ResolveError::MissingDescriptor));
        skin.palettes.push(GmdlPalette { first_word: 64, bone_count: 19 });
        data[8] = 57;
        assert_eq!(resolve(&data, &model, &skin, 1, ADMITTED), Err(ResolveError::IncompatibleActiveOffset { vertex: 1, lane: 0, offset: 57 }));
        data[8] = 53;
        assert!(matches!(resolve(&data, &model, &skin, 1, ADMITTED), Err(ResolveError::IncompatibleActiveOffset { .. })));
    }
    #[test]
    fn rejects_bad_index_extents_and_offsets() {
        let (data, mut model, mut skin) = fixture();
        model.triangle_buffers[1].indices[0] = 2;
        assert_eq!(resolve(&data, &model, &skin, 1, ADMITTED), Err(ResolveError::Influence(RawInfluenceError::VertexOutOfBounds)));
        model.triangle_buffers[1].buffer_size = 5;
        assert_eq!(resolve(&data, &model, &skin, 1, ADMITTED), Err(ResolveError::InvalidIndexBuffer));
        model.vertex_buffers[0].buffer_size = 17;
        assert_eq!(resolve(&data, &model, &skin, 0, ADMITTED), Err(ResolveError::InvalidVertexBuffer));
        model.vertex_buffers[0].buffer_size = 16;
        skin.palettes[0].first_word = u32::MAX;
        assert_eq!(resolve(&data, &model, &skin, 0, ADMITTED), Err(ResolveError::OffsetOverflow));
    }
}
