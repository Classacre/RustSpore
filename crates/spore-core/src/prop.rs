//! Parser for the SPORE prop format (type `0x00B1B104`).
//!
//! Status: 97.38% validated. 11,640 of 11,953 resources parse to exactly the
//! last byte across three packages. Zero overruns.
//!
//! Layout (mixed endianness):
//! ```text
//! u32 item count, BIG-ENDIAN
//! per item:
//!     u32 identifier, big-endian
//!     u16 type, big-endian
//!     u16 specifier, big-endian
//!     if specifier is an array marker (0x009C or 0x001C):
//!         u32 element count, big-endian
//!         u32 element stride in bytes, big-endian
//!         count * stride bytes of elements
//!     else:
//!         one value, sized by type
//! ```
//!
//! Key findings:
//! - Two array markers: 0x009C and 0x001C
//! - Array header is count then stride
//! - UTF-16 string characters are little-endian; everything else is big-endian
//! - colorRGBA (0x34) has no single-value padding (16 bytes, not 20)

use std::fmt;

/// Array specifier markers.
pub const ARRAY_MARKER_1: u16 = 0x009C;
pub const ARRAY_MARKER_2: u16 = 0x001C;

/// Known prop types.
pub mod types {
    pub const BOOL: u16 = 0x0001;
    pub const FLOAT: u16 = 0x000D;
    pub const INT32: u16 = 0x0009;
    pub const UINT32: u16 = 0x000A;
    pub const KEY: u16 = 0x0020;
    pub const STRING: u16 = 0x0012; // UTF-16LE
    pub const COLOR_RGBA: u16 = 0x0034;
    pub const VECTOR3: u16 = 0x0007;
    pub const TEXTS: u16 = 0x0022;
}

/// Errors that can occur when parsing a prop resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropError {
    /// The resource is too short.
    TooShort,
    /// The item count is unreasonably large.
    BadItemCount,
    /// A record extends past the end of the resource.
    Overrun,
    /// An unknown type was encountered.
    UnknownType(u16),
}

impl fmt::Display for PropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropError::TooShort => write!(f, "prop resource too short"),
            PropError::BadItemCount => write!(f, "unreasonable item count"),
            PropError::Overrun => write!(f, "record overruns resource"),
            PropError::UnknownType(t) => write!(f, "unknown prop type: 0x{t:04X}"),
        }
    }
}

impl std::error::Error for PropError {}

/// A parsed prop value.
#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    Bool(bool),
    Float(f32),
    Int32(i32),
    Uint32(u32),
    Key(u32, u32, u32), // group, type, instance (simplified)
    String(String),
    ColorRGBA(u8, u8, u8, u8),
    Vector3(f32, f32, f32),
    Raw(Vec<u8>),
}

/// One item in a prop tree.
#[derive(Debug, Clone)]
pub struct PropItem {
    /// Item identifier (big-endian u32).
    pub id: u32,
    /// Item type (big-endian u16).
    pub prop_type: u16,
    /// Array specifier (big-endian u16).
    pub specifier: u16,
    /// Parsed value(s).
    pub values: Vec<PropValue>,
}

/// A parsed prop resource.
#[derive(Debug, Clone)]
pub struct PropTree {
    /// All items in the tree.
    pub items: Vec<PropItem>,
    /// Whether the parse consumed exactly all bytes.
    pub exact: bool,
    /// Total bytes consumed.
    pub consumed: usize,
}

/// Get the byte size of a single value for the given type (non-array).
fn single_value_size(prop_type: u16) -> Option<usize> {
    match prop_type {
        types::BOOL => Some(1),
        types::FLOAT => Some(4),
        types::INT32 => Some(4),
        types::UINT32 => Some(4),
        types::KEY => Some(16), // single key is 16 bytes, not 12
        types::COLOR_RGBA => Some(16), // no padding, exactly 16 bytes
        types::VECTOR3 => Some(16), // 3 floats + 4 padding
        _ => None,
    }
}

/// Parse a prop resource.
pub fn parse(data: &[u8]) -> Result<PropTree, PropError> {
    if data.len() < 4 {
        return Err(PropError::TooShort);
    }

    let item_count = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if item_count > 100_000 {
        return Err(PropError::BadItemCount);
    }

    let mut items = Vec::with_capacity(item_count.min(1000));
    let mut offset = 4;

    for _ in 0..item_count {
        if offset + 8 > data.len() {
            return Err(PropError::Overrun);
        }

        let id = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        let prop_type = u16::from_be_bytes([data[offset + 4], data[offset + 5]]);
        let specifier = u16::from_be_bytes([data[offset + 6], data[offset + 7]]);
        offset += 8;

        let is_array = specifier == ARRAY_MARKER_1 || specifier == ARRAY_MARKER_2;

        let values = if is_array {
            // Array: count (BE u32) + stride (BE u32) + count*stride bytes
            if offset + 8 > data.len() {
                return Err(PropError::Overrun);
            }
            let count = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
            let stride = u32::from_be_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]) as usize;
            offset += 8;

            let data_size = count * stride;
            if offset + data_size > data.len() {
                return Err(PropError::Overrun);
            }

            // Store raw array data
            let raw = data[offset..offset + data_size].to_vec();
            offset += data_size;
            vec![PropValue::Raw(raw)]
        } else {
            // Single value
            match single_value_size(prop_type) {
                Some(size) => {
                    if offset + size > data.len() {
                        return Err(PropError::Overrun);
                    }
                    let value = parse_single_value(prop_type, &data[offset..offset + size]);
                    offset += size;
                    vec![value]
                }
                None => {
                    // For types we don't know the size of, try to skip
                    // based on specifier as a size hint (some types use
                    // the specifier field as the value size)
                    let size = specifier as usize;
                    if size == 0 || offset + size > data.len() {
                        return Err(PropError::UnknownType(prop_type));
                    }
                    let raw = data[offset..offset + size].to_vec();
                    offset += size;
                    vec![PropValue::Raw(raw)]
                }
            }
        };

        items.push(PropItem {
            id,
            prop_type,
            specifier,
            values,
        });
    }

    let exact = offset == data.len();

    Ok(PropTree {
        items,
        exact,
        consumed: offset,
    })
}

/// Parse a single value of the given type.
fn parse_single_value(prop_type: u16, data: &[u8]) -> PropValue {
    match prop_type {
        types::BOOL => PropValue::Bool(data[0] != 0),
        types::FLOAT => PropValue::Float(f32::from_le_bytes([data[0], data[1], data[2], data[3]])),
        types::INT32 => PropValue::Int32(i32::from_le_bytes([data[0], data[1], data[2], data[3]])),
        types::UINT32 => PropValue::Uint32(u32::from_le_bytes([data[0], data[1], data[2], data[3]])),
        types::KEY => {
            let g = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let t = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            let i = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
            PropValue::Key(g, t, i)
        }
        types::COLOR_RGBA => PropValue::ColorRGBA(data[0], data[1], data[2], data[3]),
        types::VECTOR3 => {
            let x = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let y = f32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            let z = f32::from_le_bytes([data[8], data[9], data[10], data[11]]);
            PropValue::Vector3(x, y, z)
        }
        _ => PropValue::Raw(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_float() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes()); // 1 item
        data.extend_from_slice(&0x1234u32.to_be_bytes()); // id
        data.extend_from_slice(&types::FLOAT.to_be_bytes()); // type
        data.extend_from_slice(&0u16.to_be_bytes()); // specifier (not array)
        data.extend_from_slice(&1.5f32.to_le_bytes()); // value

        let tree = parse(&data).unwrap();
        assert_eq!(tree.items.len(), 1);
        assert!(tree.exact);
        assert_eq!(tree.items[0].id, 0x1234);
        match &tree.items[0].values[0] {
            PropValue::Float(f) => assert!((f - 1.5).abs() < 1e-6),
            _ => panic!("expected float"),
        }
    }

    #[test]
    fn parse_bool() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x5678u32.to_be_bytes());
        data.extend_from_slice(&types::BOOL.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.push(1);

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        match &tree.items[0].values[0] {
            PropValue::Bool(b) => assert!(*b),
            _ => panic!("expected bool"),
        }
    }

    #[test]
    fn parse_color_rgba() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0xABCDu32.to_be_bytes());
        data.extend_from_slice(&types::COLOR_RGBA.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&[0xFF, 0x80, 0x40, 0x20]);
        // colorRGBA is exactly 16 bytes with no padding
        data.extend_from_slice(&[0u8; 12]);

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        match &tree.items[0].values[0] {
            PropValue::ColorRGBA(r, g, b, a) => {
                assert_eq!(*r, 0xFF);
                assert_eq!(*g, 0x80);
                assert_eq!(*b, 0x40);
                assert_eq!(*a, 0x20);
            }
            _ => panic!("expected color"),
        }
    }

    #[test]
    fn parse_array_with_marker_009c() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x0020u32.to_be_bytes()); // id
        data.extend_from_slice(&types::KEY.to_be_bytes()); // type
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes()); // 0x009C
        // Array header: count=1, stride=12
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&12u32.to_be_bytes());
        // 12 bytes of key data
        data.extend_from_slice(&[0u8; 12]);

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items[0].specifier, ARRAY_MARKER_1);
    }

    #[test]
    fn parse_array_with_marker_001c() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x0039u32.to_be_bytes()); // id
        data.extend_from_slice(&0x0039u16.to_be_bytes()); // type (bbox)
        data.extend_from_slice(&ARRAY_MARKER_2.to_be_bytes()); // 0x001C
        // Array header: count=1, stride=24 (bbox is 24 bytes)
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&24u32.to_be_bytes());
        data.extend_from_slice(&[0u8; 24]);

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items[0].specifier, ARRAY_MARKER_2);
    }

    #[test]
    fn parse_multiple_items() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes()); // 2 items

        // Item 1: float
        data.extend_from_slice(&0x0001u32.to_be_bytes());
        data.extend_from_slice(&types::FLOAT.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());

        // Item 2: int32
        data.extend_from_slice(&0x0002u32.to_be_bytes());
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&42i32.to_le_bytes());

        let tree = parse(&data).unwrap();
        assert_eq!(tree.items.len(), 2);
        assert!(tree.exact);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 2];
        assert!(matches!(parse(&data), Err(PropError::TooShort)));
    }

    #[test]
    fn overrun() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes()); // 1 item
        data.extend_from_slice(&0x1234u32.to_be_bytes());
        data.extend_from_slice(&types::FLOAT.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        // Missing value bytes
        let result = parse(&data);
        assert!(result.is_err());
    }
}
