//! Position-only diagnostic using explicitly supplied synthetic palettes.
//!
//! The direct-draw association is conditional on the caller's path assertion.
//! The supplied matrices are explicit diagnostic inputs, not an actor pose or
//! original clip. This does not reproduce the native CPU path: its palette
//! base relationship and exact arithmetic are not established. Normals and
//! tangents remain unchanged. Never feed this output to a skinning shader.

use crate::gmdl::Gmdl;
use crate::gmdl_direct_draw_influences_v1::{self, DrawPath, ResolveError};
use crate::gmdl_skin::GmdlSkin;

/// Three explicit affine rows; local register offset 3*n selects matrix n.
pub type Matrix = [[f32; 4]; 3];

#[derive(Debug, PartialEq)]
pub enum DiagnosticError {
    Resolve(ResolveError),
    PositionDeclaration,
    InfluenceDeclaration,
    BufferExtent,
    PaletteExtent,
    NonFiniteMatrix,
    NonFinitePosition,
    NonFiniteResult,
    WeightSum { vertex: usize, sum: u16 },
}

/// Returns a separate copy of the entire selected vertex buffer for this draw.
/// Only positions of referenced vertices are modified. `palette` is indexed
/// by descriptor first word plus the active raw register offset divided by 3;
/// inactive indices are never read. A sum of 255 is required rather than
/// inventing a normalization rule for other source data.
pub fn deform_positions(
    data: &[u8], model: &Gmdl, skin: &GmdlSkin, mesh_ordinal: usize,
    path: DrawPath, palette: &[Matrix],
) -> Result<Vec<u8>, DiagnosticError> {
    use DiagnosticError::*;
    let draw = gmdl_direct_draw_influences_v1::resolve(data, model, skin, mesh_ordinal, path)
        .map_err(Resolve)?;
    let mesh = model.meshes.get(mesh_ordinal).ok_or(BufferExtent)?;
    let vb = model.vertex_buffers.get(usize::try_from(mesh.vertex_index).map_err(|_| BufferExtent)?)
        .ok_or(BufferExtent)?;
    let fmt = model.vertex_formats.get(usize::try_from(vb.format_index).map_err(|_| BufferExtent)?)
        .ok_or(BufferExtent)?;
    let mut positions = fmt.elements.iter().filter(|e| e.usage == 0 && e.usage_index == 0);
    let pos = positions.next().ok_or(PositionDeclaration)?;
    if positions.next().is_some() || pos.stream != 0 || pos.element_type != 2 || pos.method != 0 || pos.offset < 0 {
        return Err(PositionDeclaration);
    }
    // The resolver validates unique UBYTE4/UBYTE4N semantics and their
    // extents, but does not enforce D3DDECLMETHOD_DEFAULT.
    for usage in [1, 2] {
        let element = fmt.elements.iter().find(|e| e.usage == usage && e.usage_index == 0)
            .ok_or(InfluenceDeclaration)?;
        if element.method != 0 { return Err(InfluenceDeclaration); }
    }
    let offset = pos.offset as usize;
    if offset.checked_add(12).filter(|&v| v <= vb.stride).is_none() {
        return Err(PositionDeclaration);
    }
    let size = usize::try_from(vb.buffer_size).map_err(|_| BufferExtent)?;
    let end = vb.data_offset.checked_add(size).ok_or(BufferExtent)?;
    let mut output = data.get(vb.data_offset..end).ok_or(BufferExtent)?.to_vec();
    let base = usize::try_from(draw.descriptor.first_word).map_err(|_| PaletteExtent)?;
    for influence in &draw.referenced {
        let sum: u16 = influence.weight_bytes.iter().map(|&w| u16::from(w)).sum();
        if sum != 255 { return Err(WeightSum { vertex: influence.vertex_index, sum }); }
        let at = influence.vertex_index.checked_mul(vb.stride)
            .and_then(|x| x.checked_add(offset)).ok_or(BufferExtent)?;
        let position_end = at.checked_add(12).ok_or(BufferExtent)?;
        let bytes = output.get(at..position_end).ok_or(BufferExtent)?;
        let mut p = [0.0f32; 4];
        for i in 0..3 {
            p[i] = f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().map_err(|_| BufferExtent)?);
        }
        if p[..3].iter().any(|v| !v.is_finite()) { return Err(NonFinitePosition); }
        p[3] = 1.0;
        let mut result = [0.0f32; 3];
        for lane in 0..4 {
            let weight = influence.weight_bytes[lane];
            if weight == 0 { continue; }
            let index = base.checked_add(usize::from(influence.register_offsets[lane] / 3))
                .ok_or(PaletteExtent)?;
            let matrix = palette.get(index).ok_or(PaletteExtent)?;
            if matrix.iter().flatten().any(|v| !v.is_finite()) { return Err(NonFiniteMatrix); }
            for row in 0..3 {
                let transformed = (0..4).map(|column| matrix[row][column] * p[column]).sum::<f32>();
                result[row] += transformed * (f32::from(weight) / 255.0);
            }
        }
        if result.iter().any(|v| !v.is_finite()) { return Err(NonFiniteResult); }
        for i in 0..3 { output[at + i * 4..at + i * 4 + 4].copy_from_slice(&result[i].to_le_bytes()); }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gmdl::{Mesh, TriangleBuffer, VertexBuffer, VertexElement, VertexFormat};
    use crate::gmdl_skin::GmdlPalette;
    use crate::Vec3;

    fn fixture() -> (Vec<u8>, Gmdl, GmdlSkin) {
        let mut data = Vec::new();
        for (p, indices, weights) in [([1.0f32, 2.0, 3.0], [0u8, 255, 0, 0], [255u8, 0, 0, 0]),
            ([4.0, 5.0, 6.0], [0, 255, 0, 0], [255, 0, 0, 0])] {
            for x in p { data.extend_from_slice(&x.to_le_bytes()); }
            data.extend_from_slice(&indices); data.extend_from_slice(&weights);
        }
        let element = |usage, kind, offset, rw_decl| VertexElement { stream: 0, offset, element_type: kind, method: 0, usage, usage_index: 0, rw_decl };
        let tri = |v| TriangleBuffer { primitive_type: 4, indices_count: 3, indices_bits: 16, buffer_size: 6, indices: vec![v, v, v] };
        let model = Gmdl { version: 8, ref_count: 0, mesh_count: 2, bbox_min: Vec3::ZERO, bbox_max: Vec3::ZERO, radius: 0.,
            triangle_buffers: vec![tri(0), tri(1)], vertex_formats: vec![VertexFormat { elements: vec![element(0, 2, 0, 0), element(2, 5, 12, 14), element(1, 8, 16, 15)] }],
            vertex_buffers: vec![VertexBuffer { format_index: 0, vertex_count: 2, buffer_size: 40, stride: 20, data_offset: 0 }],
            meshes: vec![Mesh { triangle_index: 0, vertex_index: 0, material_id: 0 }, Mesh { triangle_index: 1, vertex_index: 0, material_id: 0 }], total_size: 40, consumed: 40, exact: true };
        let skin = GmdlSkin { source_offset: 0, materials: vec![], palettes: vec![GmdlPalette { first_word: 0, bone_count: 1 }, GmdlPalette { first_word: 64, bone_count: 1 }], parts: vec![], footer: [0; 3] };
        (data, model, skin)
    }
    const PATH: DrawPath = DrawPath::DirectFresh769a80Unmodified;
    const ID: Matrix = [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]];

    #[test]
    fn identity_and_shared_buffer_draws_are_separate() {
        let (data, model, skin) = fixture();
        let mut palette = vec![ID; 65];
        assert_eq!(deform_positions(&data, &model, &skin, 0, PATH, &palette).unwrap(), data);
        assert_eq!(deform_positions(&data, &model, &skin, 1, PATH, &palette).unwrap(), data);
        palette[0][0][3] = 10.; palette[64][1][3] = -3.;
        let first = deform_positions(&data, &model, &skin, 0, PATH, &palette).unwrap();
        let second = deform_positions(&data, &model, &skin, 1, PATH, &palette).unwrap();
        let mut expected_first = data.clone(); expected_first[0..4].copy_from_slice(&11f32.to_le_bytes());
        let mut expected_second = data.clone(); expected_second[24..28].copy_from_slice(&2f32.to_le_bytes());
        assert_eq!(first, expected_first); assert_eq!(second, expected_second);
    }
    #[test]
    fn rejects_short_and_nonfinite_palettes_and_bad_weights() {
        let (mut data, model, skin) = fixture();
        assert_eq!(deform_positions(&data, &model, &skin, 1, PATH, &[ID; 64]), Err(DiagnosticError::PaletteExtent));
        let mut palette = vec![ID; 65]; palette[64][0][0] = f32::NAN;
        assert_eq!(deform_positions(&data, &model, &skin, 1, PATH, &palette), Err(DiagnosticError::NonFiniteMatrix));
        data[16] = 254;
        assert_eq!(deform_positions(&data, &model, &skin, 0, PATH, &palette[..64]), Err(DiagnosticError::WeightSum { vertex: 0, sum: 254 }));
    }
    #[test]
    fn weighted_translation_uses_each_raw_register_offset() {
        let (mut data, model, mut skin) = fixture();
        data[12..16].copy_from_slice(&[0, 3, 255, 255]);
        data[16..20].copy_from_slice(&[128, 127, 0, 0]);
        skin.palettes[0].bone_count = 2;
        let mut palette = vec![ID; 2];
        palette[1][0][3] = 255.;
        let output = deform_positions(&data, &model, &skin, 0, PATH, &palette).unwrap();
        let x = f32::from_le_bytes(output[0..4].try_into().unwrap());
        // Independent scalar expression: 1 * 128/255 + 256 * 127/255.
        assert!((x - (128.0 / 255.0 + 256.0 * 127.0 / 255.0)).abs() < 0.0001);
        assert_eq!(&output[4..], &data[4..]);
    }
    #[test]
    fn ignores_unused_nonfinite_matrices_and_rejects_nondefault_methods() {
        let (data, mut model, skin) = fixture();
        let mut palette = vec![ID; 65];
        palette[64][0][0] = f32::NAN;
        assert_eq!(deform_positions(&data, &model, &skin, 0, PATH, &palette).unwrap(), data);
        model.vertex_formats[0].elements[0].method = 1;
        assert_eq!(deform_positions(&data, &model, &skin, 0, PATH, &palette), Err(DiagnosticError::PositionDeclaration));
        model.vertex_formats[0].elements[0].method = 0;
        for i in [1, 2] {
            model.vertex_formats[0].elements[i].method = 1;
            assert_eq!(deform_positions(&data, &model, &skin, 0, PATH, &palette), Err(DiagnosticError::InfluenceDeclaration));
            model.vertex_formats[0].elements[i].method = 0;
        }
    }
}
