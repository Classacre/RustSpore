//! Bounded metadata from the original UI's SFNT resources.
//!
//! Native 8149C0 loads type 027C5CEF in namespace 0248E873; the shipped
//! Trebuchet EA face is a TrueType SFNT. This reader follows the public
//! OpenType table formats. It returns original design units and explicit
//! cmap records, without choosing a native fallback face, scaling, shaping,
//! interpreting TrueType instructions, rasterizing or uploading glyphs.
//!
//! Supported: TrueType 1.0, name format 0 with explicit UTF-16 decoding,
//! cmap format 4, hhea/hmtx, and loca/glyf header bounding boxes. Unsupported
//! encodings/formats return errors when requested. Auxiliary tables and
//! unselected cmap formats are retained, not interpreted.

use std::fmt;

pub const FONT_RESOURCE_TYPE: u32 = 0x027c_5cef;
pub const FONT_NAMESPACE: u32 = 0x0248_e873;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontError {
    Truncated(&'static str),
    Invalid(&'static str),
    UnsupportedSfnt(u32),
    UnsupportedTableVersion([u8; 4], u32),
    MissingTable([u8; 4]),
    DuplicateTable([u8; 4]),
    UnsupportedNameFormat(u16),
    UnsupportedNameEncoding(u16, u16),
    MissingCmap(u16, u16),
    UnsupportedCmapFormat(u16),
    GlyphOutOfRange(u16),
}
impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source UI SFNT: {self:?}")
    }
}
impl std::error::Error for FontError {}

fn bytes<'a>(data: &'a [u8], at: usize, len: usize) -> Result<&'a [u8], FontError> {
    let end = at
        .checked_add(len)
        .ok_or(FontError::Invalid("offset overflow"))?;
    data.get(at..end)
        .ok_or(FontError::Truncated("table or field"))
}
fn u16_at(data: &[u8], at: usize) -> Result<u16, FontError> {
    Ok(u16::from_be_bytes(bytes(data, at, 2)?.try_into().unwrap()))
}
fn i16_at(data: &[u8], at: usize) -> Result<i16, FontError> {
    Ok(u16_at(data, at)? as i16)
}
fn u32_at(data: &[u8], at: usize) -> Result<u32, FontError> {
    Ok(u32::from_be_bytes(bytes(data, at, 4)?.try_into().unwrap()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontTable {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphBounds {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}
fn bounds(data: &[u8], at: usize) -> Result<GlyphBounds, FontError> {
    Ok(GlyphBounds {
        x_min: i16_at(data, at)?,
        y_min: i16_at(data, at + 2)?,
        x_max: i16_at(data, at + 4)?,
        y_max: i16_at(data, at + 6)?,
    })
}

/// Unscaled header values. These are not native line heights or raster sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontMetrics {
    pub units_per_em: u16,
    pub glyph_count: u16,
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub bounds: GlyphBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalMetrics {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameRecord<'a> {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub language_id: u16,
    pub name_id: u16,
    pub encoded: &'a [u8],
}
impl NameRecord<'_> {
    /// Decode only explicitly Unicode records. Macintosh/code-page records
    /// remain accessible as bytes and are not silently treated as UTF-8.
    pub fn unicode_text(&self) -> Result<String, FontError> {
        if !((self.platform_id == 0 && self.encoding_id <= 6)
            || (self.platform_id == 3 && matches!(self.encoding_id, 0 | 1 | 10)))
        {
            return Err(FontError::UnsupportedNameEncoding(
                self.platform_id,
                self.encoding_id,
            ));
        }
        if self.encoded.len() % 2 != 0 {
            return Err(FontError::Invalid("odd UTF-16 name length"));
        }
        let units: Vec<_> = self
            .encoded
            .chunks_exact(2)
            .map(|p| u16::from_be_bytes([p[0], p[1]]))
            .collect();
        String::from_utf16(&units).map_err(|_| FontError::Invalid("invalid UTF-16 name"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmapRecord {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub format: u16,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct Font<'a> {
    data: &'a [u8],
    tables: Vec<FontTable>,
    metrics: FontMetrics,
    horizontal_count: u16,
    long_locations: bool,
}

pub fn parse(data: &[u8]) -> Result<Font<'_>, FontError> {
    let version = u32_at(data, 0)?;
    if version != 0x0001_0000 {
        return Err(FontError::UnsupportedSfnt(version));
    }
    let count = usize::from(u16_at(data, 4)?);
    if count == 0 || count > 4096 {
        return Err(FontError::Invalid("table count outside reader bound"));
    }
    bytes(data, 0, 12 + count * 16)?;
    let mut tables = Vec::with_capacity(count);
    for i in 0..count {
        let at = 12 + i * 16;
        let table = FontTable {
            tag: bytes(data, at, 4)?.try_into().unwrap(),
            checksum: u32_at(data, at + 4)?,
            offset: u32_at(data, at + 8)?,
            length: u32_at(data, at + 12)?,
        };
        bytes(data, table.offset as usize, table.length as usize)?;
        if tables.iter().any(|t: &FontTable| t.tag == table.tag) {
            return Err(FontError::DuplicateTable(table.tag));
        }
        tables.push(table);
    }
    let table = |tag: [u8; 4]| -> Result<&[u8], FontError> {
        let t = tables
            .iter()
            .find(|t| t.tag == tag)
            .ok_or(FontError::MissingTable(tag))?;
        bytes(data, t.offset as usize, t.length as usize)
    };
    let head = table(*b"head")?;
    let hhea = table(*b"hhea")?;
    let maxp = table(*b"maxp")?;
    for (tag, value) in [(*b"head", head), (*b"hhea", hhea), (*b"maxp", maxp)] {
        let version = u32_at(value, 0)?;
        if version != 0x0001_0000 {
            return Err(FontError::UnsupportedTableVersion(tag, version));
        }
    }
    bytes(head, 0, 54)?;
    bytes(hhea, 0, 36)?;
    bytes(maxp, 0, 32)?;
    if u32_at(head, 12)? != 0x5f0f_3cf5 || i16_at(head, 52)? != 0 || i16_at(hhea, 32)? != 0 {
        return Err(FontError::Invalid("head magic or metric/glyph data format"));
    }
    let units_per_em = u16_at(head, 18)?;
    let glyph_count = u16_at(maxp, 4)?;
    let horizontal_count = u16_at(hhea, 34)?;
    if !(16..=16384).contains(&units_per_em)
        || glyph_count == 0
        || horizontal_count == 0
        || horizontal_count > glyph_count
    {
        return Err(FontError::Invalid("units or glyph/metric count"));
    }
    let long_locations = match i16_at(head, 50)? {
        0 => false,
        1 => true,
        _ => return Err(FontError::Invalid("loca index format")),
    };
    bytes(
        table(*b"hmtx")?,
        0,
        usize::from(horizontal_count) * 4 + usize::from(glyph_count - horizontal_count) * 2,
    )?;
    let loca = table(*b"loca")?;
    let glyf = table(*b"glyf")?;
    let stride = if long_locations { 4 } else { 2 };
    bytes(loca, 0, (usize::from(glyph_count) + 1) * stride)?;
    let metrics = FontMetrics {
        units_per_em,
        glyph_count,
        ascender: i16_at(hhea, 4)?,
        descender: i16_at(hhea, 6)?,
        line_gap: i16_at(hhea, 8)?,
        bounds: bounds(head, 36)?,
    };
    let font = Font {
        data,
        tables,
        metrics,
        horizontal_count,
        long_locations,
    };
    let mut previous = 0;
    for i in 0..=glyph_count {
        let next = font.location(i)?;
        if next < previous || next > glyf.len() {
            return Err(FontError::Invalid("non-monotonic or out-of-range loca"));
        }
        if i != 0 && next != previous {
            bytes(glyf, previous, 10)?;
            if next - previous < 10 || i16_at(glyf, previous)? < -1 {
                return Err(FontError::Invalid("short or invalid glyf header"));
            }
        }
        previous = next;
    }
    font.name_records()?;
    font.cmap_records()?;
    Ok(font)
}

impl<'a> Font<'a> {
    pub fn tables(&self) -> &[FontTable] {
        &self.tables
    }
    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }
    pub fn table(&self, tag: [u8; 4]) -> Result<&'a [u8], FontError> {
        let t = self
            .tables
            .iter()
            .find(|t| t.tag == tag)
            .ok_or(FontError::MissingTable(tag))?;
        bytes(self.data, t.offset as usize, t.length as usize)
    }
    fn location(&self, glyph: u16) -> Result<usize, FontError> {
        let data = self.table(*b"loca")?;
        if self.long_locations {
            Ok(u32_at(data, usize::from(glyph) * 4)? as usize)
        } else {
            Ok(usize::from(u16_at(data, usize::from(glyph) * 2)?) * 2)
        }
    }
    pub fn horizontal_metrics(&self, glyph: u16) -> Result<HorizontalMetrics, FontError> {
        if glyph >= self.metrics.glyph_count {
            return Err(FontError::GlyphOutOfRange(glyph));
        }
        let data = self.table(*b"hmtx")?;
        let last = self.horizontal_count - 1;
        let advance_width = u16_at(data, usize::from(glyph.min(last)) * 4)?;
        let bearing_at = if glyph < self.horizontal_count {
            usize::from(glyph) * 4 + 2
        } else {
            usize::from(self.horizontal_count) * 4 + usize::from(glyph - self.horizontal_count) * 2
        };
        Ok(HorizontalMetrics {
            advance_width,
            left_side_bearing: i16_at(data, bearing_at)?,
        })
    }
    /// Raw unscaled glyf header. An empty loca interval has no bounding box.
    /// Composite outlines and hinting instructions are deliberately not read.
    pub fn glyph_bounds(&self, glyph: u16) -> Result<Option<GlyphBounds>, FontError> {
        if glyph >= self.metrics.glyph_count {
            return Err(FontError::GlyphOutOfRange(glyph));
        }
        let start = self.location(glyph)?;
        let end = self.location(glyph + 1)?;
        if start == end {
            return Ok(None);
        }
        Ok(Some(bounds(self.table(*b"glyf")?, start + 2)?))
    }
    pub fn name_records(&self) -> Result<Vec<NameRecord<'a>>, FontError> {
        let data = self.table(*b"name")?;
        let format = u16_at(data, 0)?;
        if format != 0 {
            return Err(FontError::UnsupportedNameFormat(format));
        }
        let count = usize::from(u16_at(data, 2)?);
        let strings = usize::from(u16_at(data, 4)?);
        bytes(data, 0, 6 + count * 12)?;
        if strings < 6 + count * 12 {
            return Err(FontError::Invalid("name string storage"));
        }
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let at = 6 + i * 12;
            let offset = usize::from(u16_at(data, at + 10)?);
            result.push(NameRecord {
                platform_id: u16_at(data, at)?,
                encoding_id: u16_at(data, at + 2)?,
                language_id: u16_at(data, at + 4)?,
                name_id: u16_at(data, at + 6)?,
                encoded: bytes(data, strings + offset, usize::from(u16_at(data, at + 8)?))?,
            });
        }
        Ok(result)
    }
    pub fn cmap_records(&self) -> Result<Vec<CmapRecord>, FontError> {
        let data = self.table(*b"cmap")?;
        if u16_at(data, 0)? != 0 {
            return Err(FontError::Invalid("cmap version"));
        }
        let count = usize::from(u16_at(data, 2)?);
        bytes(data, 0, 4 + count * 8)?;
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let at = 4 + i * 8;
            let platform_id = u16_at(data, at)?;
            let encoding_id = u16_at(data, at + 2)?;
            if result
                .iter()
                .any(|r: &CmapRecord| r.platform_id == platform_id && r.encoding_id == encoding_id)
            {
                return Err(FontError::Invalid("duplicate cmap encoding"));
            }
            let offset = u32_at(data, at + 4)?;
            if (offset as usize) < 4 + count * 8 {
                return Err(FontError::Invalid("cmap subtable offset"));
            }
            result.push(CmapRecord {
                platform_id,
                encoding_id,
                offset,
                format: u16_at(data, offset as usize)?,
            });
        }
        Ok(result)
    }
    /// The caller selects an encoding record explicitly. This is not native
    /// font fallback or Unicode-record priority policy.
    pub fn cmap_format4(&self, platform: u16, encoding: u16) -> Result<Cmap4<'a>, FontError> {
        let r = self
            .cmap_records()?
            .into_iter()
            .find(|r| r.platform_id == platform && r.encoding_id == encoding)
            .ok_or(FontError::MissingCmap(platform, encoding))?;
        let data = self.table(*b"cmap")?;
        Cmap4::parse(&data[r.offset as usize..], self.metrics.glyph_count)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cmap4<'a> {
    data: &'a [u8],
    segments: usize,
}
impl<'a> Cmap4<'a> {
    fn parse(data: &'a [u8], glyph_count: u16) -> Result<Self, FontError> {
        let format = u16_at(data, 0)?;
        if format != 4 {
            return Err(FontError::UnsupportedCmapFormat(format));
        }
        let data = bytes(data, 0, usize::from(u16_at(data, 2)?))?;
        let twice = u16_at(data, 6)?;
        if twice == 0 || twice % 2 != 0 {
            return Err(FontError::Invalid("cmap segment count"));
        }
        let segments = usize::from(twice / 2);
        bytes(data, 0, 16 + segments * 8)?;
        if u16_at(data, 14 + segments * 2)? != 0 {
            return Err(FontError::Invalid("cmap reserved pad"));
        }
        let map = Self { data, segments };
        let mut previous = None;
        for i in 0..segments {
            let end = map.end(i);
            let start = map.start(i);
            if start > end || previous.is_some_and(|p| start <= p) {
                return Err(FontError::Invalid("overlapping or inverted cmap segments"));
            }
            let range = usize::from(map.range(i));
            if range != 0 {
                let first = map.range_at(i) + range;
                if range % 2 != 0 || first < 16 + segments * 8 {
                    return Err(FontError::Invalid("cmap glyph array offset"));
                }
                bytes(data, first, (usize::from(end - start) + 1) * 2)?;
            }
            for c in u32::from(start)..=u32::from(end) {
                if map.in_segment(i, c as u16) >= glyph_count {
                    return Err(FontError::Invalid("cmap glyph exceeds maxp"));
                }
            }
            previous = Some(end);
        }
        if map.start(segments - 1) != 0xffff || map.end(segments - 1) != 0xffff {
            return Err(FontError::Invalid("missing cmap terminal segment"));
        }
        Ok(map)
    }
    fn word(&self, at: usize) -> u16 {
        u16_at(self.data, at).unwrap()
    }
    fn end(&self, i: usize) -> u16 {
        self.word(14 + 2 * i)
    }
    fn start(&self, i: usize) -> u16 {
        self.word(16 + 2 * self.segments + 2 * i)
    }
    fn delta(&self, i: usize) -> u16 {
        self.word(16 + 4 * self.segments + 2 * i)
    }
    fn range_at(&self, i: usize) -> usize {
        16 + 6 * self.segments + 2 * i
    }
    fn range(&self, i: usize) -> u16 {
        self.word(self.range_at(i))
    }
    fn in_segment(&self, i: usize, code: u16) -> u16 {
        if self.range(i) == 0 {
            return code.wrapping_add(self.delta(i));
        }
        let at =
            self.range_at(i) + usize::from(self.range(i)) + usize::from(code - self.start(i)) * 2;
        let glyph = self.word(at);
        if glyph == 0 {
            0
        } else {
            glyph.wrapping_add(self.delta(i))
        }
    }
    /// Original format-4 mapping of one 16-bit code. Missing glyph is zero.
    /// Supplementary-plane shaping/fallback is not supplied by this API.
    pub fn glyph_index(&self, code: u16) -> u16 {
        let mut lo = 0;
        let mut hi = self.segments;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.end(mid) < code {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        if code < self.start(lo) {
            0
        } else {
            self.in_segment(lo, code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn words(values: &[u16]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_be_bytes()).collect()
    }

    #[test]
    fn public_format4_delta_example_and_unmapped_holes() {
        // Independent example from the public OpenType format-4 table.
        let data = words(&[
            4,
            48,
            0,
            8,
            8,
            2,
            0,
            20,
            90,
            480,
            65535,
            0,
            10,
            30,
            153,
            65535,
            (-9i16) as u16,
            (-18i16) as u16,
            (-80i16) as u16,
            1,
            0,
            0,
            0,
            0,
        ]);
        let map = Cmap4::parse(&data, 401).unwrap();
        for (code, glyph) in [
            (0, 0),
            (9, 0),
            (10, 1),
            (20, 11),
            (21, 0),
            (30, 12),
            (90, 72),
            (91, 0),
            (153, 73),
            (480, 400),
            (481, 0),
            (65535, 0),
        ] {
            assert_eq!(map.glyph_index(code), glyph);
        }
    }
    #[test]
    fn range_array_delta_keeps_zero_missing_and_is_bounded() {
        let mut data = words(&[
            4, 38, 0, 4, 4, 1, 0, 67, 65535, 0, 65, 65535, 2, 1, 4, 0, 1, 0, 3,
        ]);
        let map = Cmap4::parse(&data, 6).unwrap();
        assert_eq!(
            (
                map.glyph_index(65),
                map.glyph_index(66),
                map.glyph_index(67)
            ),
            (3, 0, 5)
        );
        data[28..30].copy_from_slice(&0xfffeu16.to_be_bytes());
        assert!(Cmap4::parse(&data, 6).is_err());
        assert_eq!(
            Cmap4::parse(&[0, 12, 0, 0], 6).unwrap_err(),
            FontError::UnsupportedCmapFormat(12)
        );
    }
    #[test]
    fn metrics_share_last_advance_without_sharing_bearings() {
        let data = words(&[20, (-2i16) as u16, 30, 3, (-4i16) as u16, 5]);
        let font = Font {
            data: &data,
            tables: vec![FontTable {
                tag: *b"hmtx",
                checksum: 0,
                offset: 0,
                length: 12,
            }],
            metrics: FontMetrics {
                units_per_em: 1000,
                glyph_count: 4,
                ascender: 900,
                descender: -200,
                line_gap: 0,
                bounds: GlyphBounds {
                    x_min: 0,
                    y_min: 0,
                    x_max: 20,
                    y_max: 30,
                },
            },
            horizontal_count: 2,
            long_locations: false,
        };
        assert_eq!(
            font.horizontal_metrics(2).unwrap(),
            HorizontalMetrics {
                advance_width: 30,
                left_side_bearing: -4
            }
        );
        assert_eq!(font.horizontal_metrics(3).unwrap().left_side_bearing, 5);
        assert_eq!(
            font.horizontal_metrics(4),
            Err(FontError::GlyphOutOfRange(4))
        );
    }
    #[test]
    fn unicode_names_reject_replacement_decoding() {
        let encoded = words(&[0x0041, 0xd83d, 0xde00]);
        let name = NameRecord {
            platform_id: 3,
            encoding_id: 1,
            language_id: 0x409,
            name_id: 1,
            encoded: &encoded,
        };
        assert_eq!(name.unicode_text().unwrap(), "A\u{1f600}");
        assert!(NameRecord {
            encoded: &[0xd8, 0x3d],
            ..name
        }
        .unicode_text()
        .is_err());
        assert!(NameRecord {
            platform_id: 1,
            ..name
        }
        .unicode_text()
        .is_err());
        assert!(NameRecord {
            platform_id: 0,
            encoding_id: 65535,
            ..name
        }
        .unicode_text()
        .is_err());
        assert!(NameRecord {
            encoded: &[0],
            ..name
        }
        .unicode_text()
        .is_err());
    }
    #[test]
    fn unsupported_scaler_and_truncated_directory_are_errors() {
        assert_eq!(
            parse(b"OTTO").unwrap_err(),
            FontError::UnsupportedSfnt(0x4f54_544f)
        );
        assert!(parse(&[0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(parse(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }
}
