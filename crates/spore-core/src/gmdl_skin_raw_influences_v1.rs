//! Lossless access to the measured UBYTE4 / UBYTE4N influence fields.
//!
//! Selection is in the referenced vertex buffer's index space, not the mesh's
//! triangle indices. Meshes may share buffers. No palette association, joint
//! conversion, weight normalization, or interpretation of inactive lanes occurs.
//! Selection follows D3D usage 2/0 and 1/0, as in `gmdl_skin`; it does not
//! require native `rw_decl` 14/15 or assert that those IDs establish a palette
//! mapping. The native semantic field is checked only for duplicate IDs.

use crate::gmdl::{Gmdl, VertexElement};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawInfluenceError {
    InvalidReference,
    InvalidDeclaration,
    InvalidBuffer,
    VertexOutOfBounds,
    AllocationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInfluence {
    pub vertex_index: usize,
    pub register_offsets: [u8; 4],
    pub weight_bytes: [u8; 4],
}

struct Layout<'a> {
    data: &'a [u8],
    start: usize,
    stride: usize,
    count: usize,
    joint: usize,
    weight: usize,
}

impl<'a> Layout<'a> {
    fn new(data: &'a [u8], model: &Gmdl, mesh_index: usize) -> Result<Self, RawInfluenceError> {
        use RawInfluenceError::*;
        let mesh = model.meshes.get(mesh_index).ok_or(InvalidReference)?;
        let vb = usize::try_from(mesh.vertex_index).ok()
            .and_then(|i| model.vertex_buffers.get(i)).ok_or(InvalidReference)?;
        let format = usize::try_from(vb.format_index).ok()
            .and_then(|i| model.vertex_formats.get(i)).ok_or(InvalidReference)?;
        // Reject ambiguity even when duplicate semantics appear on another stream.
        for (i, e) in format.elements.iter().enumerate() {
            if format.elements[..i].iter().any(|p|
                (p.usage == e.usage && p.usage_index == e.usage_index)
                    || (e.rw_decl >= 0 && p.rw_decl == e.rw_decl)) {
                return Err(InvalidDeclaration);
            }
        }
        let find = |usage| -> Result<&VertexElement, RawInfluenceError> {
            format.elements.iter().find(|e| e.usage == usage && e.usage_index == 0)
                .ok_or(InvalidDeclaration)
        };
        let joint = find(2)?;
        let weight = find(1)?;
        if joint.stream != 0 || weight.stream != 0 || joint.element_type != 5
            || weight.element_type != 8 || joint.offset < 0 || weight.offset < 0 {
            return Err(InvalidDeclaration);
        }
        let stride = format.elements.iter().try_fold(0usize, |n, e| {
            if !(0..=16).contains(&e.element_type) { return None; }
            n.checked_add(e.size())
        }).ok_or(InvalidDeclaration)?;
        if stride == 0 || stride != vb.stride { return Err(InvalidBuffer); }
        // The parser exposes a single buffer and computes its stride by summing
        // all elements. Other streams cannot safely be addressed by that stride.
        // Reject overlapping or out-of-stride extents for every attribute, not
        // only the fields being returned.
        for (i, e) in format.elements.iter().enumerate() {
            if e.stream != 0 || e.offset < 0 { return Err(InvalidDeclaration); }
            let start = e.offset as usize;
            let end = start.checked_add(e.size()).ok_or(InvalidDeclaration)?;
            if end > stride { return Err(InvalidDeclaration); }
            for previous in &format.elements[..i] {
                let previous_start = previous.offset as usize;
                let previous_end = previous_start.checked_add(previous.size()).ok_or(InvalidDeclaration)?;
                if start < previous_end && previous_start < end { return Err(InvalidDeclaration); }
            }
        }
        let j = joint.offset as usize;
        let w = weight.offset as usize;
        let je = j.checked_add(4).ok_or(InvalidDeclaration)?;
        let we = w.checked_add(4).ok_or(InvalidDeclaration)?;
        if je > stride || we > stride || (j < we && w < je) {
            return Err(InvalidDeclaration);
        }
        let count = usize::try_from(vb.vertex_count).map_err(|_| InvalidBuffer)?;
        let declared = usize::try_from(vb.buffer_size).map_err(|_| InvalidBuffer)?;
        let len = count.checked_mul(stride).ok_or(InvalidBuffer)?;
        let end = vb.data_offset.checked_add(len).ok_or(InvalidBuffer)?;
        if declared < len || data.get(vb.data_offset..end).is_none() {
            return Err(InvalidBuffer);
        }
        Ok(Self { data, start: vb.data_offset, stride, count, joint: j, weight: w })
    }

    fn read(&self, index: usize) -> Result<RawInfluence, RawInfluenceError> {
        if index >= self.count { return Err(RawInfluenceError::VertexOutOfBounds); }
        let at = self.start.checked_add(index.checked_mul(self.stride)
            .ok_or(RawInfluenceError::InvalidBuffer)?)
            .ok_or(RawInfluenceError::InvalidBuffer)?;
        let field = |offset: usize| -> Result<[u8; 4], RawInfluenceError> {
            let start = at.checked_add(offset).ok_or(RawInfluenceError::InvalidBuffer)?;
            let end = start.checked_add(4).ok_or(RawInfluenceError::InvalidBuffer)?;
            self.data.get(start..end).ok_or(RawInfluenceError::InvalidBuffer)?
                .try_into().map_err(|_| RawInfluenceError::InvalidBuffer)
        };
        Ok(RawInfluence { vertex_index: index, register_offsets: field(self.joint)?, weight_bytes: field(self.weight)? })
    }
}

/// Read explicitly selected indices, preserving their order and duplicates.
pub fn decode_indices(data: &[u8], model: &Gmdl, mesh_index: usize, indices: &[usize]) -> Result<Vec<RawInfluence>, RawInfluenceError> {
    let layout = Layout::new(data, model, mesh_index)?;
    if indices.iter().any(|&i| i >= layout.count) { return Err(RawInfluenceError::VertexOutOfBounds); }
    let mut rows = Vec::new();
    rows.try_reserve_exact(indices.len()).map_err(|_| RawInfluenceError::AllocationFailed)?;
    for &index in indices { rows.push(layout.read(index)?); }
    Ok(rows)
}

/// Read a caller-selected half-open range in the buffer's vertex index space.
pub fn decode_range(data: &[u8], model: &Gmdl, mesh_index: usize, range: Range<usize>) -> Result<Vec<RawInfluence>, RawInfluenceError> {
    let layout = Layout::new(data, model, mesh_index)?;
    if range.start > range.end || range.end > layout.count { return Err(RawInfluenceError::VertexOutOfBounds); }
    let mut rows = Vec::new();
    rows.try_reserve_exact(range.end - range.start).map_err(|_| RawInfluenceError::AllocationFailed)?;
    for index in range { rows.push(layout.read(index)?); }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gmdl::{Mesh, VertexBuffer, VertexElement, VertexFormat};
    use crate::Vec3;

    fn element(usage: i8, kind: i8, offset: i16, rw_decl: i32) -> VertexElement {
        VertexElement { stream: 0, offset, element_type: kind, method: 0, usage, usage_index: 0, rw_decl }
    }
    fn fixture() -> (Vec<u8>, Gmdl) {
        let data = vec![99, 1, 2, 3, 4, 0, 7, 0, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let model = Gmdl { version: 0, ref_count: 0, mesh_count: 2, bbox_min: Vec3::ZERO,
            bbox_max: Vec3::ZERO, radius: 0., triangle_buffers: vec![],
            vertex_formats: vec![VertexFormat { elements: vec![element(2, 5, 0, 3), element(1, 8, 4, 4)] }],
            vertex_buffers: vec![VertexBuffer { format_index: 0, vertex_count: 2, buffer_size: 16, stride: 8, data_offset: 1 }],
            meshes: vec![Mesh { triangle_index: 0, vertex_index: 0, material_id: 0 }, Mesh { triangle_index: 1, vertex_index: 0, material_id: 1 }],
            total_size: 17, consumed: 17, exact: true };
        (data, model)
    }
    #[test]
    fn shared_buffer_selection_preserves_every_byte() {
        let (d, m) = fixture();
        let rows = decode_indices(&d, &m, 1, &[1, 0, 1]).unwrap();
        assert_eq!(rows[1], RawInfluence { vertex_index: 0, register_offsets: [1, 2, 3, 4], weight_bytes: [0, 7, 0, 9] });
        assert_eq!(rows[0], rows[2]);
        assert_eq!(decode_range(&d, &m, 0, 0..2).unwrap(), vec![rows[1].clone(), rows[0].clone()]);
        assert!(decode_range(&d, &m, 0, 2..2).unwrap().is_empty());
    }
    #[test]
    fn rejects_ambiguous_declarations() {
        let (d, mut m) = fixture();
        let mut duplicate = element(2, 5, 0, 9); duplicate.stream = 1;
        m.vertex_formats[0].elements.push(duplicate);
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
        m.vertex_formats[0].elements.pop();
        m.vertex_formats[0].elements[1].rw_decl = 3;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
        m.vertex_formats[0].elements[1].rw_decl = 4;
        m.vertex_formats[0].elements[1].offset = 2;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
    }
    #[test]
    fn rejects_other_streams_and_overlapping_unrelated_attributes() {
        let (d, mut m) = fixture();
        let mut extra = element(0, 0, 8, 0);
        extra.stream = 1;
        m.vertex_formats[0].elements.push(extra);
        m.vertex_buffers[0].stride = 12;
        m.vertex_buffers[0].buffer_size = 24;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
        m.vertex_formats[0].elements[2].stream = 0;
        m.vertex_formats[0].elements[2].offset = 2;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
        m.vertex_formats[0].elements[2].offset = 10;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidDeclaration));
    }
    #[test]
    fn rejects_bad_ranges_references_and_buffers() {
        let (d, mut m) = fixture();
        assert_eq!(decode_range(&d, &m, 0, 1..3), Err(RawInfluenceError::VertexOutOfBounds));
        assert_eq!(decode_indices(&d, &m, 0, &[usize::MAX]), Err(RawInfluenceError::VertexOutOfBounds));
        assert_eq!(decode_indices(&d, &m, 2, &[]), Err(RawInfluenceError::InvalidReference));
        assert_eq!(decode_indices(&d[..16], &m, 0, &[]), Err(RawInfluenceError::InvalidBuffer));
        m.vertex_buffers[0].buffer_size = 15;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidBuffer));
        m.vertex_buffers[0].buffer_size = 16;
        m.vertex_buffers[0].data_offset = usize::MAX;
        assert_eq!(decode_indices(&d, &m, 0, &[]), Err(RawInfluenceError::InvalidBuffer));
    }
}
