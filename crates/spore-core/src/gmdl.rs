//! Parser for the SPORE gmdl model format (type `0x00E6BCE5`).
//!
//! Status: **100% exact (4,209 of 4,209).** The Python reference validator is
//! `tools/gmdl_full.py`.
//!
//! ## Layout (little-endian unless noted)
//!
//! ```text
//! version                 i32
//! ref_count               i32 (BIG-ENDIAN)
//! refs                    ref_count * 12 bytes (BE instance, type, group)
//! mesh_count              i32
//! bbox                    6 f32, then radius f32
//! triangle_buffer_count   i32
//!   per buffer: primitive_type, indices_count, indices_bits, buffer_size (i32)
//!               indices: indices_count * (2 if bits==16 else 4) bytes
//! vertex_format_count     i32
//!   per format: element_count i32, then element_count * VertexElement
//!               VertexElement = <hhbbbbi (stream i16, offset i16, type i8,
//!               method i8, usage i8, usage_index i8, rw_decl i32)
//! vertex_buffer_count     i32
//!   per buffer: format_index i32, vertex_count i32, buffer_size i32
//!               vertices: vertex_count * stride bytes
//!               (stride = sum of D3DDECLTYPE sizes of the format's elements)
//! mesh_count times: t_index i32, v_index i32
//! mesh_count times: material_id i32
//! trailing (skinned models):
//!   u32 zero, material_count i32, per material: shdata_count i32 then shader
//!   blobs (0x20D: count then count*24 bytes; 0x210: 20 bytes; 0x218: 0 bytes;
//!   0x238: 1 byte)
//!   skeleton: count1 i32, count1*8 bytes, part_count i32, per part two 0x38
//!   transforms, field i32, key 3 i32, var i32, and if var!=0 bone_count i32
//!   + bone_count*0xA0 bytes
//!   fixed 12-byte footer
//! ```

use crate::Vec3;
use std::fmt;

/// The 12-byte magic is not present in gmdl; the version is the first field.
pub const MIN_SIZE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GmdlError {
    /// The resource is shorter than the fixed header.
    TooShort,
    /// A field or blob runs past the end of the buffer.
    Truncated,
}

impl fmt::Display for GmdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GmdlError::TooShort => write!(f, "gmdl resource too short"),
            GmdlError::Truncated => write!(f, "gmdl field or blob truncated"),
        }
    }
}

impl std::error::Error for GmdlError {}

/// One triangle (index) buffer.
#[derive(Debug, Clone)]
pub struct TriangleBuffer {
    pub primitive_type: i32,
    pub indices_count: i32,
    pub indices_bits: i32,
    pub buffer_size: i32,
    pub indices: Vec<u32>,
}

/// One vertex element descriptor (`<hhbbbbi`, 12 bytes).
#[derive(Debug, Clone)]
pub struct VertexElement {
    pub stream: i16,
    pub offset: i16,
    pub element_type: i8,
    pub method: i8,
    pub usage: i8,
    pub usage_index: i8,
    pub rw_decl: i32,
}

impl VertexElement {
    /// Byte size of this element, from the D3DDECLTYPE code.
    pub fn size(&self) -> usize {
        match self.element_type {
            0 => 4,   // FLOAT1
            1 => 8,   // FLOAT2
            2 => 12,  // FLOAT3
            3 => 16,  // FLOAT4
            4 => 4,   // D3DCOLOR
            5 => 4,   // UBYTE4
            6 => 4,   // SHORT2
            7 => 8,   // SHORT4
            8 => 4,   // UBYTE4N
            9 => 4,   // SHORT2N
            10 => 8,  // SHORT4N
            11 => 4,  // USHORT2N
            12 => 8,  // USHORT4N
            13 => 4,  // UDEC3
            14 => 4,  // DEC3N
            15 => 4,  // FLOAT16_2
            16 => 8,  // FLOAT16_4
            _ => 4,
        }
    }
}

/// One vertex format: an ordered list of elements.
#[derive(Debug, Clone)]
pub struct VertexFormat {
    pub elements: Vec<VertexElement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedNativeSemantic(pub i32);
impl fmt::Display for UnsupportedNativeSemantic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported native vertex semantic {}", self.0)
    }
}
impl std::error::Error for UnsupportedNativeSemantic {}

impl VertexFormat {
    /// Native 760FB0/73EDF0 bind marker shader-data 22C when unpacking is
    /// enabled and the first semantic-2 row uses UBYTE4. The original builder
    /// then selects its unpack fragment. UBYTE4N and a tangent alone do not
    /// trigger this path. The native enable bytes are caller/runtime state.
    pub fn native_normal_unpack_required(&self, enabled: bool) -> bool {
        enabled && self.elements.iter().find(|e| e.rw_decl == 2)
            .is_some_and(|e| e.element_type == 5)
    }

    /// Native 11F2FD0 ORs the semantic table's feature word into declaration
    /// +0x10. 6FC780 uses that word for source shader selector masks. Measured
    /// semantic entries 0..23 have bit 1<<ID; entry 24 is the END sentinel,
    /// not another attribute. D3D usage alone loses distinct source semantics.
    pub fn native_shader_features(&self) -> Result<u32, UnsupportedNativeSemantic> {
        self.elements.iter().try_fold(0, |flags, element| {
            if !(0..24).contains(&element.rw_decl) {
                return Err(UnsupportedNativeSemantic(element.rw_decl));
            }
            Ok(flags | (1u32 << element.rw_decl))
        })
    }

    /// Bytes per vertex for this format.
    pub fn stride(&self) -> usize {
        self.elements.iter().map(|e| e.size()).sum()
    }
}

/// One vertex buffer header (the vertex data itself is not materialised).
#[derive(Debug, Clone)]
pub struct VertexBuffer {
    pub format_index: i32,
    pub vertex_count: i32,
    pub buffer_size: i32,
    pub stride: usize,
    pub data_offset: usize,
}

/// One mesh, linking a triangle buffer and a vertex buffer.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub triangle_index: i32,
    pub vertex_index: i32,
    pub material_id: i32,
}

/// A parsed gmdl model.
#[derive(Debug, Clone)]
pub struct Gmdl {
    pub version: i32,
    pub ref_count: i32,
    pub mesh_count: i32,
    pub bbox_min: Vec3,
    pub bbox_max: Vec3,
    pub radius: f32,
    pub triangle_buffers: Vec<TriangleBuffer>,
    pub vertex_formats: Vec<VertexFormat>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub meshes: Vec<Mesh>,
    pub total_size: usize,
    /// Bytes consumed by the parse (before any trailing section failure).
    pub consumed: usize,
    /// Whether the parse consumed exactly the whole buffer.
    pub exact: bool,
}

struct Reader<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn new(d: &'a [u8]) -> Self {
        Reader { d, p: 0 }
    }
    fn i32le(&mut self) -> Result<i32, GmdlError> {
        if self.p + 4 > self.d.len() {
            return Err(GmdlError::Truncated);
        }
        let v = i32::from_le_bytes([self.d[self.p], self.d[self.p + 1], self.d[self.p + 2], self.d[self.p + 3]]);
        self.p += 4;
        Ok(v)
    }
    fn i32be(&mut self) -> Result<i32, GmdlError> {
        if self.p + 4 > self.d.len() {
            return Err(GmdlError::Truncated);
        }
        let v = i32::from_be_bytes([self.d[self.p], self.d[self.p + 1], self.d[self.p + 2], self.d[self.p + 3]]);
        self.p += 4;
        Ok(v)
    }
    fn f32le(&mut self) -> Result<f32, GmdlError> {
        if self.p + 4 > self.d.len() {
            return Err(GmdlError::Truncated);
        }
        let v = f32::from_le_bytes([self.d[self.p], self.d[self.p + 1], self.d[self.p + 2], self.d[self.p + 3]]);
        self.p += 4;
        Ok(v)
    }
    fn take(&mut self, n: usize) -> Result<(), GmdlError> {
        if self.p + n > self.d.len() {
            return Err(GmdlError::Truncated);
        }
        self.p += n;
        Ok(())
    }
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], GmdlError> {
        if self.p + n > self.d.len() {
            return Err(GmdlError::Truncated);
        }
        let s = &self.d[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
}

fn read_material(r: &mut Reader<'_>) -> Result<(), GmdlError> {
    let shdata_count = r.i32le()?;
    if shdata_count < 0 || shdata_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    for _ in 0..shdata_count {
        let idx = r.i32le()?;
        match idx {
            0x20D => {
                let tex = r.i32le()?;
                if tex < 0 || tex > 10000 {
                    return Err(GmdlError::Truncated);
                }
                r.take(tex as usize * 24)?;
            }
            0x210 => r.take(20)?,
            0x218 => {}
            // A single byte. Reading this as a length-prefixed blob is what kept
            // 132 skinned models at 96.86%: the byte merges with the following
            // count1 field, so the pair reads as a plausible-looking size.
            0x238 => r.take(1)?,
            _ => return Err(GmdlError::Truncated),
        }
    }
    Ok(())
}

/// Parse a gmdl resource. `exact` is the authoritative full-consumption signal.
pub fn parse(data: &[u8]) -> Result<Gmdl, GmdlError> {
    if data.len() < MIN_SIZE {
        return Err(GmdlError::TooShort);
    }
    let mut r = Reader::new(data);
    let version = r.i32le()?;
    let ref_count = r.i32be()?;
    if ref_count < 0 || ref_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    r.take(ref_count as usize * 12)?;
    let mesh_count = r.i32le()?;
    if mesh_count < 0 || mesh_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    let bbox_min = Vec3::new(r.f32le()?, r.f32le()?, r.f32le()?);
    let bbox_max = Vec3::new(r.f32le()?, r.f32le()?, r.f32le()?);
    let radius = r.f32le()?;

    let tri_count = r.i32le()?;
    if tri_count < 0 || tri_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    let mut triangle_buffers = Vec::with_capacity(tri_count as usize);
    for _ in 0..tri_count {
        let primitive_type = r.i32le()?;
        let indices_count = r.i32le()?;
        let indices_bits = r.i32le()?;
        let buffer_size = r.i32le()?;
        if indices_count < 0 || indices_count > 50_000_000 {
            return Err(GmdlError::Truncated);
        }
        let w = if indices_bits == 16 { 2 } else { 4 };
        let raw = r.bytes(indices_count as usize * w)?;
        let mut indices = Vec::with_capacity(indices_count as usize);
        for i in 0..indices_count as usize {
            let v = if w == 2 {
                u16::from_le_bytes([raw[2 * i], raw[2 * i + 1]]) as u32
            } else {
                u32::from_le_bytes([raw[4 * i], raw[4 * i + 1], raw[4 * i + 2], raw[4 * i + 3]])
            };
            indices.push(v);
        }
        triangle_buffers.push(TriangleBuffer {
            primitive_type,
            indices_count,
            indices_bits,
            buffer_size,
            indices,
        });
    }

    let vf_count = r.i32le()?;
    if vf_count < 0 || vf_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    let mut vertex_formats = Vec::with_capacity(vf_count as usize);
    for _ in 0..vf_count {
        let elem_count = r.i32le()?;
        if elem_count < 0 || elem_count > 1000 {
            return Err(GmdlError::Truncated);
        }
        let mut elements = Vec::with_capacity(elem_count as usize);
        for _ in 0..elem_count {
            let b = r.bytes(12)?;
            elements.push(VertexElement {
                stream: i16::from_le_bytes([b[0], b[1]]),
                offset: i16::from_le_bytes([b[2], b[3]]),
                element_type: b[4] as i8,
                method: b[5] as i8,
                usage: b[6] as i8,
                usage_index: b[7] as i8,
                rw_decl: i32::from_le_bytes([b[8], b[9], b[10], b[11]]),
            });
        }
        vertex_formats.push(VertexFormat { elements });
    }

    let vb_count = r.i32le()?;
    if vb_count < 0 || vb_count > 10000 {
        return Err(GmdlError::Truncated);
    }
    let mut vertex_buffers = Vec::with_capacity(vb_count as usize);
    for _ in 0..vb_count {
        let format_index = r.i32le()?;
        let vertex_count = r.i32le()?;
        let buffer_size = r.i32le()?;
        if format_index < 0 || format_index as usize >= vertex_formats.len() {
            return Err(GmdlError::Truncated);
        }
        if vertex_count < 0 || vertex_count > 50_000_000 {
            return Err(GmdlError::Truncated);
        }
        let stride = vertex_formats[format_index as usize].stride();
        let data_offset = r.p;
        r.take(vertex_count as usize * stride)?;
        vertex_buffers.push(VertexBuffer {
            format_index,
            vertex_count,
            buffer_size,
            stride,
            data_offset,
        });
    }

    let mut meshes = Vec::with_capacity(mesh_count as usize);
    for _ in 0..mesh_count {
        let triangle_index = r.i32le()?;
        let vertex_index = r.i32le()?;
        meshes.push(Mesh {
            triangle_index,
            vertex_index,
            material_id: 0,
        });
    }
    for m in meshes.iter_mut() {
        m.material_id = r.i32le()?;
    }

    let consumed = r.p;

    // Optional trailing (skinned) section.
    let mut exact = false;
    if r.p == data.len() {
        exact = true;
    } else {
        let trail = (|| -> Result<(), GmdlError> {
            let zero = r.i32le()?;
            if zero != 0 {
                return Err(GmdlError::Truncated);
            }
            let material_count = r.i32le()?;
            if material_count < 0 || material_count > 10000 {
                return Err(GmdlError::Truncated);
            }
            for _ in 0..material_count {
                read_material(&mut r)?;
            }
            let count1 = r.i32le()?;
            if count1 < 0 || count1 > 10000 {
                return Err(GmdlError::Truncated);
            }
            r.take(count1 as usize * 8)?;
            let part_count = r.i32le()?;
            if part_count < 0 || part_count > 100000 {
                return Err(GmdlError::Truncated);
            }
            for _ in 0..part_count {
                r.take(0x38)?;
                r.take(0x38)?;
                r.i32le()?; // field
                r.take(12)?; // key
                let var = r.i32le()?;
                if var != 0 {
                    let bone_count = r.i32le()?;
                    if bone_count < 0 || bone_count > 100000 {
                        return Err(GmdlError::Truncated);
                    }
                    r.take(bone_count as usize * 0xA0)?;
                }
            }
            r.take(12)?; // fixed footer
            Ok(())
        })();
        if trail.is_ok() && r.p == data.len() {
            exact = true;
        }
    }

    Ok(Gmdl {
        version,
        ref_count,
        mesh_count,
        bbox_min,
        bbox_max,
        radius,
        triangle_buffers,
        vertex_formats,
        vertex_buffers,
        meshes,
        total_size: data.len(),
        consumed,
        exact,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_i32(v: &mut Vec<u8>, x: i32) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn push_i32be(v: &mut Vec<u8>, x: i32) {
        v.extend_from_slice(&x.to_be_bytes());
    }
    fn push_f32(v: &mut Vec<u8>, x: f32) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    /// Minimal single-mesh model: 1 triangle buffer (3 u16 indices), 1 vertex
    /// format (position float3), 1 vertex buffer (1 vertex), 1 mesh, 1 material.
    fn build_min() -> Vec<u8> {
        let mut d = Vec::new();
        push_i32(&mut d, 8); // version
        push_i32be(&mut d, 0); // ref_count
        push_i32(&mut d, 1); // mesh_count
        for _ in 0..6 {
            push_f32(&mut d, 0.0); // bbox
        }
        push_f32(&mut d, 1.0); // radius
        push_i32(&mut d, 1); // tri_count
        push_i32(&mut d, 4); // prim
        push_i32(&mut d, 3); // indices_count
        push_i32(&mut d, 16); // bits
        push_i32(&mut d, 6); // buffer_size
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&2u16.to_le_bytes());
        push_i32(&mut d, 1); // vf_count
        push_i32(&mut d, 1); // elem_count
        // VertexElement: stream=0, offset=0, type=2 (FLOAT3), method=0, usage=0, uidx=0, rw_decl=0
        d.extend_from_slice(&0i16.to_le_bytes());
        d.extend_from_slice(&0i16.to_le_bytes());
        d.push(2);
        d.push(0);
        d.push(0);
        d.push(0);
        push_i32(&mut d, 0); // rw_decl
        push_i32(&mut d, 1); // vb_count
        push_i32(&mut d, 0); // format_index
        push_i32(&mut d, 1); // vertex_count
        push_i32(&mut d, 12); // buffer_size
        push_f32(&mut d, 1.0);
        push_f32(&mut d, 2.0);
        push_f32(&mut d, 3.0);
        push_i32(&mut d, 0); // t_index
        push_i32(&mut d, 0); // v_index
        push_i32(&mut d, 7); // material_id
        d
    }

    #[test]
    fn parse_minimal_core() {
        let d = build_min();
        let m = parse(&d).unwrap();
        assert_eq!(m.version, 8);
        assert_eq!(m.mesh_count, 1);
        assert_eq!(m.triangle_buffers.len(), 1);
        assert_eq!(m.triangle_buffers[0].indices, vec![0, 1, 2]);
        assert_eq!(m.vertex_formats.len(), 1);
        assert_eq!(m.vertex_formats[0].stride(), 12);
        assert_eq!(m.vertex_buffers.len(), 1);
        assert_eq!(m.vertex_buffers[0].stride, 12);
        assert_eq!(m.meshes.len(), 1);
        assert_eq!(m.meshes[0].material_id, 7);
        // No trailing section: core consumes the whole buffer.
        assert_eq!(m.consumed, d.len());
        assert!(m.exact);
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 10]), Err(GmdlError::TooShort)));
    }

    /// The skinned trailing section, including shader data `0x238`, which is a
    /// single byte. Reading it as a length-prefixed blob is the mistake that kept
    /// 132 models at 96.86%: the byte merges with the following `count1` field and
    /// the pair reads as a plausible size (0x111 = 273).
    #[test]
    fn trailing_with_shader_data_0x238() {
        let mut d = build_min();
        let core_len = d.len();
        push_i32(&mut d, 0); // zero
        push_i32(&mut d, 1); // material_count
        push_i32(&mut d, 3); // shdata_count
        push_i32(&mut d, 0x210);
        d.extend_from_slice(&[0u8; 20]);
        push_i32(&mut d, 0x20D);
        push_i32(&mut d, 1); // texture_count
        d.extend_from_slice(&[0u8; 24]);
        push_i32(&mut d, 0x238);
        d.push(0x11); // the whole 0x238 payload
        push_i32(&mut d, 1); // count1
        d.extend_from_slice(&[0u8; 8]); // count1 * 8 bytes
        push_i32(&mut d, 1); // part_count
        d.extend_from_slice(&[0u8; 0x38]);
        d.extend_from_slice(&[0u8; 0x38]);
        push_i32(&mut d, 0); // field_70
        d.extend_from_slice(&[0xFFu8; 12]); // key
        push_i32(&mut d, 0); // var: no bones
        push_i32(&mut d, 0); // footer
        push_i32(&mut d, -1i32);
        push_i32(&mut d, 0);

        let m = parse(&d).unwrap();
        assert_eq!(m.consumed, core_len);
        assert!(m.exact);
    }
    #[test]
    fn truncated_indices() {
        let mut d = build_min();
        // 64 bytes is above MIN_SIZE but cuts into the triangle indices (which
        // start at offset 60 and need 6 bytes).
        d.truncate(64);
        assert!(matches!(parse(&d), Err(GmdlError::Truncated)));
    }

    #[test]
    fn vertex_element_sizes() {
        let e = |t: i8| VertexElement {
            stream: 0,
            offset: 0,
            element_type: t,
            method: 0,
            usage: 0,
            usage_index: 0,
            rw_decl: 0,
        };
        assert_eq!(e(2).size(), 12); // FLOAT3
        assert_eq!(e(3).size(), 16); // FLOAT4
        assert_eq!(e(5).size(), 4); // UBYTE4
    }

    #[test]
    fn native_unpack_marker_uses_first_normal_storage_and_runtime_enable() {
        let element = |semantic, ty| VertexElement {
            stream: 0, offset: 0, element_type: ty, method: 0,
            usage: 0, usage_index: 0, rw_decl: semantic,
        };
        let mut format = VertexFormat { elements: vec![element(19, 5)] };
        assert!(!format.native_normal_unpack_required(true));
        format.elements.push(element(2, 8));
        format.elements.push(element(2, 5));
        assert!(!format.native_normal_unpack_required(true));
        format.elements[1].element_type = 5;
        assert!(format.native_normal_unpack_required(true));
        assert!(!format.native_normal_unpack_required(false));
    }
}
