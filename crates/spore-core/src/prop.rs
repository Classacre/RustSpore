//! Parser for the SPORE prop format (type `0x00B1B104`).
//!
//! Primitive numeric interpretation is independently checked on 30,885 complete
//! named property lists across 82 installed package indexes (847,835 values).
//! Four of 30,889 PROP resources remain excluded by the independent framing walk.
//! The earlier 23,762-resource `prop_v5.py` result established byte consumption,
//! not correct values or identifiers: its primitive padding could manufacture
//! anonymous entries. See `re/prop-numeric.md` for native evidence and limits.
//!
//! ## Layout
//!
//! ```text
//! u32 item count, BIG-ENDIAN
//! per item (the "named" shape):
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
//! For compatibility, the decoder retains an anonymous-item recovery shape
//! `(type u16, spec u16, value)` and separator/padding heuristics. Reaching EOF
//! through these alternatives alone does not prove the selected interpretation.
//! The independent numeric check requires named IDs and the declared item count.
//!
//! ## Key findings
//! - Two array markers: `0x009C` and `0x001C`.
//! - Array header is `element count` then `element stride in bytes`.
//! - Primitive FLOAT/INT32/UINT32 values and framing words are big-endian.
//!   UTF-16 characters, resource keys and compound vector components are little-endian.
//! - `colorRGBA` (`0x34`) and `vector4` (`0x33`) are 16 bytes with no padding.
//! - Compound-size candidates are retained separately from four-byte primitives.
//! - The legacy separator heuristic recognizes low words `12 00` / `a0 00` and
//!   constant `01 00 00 00`; it can collide with real property identifiers.
//! - String arrays (`0x12`/`0x13`) are length-prefixed per element, not a fixed
//!   `count * stride` block.

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
    pub const STRING8: u16 = 0x0012; // 1-byte characters
    pub const STRING: u16 = 0x0013; // UTF-16LE
    pub const COLOR_RGBA: u16 = 0x0034;
    pub const VECTOR3: u16 = 0x0007; // legacy alias kept for tests
    pub const TEXTS: u16 = 0x0022;
    pub const V2: u16 = 0x0030;
    pub const V3: u16 = 0x0031;
    pub const CRGB: u16 = 0x0032;
    pub const V4: u16 = 0x0033;
    pub const CRGBA: u16 = 0x0034;
    pub const XFORM: u16 = 0x0038;
    pub const BB: u16 = 0x0039;
}

const KEY_ARRAY_STRIDE: usize = 12;
const KEY_SINGLE_SIZE: usize = 16;

/// Errors that can occur when parsing a prop resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropError {
    /// The resource is too short to hold a count header.
    TooShort,
}

impl fmt::Display for PropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropError::TooShort => write!(f, "prop resource too short"),
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
    /// A resource reference, stored as `(instance low half, type id, instance high
    /// half)`. The 64-bit instance is `high << 32 | low`; see [`PropValue::key`].
    Key(u32, u32, u32),
    String8(String),
    String(String),
    /// Source colors are little-endian floats, including signed/HDR lighting
    /// coefficients. Conversion to display bytes belongs at the render output.
    ColorRGBA(f32, f32, f32, f32),
    ColorRGB(f32, f32, f32),
    Vector2(f32, f32),
    Vector3(f32, f32, f32),
    Vector4(f32, f32, f32, f32),
    Transform([f32; 16]),
    Raw(Vec<u8>),
}

/// One item in a prop tree.
#[derive(Debug, Clone)]
pub struct PropItem {
    /// Item identifier (big-endian u32). `0` for anonymous entries.
    pub id: u32,
    /// Whether this entry used the anonymous shape (no id).
    pub anonymous: bool,
    /// Item type (big-endian u16).
    pub prop_type: u16,
    /// Array specifier (big-endian u16).
    pub specifier: u16,
    /// Parsed value(s).
    pub values: Vec<PropValue>,
}

impl PropValue {
    /// The resource a key points at, as `(instance, type_id)`.
    ///
    /// The wire form splits the 64-bit instance across two fields, and the middle
    /// field is the type. Reassembling it this way resolves 4,403 model references
    /// and 306 texture references in `Spore_Content` against real resources; using
    /// the three fields as `(group, type, instance)` resolves none of them.
    pub fn key(&self) -> Option<(u64, u32)> {
        match self {
            PropValue::Key(low, type_id, high) => {
                Some((((*high as u64) << 32) | *low as u64, *type_id))
            }
            _ => None,
        }
    }
}

/// Idents whose meaning has been established by measurement rather than by
/// recovering a name.
///
/// Prop idents are hashes, and the strings behind them live in the executable, so
/// the names are not recoverable from the data. What each one *is* can still be
/// established: an ident that holds a key to a model is a model slot. The names
/// below describe the role, not the original identifier.
///
/// Measured on `Spore_Content`: 1,162 props hold `MODEL_LOD0`, and 1,095, 1,095 and
/// 1,051 hold the next three. The four keys in one prop point at four gmdl
/// resources whose instances differ only in the low byte, which is a level-of-detail
/// chain. `TEXTURE_A` and `TEXTURE_B` hold RASTER keys and both decode to the same
/// object's colour map at different sizes.
pub mod idents {
    /// The highest-detail model of this prop.
    pub const MODEL_LOD0: u32 = 0x00F9_EFBB;
    pub const MODEL_LOD1: u32 = 0x00F9_EFBC;
    pub const MODEL_LOD2: u32 = 0x00F9_EFBD;
    pub const MODEL_LOD3: u32 = 0x00F9_EFBE;
    /// A float next to the model chain, scale-like in magnitude.
    pub const MODEL_SCALE: u32 = 0x00F9_EFB9;
    /// A bounding box next to the model chain.
    pub const MODEL_BOUNDS: u32 = 0x00F9_EFBA;
    /// One of the prop's two colour maps.
    pub const TEXTURE_A: u32 = 0xD124_FCE4;
    /// The other colour map.
    pub const TEXTURE_B: u32 = 0xD124_FCE7;
    /// The prop this one derives from.
    pub const PARENT: u32 = 0x00B2_CCCB;
}

impl PropTree {
    /// The first value of the item with this ident.
    pub fn value(&self, ident: u32) -> Option<&PropValue> {
        self.items
            .iter()
            .find(|i| i.id == ident && !i.anonymous)
            .and_then(|i| i.values.first())
    }

    /// The key stored under this ident, if the item exists and holds a key.
    pub fn key(&self, ident: u32) -> Option<(u64, u32)> {
        self.value(ident).and_then(PropValue::key)
    }

    /// The best model this prop names: LOD0, falling back through the chain.
    ///
    /// A prop without any of the four idents does not describe a model, which is
    /// most of them.
    pub fn model(&self) -> Option<(u64, u32)> {
        [
            idents::MODEL_LOD0,
            idents::MODEL_LOD1,
            idents::MODEL_LOD2,
            idents::MODEL_LOD3,
        ]
        .iter()
        .find_map(|id| self.key(*id))
    }

    /// Every model slot this prop fills, highest detail first.
    pub fn models(&self) -> Vec<(u64, u32)> {
        [
            idents::MODEL_LOD0,
            idents::MODEL_LOD1,
            idents::MODEL_LOD2,
            idents::MODEL_LOD3,
        ]
        .iter()
        .filter_map(|id| self.key(*id))
        .collect()
    }

    /// The prop's colour maps.
    pub fn textures(&self) -> Vec<(u64, u32)> {
        [idents::TEXTURE_A, idents::TEXTURE_B]
            .iter()
            .filter_map(|id| self.key(*id))
            .collect()
    }

    /// The prop this one derives from.
    pub fn parent(&self) -> Option<(u64, u32)> {
        self.key(idents::PARENT)
    }
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

#[inline]
fn is_array_spec(spec: u16) -> bool {
    spec == ARRAY_MARKER_1 || spec == ARRAY_MARKER_2
}

#[inline]
fn is_separator(d: &[u8], p: usize) -> bool {
    if p + 4 > d.len() {
        return false;
    }
    if d[p..p + 4] == [0x01, 0x00, 0x00, 0x00] {
        return true;
    }
    (d[p + 2] == 0x12 && d[p + 3] == 0x00) || (d[p + 2] == 0xA0 && d[p + 3] == 0x00)
}

#[inline]
fn is_known_type(t: u16) -> bool {
    matches!(
        t,
        0x0000 | 0x0001
            | 0x0009
            | 0x000A
            | 0x000D
            | 0x0012
            | 0x0013
            | 0x0020
            | 0x0022
            | 0x0030
            | 0x0031
            | 0x0032
            | 0x0033
            | 0x0034
            | 0x0038
            | 0x0039
    )
}

#[inline]
fn is_known_spec(s: u16) -> bool {
    s == 0x0000
        || is_array_spec(s)
        || matches!(
            s,
            0x0001 | 0x0009 | 0x000A | 0x000D | 0x0030 | 0x0031 | 0x0032 | 0x0033 | 0x0034 | 0x0038 | 0x0039
        )
}

#[inline]
fn fixed_base(t: u16) -> Option<usize> {
    match t {
        0x0009 | 0x000A | 0x000D => Some(4),
        0x0030 => Some(8),
        0x0031 | 0x0032 => Some(12),
        0x0033 | 0x0034 => Some(16),
        0x0038 => Some(64),
        0x0039 => Some(24),
        _ => None,
    }
}

/// Append candidate value sizes (bytes) for the value starting at `vpos` for the given
/// `(typ, spec)`. The `out` vector is cleared and reused. Returns nothing; callers read
/// `out`. Sizes that would overrun the buffer are omitted.
fn value_sizes(d: &[u8], typ: u16, spec: u16, vpos: usize, out: &mut Vec<usize>) {
    out.clear();
    if is_array_spec(spec) {
        if vpos + 8 > d.len() {
            return;
        }
        let count = u32::from_be_bytes([d[vpos], d[vpos + 1], d[vpos + 2], d[vpos + 3]]) as usize;
        let stride = u32::from_be_bytes([d[vpos + 4], d[vpos + 5], d[vpos + 6], d[vpos + 7]]) as usize;
        if count > 200_000 || stride > 16384 {
            return;
        }
        if typ == 0x0012 || typ == 0x0013 {
            let width = if typ == 0x0012 { 1 } else { 2 };
            let mut pp = vpos + 8;
            for _ in 0..count {
                if pp + 4 > d.len() {
                    return;
                }
                let ln = u32::from_be_bytes([d[pp], d[pp + 1], d[pp + 2], d[pp + 3]]) as usize;
                pp += 4;
                if pp + ln * width > d.len() {
                    return;
                }
                pp += ln * width;
            }
            out.push(pp - vpos);
        } else if vpos + 8 + count * stride <= d.len() {
            out.push(8 + count * stride);
        }
        return;
    }
    if typ == 0x0012 || typ == 0x0013 {
        if vpos + 4 > d.len() {
            return;
        }
        let ln = u32::from_be_bytes([d[vpos], d[vpos + 1], d[vpos + 2], d[vpos + 3]]) as usize;
        let width = if typ == 0x0012 { 1 } else { 2 };
        if vpos + 4 + ln * width <= d.len() {
            out.push(4 + ln * width);
        }
        return;
    }
    if typ == 0x0022 {
        if vpos + 4 > d.len() {
            return;
        }
        let n = u32::from_be_bytes([d[vpos], d[vpos + 1], d[vpos + 2], d[vpos + 3]]) as usize;
        let mut pp = vpos + 4;
        for _ in 0..n {
            if pp + 12 > d.len() {
                return;
            }
            let sz = u32::from_be_bytes([d[pp], d[pp + 1], d[pp + 2], d[pp + 3]]) as usize;
            pp += 12;
            if pp + sz > d.len() {
                return;
            }
            pp += sz;
        }
        out.push(pp - vpos);
        return;
    }
    if typ == 0x0001 {
        out.push(1);
        return;
    }
    if typ == 0x0020 {
        // key: single is 16 bytes; array stride is handled by the array branch.
        out.push(KEY_SINGLE_SIZE);
        return;
    }
    if typ == 0x0000 {
        if (spec as usize) <= 65535 {
            out.push(spec as usize);
        }
        return;
    }
    if let Some(base) = fixed_base(typ) {
        if typ == 0x0033 || typ == 0x0034 {
            out.push(base); // no single-value padding
        } else {
            out.push(base);
            out.push(base + 4); // padding variant for other compound singles
        }
    }
}

/// Backtracking exact-consumption walk over a prop blob.
///
/// Returns `true` iff the bytes `[4..len]` (after the 4-byte count header) can be fully
/// consumed by some valid sequence of named/anonymous entries and separators. The walk
/// prefers named entries; it falls back to anonymous only when the named branch cannot
/// reach the end. Memoised on position.
pub fn exact_consumes(d: &[u8]) -> bool {
    let n = d.len();
    if n < 4 {
        return false;
    }
    let mut can: Vec<bool> = vec![false; n + 1];
    can[n] = true;
    let mut buf: Vec<usize> = Vec::with_capacity(8);
    for pos in (0..n).rev() {
        let mut res = false;
        if pos + 4 <= n && is_separator(d, pos) && can[pos + 4] {
            res = true;
        }
        if !res && pos + 8 <= n {
            let typ = u16::from_be_bytes([d[pos + 4], d[pos + 5]]);
            let spec = u16::from_be_bytes([d[pos + 6], d[pos + 7]]);
            // Named entry (id, type, spec, value) at this position.
            value_sizes(d, typ, spec, pos + 8, &mut buf);
            for &vsz in &buf {
                if pos + 8 + vsz <= n && can[pos + 8 + vsz] {
                    res = true;
                    break;
                }
            }
            // Anonymous entry (type, spec, value) when the first two bytes are a known type.
            if !res && is_known_type(typ) && is_known_spec(spec) {
                value_sizes(d, typ, spec, pos + 4, &mut buf);
                for &vsz in &buf {
                    if pos + 4 + vsz <= n && can[pos + 4 + vsz] {
                        res = true;
                        break;
                    }
                }
            }
        }
        can[pos] = res;
    }
    can[4]
}

fn decode_string8(d: &[u8], p: usize, len: usize) -> String {
    String::from_utf8_lossy(&d[p..p + len]).into_owned()
}

fn decode_string16(d: &[u8], p: usize, len: usize) -> String {
    let mut units = Vec::with_capacity(len);
    for i in 0..len {
        units.push(u16::from_le_bytes([d[p + 2 * i], d[p + 2 * i + 1]]));
    }
    let decoded: String = char::decode_utf16(units.iter().copied())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect();
    decoded
}

/// Parse a single value of `typ` whose meaningful data begins at `p` (ignoring any
/// trailing padding bytes). `avail` is the total bytes available for the value.
fn parse_single(d: &[u8], typ: u16, p: usize, avail: usize) -> PropValue {
    let take = |n: usize| -> PropValue { PropValue::Raw(d[p..(p + n.min(avail))].to_vec()) };
    match typ {
        0x0001 => PropValue::Bool(d[p] != 0),
        0x000D => {
            if avail >= 4 {
                PropValue::Float(f32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        0x0009 => {
            if avail >= 4 {
                PropValue::Int32(i32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        0x000A => {
            if avail >= 4 {
                PropValue::Uint32(u32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]))
            } else {
                take(avail)
            }
        }
        0x0020 => {
            if avail >= 12 {
                // Keys are raw little-endian structs in the mixed-endian format: the
                // game stores them as a raw struct, and the struct's members are
                // native order. Measured: read big-endian, the type field of a model
                // key comes out as 0xE5BCE600 instead of 0x00E6BCE5, and nothing
                // resolves. The 64-bit instance is split across the first and third
                // members, which is why they are not "group" and "instance".
                let low = u32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]);
                let t = u32::from_le_bytes([d[p + 4], d[p + 5], d[p + 6], d[p + 7]]);
                let high = u32::from_le_bytes([d[p + 8], d[p + 9], d[p + 10], d[p + 11]]);
                PropValue::Key(low, t, high)
            } else {
                take(avail)
            }
        }
        0x0012 => {
            if avail >= 4 {
                let ln = u32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]) as usize;
                if p + 4 + ln <= d.len() {
                    PropValue::String8(decode_string8(d, p + 4, ln))
                } else {
                    take(avail)
                }
            } else {
                take(avail)
            }
        }
        0x0013 => {
            if avail >= 4 {
                let ln = u32::from_be_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]) as usize;
                if p + 4 + ln * 2 <= d.len() {
                    PropValue::String(decode_string16(d, p + 4, ln))
                } else {
                    take(avail)
                }
            } else {
                take(avail)
            }
        }
        0x0034 => {
            if avail >= 16 {
                let f = |at| f32::from_le_bytes(d[at..at + 4].try_into().unwrap());
                PropValue::ColorRGBA(f(p), f(p + 4), f(p + 8), f(p + 12))
            } else {
                take(avail)
            }
        }
        0x0032 => {
            if avail >= 12 {
                let f = |at| f32::from_le_bytes(d[at..at + 4].try_into().unwrap());
                PropValue::ColorRGB(f(p), f(p + 4), f(p + 8))
            } else {
                take(avail)
            }
        }
        0x0030 => {
            if avail >= 8 {
                let x = f32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]);
                let y = f32::from_le_bytes([d[p + 4], d[p + 5], d[p + 6], d[p + 7]]);
                PropValue::Vector2(x, y)
            } else {
                take(avail)
            }
        }
        0x0031 => {
            if avail >= 12 {
                let x = f32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]);
                let y = f32::from_le_bytes([d[p + 4], d[p + 5], d[p + 6], d[p + 7]]);
                let z = f32::from_le_bytes([d[p + 8], d[p + 9], d[p + 10], d[p + 11]]);
                PropValue::Vector3(x, y, z)
            } else {
                take(avail)
            }
        }
        0x0033 => {
            if avail >= 16 {
                let x = f32::from_le_bytes([d[p], d[p + 1], d[p + 2], d[p + 3]]);
                let y = f32::from_le_bytes([d[p + 4], d[p + 5], d[p + 6], d[p + 7]]);
                let z = f32::from_le_bytes([d[p + 8], d[p + 9], d[p + 10], d[p + 11]]);
                let w = f32::from_le_bytes([d[p + 12], d[p + 13], d[p + 14], d[p + 15]]);
                PropValue::Vector4(x, y, z, w)
            } else {
                take(avail)
            }
        }
        0x0038 => {
            if avail >= 64 {
                let mut m = [0f32; 16];
                for i in 0..16 {
                    m[i] = f32::from_le_bytes([
                        d[p + 4 * i],
                        d[p + 4 * i + 1],
                        d[p + 4 * i + 2],
                        d[p + 4 * i + 3],
                    ]);
                }
                PropValue::Transform(m)
            } else {
                take(avail)
            }
        }
        _ => take(avail),
    }
}

/// Parse the value(s) for an entry. `vstart` is the value start and `vlen` the total
/// value length (already validated to fit in `d`).
fn parse_values(d: &[u8], typ: u16, spec: u16, vstart: usize, vlen: usize) -> Vec<PropValue> {
    if is_array_spec(spec) {
        if vstart + 8 > d.len() {
            return vec![PropValue::Raw(d[vstart..(vstart + vlen).min(d.len())].to_vec())];
        }
        let count = u32::from_be_bytes([d[vstart], d[vstart + 1], d[vstart + 2], d[vstart + 3]]) as usize;
        let stride = u32::from_be_bytes([d[vstart + 4], d[vstart + 5], d[vstart + 6], d[vstart + 7]]) as usize;
        let mut out = Vec::with_capacity(count.min(1024));
        if typ == 0x0012 || typ == 0x0013 {
            let width = if typ == 0x0012 { 1 } else { 2 };
            let mut pp = vstart + 8;
            for _ in 0..count {
                if pp + 4 > d.len() {
                    break;
                }
                let ln = u32::from_be_bytes([d[pp], d[pp + 1], d[pp + 2], d[pp + 3]]) as usize;
                pp += 4;
                if pp + ln * width > d.len() {
                    break;
                }
                let s = if typ == 0x0012 {
                    decode_string8(d, pp, ln)
                } else {
                    decode_string16(d, pp, ln)
                };
                pp += ln * width;
                out.push(if typ == 0x0012 {
                    PropValue::String8(s)
                } else {
                    PropValue::String(s)
                });
            }
            return out;
        }
        // Fixed-stride array: parse each element's meaningful (base) bytes.
        let base = if typ == 0x0020 {
            KEY_ARRAY_STRIDE
        } else {
            fixed_base(typ).unwrap_or(stride)
        };
        let mut pp = vstart + 8;
        for _ in 0..count {
            if pp + base > d.len() {
                break;
            }
            out.push(parse_single(d, typ, pp, base));
            pp += stride;
        }
        return out;
    }
    vec![parse_single(d, typ, vstart, vlen)]
}

/// Parse a prop resource into a tree. The `exact` flag reports whether the whole blob
/// was consumed; this is the authoritative 100% validation signal.
pub fn parse(data: &[u8]) -> Result<PropTree, PropError> {
    let n = data.len();
    if n < 4 {
        return Err(PropError::TooShort);
    }
    let exact = exact_consumes(data);
    // Recompute the memoised `can` table to drive a deterministic named-first build.
    let mut can: Vec<bool> = vec![false; n + 1];
    can[n] = true;
    let mut buf: Vec<usize> = Vec::with_capacity(8);
    for pos in (0..n).rev() {
        let mut res = false;
        if pos + 4 <= n && is_separator(data, pos) && can[pos + 4] {
            res = true;
        }
        if !res && pos + 8 <= n {
            let typ = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
            let spec = u16::from_be_bytes([data[pos + 6], data[pos + 7]]);
            value_sizes(data, typ, spec, pos + 8, &mut buf);
            for &vsz in &buf {
                if pos + 8 + vsz <= n && can[pos + 8 + vsz] {
                    res = true;
                    break;
                }
            }
            if !res && is_known_type(typ) && is_known_spec(spec) {
                value_sizes(data, typ, spec, pos + 4, &mut buf);
                for &vsz in &buf {
                    if pos + 4 + vsz <= n && can[pos + 4 + vsz] {
                        res = true;
                        break;
                    }
                }
            }
        }
        can[pos] = res;
    }

    let mut items: Vec<PropItem> = Vec::new();
    let mut pos = 4;
    while pos < n {
        if is_separator(data, pos) {
            pos += 4;
            continue;
        }
        if pos + 8 > n {
            break;
        }
        let id = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let typ = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
        let spec = u16::from_be_bytes([data[pos + 6], data[pos + 7]]);
        // Named first.
        let mut chosen: Option<(bool, usize)> = None;
        value_sizes(data, typ, spec, pos + 8, &mut buf);
        for &vsz in &buf {
            if pos + 8 + vsz <= n && can[pos + 8 + vsz] {
                chosen = Some((true, pos + 8 + vsz));
                break;
            }
        }
        if chosen.is_none() && is_known_type(typ) && is_known_spec(spec) {
            value_sizes(data, typ, spec, pos + 4, &mut buf);
            for &vsz in &buf {
                if pos + 4 + vsz <= n && can[pos + 4 + vsz] {
                    chosen = Some((false, pos + 4 + vsz));
                    break;
                }
            }
        }
        let (is_named, nxt) = match chosen {
            Some(c) => c,
            None => break,
        };
        let vstart = if is_named { pos + 8 } else { pos + 4 };
        let vlen = nxt - vstart;
        let values = parse_values(data, typ, spec, vstart, vlen);
        items.push(PropItem {
            id: if is_named { id } else { 0 },
            anonymous: !is_named,
            prop_type: typ,
            specifier: spec,
            values,
        });
        pos = nxt;
    }
    Ok(PropTree {
        items,
        exact,
        consumed: pos,
    })
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
        data.extend_from_slice(&1.5f32.to_be_bytes()); // scalar value, no padding

        let tree = parse(&data).unwrap();
        assert_eq!(tree.items.len(), 1);
        assert!(tree.exact);
        assert_eq!(tree.items[0].id, 0x1234);
        match &tree.items[0].values[0] {
            PropValue::Float(f) => assert!((f - 1.5).abs() < 1e-6),
            _ => panic!("expected float"),
        }
    }

    /// Exact raw primitive records from shipped EditorCameraCellThumbnail props.
    /// Byte consumption alone could not catch the previous little-endian values.
    #[test]
    fn shipped_camera_primitives_are_big_endian_and_keep_their_identifiers() {
        let data = [
            0, 0, 0, 3,
            0x00, 0xC7, 0xC4, 0xFB, 0, 0x0D, 0, 0, 0x3F, 0x80, 0, 0,
            0x00, 0xC7, 0xC4, 0xFC, 0, 0x0D, 0, 0, 0xC2, 0xB4, 0, 0,
            0x00, 0xC7, 0xC4, 0xFD, 0, 0x0D, 0, 0, 0xC2, 0xB4, 0, 0,
        ];
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items.len(), 3);
        for (ident, expected) in [(0x00C7_C4FB, 1.0), (0x00C7_C4FC, -90.0), (0x00C7_C4FD, -90.0)] {
            assert_eq!(tree.value(ident), Some(&PropValue::Float(expected)));
        }
        assert!(tree.items.iter().all(|item| !item.anonymous));
    }

    #[test]
    fn primitive_singles_are_big_endian_while_vectors_and_keys_remain_little_endian() {
        let mut data = 5u32.to_be_bytes().to_vec();
        for (ident, typ, bytes) in [
            (0x1020_3040u32, types::INT32, (-314159i32).to_be_bytes()),
            (0x1020_3041, types::UINT32, 0x1234_5678u32.to_be_bytes()),
            (0x1020_3042, types::FLOAT, (-90.0f32).to_be_bytes()),
        ] {
            data.extend_from_slice(&ident.to_be_bytes());
            data.extend_from_slice(&typ.to_be_bytes());
            data.extend_from_slice(&0u16.to_be_bytes());
            data.extend_from_slice(&bytes);
        }
        data.extend_from_slice(&0x1020_3043u32.to_be_bytes());
        data.extend_from_slice(&types::V3.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        for value in [1.0f32, -2.0, 3.5] { data.extend_from_slice(&value.to_le_bytes()); }
        data.extend_from_slice(&0x1020_3044u32.to_be_bytes());
        data.extend_from_slice(&types::KEY.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        for value in [0x780A_9EAFu32, 0x00E6_BCE5, 0x4061_7100, 0] {
            data.extend_from_slice(&value.to_le_bytes());
        }
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.value(0x1020_3040), Some(&PropValue::Int32(-314159)));
        assert_eq!(tree.value(0x1020_3041), Some(&PropValue::Uint32(0x1234_5678)));
        assert_eq!(tree.value(0x1020_3042), Some(&PropValue::Float(-90.0)));
        assert_eq!(tree.value(0x1020_3043), Some(&PropValue::Vector3(1.0, -2.0, 3.5)));
        assert_eq!(tree.key(0x1020_3044), Some((0x4061_7100_780A_9EAF, 0x00E6_BCE5)));
    }

    #[test]
    fn primitive_array_elements_preserve_big_endian_signed_unsigned_and_float_bits() {
        let mut data = 3u32.to_be_bytes().to_vec();
        let cases = [
            (0x1090_0001u32, types::FLOAT, ARRAY_MARKER_1, vec![0x8000_0000, 0x3F80_0001, 0x7FC0_1234]),
            (0x1090_0002, types::INT32, ARRAY_MARKER_2, vec![(-3i32) as u32, 0, i32::MIN as u32]),
            (0x1090_0003, types::UINT32, ARRAY_MARKER_1, vec![0x0102_0304, 0xF0E0_D0C0]),
        ];
        for (ident, typ, spec, words) in &cases {
            data.extend_from_slice(&ident.to_be_bytes());
            data.extend_from_slice(&typ.to_be_bytes());
            data.extend_from_slice(&spec.to_be_bytes());
            data.extend_from_slice(&(words.len() as u32).to_be_bytes());
            data.extend_from_slice(&4u32.to_be_bytes());
            for word in words { data.extend_from_slice(&word.to_be_bytes()); }
        }
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items.len(), 3);
        for (item, (ident, typ, _, expected)) in tree.items.iter().zip(cases) {
            assert_eq!(item.id, ident);
            let actual: Vec<u32> = item.values.iter().map(|value| match (typ, value) {
                (types::FLOAT, PropValue::Float(v)) => v.to_bits(),
                (types::INT32, PropValue::Int32(v)) => *v as u32,
                (types::UINT32, PropValue::Uint32(v)) => *v,
                _ => panic!("wrong primitive variant"),
            }).collect();
            assert_eq!(actual, expected);
        }
    }

    /// The independent Python tool supplies complete named/count-bounded props
    /// and per-resource canonical numeric hashes. No DBPF dependency is needed.
    #[test]
    #[ignore = "generate local fixtures with tools/prop_numeric_probe.py and set RUSTSPORE_PROP_NUMERIC_FIXTURES"]
    fn installed_primitive_numeric_corpus_matches_independent_named_reference() {
        use std::io::Read;
        let path = std::env::var_os("RUSTSPORE_PROP_NUMERIC_FIXTURES")
            .expect("RUSTSPORE_PROP_NUMERIC_FIXTURES must point to the independently generated local corpus");
        let mut file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let mut magic = [0; 8]; file.read_exact(&mut magic).unwrap();
        assert_eq!(&magic, b"RSPROP01");
        let mut count = [0; 4]; file.read_exact(&mut count).unwrap();
        let count = u32::from_le_bytes(count);
        assert!(count > 0 && count <= 100000);
        let mut values = 0usize;
        for resource in 0..count {
            let mut header = [0; 12]; file.read_exact(&mut header).unwrap();
            let len = u32::from_le_bytes(header[..4].try_into().unwrap()) as usize;
            assert!(len <= 32 * 1024 * 1024);
            let expected = u64::from_le_bytes(header[4..].try_into().unwrap());
            let mut bytes = vec![0; len]; file.read_exact(&mut bytes).unwrap();
            let tree = parse(&bytes).unwrap();
            assert!(tree.exact && tree.consumed == bytes.len(), "resource {resource} framing");
            let mut hash = 0xCBF2_9CE4_8422_2325u64;
            let mut add = |bytes: &[u8]| {
                for byte in bytes { hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01B3); }
            };
            for item in &tree.items {
                if !matches!(item.prop_type, types::FLOAT | types::INT32 | types::UINT32) { continue; }
                add(&item.id.to_le_bytes());
                add(&item.prop_type.to_le_bytes());
                add(&item.specifier.to_le_bytes());
                add(&(item.values.len() as u32).to_le_bytes());
                for value in &item.values {
                    let bits = match value {
                        PropValue::Float(v) => v.to_bits(),
                        PropValue::Int32(v) => *v as u32,
                        PropValue::Uint32(v) => *v,
                        _ => panic!("resource {resource}: primitive was not typed"),
                    };
                    add(&bits.to_le_bytes()); values += 1;
                }
            }
            assert_eq!(hash, expected, "resource {resource}: typed primitive digest mismatch");
        }
        let mut tail = [0]; assert_eq!(file.read(&mut tail).unwrap(), 0);
        println!("PROP_NUMERIC_CORPUS resources={count} primitive_values={values}");
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
        // HDR, signed and fractional values must survive without byte-color
        // conversion. The source field is four LE floats with no padding.
        for v in [2.38805f32, -0.590243, 0.28, 1.25] {
            data.extend_from_slice(&v.to_le_bytes());
        }

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        match &tree.items[0].values[0] {
            PropValue::ColorRGBA(r, g, b, a) => {
                assert_eq!([*r, *g, *b, *a], [2.38805, -0.590243, 0.28, 1.25]);
            }
            _ => panic!("expected color"),
        }
    }

    #[test]
    fn rgb_arrays_preserve_signed_hdr_float_bits_and_following_property() {
        let mut data = 2u32.to_be_bytes().to_vec();
        data.extend_from_slice(&0x0100_EAC5u32.to_be_bytes());
        data.extend_from_slice(&types::CRGB.to_be_bytes());
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes());
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&12u32.to_be_bytes());
        let bits = [2.38805f32.to_bits(), (-0.590243f32).to_bits(), 0x80000000,
            0x7fc01234, 1.25f32.to_bits(), 0.28f32.to_bits()];
        for word in bits { data.extend_from_slice(&word.to_le_bytes()); }
        data.extend_from_slice(&0x0100_EAC4u32.to_be_bytes());
        data.extend_from_slice(&types::FLOAT.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&0.28f32.to_be_bytes());
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items.len(), 2);
        for (value, expected) in tree.items[0].values.iter().zip(bits.chunks_exact(3)) {
            let PropValue::ColorRGB(r, g, b) = value else { panic!("expected RGB float array"); };
            assert_eq!([r.to_bits(), g.to_bits(), b.to_bits()], expected);
        }
        assert_eq!(tree.items[1].id, 0x0100_EAC4);
        assert_eq!(tree.items[1].values, [PropValue::Float(0.28)]);
    }

    #[test]
    fn parse_array_with_marker_009c() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x0020u32.to_be_bytes()); // id
        data.extend_from_slice(&types::KEY.to_be_bytes()); // type
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes()); // 0x009C
        data.extend_from_slice(&1u32.to_be_bytes()); // count
        data.extend_from_slice(&12u32.to_be_bytes()); // stride
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
        data.extend_from_slice(&1u32.to_be_bytes()); // count
        data.extend_from_slice(&24u32.to_be_bytes()); // stride
        data.extend_from_slice(&[0u8; 24]);

        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items[0].specifier, ARRAY_MARKER_2);
    }

    #[test]
    fn parse_multiple_items() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes()); // 2 items

        data.extend_from_slice(&0x0001u32.to_be_bytes());
        data.extend_from_slice(&types::FLOAT.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&1.0f32.to_be_bytes());

        data.extend_from_slice(&0x0002u32.to_be_bytes());
        data.extend_from_slice(&types::INT32.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&42i32.to_be_bytes());

        let tree = parse(&data).unwrap();
        assert_eq!(tree.items.len(), 2);
        assert!(tree.exact);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 2];
        assert!(matches!(parse(&data), Err(PropError::TooShort)));
    }

    /// A blob with one valid float entry plus a trailing byte that does not form a
    /// separator or entry must parse without error and report `exact == false`
    /// (the build loop only commits items on a path to full consumption).
    #[test]
    fn trailing_bytes_make_incomplete() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes()); // 1 item
        data.extend_from_slice(&0x1234u32.to_be_bytes()); // id
        data.extend_from_slice(&types::FLOAT.to_be_bytes()); // type
        data.extend_from_slice(&0u16.to_be_bytes()); // specifier (not array)
        data.extend_from_slice(&1.5f32.to_be_bytes()); // scalar value
        data.push(0xAB); // trailing byte, not a separator or entry

        let tree = parse(&data).unwrap();
        assert!(!tree.exact);
    }

    /// A variant blob: a string8 array (length-prefixed, not fixed stride) plus a
    /// separator. Validates the length-prefixed string array rule and exact consumption.
    #[test]
    fn variant_blob_exact() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes()); // 2 entries

        // Named string8 array: id, type 0x12, spec 0x009C, count=2, stride, two strings.
        data.extend_from_slice(&0xAAAAu32.to_be_bytes());
        data.extend_from_slice(&types::STRING8.to_be_bytes());
        data.extend_from_slice(&ARRAY_MARKER_1.to_be_bytes());
        data.extend_from_slice(&2u32.to_be_bytes()); // count
        data.extend_from_slice(&0u32.to_be_bytes()); // stride unused for strings
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend(b"ab".to_vec());
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend(b"c".to_vec());

        // Separator (low word 12 00).
        data.extend_from_slice(&[0x3c, 0xc4, 0x12, 0x00]);

        // A named float entry after the separator.
        data.extend_from_slice(&0xBBBBu32.to_be_bytes());
        data.extend_from_slice(&types::FLOAT.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&2.5f32.to_be_bytes());

        assert!(exact_consumes(&data));
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.items.len(), 2);
        // First item is the string8 array.
        assert_eq!(tree.items[0].prop_type, types::STRING8);
        match &tree.items[0].values[..] {
            [PropValue::String8(a), PropValue::String8(b)] => {
                assert_eq!(a, "ab");
                assert_eq!(b, "c");
            }
            _ => panic!("expected two string8 values"),
        }
        // Second item is the named float.
        assert!(!tree.items[1].anonymous);
        match &tree.items[1].values[0] {
            PropValue::Float(f) => assert!((f - 2.5).abs() < 1e-6),
            _ => panic!("expected float"),
        }
    }

    /// Regression: a lone 0x33 (vector4) single must be 16 bytes, not 20, so a trailing
    /// 16-byte vector4 lands exactly on the blob end.
    #[test]
    fn vector4_single_no_padding() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x1234u32.to_be_bytes());
        data.extend_from_slice(&0x0033u16.to_be_bytes()); // vector4
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&[0u8; 16]); // exactly 16 bytes, no padding
        assert!(exact_consumes(&data));
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
    }

    /// A model key's bytes, read the way the game writes them: three little-endian
    /// words whose first and third halves are the 64-bit instance.
    #[test]
    fn key_reassembles_the_instance() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&idents::MODEL_LOD0.to_be_bytes());
        data.extend_from_slice(&types::KEY.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes()); // not an array
        data.extend_from_slice(&0xD1B4BB56u32.to_le_bytes()); // instance low
        data.extend_from_slice(&0x00E6BCE5u32.to_le_bytes()); // type: gmdl
        data.extend_from_slice(&0x40667100u32.to_le_bytes()); // instance high
        data.extend_from_slice(&[0u8; 4]); // single keys carry padding

        let tree = parse(&data).unwrap();
        assert!(tree.exact, "fixture must consume exactly");
        assert_eq!(
            tree.model(),
            Some((0x4066_7100_D1B4_BB56, 0x00E6_BCE5)),
            "instance is high<<32|low, not three separate fields"
        );
    }

    /// Reading the key big-endian produces 0xE5BCE600, which is not a type id.
    #[test]
    fn key_is_not_big_endian() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&idents::MODEL_LOD0.to_be_bytes());
        data.extend_from_slice(&types::KEY.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&0xD1B4BB56u32.to_le_bytes());
        data.extend_from_slice(&0x00E6BCE5u32.to_le_bytes());
        data.extend_from_slice(&0x40667100u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        let tree = parse(&data).unwrap();
        let (_, type_id) = tree.model().unwrap();
        assert_eq!(type_id, 0x00E6_BCE5);
        assert_ne!(type_id, 0xE5BC_E600);
    }

    /// LOD0 wins when several slots are filled, and the chain is reported in order.
    #[test]
    fn model_prefers_lod0_and_keeps_the_chain() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_be_bytes());
        for (ident, instance) in [
            (idents::MODEL_LOD1, 0x40666201u32),
            (idents::MODEL_LOD0, 0x40667100u32),
        ] {
            data.extend_from_slice(&ident.to_be_bytes());
            data.extend_from_slice(&types::KEY.to_be_bytes());
            data.extend_from_slice(&0u16.to_be_bytes());
            data.extend_from_slice(&0xD1B4BB56u32.to_le_bytes());
            data.extend_from_slice(&0x00E6BCE5u32.to_le_bytes());
            data.extend_from_slice(&instance.to_le_bytes());
            data.extend_from_slice(&[0u8; 4]);
        }
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.model().unwrap().0, 0x4066_7100_D1B4_BB56);
        assert_eq!(tree.models().len(), 2);
        assert_eq!(tree.models()[0].0, 0x4066_7100_D1B4_BB56);
    }

    /// A prop that names no model reports none, which is most of them.
    #[test]
    fn prop_without_model_idents_has_no_model() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_be_bytes());
        data.extend_from_slice(&0x0254CF89u32.to_be_bytes()); // a float
        data.extend_from_slice(&0x000Du16.to_be_bytes());
        data.extend_from_slice(&0u16.to_be_bytes());
        data.extend_from_slice(&1.5f32.to_be_bytes());
        let tree = parse(&data).unwrap();
        assert!(tree.exact);
        assert_eq!(tree.model(), None);
        assert!(tree.models().is_empty());
        assert!(tree.textures().is_empty());
    }
}
