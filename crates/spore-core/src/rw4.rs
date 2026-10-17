//! RW4 container format parser.
//!
//! Type `0x2F4E681B`. RenderWare 4 asset container.
//!
//! The header is 96 bytes, big-endian throughout. Two subtypes exist:
//! 0x01 (a family of object types with variable payload sizes) and
//! 0x04 (uniform, always mesh/shader-like, footer exactly 372 bytes).

use std::fmt;

/// The 12-byte magic signature every RW4 resource begins with.
pub const MAGIC: [u8; 12] = [0x89, b'R', b'W', b'4', b'w', b'3', b'2', 0x00, 0x0D, 0x0A, 0x1A, 0x0A];

/// Size of the RW4 header in bytes.
pub const HEADER_SIZE: usize = 96;

/// Subtype discriminator at offset 28.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Subtype {
    /// Family of related types with variable payload sizes.
    Family = 1,
    /// Uniform type, footer always 372 bytes.
    Uniform = 4,
    /// Unknown subtype value.
    Unknown(u32),
}

impl From<u32> for Subtype {
    fn from(v: u32) -> Self {
        match v {
            1 => Subtype::Family,
            4 => Subtype::Uniform,
            other => Subtype::Unknown(other),
        }
    }
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Subtype::Family => write!(f, "family(1)"),
            Subtype::Uniform => write!(f, "uniform(4)"),
            Subtype::Unknown(v) => write!(f, "unknown({})", v),
        }
    }
}

/// A parsed RW4 header.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Subtype discriminator at offset 28.
    pub subtype: Subtype,

    /// Value at offset 32, always equals value at offset 36.
    pub off32: u32,

    /// Value at offset 36, always equals value at offset 32.
    pub off36: u32,

    /// Flags at offset 40. 0x04 when subtype=4, 0x10 when subtype=1.
    pub off40: u32,

    /// Variable value at offset 48.
    pub off48: u32,

    /// Payload offset at offset 52. Invariant = 152.
    pub off52: u32,

    /// Footer size at offset 68.
    pub off68: u32,

    /// Flags at offset 72. 0x04 when subtype=4, 0x10 when subtype=1.
    pub off72: u32,

    /// Footer offset at offset 76.
    pub off76: u32,

    /// Invariant value at offset 80.
    pub off80: u32,

    /// Invariant value at offset 88.
    pub off88: u32,

    /// Invariant value at offset 96.
    pub off96: u32,

    /// Variable value at offset 100.
    pub off100: u32,

    /// Invariant value at offset 104.
    pub off104: u32,

    /// Invariant value at offset 112.
    pub off112: u32,

    /// Invariant value at offset 120.
    pub off120: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rw4Error {
    /// Payload too short for a 96-byte header.
    TooShort,
    /// Magic signature mismatch.
    BadMagic,
}

impl fmt::Display for Rw4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rw4Error::TooShort => write!(f, "payload too short for RW4 header"),
            Rw4Error::BadMagic => write!(f, "bad RW4 magic signature"),
        }
    }
}

impl std::error::Error for Rw4Error {}

/// Parse the RW4 header from a payload.
///
/// The entire header is big-endian. Returns the parsed [`Header`] with all
/// documented fields extracted.
pub fn parse_header(data: &[u8]) -> Result<Header, Rw4Error> {
    if data.len() < HEADER_SIZE {
        return Err(Rw4Error::TooShort);
    }
    if data[..12] != MAGIC {
        return Err(Rw4Error::BadMagic);
    }

    let u32_at = |off: usize| -> u32 {
        u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
    };

    Ok(Header {
        subtype: Subtype::from(u32_at(28)),
        off32: u32_at(32),
        off36: u32_at(36),
        off40: u32_at(40),
        off48: u32_at(48),
        off52: u32_at(52),
        off68: u32_at(68),
        off72: u32_at(72),
        off76: u32_at(76),
        off80: u32_at(80),
        off88: u32_at(88),
        off96: u32_at(96),
        off100: u32_at(100),
        off104: u32_at(104),
        off112: u32_at(112),
        off120: u32_at(120),
    })
}

impl Header {
    /// Compute the footer size from the total payload length.
    ///
    /// The footer invariant: `off68 = len(payload) - off76`.
    pub fn footer_size(&self, payload_len: usize) -> u32 {
        payload_len as u32 - self.off76
    }

    /// Check whether the footer invariant holds for a given payload length.
    pub fn footer_invariant_ok(&self, payload_len: usize) -> bool {
        self.off68 == payload_len as u32 - self.off76
    }

    /// Compute the payload size (everything after the 96-byte header).
    pub fn payload_size(&self, total_len: usize) -> usize {
        total_len.saturating_sub(HEADER_SIZE)
    }
}

/// Known RW4 object type codes that appear in the payload dispatch table.
pub mod object_types {
    // BaseResource family
    pub const BASE_RESOURCE: u32 = 0x00010030;
    pub const BASE_RESOURCE_31: u32 = 0x00010031;
    pub const BASE_RESOURCE_32: u32 = 0x00010032;

    // Graphics objects
    pub const RASTER: u32 = 0x00020003;
    pub const VERTEX_DESCRIPTION: u32 = 0x00020004;
    pub const VERTEX_BUFFER: u32 = 0x00020005;
    pub const INDEX_BUFFER: u32 = 0x00020007;
    pub const MESH: u32 = 0x00020009;
    pub const BLEND_SHAPE_BUFFER: u32 = 0x000200AF;

    // Animation objects
    pub const KEYFRAME_ANIM: u32 = 0x00070001;
    pub const SKELETON: u32 = 0x00070002;

    /// Classify a type code by its high 16 bits.
    pub fn category(code: u32) -> &'static str {
        match code >> 16 {
            0x0001 => "BaseResource",
            0x0002 => "Graphics",
            0x0007 => "Animation",
            _ => "Unknown",
        }
    }

    /// Human-readable name for a known type code.
    pub fn name(code: u32) -> Option<&'static str> {
        match code {
            BASE_RESOURCE => Some("BaseResource"),
            BASE_RESOURCE_31 => Some("BaseResource_31"),
            BASE_RESOURCE_32 => Some("BaseResource_32"),
            RASTER => Some("Raster"),
            VERTEX_DESCRIPTION => Some("VertexDescription"),
            VERTEX_BUFFER => Some("VertexBuffer"),
            INDEX_BUFFER => Some("IndexBuffer"),
            MESH => Some("Mesh"),
            BLEND_SHAPE_BUFFER => Some("BlendShapeBuffer"),
            KEYFRAME_ANIM => Some("KeyframeAnim"),
            SKELETON => Some("Skeleton"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_rw4(subtype: u32, off48: u32, off68: u32, off76: u32, off100: u32) -> Vec<u8> {
        let mut d = vec![0u8; 128];
        d[..12].copy_from_slice(&MAGIC);
        // Platform "454\0" at +16, version "000\0" at +20
        d[16..20].copy_from_slice(b"454\0");
        d[20..24].copy_from_slice(b"000\0");
        // off32 = off36
        d[32..36].copy_from_slice(&100u32.to_be_bytes());
        d[36..40].copy_from_slice(&100u32.to_be_bytes());
        // off40: 0x04 or 0x10
        d[40..44].copy_from_slice(&if subtype == 4 { 4u32 } else { 0x10u32 }.to_be_bytes());
        // subtype at +28
        d[28..32].copy_from_slice(&subtype.to_be_bytes());
        d[48..52].copy_from_slice(&off48.to_be_bytes());
        d[52..56].copy_from_slice(&152u32.to_be_bytes());
        d[68..72].copy_from_slice(&off68.to_be_bytes());
        d[72..76].copy_from_slice(&if subtype == 4 { 4u32 } else { 0x10u32 }.to_be_bytes());
        d[76..80].copy_from_slice(&off76.to_be_bytes());
        d[80..84].copy_from_slice(&4u32.to_be_bytes());
        d[88..92].copy_from_slice(&1u32.to_be_bytes());
        d[96..100].copy_from_slice(&1u32.to_be_bytes());
        d[100..104].copy_from_slice(&off100.to_be_bytes());
        d[104..108].copy_from_slice(&4u32.to_be_bytes());
        d[112..116].copy_from_slice(&1u32.to_be_bytes());
        d[120..124].copy_from_slice(&1u32.to_be_bytes());
        d
    }

    #[test]
    fn parse_valid_header() {
        let data = build_test_rw4(1, 200, 300, 500, 42);
        let h = parse_header(&data).unwrap();
        assert_eq!(h.subtype, Subtype::Family);
        assert_eq!(h.off32, 100);
        assert_eq!(h.off36, 100);
        assert_eq!(h.off40, 0x10);
        assert_eq!(h.off48, 200);
        assert_eq!(h.off68, 300);
        assert_eq!(h.off72, 0x10);
        assert_eq!(h.off76, 500);
        assert_eq!(h.off96, 1);
        assert_eq!(h.off100, 42);
    }

    #[test]
    fn parse_subtype_4() {
        let data = build_test_rw4(4, 324, 372, 600, 440);
        let h = parse_header(&data).unwrap();
        assert_eq!(h.subtype, Subtype::Uniform);
        assert_eq!(h.off40, 4);
        assert_eq!(h.off72, 4);
        assert_eq!(h.off48, 324);
        assert_eq!(h.off68, 372);
    }

    #[test]
    fn bad_magic() {
        let mut data = build_test_rw4(1, 0, 0, 0, 0);
        data[0] = 0x00;
        assert_eq!(parse_header(&data), Err(Rw4Error::BadMagic));
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 50];
        assert_eq!(parse_header(&data), Err(Rw4Error::TooShort));
    }

    #[test]
    fn footer_invariant() {
        let data = build_test_rw4(4, 324, 372, 600, 440);
        let h = parse_header(&data).unwrap();
        // len = 128, off76 = 600 -> footer would be negative, but saturating
        // so check with correct values
        let data2 = build_test_rw4(4, 324, 372, 100, 440);
        // data2 is only 128 bytes, footer at 100 = 28 bytes
        let h2 = parse_header(&data2).unwrap();
        assert_eq!(h2.footer_size(128), 28);
        assert_eq!(h2.footer_invariant_ok(128), false); // off68=372 != 28
    }

    #[test]
    fn object_type_classification() {
        assert_eq!(object_types::category(object_types::BASE_RESOURCE), "BaseResource");
        assert_eq!(object_types::category(object_types::RASTER), "Graphics");
        assert_eq!(object_types::category(object_types::SKELETON), "Animation");
        assert_eq!(object_types::name(object_types::MESH), Some("Mesh"));
        assert_eq!(object_types::name(0xDEADBEEF), None);
    }
}
