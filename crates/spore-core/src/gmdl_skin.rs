//! Lossless typed GMDL trailing records and declaration-stored skin influences.
//!
//! The existing geometry parser intentionally skips this tail. This companion
//! decoder retains all source records without inventing a hierarchy from fields
//! that can contain saved process addresses. `GmdlBoneRecord` is deliberately raw:
//! a valid tail does not yet imply a usable assembled animation skeleton.

use crate::{gmdl, rw4_animation::VertexSkinning};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GmdlSkinError {
    Truncated,
    InvalidCount,
    UnsupportedShaderData(u32),
    InvalidTail,
    InvalidReference,
    UnsupportedDeclaration,
    InvalidInfluence,
    UnsupportedPaletteMapping,
    UnsupportedVersion(i32),
}
impl fmt::Display for GmdlSkinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GMDL skin: {self:?}")
    }
}
impl std::error::Error for GmdlSkinError {}

/// The serialized 0x38-byte Transform layout, stored without applying flags.
/// Rotation multiplication convention and the relationship of the two per-part
/// transforms must be established before they are used to deform a creature.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GmdlTransform {
    pub flags: u16,
    pub transform_count: u16,
    pub translation: [f32; 3],
    pub scale: f32,
    pub rotation: [[f32; 3]; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmdlShaderData {
    pub id: u32,
    /// Exact source bytes after the shader-data identifier, including the texture
    /// count for 0x20D. Texture-slot roles are not inferred from record order.
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmdlMaterial {
    /// Version9 adds a big-endian presence word. Zero denotes an empty material
    /// slot with no following shader-data count; nonzero precedes the count.
    /// Version8 has no such word.
    pub version9_prefix: Option<u32>,
    pub shader_data: Vec<GmdlShaderData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GmdlPalette {
    /// First word of a source 8-byte palette descriptor. Do not assume it names
    /// a mesh rather than a vertex buffer or another source slot.
    pub first_word: u32,
    /// Second word. Measured against declaration indices and per-part counts.
    pub bone_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmdlBoneRecord {
    /// All 0xA0 source bytes, as 40 little-endian words. Padding and unknown fields
    /// remain intact; some words are non-finite when misread as floats.
    pub words: [u32; 40],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GmdlPart {
    pub source_offset: usize,
    pub transform_a: GmdlTransform,
    pub transform_b: GmdlTransform,
    /// Non-sentinel values contribute the body palette count in measured models.
    /// Other records contain u32::MAX. This is not a bone-parent index.
    pub count_or_sentinel: u32,
    /// Exact source resource-key words: instance-low, type, instance-high. The
    /// middle word is a type here, not a DBPF group identifier.
    pub model_key_words: [u32; 3],
    pub bone_presence: u32,
    pub bone_data_offset: usize,
    pub bones: Vec<GmdlBoneRecord>,
}
impl GmdlPart {
    pub fn model_instance(&self) -> Option<u64> {
        (self.model_key_words[1] != u32::MAX).then_some(
            (u64::from(self.model_key_words[2]) << 32) | u64::from(self.model_key_words[0]),
        )
    }
    pub fn model_type(&self) -> Option<u32> {
        self.model_instance().map(|_| self.model_key_words[1])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GmdlSkin {
    pub source_offset: usize,
    pub materials: Vec<GmdlMaterial>,
    pub palettes: Vec<GmdlPalette>,
    pub parts: Vec<GmdlPart>,
    pub footer: [u32; 3],
}
impl GmdlSkin {
    /// The measured palette-size equation. It is a consistency check, not a
    /// recovered parent hierarchy or proof that record order is joint order.
    pub fn part_palette_count(&self) -> Option<usize> {
        self.parts.iter().try_fold(0usize, |sum, p| {
            let count = if p.count_or_sentinel == u32::MAX {
                0
            } else {
                p.count_or_sentinel as usize
            };
            sum.checked_add(count)?.checked_add(p.bones.len())
        })
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], GmdlSkinError> {
        let end = self.at.checked_add(len).ok_or(GmdlSkinError::Truncated)?;
        let b = self
            .data
            .get(self.at..end)
            .ok_or(GmdlSkinError::Truncated)?;
        self.at = end;
        Ok(b)
    }
    fn word(&mut self) -> Result<u32, GmdlSkinError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, minimum_record_bytes: usize) -> Result<usize, GmdlSkinError> {
        let count = self.word()? as usize;
        if count > 100_000
            || count > self.data.len().saturating_sub(self.at) / minimum_record_bytes.max(1)
        {
            return Err(GmdlSkinError::InvalidCount);
        }
        Ok(count)
    }
    fn transform(&mut self) -> Result<GmdlTransform, GmdlSkinError> {
        let b = self.take(56)?;
        let f = |at| f32::from_le_bytes(b[at..at + 4].try_into().unwrap());
        let mut rotation = [[0.; 3]; 3];
        for (r, row) in rotation.iter_mut().enumerate() {
            for (c, value) in row.iter_mut().enumerate() {
                *value = f(20 + (r * 3 + c) * 4);
            }
        }
        Ok(GmdlTransform {
            flags: u16::from_le_bytes(b[0..2].try_into().unwrap()),
            transform_count: u16::from_le_bytes(b[2..4].try_into().unwrap()),
            translation: [f(4), f(8), f(12)],
            scale: f(16),
            rotation,
        })
    }
}

/// Decode the optional tail using the existing parser's exact core boundary.
/// Partial tails are errors rather than silently disappearing. Core-only models
/// return None. The borrowed model must be parsed from these same source bytes.
pub fn parse(data: &[u8], model: &gmdl::Gmdl) -> Result<Option<GmdlSkin>, GmdlSkinError> {
    if !matches!(model.version, 8 | 9) {
        return Err(GmdlSkinError::UnsupportedVersion(model.version));
    }
    if model.total_size != data.len() || model.consumed > data.len() {
        return Err(GmdlSkinError::InvalidTail);
    }
    if model.consumed == data.len() {
        return Ok(None);
    }
    let mut r = Reader {
        data,
        at: model.consumed,
    };
    if r.word()? != 0 {
        return Err(GmdlSkinError::InvalidTail);
    }
    let material_count = r.count(4)?;
    let mut materials = Vec::with_capacity(material_count);
    for _ in 0..material_count {
        let version9_prefix = if model.version == 9 {
            Some(u32::from_be_bytes(r.take(4)?.try_into().unwrap()))
        } else {
            None
        };
        if version9_prefix == Some(0) {
            materials.push(GmdlMaterial {
                version9_prefix,
                shader_data: Vec::new(),
            });
            continue;
        }
        let count = r.count(4)?;
        let mut material = Vec::with_capacity(count);
        for _ in 0..count {
            let id = r.word()?;
            let bytes = match id {
                0x20D => {
                    let start = r.at;
                    let count = r.count(24)?;
                    r.take(count * 24)?;
                    data[start..r.at].to_vec()
                }
                0x210 => r.take(20)?.to_vec(),
                0x21C => r.take(32)?.to_vec(),
                0x218 => Vec::new(),
                0x238 => r.take(1)?.to_vec(),
                _ => return Err(GmdlSkinError::UnsupportedShaderData(id)),
            };
            material.push(GmdlShaderData { id, payload: bytes });
        }
        materials.push(GmdlMaterial {
            version9_prefix,
            shader_data: material,
        });
    }
    let count = r.count(8)?;
    let mut palettes = Vec::with_capacity(count);
    for _ in 0..count {
        palettes.push(GmdlPalette {
            first_word: r.word()?,
            bone_count: r.word()?,
        });
    }
    let count = r.count(132)?;
    let mut parts = Vec::with_capacity(count);
    for _ in 0..count {
        let source_offset = r.at;
        let transform_a = r.transform()?;
        let transform_b = r.transform()?;
        let count_or_sentinel = r.word()?;
        let model_key_words = [r.word()?, r.word()?, r.word()?];
        let bone_presence = r.word()?;
        let mut bones = Vec::new();
        let mut bone_data_offset = r.at;
        if bone_presence != 0 {
            let count = r.count(160)?;
            bone_data_offset = r.at;
            bones.reserve(count);
            for _ in 0..count {
                let mut words = [0; 40];
                for v in &mut words {
                    *v = r.word()?;
                }
                bones.push(GmdlBoneRecord { words });
            }
        }
        parts.push(GmdlPart {
            source_offset,
            transform_a,
            transform_b,
            count_or_sentinel,
            model_key_words,
            bone_presence,
            bone_data_offset,
            bones,
        });
    }
    let footer = [r.word()?, r.word()?, r.word()?];
    if r.at != data.len() {
        return Err(GmdlSkinError::InvalidTail);
    }
    Ok(Some(GmdlSkin {
        source_offset: model.consumed,
        materials,
        palettes,
        parts,
        footer,
    }))
}

/// Decode the measured UBYTE4 joint-register / UBYTE4N weight declaration.
/// Active register offsets must be divisible by three and fit the exact mesh's
/// declared palette. Weight bytes are divided by 255 without renormalization.
pub fn vertex_skinning(
    data: &[u8],
    model: &gmdl::Gmdl,
    skin: &GmdlSkin,
    mesh_index: usize,
) -> Result<Vec<VertexSkinning>, GmdlSkinError> {
    let mesh = model
        .meshes
        .get(mesh_index)
        .ok_or(GmdlSkinError::InvalidReference)?;
    let vb = usize::try_from(mesh.vertex_index)
        .ok()
        .and_then(|i| model.vertex_buffers.get(i))
        .ok_or(GmdlSkinError::InvalidReference)?;
    let format = usize::try_from(vb.format_index)
        .ok()
        .and_then(|i| model.vertex_formats.get(i))
        .ok_or(GmdlSkinError::InvalidReference)?;
    let joint = format
        .elements
        .iter()
        .find(|e| e.stream == 0 && e.usage == 2 && e.usage_index == 0)
        .ok_or(GmdlSkinError::UnsupportedDeclaration)?;
    let weight = format
        .elements
        .iter()
        .find(|e| e.stream == 0 && e.usage == 1 && e.usage_index == 0)
        .ok_or(GmdlSkinError::UnsupportedDeclaration)?;
    if joint.element_type != 5 || weight.element_type != 8 || joint.offset < 0 || weight.offset < 0
    {
        return Err(GmdlSkinError::UnsupportedDeclaration);
    }
    // Only the unambiguous observed one-mesh/one-palette case is supported. The
    // descriptor's first word has not yet been established as a mesh selector.
    if model.meshes.len() != 1 || skin.palettes.len() != 1 || skin.palettes[0].first_word != 0 {
        return Err(GmdlSkinError::UnsupportedPaletteMapping);
    }
    let count = skin.palettes[0].bone_count as usize;
    if count == 0 || count > u16::MAX as usize {
        return Err(GmdlSkinError::InvalidReference);
    }
    if joint.offset as usize + 4 > vb.stride || weight.offset as usize + 4 > vb.stride {
        return Err(GmdlSkinError::InvalidReference);
    }
    let vertices = usize::try_from(vb.vertex_count).map_err(|_| GmdlSkinError::InvalidCount)?;
    let end = vertices
        .checked_mul(vb.stride)
        .and_then(|n| vb.data_offset.checked_add(n))
        .ok_or(GmdlSkinError::Truncated)?;
    if data.get(vb.data_offset..end).is_none() {
        return Err(GmdlSkinError::Truncated);
    }
    let mut out = Vec::with_capacity(vertices);
    for v in 0..vertices {
        let at = vb.data_offset + v * vb.stride;
        let mut row = VertexSkinning {
            joints: [0; 4],
            weights: [0.; 4],
        };
        for i in 0..4 {
            let w = data[at + weight.offset as usize + i];
            if w != 0 {
                let raw = data[at + joint.offset as usize + i];
                if raw % 3 != 0 || raw as usize / 3 >= count {
                    return Err(GmdlSkinError::InvalidInfluence);
                }
                row.joints[i] = u16::from(raw / 3);
                row.weights[i] = f32::from(w) / 255.;
            }
        }
        out.push(row);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(d: &mut Vec<u8>, v: u32) {
        d.extend(v.to_le_bytes());
    }
    fn base() -> (Vec<u8>, gmdl::Gmdl) {
        // A valid core-only model, 1 position vertex and a degenerate triangle.
        let mut d = Vec::new();
        for v in [8, 0, 1] {
            word(&mut d, v);
        }
        for _ in 0..7 {
            word(&mut d, 0);
        }
        for v in [1, 4, 3, 16, 6] {
            word(&mut d, v);
        }
        d.extend([0; 6]);
        for v in [1, 1] {
            word(&mut d, v);
        }
        d.extend([0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
        for v in [1, 0, 1, 12, 0, 0, 0, 0, 0, 0] {
            word(&mut d, v);
        }
        let m = gmdl::parse(&d).unwrap();
        assert!(m.exact);
        (d, m)
    }
    fn transform(d: &mut Vec<u8>) {
        d.extend(6u16.to_le_bytes());
        d.extend(1u16.to_le_bytes());
        for f in [1f32, 2., 3., 0.5, 1., 0., 0., 0., 1., 0., 0., 0., 1.] {
            word(d, f.to_bits());
        }
    }
    fn fixture() -> (Vec<u8>, gmdl::Gmdl) {
        let (mut d, _) = base();
        for v in [0, 1, 1, 0x238] {
            word(&mut d, v);
        }
        d.push(7);
        for v in [1, 0, 2, 1] {
            word(&mut d, v);
        }
        transform(&mut d);
        transform(&mut d);
        for v in [1, 0xABCDEF01, 0x2F4E681B, 0x41606100, 1, 1] {
            word(&mut d, v);
        }
        for v in 0..40 {
            word(&mut d, v);
        }
        for v in [0, u32::MAX, 0] {
            word(&mut d, v);
        }
        let m = gmdl::parse(&d).unwrap();
        assert!(m.exact);
        (d, m)
    }
    #[test]
    fn core_only_has_no_tail() {
        let (d, m) = base();
        assert!(parse(&d, &m).unwrap().is_none());
    }
    #[test]
    fn typed_tail_preserves_transforms_raw_bones_and_key_order() {
        let (d, m) = fixture();
        let s = parse(&d, &m).unwrap().unwrap();
        assert_eq!(s.parts.len(), 1);
        assert_eq!(s.parts[0].transform_a.translation, [1., 2., 3.]);
        assert_eq!(s.parts[0].transform_a.scale, 0.5);
        assert_eq!(s.parts[0].transform_a.flags, 6);
        assert_eq!(s.parts[0].model_instance(), Some(0x41606100ABCDEF01));
        assert_eq!(s.parts[0].bones[0].words[39], 39);
        assert_eq!(s.part_palette_count(), Some(2));
        assert_eq!(s.materials[0].shader_data[0].payload, vec![7]);
        assert_eq!(s.footer, [0, u32::MAX, 0]);
    }
    #[test]
    fn tail_rejects_every_prefix_and_trailing_bytes() {
        let (d, m) = fixture();
        for end in m.consumed + 1..d.len() {
            let mut view = m.clone();
            view.total_size = end;
            assert!(parse(&d[..end], &view).is_err(), "prefix{end}");
        }
        let mut extra = d.clone();
        extra.push(0);
        let mut view = m.clone();
        view.total_size += 1;
        assert_eq!(parse(&extra, &view), Err(GmdlSkinError::InvalidTail));
    }
    #[test]
    fn unknown_bone_bits_are_not_misread_as_finite_matrices() {
        let (mut d, m) = fixture();
        let s = parse(&d, &m).unwrap().unwrap();
        d[s.parts[0].bone_data_offset + 12..s.parts[0].bone_data_offset + 16]
            .copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        let s = parse(&d, &m).unwrap().unwrap();
        assert_eq!(s.parts[0].bones[0].words[3], u32::MAX);
    }
    #[test]
    fn huge_counts_fail_before_allocation() {
        let (mut d, m) = fixture();
        d[m.consumed + 4..m.consumed + 8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(parse(&d, &m), Err(GmdlSkinError::InvalidCount));
    }
    #[test]
    fn version9_material_prefix_is_big_endian_and_preserved() {
        let (mut d, m) = fixture();
        d[0..4].copy_from_slice(&9u32.to_le_bytes());
        d.splice(m.consumed + 8..m.consumed + 8, 0x01234567u32.to_be_bytes());
        let model = gmdl::parse(&d).unwrap();
        // The original geometry parser's legacy skipped-tail exact flag is false;
        // the strict companion parser consumes the actual version9 tail.
        assert!(!model.exact);
        let s = parse(&d, &model).unwrap().unwrap();
        assert_eq!(s.materials[0].version9_prefix, Some(0x01234567));
        assert_eq!(s.materials[0].shader_data[0].payload, vec![7]);
    }
    #[test]
    fn shader_21c_retains_all_32_bytes_without_assigning_semantics() {
        let (mut d, m) = fixture();
        let at = m.consumed + 12;
        d[at..at + 4].copy_from_slice(&0x21Cu32.to_le_bytes());
        d.splice(at + 4..at + 5, 0u8..32);
        let model = gmdl::parse(&d).unwrap();
        let s = parse(&d, &model).unwrap().unwrap();
        assert_eq!(
            s.materials[0].shader_data[0].payload,
            (0u8..32).collect::<Vec<_>>()
        );
    }
    #[test]
    fn version9_empty_material_slot_has_no_shader_count_and_keeps_slot_order() {
        let (mut d, m) = fixture();
        d[0..4].copy_from_slice(&9u32.to_le_bytes());
        d[m.consumed + 4..m.consumed + 8].copy_from_slice(&2u32.to_le_bytes());
        d.splice(m.consumed + 8..m.consumed + 8, 1u32.to_be_bytes());
        d.splice(m.consumed + 21..m.consumed + 21, 0u32.to_be_bytes());
        let model = gmdl::parse(&d).unwrap();
        let s = parse(&d, &model).unwrap().unwrap();
        assert_eq!(s.materials.len(), 2);
        assert_eq!(s.materials[0].shader_data[0].payload, vec![7]);
        assert_eq!(s.materials[1].version9_prefix, Some(0));
        assert!(s.materials[1].shader_data.is_empty());
        assert_eq!(s.palettes.len(), 1);
    }
    #[test]
    fn source_model_mismatch_fails() {
        let (mut d, m) = fixture();
        d.pop();
        assert_eq!(parse(&d, &m), Err(GmdlSkinError::InvalidTail));
    }

    #[test]
    fn absent_model_is_decided_by_type_word_not_stale_instance_bits() {
        let (d, m) = fixture();
        let mut s = parse(&d, &m).unwrap().unwrap();
        s.parts[0].model_key_words = [0x12345678, u32::MAX, 0xABCDEF01];
        assert_eq!(s.parts[0].model_instance(), None);
        assert_eq!(s.parts[0].model_type(), None);
    }

    fn influence_fixture() -> (Vec<u8>, gmdl::Gmdl, GmdlSkin) {
        let (mut d, mut m) = fixture();
        let mut skin = parse(&d, &m).unwrap().unwrap();
        skin.palettes[0].bone_count = 13;
        m.vertex_buffers[0].data_offset = d.len();
        m.vertex_buffers[0].stride = 20;
        m.vertex_buffers[0].buffer_size = 20;
        for (offset, ty, usage) in [(12, 5, 2), (16, 8, 1)] {
            m.vertex_formats[0].elements.push(gmdl::VertexElement {
                stream: 0,
                offset,
                element_type: ty,
                method: 0,
                usage,
                usage_index: 0,
                rw_decl: 0,
            });
        }
        d.extend([0; 12]);
        d.extend([0, 30, 33, 36, 66, 63, 63, 63]);
        (d, m, skin)
    }

    #[test]
    fn byte_weights_and_three_register_joint_storage_decode_exactly() {
        let (d, m, skin) = influence_fixture();
        let v = vertex_skinning(&d, &m, &skin, 0).unwrap();
        assert_eq!(v[0].joints, [0, 10, 11, 12]);
        assert_eq!(
            v[0].weights,
            [66. / 255., 63. / 255., 63. / 255., 63. / 255.]
        );
        assert!((v[0].weights.iter().sum::<f32>() - 1.).abs() < 1e-6);
    }

    #[test]
    fn zero_weight_garbage_indices_ignored_but_active_invalid_rejected() {
        let (mut d, m, skin) = influence_fixture();
        let at = m.vertex_buffers[0].data_offset;
        d[at + 15] = 255;
        d[at + 19] = 0;
        let v = vertex_skinning(&d, &m, &skin, 0).unwrap();
        assert_eq!(v[0].joints[3], 0);
        d[at + 19] = 1;
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::InvalidInfluence)
        );
        d[at + 15] = 1;
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::InvalidInfluence)
        );
    }

    #[test]
    fn influence_data_and_palette_references_are_bounded() {
        let (mut d, mut m, mut skin) = influence_fixture();
        skin.palettes[0].bone_count = 12;
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::InvalidInfluence)
        );
        skin.palettes[0].bone_count = 13;
        skin.palettes.push(skin.palettes[0]);
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::UnsupportedPaletteMapping)
        );
        skin.palettes.pop();
        d.pop();
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::Truncated)
        );
        m.vertex_formats[0].elements[1].element_type = 4;
        assert_eq!(
            vertex_skinning(&d, &m, &skin, 0),
            Err(GmdlSkinError::UnsupportedDeclaration)
        );
    }
}
