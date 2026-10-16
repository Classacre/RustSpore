//! Parser for the SPORE gmsh mesh format (type `0x01C135DA`).
//!
//! Status: 96.6% validated across 560 samples. Header, position data,
//! attribute sections, and index section version 1 are fully decoded.
//! Index section version 2+ is 91.3% decoded.
//!
//! Layout (all multi-byte integers in the indicated endianness):
//!
//! ```text
//! Header (24 bytes):
//!   offset  0: BE u32 = 1        (version)
//!   offset  4: BE u32 = 1        (flags)
//!   offset  8: LE u32 = 1024     (constant)
//!   offset 12: BE u16 = N        (section count)
//!   offset 14: BE u16 = 256      (constant)
//!   offset 16: LE u32 = 3        (floats per position vertex)
//!   offset 20: BE u16 = vcount   (vertex count)
//!   offset 22: BE u16 = 12       (position stride in bytes)
//!
//! Position data: vcount * 12 bytes of LE float3 XYZ
//!
//! Attribute sections (one or more):
//!   4 bytes: magic (varies by attribute type)
//!   4 bytes: BE u32 = vcount
//!   2 bytes: BE u16 = stride
//!   vcount * stride bytes of data
//!
//! Index section:
//!   Version 1: magic(15 00 06 08) + BE u32=1 + 24-byte header +
//!              count(BE u16) + stride(BE u16=2) + indices
//!   Version 2+: variable header with transformation matrices
//! ```

use crate::Vec3;

/// Errors that can occur when parsing a gmsh resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GmshError {
    /// The resource is too short to contain a valid header.
    TooShort,
    /// The position data extends past the end of the resource.
    PositionOverrun,
    /// An attribute section extends past the end of the resource.
    AttributeOverrun,
    /// The index section could not be decoded.
    IndexError,
}

impl std::fmt::Display for GmshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GmshError::TooShort => write!(f, "gmsh resource too short"),
            GmshError::PositionOverrun => write!(f, "position data overruns resource"),
            GmshError::AttributeOverrun => write!(f, "attribute section overruns resource"),
            GmshError::IndexError => write!(f, "index section decode failed"),
        }
    }
}

impl std::error::Error for GmshError {}

/// A parsed gmsh mesh resource.
#[derive(Debug, Clone)]
pub struct GmshMesh {
    /// Vertex positions (float3 XYZ).
    pub positions: Vec<Vec3>,
    /// Number of sections declared in the header.
    pub section_count: u16,
    /// Attribute sections (magic, stride, raw data).
    pub attributes: Vec<GmshAttribute>,
    /// Index data, if decoded.
    pub indices: Option<Vec<u16>>,
    /// Total resource size in bytes.
    pub total_size: usize,
}

/// One attribute section from a gmsh resource.
#[derive(Debug, Clone)]
pub struct GmshAttribute {
    /// 4-byte magic identifying the attribute type.
    pub magic: [u8; 4],
    /// Stride in bytes per vertex.
    pub stride: u16,
    /// Raw attribute data.
    pub data: Vec<u8>,
}

/// Parse a gmsh mesh resource.
pub fn parse(data: &[u8]) -> Result<GmshMesh, GmshError> {
    if data.len() < 24 {
        return Err(GmshError::TooShort);
    }

    let vcount = u16::from_be_bytes([data[20], data[21]]) as usize;
    let pos_stride = u16::from_be_bytes([data[22], data[23]]) as usize;
    let section_count = u16::from_be_bytes([data[12], data[13]]);

    // Position data
    let pos_size = vcount * pos_stride;
    let pos_end = 24 + pos_size;
    if pos_end > data.len() {
        return Err(GmshError::PositionOverrun);
    }

    let mut positions = Vec::with_capacity(vcount);
    for i in 0..vcount {
        let base = 24 + i * pos_stride;
        let x = f32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]);
        let y = f32::from_le_bytes([data[base + 4], data[base + 5], data[base + 6], data[base + 7]]);
        let z = f32::from_le_bytes([data[base + 8], data[base + 9], data[base + 10], data[base + 11]]);
        positions.push(Vec3::new(x, y, z));
    }

    // Attribute sections and index section
    let mut offset = pos_end;
    let mut attributes = Vec::new();
    let mut indices = None;
    let mut parsed_sections = 1; // position already counted

    while offset < data.len() && parsed_sections < section_count as usize {
        let remaining = data.len() - offset;

        // Check for index section magic
        if remaining >= 4 && &data[offset..offset + 4] == b"\x15\x00\x06\x08" {
            // Index section
            if remaining < 24 {
                return Err(GmshError::IndexError);
            }

            let version = u32::from_be_bytes([
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);

            if version == 1 {
                // Version 1: count at +20, stride at +22
                let count = u16::from_be_bytes([data[offset + 20], data[offset + 21]]) as usize;
                let stride = u16::from_be_bytes([data[offset + 22], data[offset + 23]]) as usize;
                let data_start = offset + 24;
                let data_size = count * stride;
                let data_end = data_start + data_size;

                if data_end > data.len() {
                    return Err(GmshError::IndexError);
                }

                if stride == 2 {
                    let mut idx = Vec::with_capacity(count);
                    for j in 0..count {
                        let base = data_start + j * 2;
                        idx.push(u16::from_le_bytes([data[base], data[base + 1]]));
                    }
                    indices = Some(idx);
                }

                offset = data_end;
                parsed_sections += 1;
                continue;
            } else {
                // Version 2+: try header-based decode
                let header_size = 16 + version as usize * 4;
                let count_offset = offset + header_size;
                if count_offset + 4 > data.len() {
                    return Err(GmshError::IndexError);
                }

                let count = u16::from_be_bytes([data[count_offset], data[count_offset + 1]]) as usize;
                let stride = u16::from_be_bytes([data[count_offset + 2], data[count_offset + 3]]) as usize;

                if stride != 48 {
                    return Err(GmshError::IndexError);
                }

                let matrix_end = count_offset + 4 + count * 48;

                // Try footer=8 then footer=10
                for footer_size in [8, 10] {
                    let idx_start = matrix_end + footer_size;
                    if idx_start + 20 > data.len() {
                        continue;
                    }

                    // Check first 10 u16 indices
                    let mut ok = true;
                    for j in 0..10 {
                        let base = idx_start + j * 2;
                        let v = u16::from_le_bytes([data[base], data[base + 1]]);
                        if v as usize >= vcount {
                            ok = false;
                            break;
                        }
                    }

                    if ok {
                        let mut idx = Vec::new();
                        let mut j = 0;
                        while idx_start + j * 2 + 2 <= data.len() {
                            let base = idx_start + j * 2;
                            let v = u16::from_le_bytes([data[base], data[base + 1]]);
                            if v as usize >= vcount {
                                break;
                            }
                            idx.push(v);
                            j += 1;
                        }
                        indices = Some(idx);
                        break;
                    }
                }

                // Skip to end (we don't fully decode v2+ structure)
                offset = data.len();
                parsed_sections += 1;
                continue;
            }
        }

        // Attribute section
        if remaining < 10 {
            break;
        }

        let magic = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
        let attr_vcount = u32::from_be_bytes([
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]) as usize;
        let attr_stride = u16::from_be_bytes([data[offset + 8], data[offset + 9]]) as usize;
        let data_size = attr_vcount * attr_stride;
        let data_end = offset + 10 + data_size;

        if data_end > data.len() {
            return Err(GmshError::AttributeOverrun);
        }

        attributes.push(GmshAttribute {
            magic,
            stride: attr_stride as u16,
            data: data[offset + 10..data_end].to_vec(),
        });

        offset = data_end;
        parsed_sections += 1;
    }

    Ok(GmshMesh {
        positions,
        section_count,
        attributes,
        indices,
        total_size: data.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal version 1 gmsh resource for testing.
    fn build_test_gmsh_v1() -> Vec<u8> {
        let mut data = Vec::new();

        // Header (24 bytes)
        data.extend_from_slice(&1u32.to_be_bytes()); // version
        data.extend_from_slice(&1u32.to_be_bytes()); // flags
        data.extend_from_slice(&1024u32.to_le_bytes()); // constant
        data.extend_from_slice(&3u16.to_be_bytes()); // section count (pos + attr + index)
        data.extend_from_slice(&256u16.to_be_bytes()); // constant
        data.extend_from_slice(&3u32.to_le_bytes()); // floats per position
        data.extend_from_slice(&14u16.to_be_bytes()); // vcount
        data.extend_from_slice(&12u16.to_be_bytes()); // pos stride

        // Position data: 14 vertices * 12 bytes
        for i in 0..14 {
            let x = (i as f32) * 0.1;
            data.extend_from_slice(&x.to_le_bytes());
            data.extend_from_slice(&x.to_le_bytes());
            data.extend_from_slice(&x.to_le_bytes());
        }

        // Attribute section: magic 02 00 07 01, vcount=14, stride=4
        data.extend_from_slice(&[0x02, 0x00, 0x07, 0x01]);
        data.extend_from_slice(&14u32.to_be_bytes());
        data.extend_from_slice(&4u16.to_be_bytes());
        for _ in 0..14 * 4 {
            data.push(0x7f);
        }

        // Index section v1
        data.extend_from_slice(b"\x15\x00\x06\x08");
        data.extend_from_slice(&1u32.to_be_bytes()); // version
        data.extend_from_slice(&1024u32.to_le_bytes()); // constant
        data.extend_from_slice(&0u32.to_be_bytes()); // zeros
        data.extend_from_slice(&1u16.to_be_bytes()); // 1
        data.extend_from_slice(&0u16.to_be_bytes()); // 0
        data.extend_from_slice(&36u16.to_be_bytes()); // count
        data.extend_from_slice(&2u16.to_be_bytes()); // stride
        for i in 0..36 {
            data.extend_from_slice(&(i as u16).to_le_bytes());
        }

        // Footer
        data.extend_from_slice(&8u16.to_be_bytes());
        data.extend_from_slice(&[0xff; 8]);

        data
    }

    #[test]
    fn parse_v1_header() {
        let data = build_test_gmsh_v1();
        let mesh = parse(&data).unwrap();
        assert_eq!(mesh.positions.len(), 14);
        assert_eq!(mesh.section_count, 3);
        assert_eq!(mesh.attributes.len(), 1);
        assert_eq!(mesh.attributes[0].magic, [0x02, 0x00, 0x07, 0x01]);
        assert_eq!(mesh.attributes[0].stride, 4);
    }

    #[test]
    fn parse_v1_indices() {
        let data = build_test_gmsh_v1();
        let mesh = parse(&data).unwrap();
        let indices = mesh.indices.unwrap();
        assert_eq!(indices.len(), 36);
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(idx, i as u16);
        }
    }

    #[test]
    fn parse_v1_positions() {
        let data = build_test_gmsh_v1();
        let mesh = parse(&data).unwrap();
        for (i, pos) in mesh.positions.iter().enumerate() {
            let expected = (i as f32) * 0.1;
            assert!((pos.x - expected).abs() < 1e-6);
            assert!((pos.y - expected).abs() < 1e-6);
            assert!((pos.z - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 10];
        assert!(parse(&data).is_err());
    }
}
