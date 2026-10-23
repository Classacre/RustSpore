//! Ordinary TrueType glyph packets from the original UI font resources.
//!
//! This returns unscaled design points, original expanded flags, borrowed hint
//! bytes and composite descriptors. It does not flatten composites, append the
//! native scaler's four phantom points, execute hints or rasterize. Native
//! 8CD700 stores accumulated coordinates as 16-bit words and returns immediately
//! for a zero-contour header. Those two packet semantics are explicit here.
//! Malformed repeat runs are rejected; the native reader's clamping of such a
//! run is not an admitted malformed-font compatibility policy.

use crate::spui_font::{Font, FontError, GlyphBounds};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutlineError {
    Font(FontError),
    Truncated(&'static str),
    Invalid(&'static str),
    UnsupportedPointCount(u32),
    UnsupportedInstructionLength(u16),
}
impl From<FontError> for OutlineError {
    fn from(value: FontError) -> Self {
        Self::Font(value)
    }
}
impl fmt::Display for OutlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source UI glyph packet: {self:?}")
    }
}
impl std::error::Error for OutlineError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignPoint {
    pub x: i16,
    pub y: i16,
    /// Original flag byte, including compression/repeat and overlap bits.
    pub flag: u8,
}
impl DesignPoint {
    pub fn on_curve(self) -> bool {
        self.flag & 1 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentArguments {
    Bytes([u8; 2]),
    Words([u16; 2]),
}

/// Original signed F2DOT14 coefficients; no float conversion or rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentTransform {
    Identity,
    Uniform(i16),
    ScaleXY { x: i16, y: i16 },
    Matrix { xx: i16, xy: i16, yx: i16, yy: i16 },
}
impl ComponentTransform {
    pub fn matrix(self) -> [i16; 4] {
        match self {
            Self::Identity => [0x4000, 0, 0, 0x4000],
            Self::Uniform(v) => [v, 0, 0, v],
            Self::ScaleXY { x, y } => [x, 0, 0, y],
            Self::Matrix { xx, xy, yx, yy } => [xx, xy, yx, yy],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Component {
    pub flags: u16,
    pub glyph_index: u16,
    pub arguments: ComponentArguments,
    pub transform: ComponentTransform,
}
impl Component {
    /// Interpret arguments only when the original XY-offset flag is set.
    pub fn xy_offset(self) -> Option<[i16; 2]> {
        if self.flags & 2 == 0 {
            return None;
        }
        Some(match self.arguments {
            ComponentArguments::Bytes([a, b]) => [i16::from(a as i8), i16::from(b as i8)],
            ComponentArguments::Words([a, b]) => [a as i16, b as i16],
        })
    }

    pub fn matched_points(self) -> Option<[u16; 2]> {
        if self.flags & 2 != 0 {
            return None;
        }
        Some(match self.arguments {
            ComponentArguments::Bytes([a, b]) => [u16::from(a), u16::from(b)],
            ComponentArguments::Words(v) => v,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleGlyph<'a> {
    /// Inclusive last point index of each contour, in source order.
    pub end_points: Vec<u16>,
    pub points: Vec<DesignPoint>,
    pub instructions: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeGlyph<'a> {
    /// The native reader selects composite packets for every negative marker.
    pub contour_marker: i16,
    pub components: Vec<Component>,
    pub instructions: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlyphKind<'a> {
    /// Equal consecutive loca entries: no glyf header is present.
    Empty,
    /// Native zero-contour early return; any remaining bytes are uninterpreted.
    ZeroContours,
    Simple(SimpleGlyph<'a>),
    Composite(CompositeGlyph<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphPacket<'a> {
    pub bounds: Option<GlyphBounds>,
    pub kind: GlyphKind<'a>,
    /// Original bytes after the consumed packet, commonly alignment padding.
    pub remainder: &'a [u8],
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], OutlineError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(OutlineError::Invalid("packet offset overflow"))?;
        let data = self
            .data
            .get(self.at..end)
            .ok_or(OutlineError::Truncated("glyph field"))?;
        self.at = end;
        Ok(data)
    }
    fn byte(&mut self) -> Result<u8, OutlineError> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> Result<u16, OutlineError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn signed(&mut self) -> Result<i16, OutlineError> {
        Ok(self.word()? as i16)
    }
    fn instructions(&mut self) -> Result<&'a [u8], OutlineError> {
        let n = self.word()?;
        // 8CD8AB/8CD99F sign-extend the original 16-bit instruction length.
        if n > i16::MAX as u16 {
            return Err(OutlineError::UnsupportedInstructionLength(n));
        }
        self.take(usize::from(n))
    }
    fn coordinate(
        &mut self,
        flag: u8,
        short: u8,
        same_or_positive: u8,
    ) -> Result<i16, OutlineError> {
        if flag & short != 0 {
            let delta = i16::from(self.byte()?);
            Ok(if flag & same_or_positive != 0 {
                delta
            } else {
                -delta
            })
        } else if flag & same_or_positive != 0 {
            Ok(0)
        } else {
            self.signed()
        }
    }
}

/// Read one unscaled ordinary packet. No component recursion is performed.
pub fn parse_packet(data: &[u8], glyph_count: u16) -> Result<GlyphPacket<'_>, OutlineError> {
    if data.is_empty() {
        return Ok(GlyphPacket {
            bounds: None,
            kind: GlyphKind::Empty,
            remainder: data,
        });
    }
    let mut r = Reader { data, at: 0 };
    let contours = r.signed()?;
    let bounds = Some(GlyphBounds {
        x_min: r.signed()?,
        y_min: r.signed()?,
        x_max: r.signed()?,
        y_max: r.signed()?,
    });
    let kind = if contours == 0 {
        GlyphKind::ZeroContours
    } else if contours > 0 {
        let mut end_points = Vec::with_capacity(contours as usize);
        for _ in 0..contours {
            let end = r.word()?;
            if end_points.last().is_some_and(|old| end <= *old) {
                return Err(OutlineError::Invalid("non-increasing contour endpoint"));
            }
            end_points.push(end);
        }
        let count = u32::from(*end_points.last().unwrap()) + 1;
        // 8CD990 stores the accumulated point count as a signed 16-bit field.
        if count > i16::MAX as u32 {
            return Err(OutlineError::UnsupportedPointCount(count));
        }
        let count = count as usize;
        let instructions = r.instructions()?;
        let mut points = Vec::with_capacity(count);
        while points.len() < count {
            let flag = r.byte()?;
            let run = if flag & 8 != 0 {
                usize::from(r.byte()?) + 1
            } else {
                1
            };
            if run > count - points.len() {
                return Err(OutlineError::Invalid("flag repeat exceeds point count"));
            }
            points.extend((0..run).map(|_| DesignPoint { x: 0, y: 0, flag }));
        }
        let mut coordinate = 0i16;
        for point in &mut points {
            coordinate = coordinate.wrapping_add(r.coordinate(point.flag, 2, 0x10)?);
            point.x = coordinate;
        }
        coordinate = 0;
        for point in &mut points {
            coordinate = coordinate.wrapping_add(r.coordinate(point.flag, 4, 0x20)?);
            point.y = coordinate;
        }
        GlyphKind::Simple(SimpleGlyph {
            end_points,
            points,
            instructions,
        })
    } else {
        let mut components = Vec::new();
        let last_flags = loop {
            let flags = r.word()?;
            let glyph_index = r.word()?;
            if glyph_index >= glyph_count {
                return Err(OutlineError::Font(FontError::GlyphOutOfRange(glyph_index)));
            }
            let arguments = if flags & 1 != 0 {
                ComponentArguments::Words([r.word()?, r.word()?])
            } else {
                ComponentArguments::Bytes([r.byte()?, r.byte()?])
            };
            // The source parser selects these fields in this priority order.
            let transform = if flags & 8 != 0 {
                ComponentTransform::Uniform(r.signed()?)
            } else if flags & 0x40 != 0 {
                ComponentTransform::ScaleXY {
                    x: r.signed()?,
                    y: r.signed()?,
                }
            } else if flags & 0x80 != 0 {
                ComponentTransform::Matrix {
                    xx: r.signed()?,
                    xy: r.signed()?,
                    yx: r.signed()?,
                    yy: r.signed()?,
                }
            } else {
                ComponentTransform::Identity
            };
            components.push(Component {
                flags,
                glyph_index,
                arguments,
                transform,
            });
            if flags & 0x20 == 0 {
                break flags;
            }
        };
        // 8CD7E4..8CD7F5 updates this bit for each component; the last wins.
        let instructions = if last_flags & 0x100 != 0 {
            r.instructions()?
        } else {
            &[]
        };
        GlyphKind::Composite(CompositeGlyph {
            contour_marker: contours,
            components,
            instructions,
        })
    };
    Ok(GlyphPacket {
        bounds,
        kind,
        remainder: &data[r.at..],
    })
}

pub fn parse_glyph<'a>(font: &Font<'a>, glyph: u16) -> Result<GlyphPacket<'a>, OutlineError> {
    if glyph >= font.metrics().glyph_count {
        return Err(OutlineError::Font(FontError::GlyphOutOfRange(glyph)));
    }
    let head = font.table(*b"head")?;
    let loca = font.table(*b"loca")?;
    let glyf = font.table(*b"glyf")?;
    let format = i16::from_be_bytes(head[50..52].try_into().unwrap());
    let location = |index: u16| -> Result<usize, OutlineError> {
        let index = usize::from(index);
        match format {
            0 => Ok(usize::from(u16::from_be_bytes(
                loca[index * 2..index * 2 + 2].try_into().unwrap(),
            )) * 2),
            1 => {
                Ok(u32::from_be_bytes(loca[index * 4..index * 4 + 4].try_into().unwrap()) as usize)
            }
            _ => Err(OutlineError::Invalid("unsupported loca format")),
        }
    };
    // Font::parse has already checked every loca interval and table bound.
    let start = location(glyph)?;
    let end = location(glyph + 1)?;
    parse_packet(&glyf[start..end], font.metrics().glyph_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(contours: i16, tail: &[u8]) -> Vec<u8> {
        let mut data = contours.to_be_bytes().to_vec();
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(tail);
        data
    }

    #[test]
    fn repeated_short_coordinates_keep_flags_and_instruction_bytes() {
        // Three on-curve points: repeated positive X/negative Y byte deltas.
        let data = packet(1, &[0, 2, 0, 2, 0x40, 0, 0x1f, 2, 1, 2, 3, 4, 5, 6, 0xaa]);
        let p = parse_packet(&data, 10).unwrap();
        let GlyphKind::Simple(g) = p.kind else {
            panic!("simple packet")
        };
        assert_eq!(g.instructions, [0x40, 0]);
        assert_eq!(g.end_points, [2]);
        assert_eq!(
            g.points
                .iter()
                .map(|p| (p.x, p.y, p.flag))
                .collect::<Vec<_>>(),
            [(1, -4, 0x1f), (3, -9, 0x1f), (6, -15, 0x1f)]
        );
        assert_eq!(p.remainder, [0xaa]);
    }

    #[test]
    fn native_word_accumulation_wraps_without_adding_phantom_points() {
        let data = packet(1, &[0, 1, 0, 0, 0x21, 0x21, 0x7f, 0xff, 0, 1]);
        let p = parse_packet(&data, 10).unwrap();
        let GlyphKind::Simple(g) = p.kind else {
            panic!("simple packet")
        };
        assert_eq!(g.points.len(), 2);
        assert_eq!((g.points[0].x, g.points[1].x), (32767, -32768));
        assert_eq!((g.points[0].y, g.points[1].y), (0, 0));
    }

    #[test]
    fn zero_contours_does_not_interpret_a_truncated_instruction_tail() {
        let data = packet(0, &[0xff]);
        let p = parse_packet(&data, 1).unwrap();
        assert_eq!(p.kind, GlyphKind::ZeroContours);
        assert_eq!(p.remainder, [0xff]);
        assert_eq!(parse_packet(&[], 1).unwrap().kind, GlyphKind::Empty);
    }

    #[test]
    fn composite_preserves_argument_signedness_transform_and_last_instruction_bit() {
        let data = packet(
            -1,
            &[
                0x01, 0x22, 0, 2, 0xff, 3, // First component has instructions bit, but MORE.
                0x00, 0x81, 0, 3, 0x80, 0, 0, 4, // Matched point words, not signed offsets.
                0x40, 0, 0, 1, 0xff, 0xff, 0x20, 0, 0xaa,
            ],
        );
        let p = parse_packet(&data, 4).unwrap();
        let GlyphKind::Composite(g) = p.kind else {
            panic!("composite packet")
        };
        assert_eq!(g.components[0].xy_offset(), Some([-1, 3]));
        assert_eq!(g.components[1].matched_points(), Some([32768, 4]));
        assert_eq!(g.components[1].transform.matrix(), [16384, 1, -1, 8192]);
        assert!(g.instructions.is_empty());
        assert_eq!(p.remainder, [0xaa]);
    }

    #[test]
    fn packet_bounds_reject_repeat_overflow_and_signed_length_overflow() {
        assert_eq!(
            parse_packet(&packet(1, &[0, 0, 0, 0, 8, 1]), 1),
            Err(OutlineError::Invalid("flag repeat exceeds point count"))
        );
        assert_eq!(
            parse_packet(&packet(1, &[0, 0, 0x80, 0]), 1),
            Err(OutlineError::UnsupportedInstructionLength(0x8000))
        );
        assert_eq!(
            parse_packet(&packet(1, &[0x7f, 0xff]), 1),
            Err(OutlineError::UnsupportedPointCount(32768))
        );
        assert!(matches!(
            parse_packet(&[0], 1),
            Err(OutlineError::Truncated(_))
        ));
    }
}
