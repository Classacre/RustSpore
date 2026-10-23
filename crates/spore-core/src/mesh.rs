//! Engine-agnostic renderable geometry.
//!
//! A parser says what is in the file. This module says what a renderer needs:
//! triangle lists with positions, normals and texture coordinates, all indexed
//! consistently. It deliberately does not know about Bevy, so it can be tested and
//! validated headlessly against the whole install.
//!
//! ## Why normals are computed here
//!
//! Only 672 of the 2,756 gmsh resources carry float3 normals and 2,073 carry packed
//! normals, so 11 have none. Computing area-weighted vertex normals from the
//! triangles fills those in and gives one code path for every mesh. Where the file
//! does carry normals they are used as-is, because the artist's smoothing groups
//! are not recoverable from the geometry alone.

use crate::{gmsh, Vec3};

/// Why a mesh could not be turned into renderable geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshError {
    /// The mesh has no positions.
    Empty,
    /// There are no triangles to draw.
    NoIndices,
    /// An index pointed past the end of the vertex array.
    IndexOutOfRange { index: u32, vertices: usize },
    /// The attribute arrays are not all one entry per vertex.
    MismatchedAttributes,
    /// A position, normal or UV was not finite.
    NotFinite,
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::Empty => write!(f, "mesh has no vertices"),
            MeshError::NoIndices => write!(f, "mesh has no triangles"),
            MeshError::IndexOutOfRange { index, vertices } => {
                write!(f, "index {index} out of range for {vertices} vertices")
            }
            MeshError::MismatchedAttributes => {
                write!(f, "attribute arrays do not match the vertex count")
            }
            MeshError::NotFinite => write!(f, "mesh contains a non-finite value"),
        }
    }
}

impl std::error::Error for MeshError {}

/// A triangle mesh ready for upload.
#[derive(Debug, Clone, Default)]
pub struct MeshData {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// One pair per vertex. Filled with zeros when the file has no UV section.
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Whether normals came from the file rather than being computed.
    pub normals_from_file: bool,
    /// How many shading values were replaced because they were not finite.
    ///
    /// Some gmdl models store `NaN` in the UVs of unused vertices. A single such
    /// value poisons the interpolation for a whole triangle, so they are replaced
    /// with zero and counted here rather than passed to the GPU.
    pub sanitised: usize,
}

impl MeshData {
    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Axis-aligned bounds, or `None` when empty.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for p in &self.positions {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }
        if self.positions.is_empty() {
            None
        } else {
            Some((min, max))
        }
    }

    /// Longest axis of the bounding box, or 1.0 when degenerate.
    pub fn radius(&self) -> f32 {
        match self.bounds() {
            Some((min, max)) => {
                let d = Vec3::new(max.x - min.x, max.y - min.y, max.z - min.z);
                let r = d.x.max(d.y).max(d.z) * 0.5;
                if r.is_finite() && r > 0.0 {
                    r
                } else {
                    1.0
                }
            }
            None => 1.0,
        }
    }

    /// Check the invariants a renderer depends on.
    pub fn validate(&self) -> Result<(), MeshError> {
        if self.positions.is_empty() {
            return Err(MeshError::Empty);
        }
        if self.indices.is_empty() || self.indices.len() % 3 != 0 {
            return Err(MeshError::NoIndices);
        }
        for p in &self.positions {
            if !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite() {
                return Err(MeshError::NotFinite);
            }
        }
        for n in &self.normals {
            if !n.x.is_finite() || !n.y.is_finite() || !n.z.is_finite() {
                return Err(MeshError::NotFinite);
            }
        }
        for uv in &self.uvs {
            if !uv[0].is_finite() || !uv[1].is_finite() {
                return Err(MeshError::NotFinite);
            }
        }
        let n = self.positions.len() as u32;
        for &i in &self.indices {
            if i >= n {
                return Err(MeshError::IndexOutOfRange {
                    index: i,
                    vertices: self.positions.len(),
                });
            }
        }
        if self.normals.len() != self.positions.len() || self.uvs.len() != self.positions.len() {
            return Err(MeshError::MismatchedAttributes);
        }
        Ok(())
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}

fn normalized(v: Vec3) -> Vec3 {
    let len = v.length();
    if len > 1e-12 && len.is_finite() {
        Vec3::new(v.x / len, v.y / len, v.z / len)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    }
}

/// Area-weighted vertex normals from the triangle list.
fn compute_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut acc = vec![Vec3::ZERO; positions.len()];
    for tri in indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let n = cross(
            sub(positions[i1], positions[i0]),
            sub(positions[i2], positions[i0]),
        );
        for i in [i0, i1, i2] {
            acc[i] = Vec3::new(acc[i].x + n.x, acc[i].y + n.y, acc[i].z + n.z);
        }
    }
    acc.into_iter().map(normalized).collect()
}

/// Convert a parsed gmsh mesh into renderable geometry.
pub fn from_gmsh(mesh: &gmsh::GmshMesh) -> Result<MeshData, MeshError> {
    let positions = mesh.positions.clone();
    if positions.is_empty() {
        return Err(MeshError::Empty);
    }
    let indices: Vec<u32> = match &mesh.indices {
        Some(v) => v.iter().map(|&i| i as u32).collect(),
        None => return Err(MeshError::NoIndices),
    };
    if indices.is_empty() {
        return Err(MeshError::NoIndices);
    }
    // Bounds-check before computing normals: the normal pass indexes the vertex
    // array directly, so a bad index would panic there instead of returning an error.
    let n_vertices = positions.len() as u32;
    for &i in &indices {
        if i >= n_vertices {
            return Err(MeshError::IndexOutOfRange {
                index: i,
                vertices: positions.len(),
            });
        }
    }

    let (normals, normals_from_file) = match &mesh.normals {
        Some(n) if n.len() == positions.len() => (n.clone(), true),
        _ => (compute_normals(&positions, &indices), false),
    };

    let mut uvs = match &mesh.uvs {
        Some(u) if u.len() == positions.len() => u.clone(),
        _ => vec![[0.0, 0.0]; positions.len()],
    };
    let mut sanitised = 0usize;
    for uv in uvs.iter_mut() {
        if !uv[0].is_finite() || !uv[1].is_finite() {
            *uv = [0.0, 0.0];
            sanitised += 1;
        }
    }
    let mut normals = normals;
    for n in normals.iter_mut() {
        if !n.x.is_finite() || !n.y.is_finite() || !n.z.is_finite() {
            *n = Vec3::new(0.0, 1.0, 0.0);
            sanitised += 1;
        }
    }

    let out = MeshData {
        positions,
        normals,
        uvs,
        indices,
        normals_from_file,
        sanitised,
    };
    out.validate()?;
    Ok(out)
}


/// Append one part to an assembly, rotated then translated.
///
/// The rig stores a 3x3 rotation and a translation per part, so a part's vertices
/// are `R * v + t`. Normals get the rotation only: translating a direction is not a
/// thing. Indices are offset by the number of vertices already present.
///
/// Returns the number of vertices added, which is also the index offset used.
pub fn append_transformed(
    assembly: &mut MeshData,
    part: &MeshData,
    rotation: [[f32; 3]; 3],
    translation: [f32; 3],
) -> usize {
    let base = assembly.positions.len() as u32;
    let rot = |v: Vec3| -> Vec3 {
        Vec3::new(
            rotation[0][0] * v.x + rotation[0][1] * v.y + rotation[0][2] * v.z,
            rotation[1][0] * v.x + rotation[1][1] * v.y + rotation[1][2] * v.z,
            rotation[2][0] * v.x + rotation[2][1] * v.y + rotation[2][2] * v.z,
        )
    };
    for p in &part.positions {
        let r = rot(*p);
        assembly.positions.push(Vec3::new(
            r.x + translation[0],
            r.y + translation[1],
            r.z + translation[2],
        ));
    }
    for n in &part.normals {
        let r = rot(*n);
        let len = (r.x * r.x + r.y * r.y + r.z * r.z).sqrt();
        assembly.normals.push(if len > 1e-6 {
            Vec3::new(r.x / len, r.y / len, r.z / len)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        });
    }
    assembly.uvs.extend_from_slice(&part.uvs);
    assembly
        .indices
        .extend(part.indices.iter().map(|i| i + base));
    part.positions.len()
}

impl MeshData {
    /// An empty assembly to append parts to.
    pub fn assembly() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            normals_from_file: true,
            sanitised: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gmsh::{GmshMesh, GmshAttribute};

    fn tri_mesh(normals: Option<Vec<Vec3>>, uvs: Option<Vec<[f32; 2]>>) -> GmshMesh {
        GmshMesh {
            positions: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            normals,
            tangents: None,
            uvs,
            section_count: 2,
            attributes: Vec::<GmshAttribute>::new(),
            indices: Some(vec![0, 1, 2]),
            matrix_count: 0,
            total_size: 0,
        }
    }

    /// With no normal section the normals come from the triangles. A triangle in
    /// the XY plane wound counter-clockwise faces +Z.
    #[test]
    fn computes_normals_when_absent() {
        let m = from_gmsh(&tri_mesh(None, None)).unwrap();
        assert!(!m.normals_from_file);
        assert_eq!(m.normals.len(), 3);
        for n in &m.normals {
            assert!((n.z - 1.0).abs() < 1e-6, "expected +Z, got {n:?}");
        }
        assert_eq!(m.triangle_count(), 1);
    }

    /// File normals are kept, because smoothing groups are not recoverable from
    /// the geometry.
    #[test]
    fn keeps_file_normals() {
        let given = vec![Vec3::new(0.0, 0.0, -1.0); 3];
        let m = from_gmsh(&tri_mesh(Some(given), None)).unwrap();
        assert!(m.normals_from_file);
        assert!((m.normals[0].z + 1.0).abs() < 1e-6);
    }

    /// A missing UV section becomes zeros rather than a shorter array.
    #[test]
    fn fills_missing_uvs() {
        let m = from_gmsh(&tri_mesh(None, None)).unwrap();
        assert_eq!(m.uvs.len(), 3);
        assert_eq!(m.uvs[0], [0.0, 0.0]);
    }

    /// A UV section with the wrong length is ignored rather than trusted.
    #[test]
    fn ignores_mismatched_uvs() {
        let m = from_gmsh(&tri_mesh(None, Some(vec![[0.5, 0.5]]))).unwrap();
        assert_eq!(m.uvs.len(), 3);
        assert_eq!(m.uvs[1], [0.0, 0.0]);
    }

    #[test]
    fn rejects_out_of_range_indices() {
        let mut mesh = tri_mesh(None, None);
        mesh.indices = Some(vec![0, 1, 9]);
        match from_gmsh(&mesh) {
            Err(MeshError::IndexOutOfRange { index: 9, vertices: 3 }) => {}
            other => panic!("expected IndexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_indices() {
        let mut mesh = tri_mesh(None, None);
        mesh.indices = None;
        assert_eq!(from_gmsh(&mesh).unwrap_err(), MeshError::NoIndices);
    }

    #[test]
    fn bounds_and_radius() {
        let m = from_gmsh(&tri_mesh(None, None)).unwrap();
        let (min, max) = m.bounds().unwrap();
        assert_eq!((min.x, min.y, min.z), (0.0, 0.0, 0.0));
        assert_eq!((max.x, max.y, max.z), (1.0, 1.0, 0.0));
        assert!((m.radius() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn degenerate_positions_are_rejected() {
        let mut mesh = tri_mesh(None, None);
        mesh.positions[0] = Vec3::new(f32::NAN, 0.0, 0.0);
        assert_eq!(from_gmsh(&mesh).unwrap_err(), MeshError::NotFinite);
    }

    /// A part is rotated, then translated. A 90 degree turn about Z sends +X to +Y,
    /// and the translation lands on top of that.
    #[test]
    fn append_transformed_rotates_then_translates() {
        let part = from_gmsh(&tri_mesh(None, None)).unwrap();
        let mut assembly = MeshData::assembly();
        let rot = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let added = append_transformed(&mut assembly, &part, rot, [10.0, 0.0, 0.0]);
        assert_eq!(added, 3);
        assert_eq!(assembly.positions.len(), 3);
        assert_eq!(assembly.indices, vec![0, 1, 2]);
        // (0,0,0) stays at the origin of the part, then moves to x=10.
        assert_eq!(assembly.positions[0], Vec3::new(10.0, 0.0, 0.0));
        // (1,0,0) becomes (0,1,0), then x=10.
        assert!((assembly.positions[1].x - 10.0).abs() < 1e-6);
        assert!((assembly.positions[1].y - 1.0).abs() < 1e-6);
    }

    /// Appending twice offsets the second part's indices, so one draw call covers
    /// both parts.
    #[test]
    fn append_transformed_offsets_indices() {
        let part = from_gmsh(&tri_mesh(None, None)).unwrap();
        let mut assembly = MeshData::assembly();
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        append_transformed(&mut assembly, &part, identity, [0.0, 0.0, 0.0]);
        append_transformed(&mut assembly, &part, identity, [5.0, 0.0, 0.0]);
        assert_eq!(assembly.positions.len(), 6);
        assert_eq!(assembly.indices, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(assembly.triangle_count(), 2);
    }

    /// Normals are rotated but not translated, and stay unit length.
    #[test]
    fn append_transformed_rotates_normals_only() {
        let given = vec![Vec3::new(0.0, 0.0, 1.0); 3];
        let part = from_gmsh(&tri_mesh(Some(given), None)).unwrap();
        let mut assembly = MeshData::assembly();
        let rot = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        append_transformed(&mut assembly, &part, rot, [100.0, 0.0, 0.0]);
        for n in &assembly.normals {
            assert!((n.z - 1.0).abs() < 1e-6, "Z is the rotation axis, got {n:?}");
        }
    }
}

fn read_f32(data: &[u8], at: usize) -> f32 {
    if at + 4 > data.len() {
        return 0.0;
    }
    f32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

/// Decode up to four components of a D3D vertex element.
fn read_element(data: &[u8], at: usize, ty: i8) -> [f32; 4] {
    let f32_at = |o: usize| -> f32 {
        if at + o + 4 <= data.len() {
            f32::from_le_bytes([data[at + o], data[at + o + 1], data[at + o + 2], data[at + o + 3]])
        } else {
            0.0
        }
    };
    let u8_at = |o: usize| -> u8 { data.get(at + o).copied().unwrap_or(0) };
    let i16_at = |o: usize| -> i16 {
        if at + o + 2 <= data.len() {
            i16::from_le_bytes([data[at + o], data[at + o + 1]])
        } else {
            0
        }
    };
    let u16_at = |o: usize| -> u16 {
        if at + o + 2 <= data.len() {
            u16::from_le_bytes([data[at + o], data[at + o + 1]])
        } else {
            0
        }
    };
    let u32_at = |o: usize| -> u32 {
        if at + o + 4 <= data.len() {
            u32::from_le_bytes([data[at + o], data[at + o + 1], data[at + o + 2], data[at + o + 3]])
        } else {
            0
        }
    };

    match ty {
        0 => [f32_at(0), 0.0, 0.0, 0.0],                          // FLOAT1
        1 => [f32_at(0), f32_at(4), 0.0, 0.0],                    // FLOAT2
        2 => [f32_at(0), f32_at(4), f32_at(8), 0.0],              // FLOAT3
        3 => [f32_at(0), f32_at(4), f32_at(8), f32_at(12)],       // FLOAT4
        4 => [                                                      // D3DCOLOR, BGRA
            u8_at(2) as f32 / 255.0,
            u8_at(1) as f32 / 255.0,
            u8_at(0) as f32 / 255.0,
            u8_at(3) as f32 / 255.0,
        ],
        5 | 8 => [                                                  // UBYTE4, UBYTE4N
            u8_at(0) as f32 / 255.0,
            u8_at(1) as f32 / 255.0,
            u8_at(2) as f32 / 255.0,
            u8_at(3) as f32 / 255.0,
        ],
        6 => [i16_at(0) as f32, i16_at(2) as f32, 0.0, 0.0],        // SHORT2
        7 => [                                                      // SHORT4
            i16_at(0) as f32,
            i16_at(2) as f32,
            i16_at(4) as f32,
            i16_at(6) as f32,
        ],
        9 => [                                                      // SHORT2N
            i16_at(0) as f32 / 32767.0,
            i16_at(2) as f32 / 32767.0,
            0.0,
            0.0,
        ],
        10 => [                                                     // SHORT4N
            i16_at(0) as f32 / 32767.0,
            i16_at(2) as f32 / 32767.0,
            i16_at(4) as f32 / 32767.0,
            i16_at(6) as f32 / 32767.0,
        ],
        11 => [                                                     // USHORT2N
            u16_at(0) as f32 / 65535.0,
            u16_at(2) as f32 / 65535.0,
            0.0,
            0.0,
        ],
        12 => [                                                     // USHORT4N
            u16_at(0) as f32 / 65535.0,
            u16_at(2) as f32 / 65535.0,
            u16_at(4) as f32 / 65535.0,
            u16_at(6) as f32 / 65535.0,
        ],
        13 => {                                                     // UDEC3
            let v = u32_at(0);
            [
                (v & 0x3FF) as f32 / 1023.0,
                ((v >> 10) & 0x3FF) as f32 / 1023.0,
                ((v >> 20) & 0x3FF) as f32 / 1023.0,
                1.0,
            ]
        }
        14 => {                                                     // DEC3N
            let v = u32_at(0) as i32;
            let comp = |shift: u32| -> f32 {
                let raw = ((v >> shift) & 0x3FF) as i32;
                let signed = if raw > 511 { raw - 1024 } else { raw };
                signed as f32 / 511.0
            };
            [comp(0), comp(10), comp(20), 1.0]
        }
        15 => [f16_at(data, at), f16_at(data, at + 2), 0.0, 0.0],   // FLOAT16_2
        16 => [                                                     // FLOAT16_4
            f16_at(data, at),
            f16_at(data, at + 2),
            f16_at(data, at + 4),
            f16_at(data, at + 6),
        ],
        _ => [0.0, 0.0, 0.0, 0.0],
    }
}

/// Decode an IEEE 754 half float. Rust has no stable `f16`, and the alternative is
/// another dependency for six lines of bit twiddling.
fn f16_at(data: &[u8], at: usize) -> f32 {
    if at + 2 > data.len() {
        return 0.0;
    }
    let h = u16::from_le_bytes([data[at], data[at + 1]]);
    let sign = if h & 0x8000 != 0 { -1.0f32 } else { 1.0 };
    let exp = ((h >> 10) & 0x1F) as i32;
    let mant = (h & 0x3FF) as f32;
    match exp {
        0 => sign * mant * 2.0f32.powi(-24),
        31 => {
            if mant == 0.0 {
                sign * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => sign * (1.0 + mant / 1024.0) * 2.0f32.powi(exp - 15),
    }
}


/// Read a normal element.
///
/// D3DDECLTYPE 5 is `UBYTE4`, which nominally means unsigned 0..255, but SPORE
/// stores normals in it as signed bytes: `b / 127.5 - 1`. The gmsh packed-normal
/// sections use the same convention, and decoding them as unsigned would put every
/// normal in the positive octant. Measured: unit length on 100% of samples.
fn read_normal(data: &[u8], at: usize, ty: i8) -> Vec3 {
    if ty == 5 || ty == 8 {
        let b = |o: usize| -> f32 { (data.get(at + o).copied().unwrap_or(127) as f64 / 127.5 - 1.0) as f32 };
        return Vec3::new(b(0), b(1), b(2));
    }
    let v = read_element(data, at, ty);
    Vec3::new(v[0], v[1], v[2])
}

/// Convert a parsed gmdl model into renderable geometry.
///
/// A gmdl holds several meshes, each pairing a triangle buffer with a vertex buffer
/// whose format is a list of D3D vertex elements. All of them are merged into one
/// indexed mesh, offsetting the indices as each part is appended.
///
/// `data` is the resource the model was parsed from; gmdl keeps offsets rather than
/// copying vertex data, so the raw bytes are needed here.
pub fn from_gmdl(data: &[u8], model: &crate::gmdl::Gmdl) -> Result<MeshData, MeshError> {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut any_normals = true;
    let mut any_uvs = true;

    for mesh in &model.meshes {
        if mesh.triangle_index < 0 || mesh.vertex_index < 0 {
            continue;
        }
        let Some(tri) = model.triangle_buffers.get(mesh.triangle_index as usize) else {
            continue;
        };
        let Some(vb) = model.vertex_buffers.get(mesh.vertex_index as usize) else {
            continue;
        };
        let Some(format) = model.vertex_formats.get(vb.format_index as usize) else {
            continue;
        };

        let base = positions.len() as u32;
        let count = vb.vertex_count.max(0) as usize;
        let stride = vb.stride;

        // Locate the elements the renderer needs. Usage values are D3DDECLUSAGE:
        // 0 position, 3 normal, 5 texture coordinate.
        let pos_el = format
            .elements
            .iter()
            .find(|e| e.usage == 0 && e.stream == 0);
        let nrm_el = format.elements.iter().find(|e| e.usage == 3 && e.stream == 0);
        let uv_el = format
            .elements
            .iter()
            .find(|e| e.usage == 5 && e.usage_index == 0 && e.stream == 0);

        let Some(pos_el) = pos_el else {
            continue;
        };
        if nrm_el.is_none() {
            any_normals = false;
        }
        if uv_el.is_none() {
            any_uvs = false;
        }

        for i in 0..count {
            let at = vb.data_offset + i * stride;
            if at + stride > data.len() {
                break;
            }
            let p = read_element(data, at + pos_el.offset.max(0) as usize, pos_el.element_type);
            positions.push(Vec3::new(p[0], p[1], p[2]));
            match nrm_el {
                Some(e) => normals.push(read_normal(
                    data,
                    at + e.offset.max(0) as usize,
                    e.element_type,
                )),
                None => normals.push(Vec3::ZERO),
            }
            match uv_el {
                Some(e) => {
                    let t = read_element(data, at + e.offset.max(0) as usize, e.element_type);
                    uvs.push([t[0], t[1]]);
                }
                None => uvs.push([0.0, 0.0]),
            }
        }

        for &i in &tri.indices {
            indices.push(base + i);
        }
    }

    if positions.is_empty() {
        return Err(MeshError::Empty);
    }
    if indices.is_empty() {
        return Err(MeshError::NoIndices);
    }

    let n_vertices = positions.len() as u32;
    for &i in &indices {
        if i >= n_vertices {
            return Err(MeshError::IndexOutOfRange {
                index: i,
                vertices: positions.len(),
            });
        }
    }

    // Only compute normals when the file has none for at least one part; a partial
    // mix would make the lighting inconsistent between parts.
    let normals_from_file = any_normals;
    let normals = if any_normals {
        normals.into_iter().map(normalized).collect()
    } else {
        compute_normals(&positions, &indices)
    };
    let mut uvs = if any_uvs {
        uvs
    } else {
        vec![[0.0, 0.0]; positions.len()]
    };
    let mut sanitised = 0usize;
    for uv in uvs.iter_mut() {
        if !uv[0].is_finite() || !uv[1].is_finite() {
            *uv = [0.0, 0.0];
            sanitised += 1;
        }
    }
    let mut normals: Vec<Vec3> = normals;
    for n in normals.iter_mut() {
        if !n.x.is_finite() || !n.y.is_finite() || !n.z.is_finite() {
            *n = Vec3::new(0.0, 1.0, 0.0);
            sanitised += 1;
        }
    }

    let out = MeshData {
        positions,
        normals,
        uvs,
        indices,
        normals_from_file,
        sanitised,
    };
    out.validate()?;
    Ok(out)
}

/// Convert an RW4 container's geometry into renderable geometry.
///
/// An RW4 file holds a table of objects; a `Mesh` object links an `IndexBuffer`
/// to one or more `VertexBuffer`s, each of which points at a `VertexDescription`
/// (the element list) and a `BaseResource` (the bytes). Every link is a section
/// index, not an offset.
///
/// When the vertex buffer reference is the "no object" sentinel, the geometry is
/// in a `BlendShapeBuffer` instead: parallel arrays at fixed offsets, with 16 bytes
/// per vertex whatever the array holds.
///
/// Only stream 0 is handled. Where a mesh declares several vertex buffers they are
/// separate streams, and nothing in this install uses more than one.
pub fn from_rw4(data: &[u8], rw: &crate::rw4::Rw4) -> Result<MeshData, MeshError> {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut any_normals = true;
    let mut any_uvs = true;
    let blend = rw.blend_shape(data);

    for m in rw.meshes(data) {
        // Pick the vertex buffer that declares a position element.
        let mut chosen = None;
        for vb_index in &m.vertex_buffers {
            let Some(vb) = rw.vertex_buffer(data, *vb_index) else {
                continue;
            };
            let Some(desc) = rw.vertex_description(data, vb.description) else {
                continue;
            };
            if desc.elements.iter().any(|e| e.usage == 0) {
                chosen = Some((vb, desc));
                break;
            }
        }

        let base = positions.len() as u32;

        if let Some((vb, desc)) = chosen {
            let Some(vbuf) = rw.object(data, vb.data) else {
                continue;
            };
            let count = vb.vertex_count.max(0) as usize;
            let stride = if vb.vertex_size > 0 {
                vb.vertex_size as usize
            } else {
                desc.vertex_size as usize
            };
            if stride == 0 {
                continue;
            }
            let pos_el = desc.elements.iter().find(|e| e.usage == 0 && e.stream == 0);
            let nrm_el = desc.elements.iter().find(|e| e.usage == 3 && e.stream == 0);
            let uv_el = desc
                .elements
                .iter()
                .find(|e| e.usage == 5 && e.usage_index == 0 && e.stream == 0);
            let Some(pos_el) = pos_el else { continue };
            if nrm_el.is_none() {
                any_normals = false;
            }
            if uv_el.is_none() {
                any_uvs = false;
            }
            for i in 0..count {
                let at = i * stride;
                if at + stride > vbuf.len() {
                    break;
                }
                let p = read_element(vbuf, at + pos_el.offset.max(0) as usize, pos_el.element_type);
                positions.push(Vec3::new(p[0], p[1], p[2]));
                match nrm_el {
                    Some(e) => normals.push(read_normal(
                        vbuf,
                        at + e.offset.max(0) as usize,
                        e.element_type,
                    )),
                    None => normals.push(Vec3::ZERO),
                }
                match uv_el {
                    Some(e) => {
                        let t = read_element(vbuf, at + e.offset.max(0) as usize, e.element_type);
                        uvs.push([t[0], t[1]]);
                    }
                    None => uvs.push([0.0, 0.0]),
                }
            }
        } else if let Some(bs) = &blend {
            // Blend-shape geometry: position at offsets[0], UV at offsets[3], both
            // 16 bytes per vertex. Normals are not stored, so they are computed.
            let count = bs.vertex_count.max(0) as usize;
            let pos_off = bs.offsets[crate::rw4::BlendShapeBuffer::POSITION];
            if pos_off < 0 {
                continue;
            }
            let pos_at = bs.data_at as i64 + pos_off as i64;
            let uv_off = bs.offsets[crate::rw4::BlendShapeBuffer::TEXCOORD];
            let uv_at = bs.data_at as i64 + uv_off as i64;
            any_normals = false;
            for i in 0..count {
                let at = pos_at + (i as i64) * 16;
                if at < 0 || at as usize + 12 > data.len() {
                    break;
                }
                let a = at as usize;
                positions.push(Vec3::new(
                    read_f32(data, a),
                    read_f32(data, a + 4),
                    read_f32(data, a + 8),
                ));
                normals.push(Vec3::ZERO);
                if uv_off >= 0 {
                    let u = uv_at + (i as i64) * 16;
                    if u >= 0 && u as usize + 8 <= data.len() {
                        let ua = u as usize;
                        uvs.push([read_f32(data, ua), read_f32(data, ua + 4)]);
                    } else {
                        uvs.push([0.0, 0.0]);
                    }
                } else {
                    uvs.push([0.0, 0.0]);
                }
            }
            if uv_off < 0 {
                any_uvs = false;
            }
        } else {
            continue;
        }

        // Indices. `first_index` and `triangle_count` select this mesh's slice of
        // the shared index buffer; `start_index` is added to every value.
        let Some(ib) = rw.index_buffer(data, m.index_buffer) else {
            continue;
        };
        let Some(ibuf) = rw.object(data, ib.data) else {
            continue;
        };
        let wide = ib.format != 101;
        let total = ib.primitive_count.max(0) as usize;
        let first = m.first_index.max(0) as usize;
        let tri = m.triangle_count.max(0) as usize;
        let take = if tri > 0 { tri * 3 } else { total };
        let end = (first + take).min(total);
        let add = ib.start_index.max(0) as u32;
        for i in first..end {
            let at = if wide { i * 4 } else { i * 2 };
            if at + if wide { 4 } else { 2 } > ibuf.len() {
                break;
            }
            let v = if wide {
                u32::from_le_bytes([ibuf[at], ibuf[at + 1], ibuf[at + 2], ibuf[at + 3]])
            } else {
                u16::from_le_bytes([ibuf[at], ibuf[at + 1]]) as u32
            };
            indices.push(base + v + add);
        }
    }

    if positions.is_empty() {
        return Err(MeshError::Empty);
    }
    if indices.is_empty() {
        return Err(MeshError::NoIndices);
    }

    let n_vertices = positions.len() as u32;
    for &i in &indices {
        if i >= n_vertices {
            return Err(MeshError::IndexOutOfRange {
                index: i,
                vertices: positions.len(),
            });
        }
    }

    let normals_from_file = any_normals;
    let mut normals: Vec<Vec3> = if any_normals {
        normals.into_iter().map(normalized).collect()
    } else {
        compute_normals(&positions, &indices)
    };
    let mut uvs = if any_uvs {
        uvs
    } else {
        vec![[0.0, 0.0]; positions.len()]
    };
    let mut sanitised = 0usize;
    for uv in uvs.iter_mut() {
        if !uv[0].is_finite() || !uv[1].is_finite() {
            *uv = [0.0, 0.0];
            sanitised += 1;
        }
    }
    for n in normals.iter_mut() {
        if !n.x.is_finite() || !n.y.is_finite() || !n.z.is_finite() {
            *n = Vec3::new(0.0, 1.0, 0.0);
            sanitised += 1;
        }
    }

    let out = MeshData {
        positions,
        normals,
        uvs,
        indices,
        normals_from_file,
        sanitised,
    };
    out.validate()?;
    Ok(out)


}
