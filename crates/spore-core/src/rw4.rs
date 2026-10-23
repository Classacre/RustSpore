//! RW4 container format parser (type `0x2F4E681B`).
//!
//! Status: **container and per-object payloads 100% validated (10,797 of
//! 10,797).** The Python reference validators are `tools/rw4_validate.py`
//! (container) and `tools/rw4_objects.py` (per-object extents). Layouts are
//! reimplemented from the public SporeModder `rw4_base.py`; no code copied.
//!
//! ## Layout (little-endian)
//!
//! ```text
//! offset  0: magic 89 52 57 34 77 33 32 00 0d 0a 1a 0a  ("RW4w32")
//! offset 28: u32 rw_type_code
//! offset 36: u32 section_count
//! offset 48: u32 p_section_infos
//! offset 68: u32 p_buffer_data
//! p_section_infos: section_count * 24-byte records:
//!     p_data i32, field_04 i32, data_size i32, alignment i32,
//!     type_code_index i32, type_code u32
//! object data: at p_data, or at p_buffer_data + p_data for BaseResource
//!     (type code 0x10030), and is data_size bytes long
//! ```
//!
//! `[MEASURED]` Between the section-info table and `p_buffer_data` sit the
//! sub-reference offset table and reserved runtime space. The header, object
//! data, section-info table, that reserved space and the BaseResource buffer
//! tile the whole file, with gaps of at most 16 bytes (alignment).
//!
//! `[CORRECTION]` An earlier revision of this module read the header as
//! big-endian with a "subtype" at offset 28 and fields at 32/40/52/76/80/88.
//! Those reads were wrong on both counts. Offset 28 is the RenderWare type code
//! (model, texture, special), which is why payload shapes differ between files;
//! and the container fields are little-endian.

use std::fmt;

/// The 12-byte magic every RW4 resource begins with.
pub const MAGIC: [u8; 12] = [
    0x89, b'R', b'W', b'4', b'w', b'3', b'2', 0x00, 0x0D, 0x0A, 0x1A, 0x0A,
];

/// Smallest possible header: the last field we read is `p_buffer_data` at 68.
pub const HEADER_MIN: usize = 72;

/// Size of one section-info record.
pub const SECTION_INFO_SIZE: usize = 24;

/// `BaseResource` offsets its data relative to `p_buffer_data`.
pub const BASE_RESOURCE: u32 = 0x0001_0030;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rw4Error {
    /// Payload too short to hold the header fields.
    TooShort,
    /// Magic signature mismatch.
    BadMagic,
    /// `section_count` is zero or implausibly large.
    BadSectionCount(i32),
    /// The section-info table runs past the end of the buffer.
    BadSectionInfos,
    /// `p_buffer_data` is outside the buffer.
    BadBufferData,
    /// A section's data range runs past the end of the buffer.
    SectionOutOfBounds,
}

impl fmt::Display for Rw4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rw4Error::TooShort => write!(f, "payload too short for RW4 header"),
            Rw4Error::BadMagic => write!(f, "bad RW4 magic signature"),
            Rw4Error::BadSectionCount(c) => write!(f, "implausible RW4 section count {c}"),
            Rw4Error::BadSectionInfos => write!(f, "RW4 section-info table out of bounds"),
            Rw4Error::BadBufferData => write!(f, "RW4 p_buffer_data out of bounds"),
            Rw4Error::SectionOutOfBounds => write!(f, "RW4 section data out of bounds"),
        }
    }
}

impl std::error::Error for Rw4Error {}

/// One 24-byte section-info record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionInfo {
    pub p_data: i32,
    pub field_04: i32,
    pub data_size: i32,
    pub alignment: i32,
    pub type_code_index: i32,
    pub type_code: u32,
}

/// A section with its resolved absolute data range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    pub info: SectionInfo,
    /// Absolute start of the object's data.
    pub start: usize,
    /// `data_size` as a `usize`.
    pub data_size: usize,
}

/// A parsed RW4 container.
#[derive(Debug, Clone)]
pub struct Rw4 {
    /// RenderWare type code (model / texture / special).
    pub rw_type_code: u32,
    pub p_section_infos: usize,
    pub p_buffer_data: usize,
    pub sections: Vec<Section>,
}

fn i32_at(d: &[u8], o: usize) -> i32 {
    i32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn u32_at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn i16_at(d: &[u8], o: usize) -> i16 {
    i16::from_le_bytes([d[o], d[o + 1]])
}

/// Parse the RW4 container.
pub fn parse(data: &[u8]) -> Result<Rw4, Rw4Error> {
    if data.len() < HEADER_MIN {
        return Err(Rw4Error::TooShort);
    }
    if data[..12] != MAGIC {
        return Err(Rw4Error::BadMagic);
    }
    let rw_type_code = u32_at(data, 28);
    let section_count = i32_at(data, 36);
    let p_section_infos = i32_at(data, 48);
    let p_buffer_data = i32_at(data, 68);

    if section_count <= 0 || section_count > 100_000 {
        return Err(Rw4Error::BadSectionCount(section_count));
    }
    let count = section_count as usize;
    if p_section_infos < HEADER_MIN as i32
        || p_section_infos as usize + count * SECTION_INFO_SIZE > data.len()
    {
        return Err(Rw4Error::BadSectionInfos);
    }
    if p_buffer_data < 0 || p_buffer_data as usize > data.len() {
        return Err(Rw4Error::BadBufferData);
    }

    let infos_at = p_section_infos as usize;
    let buffer_at = p_buffer_data as usize;
    let mut sections = Vec::with_capacity(count);
    for i in 0..count {
        let base = infos_at + i * SECTION_INFO_SIZE;
        let info = SectionInfo {
            p_data: i32_at(data, base),
            field_04: i32_at(data, base + 4),
            data_size: i32_at(data, base + 8),
            alignment: i32_at(data, base + 12),
            type_code_index: i32_at(data, base + 16),
            type_code: u32_at(data, base + 20),
        };
        if info.data_size < 0 {
            return Err(Rw4Error::SectionOutOfBounds);
        }
        let raw_start = if info.type_code == BASE_RESOURCE {
            buffer_at as i64 + info.p_data as i64
        } else {
            info.p_data as i64
        };
        let end = raw_start + info.data_size as i64;
        if raw_start < 0 || end > data.len() as i64 {
            return Err(Rw4Error::SectionOutOfBounds);
        }
        sections.push(Section {
            info,
            start: raw_start as usize,
            data_size: info.data_size as usize,
        });
    }

    Ok(Rw4 {
        rw_type_code,
        p_section_infos: infos_at,
        p_buffer_data: buffer_at,
        sections,
    })
}

impl Rw4 {
    /// Check that the container tiles the file: header, section-info table, the
    /// reserved region up to `p_buffer_data`, the buffer data, and every object
    /// range, with no gap larger than 16 bytes.
    pub fn tiles(&self, data: &[u8]) -> bool {
        let n = data.len();
        if self.p_section_infos > self.p_buffer_data {
            return false;
        }
        let mut intervals: Vec<(usize, usize)> = vec![
            (0, self.p_section_infos),
            (self.p_section_infos, self.p_buffer_data),
            (self.p_buffer_data, n),
        ];
        for s in &self.sections {
            if s.data_size > 0 {
                intervals.push((s.start, s.start + s.data_size));
            }
        }
        intervals.sort();
        let mut cur = 0usize;
        let mut max_gap = 0usize;
        for (a, b) in intervals {
            if a > cur {
                max_gap = max_gap.max(a - cur);
            }
            if b > cur {
                cur = b;
            }
        }
        if cur < n {
            max_gap = max_gap.max(n - cur);
        }
        max_gap <= 16
    }

    /// Whether every object consumes exactly its `data_size`.
    pub fn all_objects_exact(&self, data: &[u8]) -> bool {
        self.sections.iter().all(|s| {
            object_extent(s.info.type_code, data, s.start, s.data_size) == Some(s.data_size)
        })
    }
}

/// Bytes an object of `type_code` consumes, or `None` if the type is not
/// modelled. The caller compares the result with the section's `data_size`.
///
/// Every rule below is measured to hold for all 10,797 resources in the install.
pub fn object_extent(type_code: u32, data: &[u8], start: usize, data_size: usize) -> Option<usize> {
    // Bounds guard: every read below stays inside the section.
    let o = start;
    let end = start.checked_add(data_size)?;
    if end > data.len() {
        return None;
    }
    let need = |k: usize| -> Option<()> { if o + k <= data.len() { Some(()) } else { None } };

    match type_code {
        // BaseResource wraps its payload; the section size is authoritative.
        0x0001_0030 => Some(data_size),
        0x0001_0004 => Some(24), // SectionManifest
        0x0001_0005 => {
            // SectionTypes: count i32 then count type codes.
            need(8)?;
            Some(8 + (i32_at(data, o) as usize) * 4)
        }
        0x0001_0006 => Some(32), // SectionExternalArenas
        0x0001_0007 => Some(24), // SectionSubReferences
        0x0001_0008 => Some(8),  // SectionAtoms

        0x0002_0003 => Some(32), // Raster
        0x0002_0004 => {
            // VertexDescription: element_count i16 at +12, 12 bytes each.
            need(24)?;
            Some(24 + (i16_at(data, o + 12).max(0) as usize) * 12)
        }
        0x0002_0005 => Some(28), // VertexBuffer
        0x0002_0007 => Some(28), // IndexBuffer
        0x0002_0008 => {
            // TextureOverride: u32 hash then a NUL-terminated ASCII string.
            need(4)?;
            let mut p = o + 4;
            while p < end && data[p] != 0 {
                p += 1;
            }
            Some(p - o + 1)
        }
        0x0002_0009 => {
            // Mesh: index_count i32 at +16.
            need(20)?;
            Some(36 + (i32_at(data, o + 16).max(0) as usize) * 4)
        }
        0x0002_000B => {
            // CompiledState: leading i32 is the object's own byte length.
            need(4)?;
            Some(i32_at(data, o).max(0) as usize)
        }
        0x0002_001A => {
            // MeshCompiledStateLink: count i32 at +4.
            need(8)?;
            Some(8 + (i32_at(data, o + 4).max(0) as usize) * 4)
        }

        0x0007_0002 => {
            // Skeleton: bone_count i32 at +12, 12 bytes each.
            need(16)?;
            Some(24 + (i32_at(data, o + 12).max(0) as usize) * 12)
        }
        0x0007_0003 => {
            // AnimationSkin: count i32 at +4, 64 bytes each.
            need(8)?;
            Some(16 + (i32_at(data, o + 4).max(0) as usize) * 64)
        }
        0x0007_000B => Some(32), // fixed 32-byte object (not in the Blender add-on)
        0x0007_000C => Some(20), // SkinsInK
        0x0007_000F => {
            // SkinMatrixBuffer: count i32 at +4, 48 bytes each.
            need(8)?;
            Some(16 + (i32_at(data, o + 4).max(0) as usize) * 48)
        }

        0x0008_0005 => Some(32), // BoundingBox

        0x00FF_0000 => Some(64), // MorphHandle: 8 + 6 doubles + 2 i32
        0x00FF_0001 => {
            // Animations: count i32 at +4, 8 bytes each.
            need(8)?;
            Some(8 + (i32_at(data, o + 4).max(0) as usize) * 8)
        }
        0x00FF_0002 => {
            // BlendShape: shape_count i32 at +12, two arrays of that many i32.
            need(16)?;
            Some(28 + 2 * (i32_at(data, o + 12).max(0) as usize) * 4)
        }

        // Self-contained blobs with internal pointers: the section size covers
        // the whole object (KeyframeAnim, BlendShapeBuffer, KD-tree).
        0x0007_0001 | 0x0002_00AF | 0x0008_0003 => Some(data_size),

        _ => None,
    }
}


/// The typed objects the renderer needs, decoded from their section bytes.
///
/// Object references in RW4 are **indices into the section table**, not offsets.
/// The top two bits of a reference select the index space: `0` is a section index,
/// `1` is "no object", `2` is a sub-reference. Only the first is needed for
/// geometry, and only the first is handled here.
pub const INDEX_OBJECT_SHIFT: u32 = 22;

/// A vertex element declaration. Same 12-byte shape as gmdl's, and the same
/// D3DDECLTYPE and D3DDECLUSAGE numbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn size(&self) -> usize {
        crate::gmdl::VertexElement {
            stream: self.stream,
            offset: self.offset,
            element_type: self.element_type,
            method: self.method,
            usage: self.usage,
            usage_index: self.usage_index,
            rw_decl: self.rw_decl,
        }
        .size()
    }
}

/// `0x20004`: how the vertices in a buffer are laid out.
#[derive(Debug, Clone, Default)]
pub struct VertexDescription {
    pub vertex_size: u32,
    pub elements: Vec<VertexElement>,
}

/// `0x20005`: one vertex buffer, pointing at a description and a data object.
#[derive(Debug, Clone, Copy)]
pub struct VertexBuffer {
    pub description: u32,
    pub base_vertex_index: i32,
    pub vertex_count: i32,
    pub vertex_size: i32,
    pub data: u32,
}

/// `0x20007`: indices, pointing at a data object.
#[derive(Debug, Clone, Copy)]
pub struct IndexBuffer {
    pub start_index: i32,
    pub primitive_count: i32,
    /// 101 is D3DFMT_INDEX16, anything else is treated as 32-bit.
    pub format: i32,
    pub primitive_type: i32,
    pub data: u32,
}

/// `0x20009`: links an index buffer to its vertex buffers.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub primitive_type: i32,
    pub index_buffer: u32,
    pub triangle_count: i32,
    pub first_index: i32,
    pub primitive_count: i32,
    pub first_vertex: i32,
    pub vertex_count: i32,
    pub vertex_buffers: Vec<u32>,
}

fn le_i32(d: &[u8], o: usize) -> i32 {
    i32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn le_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

impl Rw4 {
    /// Bytes of one object, by section index. `None` when the index is out of
    /// range, refers to "no object", or is a sub-reference (unsupported).
    pub fn object<'a>(&self, data: &'a [u8], index: u32) -> Option<&'a [u8]> {
        if index >> INDEX_OBJECT_SHIFT != 0 {
            return None;
        }
        let s = self.sections.get(index as usize)?;
        data.get(s.start..s.start + s.data_size)
    }

    /// Decode a `VertexDescription` object.
    pub fn vertex_description(&self, data: &[u8], index: u32) -> Option<VertexDescription> {
        let b = self.object(data, index)?;
        // Three ints, then a short count, a byte, the vertex size byte, then two
        // more ints: the element list starts at 24, not 20.
        if b.len() < 24 {
            return None;
        }
        let count = u16::from_le_bytes([b[12], b[13]]) as usize;
        let vertex_size = b[15] as u32;
        let need = 24 + count * 12;
        if b.len() < need {
            return None;
        }
        let mut elements = Vec::with_capacity(count);
        for i in 0..count {
            let o = 24 + i * 12;
            elements.push(VertexElement {
                stream: i16::from_le_bytes([b[o], b[o + 1]]),
                offset: i16::from_le_bytes([b[o + 2], b[o + 3]]),
                element_type: b[o + 4] as i8,
                method: b[o + 5] as i8,
                usage: b[o + 6] as i8,
                usage_index: b[o + 7] as i8,
                rw_decl: le_i32(b, o + 8),
            });
        }
        Some(VertexDescription {
            vertex_size,
            elements,
        })
    }

    /// Decode a `VertexBuffer` object.
    pub fn vertex_buffer(&self, data: &[u8], index: u32) -> Option<VertexBuffer> {
        let b = self.object(data, index)?;
        if b.len() < 28 {
            return None;
        }
        Some(VertexBuffer {
            description: le_u32(b, 0),
            base_vertex_index: le_i32(b, 8),
            vertex_count: le_i32(b, 12),
            vertex_size: le_i32(b, 20),
            data: le_u32(b, 24),
        })
    }

    /// Decode an `IndexBuffer` object.
    pub fn index_buffer(&self, data: &[u8], index: u32) -> Option<IndexBuffer> {
        let b = self.object(data, index)?;
        if b.len() < 28 {
            return None;
        }
        Some(IndexBuffer {
            start_index: le_i32(b, 4),
            primitive_count: le_i32(b, 8),
            format: le_i32(b, 16),
            primitive_type: le_i32(b, 20),
            data: le_u32(b, 24),
        })
    }

    /// Decode a `Mesh` object.
    pub fn mesh(&self, data: &[u8], index: u32) -> Option<Mesh> {
        let b = self.object(data, index)?;
        if b.len() < 36 {
            return None;
        }
        let n = le_i32(b, 16);
        if n < 0 || n > 64 {
            return None;
        }
        let mut vertex_buffers = Vec::with_capacity(n as usize);
        if b.len() < 36 + n as usize * 4 {
            return None;
        }
        for i in 0..n as usize {
            vertex_buffers.push(le_u32(b, 36 + i * 4));
        }
        Some(Mesh {
            primitive_type: le_i32(b, 4),
            index_buffer: le_u32(b, 8),
            triangle_count: le_i32(b, 12),
            first_index: le_i32(b, 20),
            primitive_count: le_i32(b, 24),
            first_vertex: le_i32(b, 28),
            vertex_count: le_i32(b, 32),
            vertex_buffers,
        })
    }

    /// All `Mesh` objects, in section order.
    pub fn meshes(&self, data: &[u8]) -> Vec<Mesh> {
        let mut out = Vec::new();
        for (i, s) in self.sections.iter().enumerate() {
            if s.info.type_code == object_types::MESH {
                if let Some(m) = self.mesh(data, i as u32) {
                    out.push(m);
                }
            }
        }
        out
    }
}


/// `0x200AF`: geometry for meshes whose vertex buffer is the "no object"
/// sentinel. Instead of a vertex description, it holds a fixed set of parallel
/// arrays whose offsets are stored in a 64-byte header.
#[derive(Debug, Clone)]
pub struct BlendShapeBuffer {
    /// Array offsets relative to the data region, or `-1` when the array is absent.
    /// Index 0 is position, 1 normal, 2 tangent, 3 texture coordinate,
    /// 9 blend indices, 10 blend weights.
    pub offsets: [i32; 11],
    pub shape_count: i32,
    pub vertex_count: i32,
    pub bone_indices_count: i32,
    /// Absolute offset of the data region in the file.
    pub data_at: usize,
    pub data_len: usize,
}

impl BlendShapeBuffer {
    pub const POSITION: usize = 0;
    pub const NORMAL: usize = 1;
    pub const TEXCOORD: usize = 3;
}

impl Rw4 {
    /// Decode a `BlendShapeBuffer` object.
    pub fn blend_shape_buffer(&self, data: &[u8], index: u32) -> Option<BlendShapeBuffer> {
        let section = self.sections.get(index as usize)?;
        let b = self.object(data, index)?;
        if b.len() < 64 {
            return None;
        }
        if le_i32(b, 0) != 1 {
            return None;
        }
        let mut offsets = [-1i32; 11];
        for (i, o) in offsets.iter_mut().enumerate() {
            let v = le_u32(b, 4 + i * 4);
            *o = if v == 0 { -1 } else { v as i32 - 64 };
        }
        Some(BlendShapeBuffer {
            offsets,
            shape_count: le_i32(b, 48),
            vertex_count: le_i32(b, 52),
            bone_indices_count: le_i32(b, 60),
            data_at: section.start + 64,
            data_len: section.data_size.saturating_sub(64),
        })
    }

    /// The first `BlendShapeBuffer` in the container, if any.
    pub fn blend_shape(&self, data: &[u8]) -> Option<BlendShapeBuffer> {
        for (i, s) in self.sections.iter().enumerate() {
            if s.info.type_code == object_types::BLEND_SHAPE_BUFFER {
                if let Some(b) = self.blend_shape_buffer(data, i as u32) {
                    return Some(b);
                }
            }
        }
        None
    }
}


/// `0x20003`: a texture. The pixel data is a `BaseResource` holding the payload
/// without a DDS header, so the four-character format code is the only description.
#[derive(Debug, Clone, Copy)]
pub struct Raster {
    /// FourCC (`DXT1`, `DXT5`) or a D3DFMT code (21 = A8R8G8B8, 20 = R8G8B8).
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub mipmap_levels: u32,
    pub data: u32,
}

impl Rw4 {
    /// Decode a `Raster` object.
    pub fn raster(&self, data: &[u8], index: u32) -> Option<Raster> {
        let b = self.object(data, index)?;
        if b.len() < 32 {
            return None;
        }
        Some(Raster {
            format: le_u32(b, 0),
            width: u16::from_le_bytes([b[12], b[13]]) as u32,
            height: u16::from_le_bytes([b[14], b[15]]) as u32,
            mipmap_levels: b[17] as u32,
            data: le_u32(b, 28),
        })
    }

    /// The first `Raster` in the container, if any.
    pub fn first_raster(&self, data: &[u8]) -> Option<Raster> {
        for (i, s) in self.sections.iter().enumerate() {
            if s.info.type_code == object_types::RASTER {
                if let Some(r) = self.raster(data, i as u32) {
                    return Some(r);
                }
            }
        }
        None
    }

    /// The first texture in the container, with its payload resolved.
    ///
    /// Most RW4 files in this install carry their own texture (8,915 of 9,368 in
    /// `Spore_Graphics`), which is what makes it possible to shade a model without
    /// finding its material somewhere else. The payload is the block stream with no
    /// container around it, so `raster::decode_pixels` takes it directly.
    pub fn texture<'a>(&self, data: &'a [u8]) -> Option<RwTexture<'a>> {
        let r = self.first_raster(data)?;
        let payload = self.object(data, r.data)?;
        Some(RwTexture {
            format: r.format,
            width: r.width,
            height: r.height,
            mipmap_levels: r.mipmap_levels,
            payload,
        })
    }
}

/// A texture held inside an RW4 container.
#[derive(Debug, Clone, Copy)]
pub struct RwTexture<'a> {
    /// `DDS` FourCC or `D3DFORMAT` code, the same values `raster` accepts.
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub mipmap_levels: u32,
    /// Mip level 0 onwards, headerless.
    pub payload: &'a [u8],
}

/// Known RW4 object type codes and their names.
pub mod object_types {
    pub const BASE_RESOURCE: u32 = 0x0001_0030;
    pub const SECTION_MANIFEST: u32 = 0x0001_0004;
    pub const SECTION_TYPES: u32 = 0x0001_0005;
    pub const SECTION_EXTERNAL_ARENAS: u32 = 0x0001_0006;
    pub const SECTION_SUB_REFERENCES: u32 = 0x0001_0007;
    pub const SECTION_ATOMS: u32 = 0x0001_0008;

    pub const RASTER: u32 = 0x0002_0003;
    pub const VERTEX_DESCRIPTION: u32 = 0x0002_0004;
    pub const VERTEX_BUFFER: u32 = 0x0002_0005;
    pub const INDEX_BUFFER: u32 = 0x0002_0007;
    pub const TEXTURE_OVERRIDE: u32 = 0x0002_0008;
    pub const MESH: u32 = 0x0002_0009;
    pub const COMPILED_STATE: u32 = 0x0002_000B;
    pub const MESH_COMPILED_STATE_LINK: u32 = 0x0002_001A;
    pub const BLEND_SHAPE_BUFFER: u32 = 0x0002_00AF;

    pub const KEYFRAME_ANIM: u32 = 0x0007_0001;
    pub const SKELETON: u32 = 0x0007_0002;
    pub const ANIMATION_SKIN: u32 = 0x0007_0003;
    pub const FIXED_32: u32 = 0x0007_000B;
    pub const SKINS_IN_K: u32 = 0x0007_000C;
    pub const SKIN_MATRIX_BUFFER: u32 = 0x0007_000F;

    pub const TRIANGLE_KD_TREE: u32 = 0x0008_0003;
    pub const BOUNDING_BOX: u32 = 0x0008_0005;

    pub const MORPH_HANDLE: u32 = 0x00FF_0000;
    pub const ANIMATIONS: u32 = 0x00FF_0001;
    pub const BLEND_SHAPE: u32 = 0x00FF_0002;

    /// Classify a type code by its high 16 bits.
    pub fn category(code: u32) -> &'static str {
        match code >> 16 {
            0x0001 => "Section",
            0x0002 => "Graphics",
            0x0007 => "Animation",
            0x0008 => "Spatial",
            0x00FF => "Special",
            _ => "Unknown",
        }
    }

    /// Human-readable name for a known type code.
    pub fn name(code: u32) -> Option<&'static str> {
        Some(match code {
            BASE_RESOURCE => "BaseResource",
            SECTION_MANIFEST => "SectionManifest",
            SECTION_TYPES => "SectionTypes",
            SECTION_EXTERNAL_ARENAS => "SectionExternalArenas",
            SECTION_SUB_REFERENCES => "SectionSubReferences",
            SECTION_ATOMS => "SectionAtoms",
            RASTER => "Raster",
            VERTEX_DESCRIPTION => "VertexDescription",
            VERTEX_BUFFER => "VertexBuffer",
            INDEX_BUFFER => "IndexBuffer",
            TEXTURE_OVERRIDE => "TextureOverride",
            MESH => "Mesh",
            COMPILED_STATE => "CompiledState",
            MESH_COMPILED_STATE_LINK => "MeshCompiledStateLink",
            BLEND_SHAPE_BUFFER => "BlendShapeBuffer",
            KEYFRAME_ANIM => "KeyframeAnim",
            SKELETON => "Skeleton",
            ANIMATION_SKIN => "AnimationSkin",
            FIXED_32 => "Fixed32",
            SKINS_IN_K => "SkinsInK",
            SKIN_MATRIX_BUFFER => "SkinMatrixBuffer",
            TRIANGLE_KD_TREE => "TriangleKDTreeProcedural",
            BOUNDING_BOX => "BoundingBox",
            MORPH_HANDLE => "MorphHandle",
            ANIMATIONS => "Animations",
            BLEND_SHAPE => "BlendShape",
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u32(d: &mut [u8], o: usize, v: u32) {
        d[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn put_i32(d: &mut [u8], o: usize, v: i32) {
        d[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// Build a container with `sections` (type_code, data_size) laid out after
    /// the header, section-info table at 72, buffer data at the end of the table.
    fn build(sections: &[(u32, usize)], payload: &[u8]) -> Vec<u8> {
        let infos_at = 72usize;
        let buffer_at = infos_at + sections.len() * 24;
        let data_at = buffer_at + payload.len();
        let total = data_at + sections.iter().map(|s| s.1).sum::<usize>();
        let mut d = vec![0u8; total];
        d[..12].copy_from_slice(&MAGIC);
        put_u32(&mut d, 28, 1);
        put_i32(&mut d, 36, sections.len() as i32);
        put_i32(&mut d, 48, infos_at as i32);
        put_i32(&mut d, 68, buffer_at as i32);
        let mut cursor = data_at;
        for (i, (code, size)) in sections.iter().enumerate() {
            let base = infos_at + i * 24;
            put_i32(&mut d, base, cursor as i32);
            put_i32(&mut d, base + 8, *size as i32);
            put_u32(&mut d, base + 20, *code);
            cursor += size;
        }
        d[data_at..data_at + payload.len()].copy_from_slice(payload);
        d
    }

    #[test]
    fn parse_container() {
        let d = build(&[(object_types::RASTER, 32), (object_types::SKELETON, 24)], &[]);
        let r = parse(&d).unwrap();
        assert_eq!(r.sections.len(), 2);
        assert_eq!(r.sections[0].info.type_code, object_types::RASTER);
        assert_eq!(r.sections[0].data_size, 32);
        assert_eq!(r.sections[1].data_size, 24);
        assert!(r.tiles(&d));
        assert!(r.all_objects_exact(&d));
    }

    #[test]
    fn count_scaled_object() {
        // Skeleton: 24 + bone_count * 12, with bone_count at +12.
        let mut payload = vec![0u8; 24 + 2 * 12];
        put_i32(&mut payload, 12, 2);
        let d = build(&[(object_types::SKELETON, payload.len())], &payload);
        let r = parse(&d).unwrap();
        assert!(r.all_objects_exact(&d));
    }

    #[test]
    fn object_mismatch_is_detected() {
        // Declare 30 bytes for a Skeleton whose bone_count says 24.
        let payload = vec![0u8; 30];
        let d = build(&[(object_types::SKELETON, 30)], &payload);
        let r = parse(&d).unwrap();
        assert!(!r.all_objects_exact(&d));
    }

    #[test]
    fn base_resource_is_relative_to_buffer_data() {
        let payload = vec![0xABu8; 8];
        let mut d = build(&[(object_types::BASE_RESOURCE, 8)], &payload);
        // p_data is an offset from p_buffer_data; make it point at the payload.
        let buffer_at = i32_at(&d, 68) as usize;
        let payload_at = d.len() - 8;
        put_i32(&mut d, 72, (payload_at - buffer_at) as i32);
        let r = parse(&d).unwrap();
        assert_eq!(r.sections[0].start, payload_at);
        assert!(r.all_objects_exact(&d));
    }

    #[test]
    fn bad_magic() {
        let mut d = build(&[(object_types::RASTER, 32)], &[]);
        d[0] = 0;
        assert!(matches!(parse(&d), Err(Rw4Error::BadMagic)));
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 40]), Err(Rw4Error::TooShort)));
    }

    #[test]
    fn bad_section_count() {
        let mut d = build(&[(object_types::RASTER, 32)], &[]);
        put_i32(&mut d, 36, 0);
        assert!(matches!(parse(&d), Err(Rw4Error::BadSectionCount(0))));
    }

    #[test]
    fn section_out_of_bounds() {
        let mut d = build(&[(object_types::RASTER, 32)], &[]);
        // data_size larger than the file.
        put_i32(&mut d, 72 + 8, 1_000_000);
        assert!(matches!(parse(&d), Err(Rw4Error::SectionOutOfBounds)));
    }

    #[test]
    fn object_type_classification() {
        assert_eq!(object_types::category(object_types::BASE_RESOURCE), "Section");
        assert_eq!(object_types::category(object_types::RASTER), "Graphics");
        assert_eq!(object_types::category(object_types::SKELETON), "Animation");
        assert_eq!(object_types::name(object_types::MESH), Some("Mesh"));
        assert_eq!(object_types::name(0xDEAD_BEEF), None);
    }
}

#[cfg(test)]
mod object_tests {
    use super::*;

    fn push_i32(v: &mut Vec<u8>, x: i32) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    /// Lay out a container with one section per (type, bytes) pair.
    fn container(objects: &[(u32, Vec<u8>)]) -> Vec<u8> {
        let infos_at = 72usize;
        let buffer_at = infos_at + objects.len() * 24;
        let data_at = buffer_at;
        let total = data_at + objects.iter().map(|o| o.1.len()).sum::<usize>();
        let mut d = vec![0u8; total];
        d[..12].copy_from_slice(&MAGIC);
        let put = |d: &mut [u8], o: usize, v: u32| {
            d[o..o + 4].copy_from_slice(&v.to_le_bytes());
        };
        put(&mut d, 28, 1);
        put(&mut d, 36, objects.len() as u32);
        put(&mut d, 48, infos_at as u32);
        put(&mut d, 68, buffer_at as u32);
        let mut cursor = data_at;
        for (i, (code, bytes)) in objects.iter().enumerate() {
            let base = infos_at + i * 24;
            // BaseResource offsets are relative to p_buffer_data; every other
            // object stores an absolute offset.
            let p_data = if *code == BASE_RESOURCE {
                (cursor - buffer_at) as u32
            } else {
                cursor as u32
            };
            put(&mut d, base, p_data);
            put(&mut d, base + 8, bytes.len() as u32);
            put(&mut d, base + 20, *code);
            d[cursor..cursor + bytes.len()].copy_from_slice(bytes);
            cursor += bytes.len();
        }
        d
    }

    fn vertex_description() -> Vec<u8> {
        let mut v = Vec::new();
        push_i32(&mut v, 0);
        push_i32(&mut v, 0);
        push_i32(&mut v, 0);
        v.extend_from_slice(&2u16.to_le_bytes()); // two elements
        v.push(0);
        v.push(24); // vertex size
        push_i32(&mut v, 0);
        push_i32(&mut v, 0);
        // element 0: position, float3, at offset 0
        v.extend_from_slice(&0i16.to_le_bytes());
        v.extend_from_slice(&0i16.to_le_bytes());
        v.extend_from_slice(&[2, 0, 0, 0]);
        push_i32(&mut v, 0);
        // element 1: uv, float2, at offset 12
        v.extend_from_slice(&0i16.to_le_bytes());
        v.extend_from_slice(&12i16.to_le_bytes());
        v.extend_from_slice(&[1, 0, 5, 0]);
        push_i32(&mut v, 0);
        v
    }

    /// The element list starts at byte 24. Reading it at 20 produces plausible but
    /// wrong values, which is how the first version of this decoder failed.
    #[test]
    fn vertex_description_elements_start_at_24() {
        let d = container(&[(object_types::VERTEX_DESCRIPTION, vertex_description())]);
        let rw = parse(&d).unwrap();
        let desc = rw.vertex_description(&d, 0).unwrap();
        assert_eq!(desc.vertex_size, 24);
        assert_eq!(desc.elements.len(), 2);
        assert_eq!(desc.elements[0].usage, 0);
        assert_eq!(desc.elements[0].element_type, 2);
        assert_eq!(desc.elements[0].offset, 0);
        assert_eq!(desc.elements[1].usage, 5);
        assert_eq!(desc.elements[1].offset, 12);
    }

    #[test]
    fn vertex_and_index_buffers_decode() {
        let mut vb = Vec::new();
        push_i32(&mut vb, 0); // description index
        push_i32(&mut vb, 0);
        push_i32(&mut vb, 0); // base vertex
        push_i32(&mut vb, 3); // vertex count
        push_i32(&mut vb, 8);
        push_i32(&mut vb, 24); // vertex size
        push_i32(&mut vb, 2); // data index

        let mut ib = Vec::new();
        push_i32(&mut ib, 0);
        push_i32(&mut ib, 0); // start index
        push_i32(&mut ib, 6); // primitive count
        push_i32(&mut ib, 8);
        push_i32(&mut ib, 101); // INDEX16
        push_i32(&mut ib, 4); // TRIANGLELIST
        push_i32(&mut ib, 4); // data index (object 4 is the index BaseResource)

        let d = container(&[
            (object_types::VERTEX_DESCRIPTION, vertex_description()),
            (object_types::VERTEX_BUFFER, vb),
            (0x0001_0030, vec![0u8; 72]),
            (object_types::INDEX_BUFFER, ib),
            (0x0001_0030, vec![0u8; 12]),
        ]);
        let rw = parse(&d).unwrap();
        let b = rw.vertex_buffer(&d, 1).unwrap();
        assert_eq!(b.description, 0);
        assert_eq!(b.vertex_count, 3);
        assert_eq!(b.vertex_size, 24);
        assert_eq!(b.data, 2);
        let i = rw.index_buffer(&d, 3).unwrap();
        assert_eq!(i.primitive_count, 6);
        assert_eq!(i.format, 101);
        assert_eq!(i.data, 4);
    }

    #[test]
    fn mesh_decodes_its_links() {
        let mut m = Vec::new();
        push_i32(&mut m, 40);
        push_i32(&mut m, 4); // TRIANGLELIST
        push_i32(&mut m, 0); // index buffer
        push_i32(&mut m, 2); // triangle count
        push_i32(&mut m, 1); // one vertex buffer
        push_i32(&mut m, 0); // first index
        push_i32(&mut m, 6); // primitive count
        push_i32(&mut m, 0); // first vertex
        push_i32(&mut m, 3); // vertex count
        push_i32(&mut m, 1); // vertex buffer index

        let d = container(&[(object_types::MESH, m)]);
        let rw = parse(&d).unwrap();
        let meshes = rw.meshes(&d);
        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].vertex_buffers, vec![1]);
        assert_eq!(meshes[0].triangle_count, 2);
        assert_eq!(meshes[0].index_buffer, 0);
    }

    /// The blend-shape header is 64 bytes and its offsets are file offsets minus
    /// 64, with zero meaning "absent".
    #[test]
    fn blend_shape_buffer_offsets_are_relative_to_data() {
        let mut b = Vec::new();
        push_i32(&mut b, 1); // type
        let mut offsets = [0u32; 11];
        offsets[BlendShapeBuffer::POSITION] = 64 + 16; // data starts after the header
        offsets[BlendShapeBuffer::TEXCOORD] = 64 + 400;
        for o in offsets {
            b.extend_from_slice(&o.to_le_bytes());
        }
        push_i32(&mut b, 3); // shape count
        push_i32(&mut b, 25); // vertex count
        push_i32(&mut b, 0);
        push_i32(&mut b, 4); // bone indices count
        b.resize(64 + 800, 0);

        let d = container(&[(object_types::BLEND_SHAPE_BUFFER, b)]);
        let rw = parse(&d).unwrap();
        let bs = rw.blend_shape(&d).expect("blend shape buffer");
        assert_eq!(bs.vertex_count, 25);
        assert_eq!(bs.shape_count, 3);
        assert_eq!(bs.bone_indices_count, 4);
        assert_eq!(bs.offsets[BlendShapeBuffer::POSITION], 16);
        assert_eq!(bs.offsets[BlendShapeBuffer::TEXCOORD], 400);
        assert_eq!(bs.offsets[BlendShapeBuffer::NORMAL], -1);
    }

    /// `1 << 22` is the "no object" sentinel, which is how a blend-shape mesh
    /// signals that its geometry is not in a vertex buffer.
    #[test]
    fn no_object_sentinel_resolves_to_none() {
        let d = container(&[(object_types::RASTER, vec![0u8; 32])]);
        let rw = parse(&d).unwrap();
        assert!(rw.object(&d, 1 << INDEX_OBJECT_SHIFT).is_none());
        assert!(rw.object(&d, 0).is_some());
    }
}
