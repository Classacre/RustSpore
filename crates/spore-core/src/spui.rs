//! Parser for the SPORE user-interface layout (type `0x250FE9A2`, `.spui`).
//!
//! The full installed corpus has 748 resources across 82 package indexes.
//! Container and typed bodies decode exactly and independent Python/Rust digests
//! agree; see `re/spui-layout.md`. Earlier container work used a 601-file subset.
//!
//! ## Layout (little-endian)
//!
//! ```text
//! u32 magic = 0xE3FE3FB8
//! u16 version (1..=3)
//! u16 image_count, u16 atlas_count, u16 hit_mask_count, u16 proxy_count
//! image_count * ResourceKey (u32 instance, type, group)
//! atlas_count * ResourceKey
//! hit_mask_count * hit mask:
//!     if version >= 3: u8 use_rle
//!     if use_rle: i32 width, i32 height, i32 word_count, then word_count*2 bytes
//!     else: ResourceKey
//! proxy_count * i32
//! element section: the remaining bytes (verbatim)
//! ```
//!
//! The container retains the original element bytes. `parse_elements` decodes
//! their self-typed property stream, including nested structures and localized
//! text. Designer class/property roles remain separate measured semantics.

use std::fmt;

pub const MAGIC: u32 = 0xE3FE3FB8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpuiError {
    TooShort,
    BadMagic(u32),
    BadVersion(u16),
    Truncated,
    BadElementMarker(u16),
    BadElementIndex(u16),
    DuplicateElement(u16),
    BadElementCount,
    UnsupportedPropertyType(u16),
    NestingLimit,
    PropertyLimit,
    BadPropertyCount(usize),
    BadReference(u16),
    BadFooter,
    TrailingBytes,
}

impl fmt::Display for SpuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpuiError::TooShort => write!(f, "spui too short"),
            SpuiError::BadMagic(m) => write!(f, "unexpected spui magic {m:#010X}"),
            SpuiError::BadVersion(v) => write!(f, "unsupported spui version {v}"),
            SpuiError::Truncated => write!(f, "spui truncated"),
            other => write!(f, "spui element stream: {other:?}"),
        }
    }
}

impl std::error::Error for SpuiError {}

/// A resource key (instance, type, group).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpuiKey {
    pub instance: u32,
    pub type_id: u32,
    pub group: u32,
}

/// One hit-mask record.
#[derive(Debug, Clone)]
pub struct SpuiHitMask {
    pub use_rle: bool,
    pub width: i32,
    pub height: i32,
    pub key: SpuiKey,
    /// The raw bytes of the record (kept verbatim).
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SpuiFile {
    pub version: u16,
    pub image_resources: Vec<SpuiKey>,
    pub atlas_resources: Vec<SpuiKey>,
    pub hit_masks: Vec<SpuiHitMask>,
    pub proxy_ids: Vec<i32>,
    /// The element section (everything after the container), preserved verbatim.
    pub element_data: Vec<u8>,
    /// Bytes consumed by the container.
    pub consumed: usize,
}

/// Self-typed property values. Scalar bytes retain exact source bit patterns;
/// class-specific signedness, colour packing and units are not inferred here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpuiValue {
    Scalars {
        width: u8,
        bytes: Vec<u8>,
    },
    Text(Vec<SpuiText>),
    Structures {
        type_id: u32,
        values: Vec<Vec<SpuiProperty>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpuiText {
    Literal(Vec<u16>),
    Localized { table: u32, instance: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpuiProperty {
    pub id: u32,
    pub type_code: u16,
    pub count: u16,
    pub value: SpuiValue,
}
impl SpuiProperty {
    pub fn scalar_bytes(&self) -> Option<&[u8]> {
        match &self.value {
            SpuiValue::Scalars { bytes, .. } => Some(bytes),
            _ => None,
        }
    }
    pub fn u32_value(&self) -> Option<u32> {
        if self.count != 1 || !matches!(self.type_code, 5 | 9) {
            return None;
        }
        Some(u32::from_le_bytes(self.scalar_bytes()?.try_into().ok()?))
    }
    pub fn reference(&self) -> Option<u16> {
        if self.count != 1 || self.type_code != 19 {
            return None;
        }
        Some(u16::from_le_bytes(self.scalar_bytes()?.try_into().ok()?))
    }
    pub fn rectangle(&self) -> Option<[f32; 4]> {
        if self.count != 1 || self.type_code != 16 {
            return None;
        }
        let bytes = self.scalar_bytes()?;
        if bytes.len() != 16 { return None; }
        Some(std::array::from_fn(|i| {
            f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())
        }))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpuiElement {
    /// Global reference index, after the image, atlas and hit-mask resources.
    pub index: u16,
    pub proxy_id: u32,
    pub root: bool,
    pub properties: Vec<SpuiProperty>,
}
impl SpuiElement {
    pub fn property(&self, id: u32) -> Option<&SpuiProperty> {
        self.properties.iter().find(|property| property.id == id)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpuiElements {
    pub elements: Vec<SpuiElement>,
}

/// Parse every property and verify exact framing. Unsupported type codes are
/// explicit errors; the original bytes remain available in `SpuiFile`.
pub fn parse_elements(file: &SpuiFile) -> Result<SpuiElements, SpuiError> {
    let base = file.image_resources.len() + file.atlas_resources.len() + file.hit_masks.len();
    let mut reader = R {
        d: &file.element_data,
        p: 0,
    };
    let mut elements = Vec::with_capacity(file.proxy_ids.len());
    let mut seen = vec![false; file.proxy_ids.len()];
    let mut remaining = 2_000_000usize;
    loop {
        let marker = reader.u16()?;
        if marker != 0x5FF5 {
            return Err(SpuiError::BadElementMarker(marker));
        }
        let index = reader.u16()?;
        if index == 0xFFFF {
            break;
        }
        let slot = (index as usize)
            .checked_sub(base)
            .filter(|i| *i < seen.len())
            .ok_or(SpuiError::BadElementIndex(index))?;
        if seen[slot] {
            return Err(SpuiError::DuplicateElement(index));
        }
        seen[slot] = true;
        let flags = reader.u16()?;
        let properties =
            read_properties(&mut reader, (flags & 0x7FFF) as usize, 0, &mut remaining)?;
        elements.push(SpuiElement {
            index,
            proxy_id: file.proxy_ids[slot] as u32,
            root: flags & 0x8000 != 0,
            properties,
        });
    }
    if elements.len() != file.proxy_ids.len() {
        return Err(SpuiError::BadElementCount);
    }
    if reader.u32()? != 0x1C01C047 {
        return Err(SpuiError::BadFooter);
    }
    if reader.p != reader.d.len() {
        return Err(SpuiError::TrailingBytes);
    }
    let parsed = SpuiElements { elements };
    parsed.validate_references(base + file.proxy_ids.len())?;
    Ok(parsed)
}
fn read_properties(
    reader: &mut R<'_>,
    count: usize,
    depth: usize,
    remaining: &mut usize,
) -> Result<Vec<SpuiProperty>, SpuiError> {
    if depth >= 32 {
        return Err(SpuiError::NestingLimit);
    }
    if count > 200 {
        return Err(SpuiError::BadPropertyCount(count));
    }
    *remaining = remaining
        .checked_sub(count)
        .ok_or(SpuiError::PropertyLimit)?;
    // Every property has an eight-byte header; check before allocating.
    if count > reader.d.len().saturating_sub(reader.p) / 8 {
        return Err(SpuiError::Truncated);
    }
    let mut properties = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.u32()?;
        let type_code = reader.u16()?;
        let count = reader.u16()?;
        let value = match type_code {
            2 | 3 | 5 | 7 | 9 | 11 | 15 | 16 | 17 | 19 => {
                let width = match type_code {
                    2 | 3 | 7 => 1,
                    19 => 2,
                    5 | 9 | 11 => 4,
                    15 | 17 => 8,
                    16 => 16,
                    _ => unreachable!(),
                };
                SpuiValue::Scalars {
                    width: width as u8,
                    bytes: reader.bytes(count as usize * width)?.to_vec(),
                }
            }
            18 => {
                let mut values = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let length = reader.u16()?;
                    values.push(if length == 0xFFFF {
                        SpuiText::Localized {
                            table: reader.u32()?,
                            instance: reader.u32()?,
                        }
                    } else {
                        let bytes = reader.bytes(length as usize * 2)?;
                        SpuiText::Literal(
                            bytes
                                .chunks_exact(2)
                                .map(|v| u16::from_le_bytes([v[0], v[1]]))
                                .collect(),
                        )
                    });
                }
                SpuiValue::Text(values)
            }
            20 => {
                let type_id = reader.u32()?;
                let mut values = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let child_count = reader.u16()? as usize;
                    values.push(read_properties(reader, child_count, depth + 1, remaining)?);
                }
                SpuiValue::Structures { type_id, values }
            }
            unknown => return Err(SpuiError::UnsupportedPropertyType(unknown)),
        };
        properties.push(SpuiProperty {
            id,
            type_code,
            count,
            value,
        });
    }
    Ok(properties)
}

impl SpuiElements {
    pub fn element(&self, index: u16) -> Option<&SpuiElement> {
        self.elements.iter().find(|element| element.index == index)
    }
    fn validate_references(&self, total: usize) -> Result<(), SpuiError> {
        fn check(properties: &[SpuiProperty], total: usize) -> Result<(), SpuiError> {
            for property in properties {
                match &property.value {
                    SpuiValue::Scalars { bytes, .. } if property.type_code == 19 => {
                        for bytes in bytes.chunks_exact(2) {
                            let reference = u16::from_le_bytes([bytes[0], bytes[1]]);
                            if reference != 0xFFFF && reference as usize >= total {
                                return Err(SpuiError::BadReference(reference));
                            }
                        }
                    }
                    SpuiValue::Structures { values, .. } => {
                        for value in values {
                            check(value, total)?;
                        }
                    }
                    _ => (),
                }
            }
            Ok(())
        }
        for element in &self.elements {
            check(&element.properties, total)?;
        }
        Ok(())
    }
    /// Re-encode the measured framing for independent byte-level verification.
    /// This does not manufacture class defaults or reinterpret unknown fields.
    pub fn to_bytes(&self) -> Vec<u8> {
        fn properties(output: &mut Vec<u8>, values: &[SpuiProperty]) {
            for property in values {
                output.extend(property.id.to_le_bytes());
                output.extend(property.type_code.to_le_bytes());
                output.extend(property.count.to_le_bytes());
                match &property.value {
                    SpuiValue::Scalars { bytes, .. } => output.extend(bytes),
                    SpuiValue::Text(values) => {
                        for value in values {
                            match value {
                                SpuiText::Literal(words) => {
                                    output.extend((words.len() as u16).to_le_bytes());
                                    for word in words {
                                        output.extend(word.to_le_bytes());
                                    }
                                }
                                SpuiText::Localized { table, instance } => {
                                    output.extend(0xFFFFu16.to_le_bytes());
                                    output.extend(table.to_le_bytes());
                                    output.extend(instance.to_le_bytes());
                                }
                            }
                        }
                    }
                    SpuiValue::Structures { type_id, values } => {
                        output.extend(type_id.to_le_bytes());
                        for value in values {
                            output.extend((value.len() as u16).to_le_bytes());
                            properties(output, value);
                        }
                    }
                }
            }
        }
        let mut output = Vec::new();
        for element in &self.elements {
            output.extend(0x5FF5u16.to_le_bytes());
            output.extend(element.index.to_le_bytes());
            output.extend(
                ((element.properties.len() as u16) | if element.root { 0x8000 } else { 0 })
                    .to_le_bytes(),
            );
            properties(&mut output, &element.properties);
        }
        output.extend(0x5FF5u16.to_le_bytes());
        output.extend(0xFFFFu16.to_le_bytes());
        output.extend(0x1C01C047u32.to_le_bytes());
        output
    }
}

struct R<'a> {
    d: &'a [u8],
    p: usize,
}

impl<'a> R<'a> {
    fn bytes(&mut self, count: usize) -> Result<&'a [u8], SpuiError> {
        let end = self
            .p
            .checked_add(count)
            .filter(|end| *end <= self.d.len())
            .ok_or(SpuiError::Truncated)?;
        let value = &self.d[self.p..end];
        self.p = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, SpuiError> {
        if self.p + 1 > self.d.len() {
            return Err(SpuiError::Truncated);
        }
        let v = self.d[self.p];
        self.p += 1;
        Ok(v)
    }
    fn u16(&mut self) -> Result<u16, SpuiError> {
        if self.p + 2 > self.d.len() {
            return Err(SpuiError::Truncated);
        }
        let v = u16::from_le_bytes([self.d[self.p], self.d[self.p + 1]]);
        self.p += 2;
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, SpuiError> {
        if self.p + 4 > self.d.len() {
            return Err(SpuiError::Truncated);
        }
        let v = u32::from_le_bytes([
            self.d[self.p],
            self.d[self.p + 1],
            self.d[self.p + 2],
            self.d[self.p + 3],
        ]);
        self.p += 4;
        Ok(v)
    }
    fn i32(&mut self) -> Result<i32, SpuiError> {
        Ok(self.u32()? as i32)
    }
    fn key(&mut self) -> Result<SpuiKey, SpuiError> {
        let instance = self.u32()?;
        let type_id = self.u32()?;
        let group = self.u32()?;
        Ok(SpuiKey {
            instance,
            type_id,
            group,
        })
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], SpuiError> {
        if self.p + n > self.d.len() {
            return Err(SpuiError::Truncated);
        }
        let s = &self.d[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
}

/// Parse a `.spui` resource container.
pub fn parse(data: &[u8]) -> Result<SpuiFile, SpuiError> {
    if data.len() < 14 {
        return Err(SpuiError::TooShort);
    }
    let mut r = R { d: data, p: 0 };
    let magic = r.u32()?;
    if magic != MAGIC {
        return Err(SpuiError::BadMagic(magic));
    }
    let version = r.u16()?;
    if !(1..=3).contains(&version) {
        return Err(SpuiError::BadVersion(version));
    }
    let ic = r.u16()? as usize;
    let ac = r.u16()? as usize;
    let hc = r.u16()? as usize;
    let pc = r.u16()? as usize;
    if ic + ac + hc + pc > 6000 {
        return Err(SpuiError::Truncated);
    }

    let mut image_resources = Vec::with_capacity(ic);
    for _ in 0..ic {
        image_resources.push(r.key()?);
    }
    let mut atlas_resources = Vec::with_capacity(ac);
    for _ in 0..ac {
        atlas_resources.push(r.key()?);
    }

    let mut hit_masks = Vec::with_capacity(hc);
    for _ in 0..hc {
        let start = r.p;
        let mut mask = SpuiHitMask {
            use_rle: false,
            width: 0,
            height: 0,
            key: SpuiKey {
                instance: 0,
                type_id: 0,
                group: 0,
            },
            raw: Vec::new(),
        };
        if version >= 3 {
            mask.use_rle = r.u8()? != 0;
        }
        if mask.use_rle {
            mask.width = r.i32()?;
            mask.height = r.i32()?;
            let wc = r.i32()?;
            if wc < 0 || wc > 10_000_000 {
                return Err(SpuiError::Truncated);
            }
            r.take(wc as usize * 2)?;
        } else {
            mask.key = r.key()?;
        }
        mask.raw = data[start..r.p].to_vec();
        hit_masks.push(mask);
    }

    let mut proxy_ids = Vec::with_capacity(pc);
    for _ in 0..pc {
        proxy_ids.push(r.i32()?);
    }

    let consumed = r.p;
    let element_data = data[consumed..].to_vec();

    Ok(SpuiFile {
        version,
        image_resources,
        atlas_resources,
        hit_masks,
        proxy_ids,
        element_data,
        consumed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(v: &mut Vec<u8>, x: u16) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn push_u32(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn push_key(v: &mut Vec<u8>, i: u32, t: u32, g: u32) {
        push_u32(v, i);
        push_u32(v, t);
        push_u32(v, g);
    }

    fn build(version: u16, images: u16, atlases: u16, hit: u16, proxies: u16) -> Vec<u8> {
        let mut d = Vec::new();
        push_u32(&mut d, MAGIC);
        push_u16(&mut d, version);
        push_u16(&mut d, images);
        push_u16(&mut d, atlases);
        push_u16(&mut d, hit);
        push_u16(&mut d, proxies);
        for i in 0..images {
            push_key(&mut d, i as u32, 1, 2);
        }
        for i in 0..atlases {
            push_key(&mut d, 100 + i as u32, 3, 4);
        }
        for _ in 0..hit {
            // key form (use_rle = 0 for v3)
            d.push(0);
            push_key(&mut d, 7, 8, 9);
        }
        for i in 0..proxies {
            push_u32(&mut d, 0xA000 + i as u32);
        }
        // element section
        d.extend_from_slice(&[1, 2, 3, 4, 5]);
        d
    }

    #[test]
    fn parse_container() {
        let d = build(3, 1, 1, 1, 1);
        let s = parse(&d).unwrap();
        assert_eq!(s.version, 3);
        assert_eq!(s.image_resources.len(), 1);
        assert_eq!(
            s.image_resources[0],
            SpuiKey {
                instance: 0,
                type_id: 1,
                group: 2
            }
        );
        assert_eq!(s.atlas_resources[0].instance, 100);
        assert_eq!(s.hit_masks.len(), 1);
        assert!(!s.hit_masks[0].use_rle);
        assert_eq!(
            s.hit_masks[0].key,
            SpuiKey {
                instance: 7,
                type_id: 8,
                group: 9
            }
        );
        assert_eq!(s.proxy_ids, vec![0xA000]);
        assert_eq!(s.element_data, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn parse_rle_hit_mask() {
        let mut d = Vec::new();
        push_u32(&mut d, MAGIC);
        push_u16(&mut d, 3);
        push_u16(&mut d, 0); // images
        push_u16(&mut d, 0); // atlases
        push_u16(&mut d, 1); // hit masks
        push_u16(&mut d, 0); // proxies
        d.push(1); // use_rle
        push_u32(&mut d, 4); // width
        push_u32(&mut d, 3); // height
        push_u32(&mut d, 2); // word_count
        d.extend_from_slice(&[0u8; 4]); // 2 words
        let s = parse(&d).unwrap();
        assert!(s.hit_masks[0].use_rle);
        assert_eq!(s.hit_masks[0].width, 4);
        assert_eq!(s.hit_masks[0].height, 3);
        assert!(s.element_data.is_empty());
    }

    #[test]
    fn bad_magic() {
        let mut d = build(3, 0, 0, 0, 0);
        d[0] = 0;
        assert!(matches!(parse(&d), Err(SpuiError::BadMagic(_))));
    }

    #[test]
    fn native_reader_version_range_excludes_zero_and_future_versions() {
        for version in [0u16, 4, u16::MAX] {
            let mut data = build(3, 0, 0, 0, 0);
            data[4..6].copy_from_slice(&version.to_le_bytes());
            assert!(matches!(parse(&data), Err(SpuiError::BadVersion(v)) if v == version));
        }
    }

    #[test]
    fn too_short() {
        assert!(matches!(parse(&[0u8; 6]), Err(SpuiError::TooShort)));
    }

    fn typed_fixture() -> SpuiFile {
        let mut file = parse(&build(3, 1, 1, 1, 1)).unwrap();
        let element = SpuiElement {
            index: 3,
            proxy_id: 0xA000,
            root: true,
            properties: vec![
                SpuiProperty {
                    id: 1,
                    type_code: 16,
                    count: 1,
                    value: SpuiValue::Scalars {
                        width: 16,
                        bytes: [1f32, 2., 3., 4.]
                            .into_iter()
                            .flat_map(f32::to_le_bytes)
                            .collect(),
                    },
                },
                SpuiProperty {
                    id: 2,
                    type_code: 18,
                    count: 2,
                    value: SpuiValue::Text(vec![
                        SpuiText::Literal("SPORE 🧬".encode_utf16().collect()),
                        SpuiText::Localized {
                            table: 0x1234,
                            instance: 0x5678,
                        },
                    ]),
                },
                SpuiProperty {
                    id: 3,
                    type_code: 20,
                    count: 3,
                    value: SpuiValue::Structures {
                        type_id: 0x32,
                        values: vec![
                            vec![],
                            vec![SpuiProperty {
                                id: 4,
                                type_code: 19,
                                count: 1,
                                value: SpuiValue::Scalars {
                                    width: 2,
                                    bytes: 0xFFFFu16.to_le_bytes().to_vec(),
                                },
                            }],
                            vec![],
                        ],
                    },
                },
            ],
        };
        file.element_data = SpuiElements {
            elements: vec![element],
        }
        .to_bytes();
        file
    }
    #[test]
    fn typed_stream_keeps_nested_array_framing_text_bits_and_resource_base() {
        let file = typed_fixture();
        let typed = parse_elements(&file).unwrap();
        assert_eq!(typed.elements[0].index, 3);
        assert_eq!(typed.elements[0].proxy_id, 0xA000);
        assert!(typed.elements[0].root);
        assert_eq!(
            typed.elements[0].property(1).unwrap().rectangle(),
            Some([1., 2., 3., 4.])
        );
        assert_eq!(typed.to_bytes(), file.element_data);
        let SpuiValue::Structures { values, .. } = &typed.elements[0].property(3).unwrap().value
        else {
            panic!()
        };
        assert_eq!(values[1][0].reference(), Some(0xFFFF));
        assert_eq!(values.len(), 3);
    }
    #[test]
    fn typed_stream_rejects_every_truncated_prefix_unknown_tag_and_trailing_data() {
        let mut file = typed_fixture();
        let original = file.element_data.clone();
        for length in 0..original.len() {
            file.element_data = original[..length].to_vec();
            assert!(parse_elements(&file).is_err(), "accepted prefix {length}");
        }
        file.element_data = original.clone();
        file.element_data[10..12].copy_from_slice(&0x7000u16.to_le_bytes());
        assert_eq!(
            parse_elements(&file),
            Err(SpuiError::UnsupportedPropertyType(0x7000))
        );
        file.element_data = original;
        file.element_data.push(0);
        assert_eq!(parse_elements(&file), Err(SpuiError::TrailingBytes));
    }
    #[test]
    fn typed_stream_rejects_out_of_range_duplicate_and_missing_objects() {
        let mut file = typed_fixture();
        file.element_data[2..4].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(parse_elements(&file), Err(SpuiError::BadElementIndex(2)));
        let mut file = typed_fixture();
        let mut typed = parse_elements(&file).unwrap();
        typed.elements.push(typed.elements[0].clone());
        file.element_data = typed.to_bytes();
        assert_eq!(parse_elements(&file), Err(SpuiError::DuplicateElement(3)));
        file.element_data = SpuiElements { elements: vec![] }.to_bytes();
        assert_eq!(parse_elements(&file), Err(SpuiError::BadElementCount));
    }
}
