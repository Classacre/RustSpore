//! Measured byte-alpha branch of the source glyph cache's DXT3 page writer.
//!
//! 88A8C0 clears white RGB with zero alpha. 88A400 packs alpha's high four
//! bits without rounding. This module does not rasterize fonts, place glyphs,
//! infer page dimensions, or implement the separate monochrome/effect branches.
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasError {
    InvalidLayout,
    InvalidRectangle,
    UnalignedX,
    ShortDestination,
    ShortSource,
}
impl fmt::Display for AtlasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source DXT3 glyph atlas: {self:?}")
    }
}
impl std::error::Error for AtlasError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dxt3Layout {
    pub side: usize,
    /// Bytes per four-pixel-high compressed block row, including padding.
    pub row_pitch: usize,
}
impl Dxt3Layout {
    fn extent(self) -> Result<(usize, usize), AtlasError> {
        if self.side == 0 || self.side > i32::MAX as usize {
            return Err(AtlasError::InvalidLayout);
        }
        let blocks = self.side.div_ceil(4);
        let row_bytes = blocks.checked_mul(16).ok_or(AtlasError::InvalidLayout)?;
        if self.row_pitch < row_bytes {
            return Err(AtlasError::InvalidLayout);
        }
        let required = (blocks - 1)
            .checked_mul(self.row_pitch)
            .and_then(|v| v.checked_add(row_bytes))
            .ok_or(AtlasError::InvalidLayout)?;
        Ok((blocks, required))
    }
}

/// Clear source format3 without changing row padding or bytes outside the page.
pub fn clear_dxt3(destination: &mut [u8], layout: Dxt3Layout) -> Result<(), AtlasError> {
    let (blocks, required) = layout.extent()?;
    if destination.len() < required {
        return Err(AtlasError::ShortDestination);
    }
    for row in 0..blocks {
        for column in 0..blocks {
            let block = &mut destination[row * layout.row_pitch + column * 16..][..16];
            block[..8].fill(0);
            block[8..12].fill(0xff);
            block[12..].fill(0);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphRectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Copy positive-pitch 8-bit source alpha into a block-aligned X position.
/// A partial group's unused alpha nibbles become zero, as in the source loop.
/// The RGB half of every compressed block and unrelated rows are preserved.
/// Layout/source bounds are checked before the first mutation.
pub fn write_alpha8_dxt3(
    destination: &mut [u8],
    layout: Dxt3Layout,
    rectangle: GlyphRectangle,
    source: &[u8],
    source_pitch: usize,
) -> Result<(), AtlasError> {
    let (_, required) = layout.extent()?;
    if destination.len() < required {
        return Err(AtlasError::ShortDestination);
    }
    let GlyphRectangle {
        x,
        y,
        width,
        height,
    } = rectangle;
    if x % 4 != 0 {
        return Err(AtlasError::UnalignedX);
    }
    if x.checked_add(width).is_none_or(|v| v > layout.side)
        || y.checked_add(height).is_none_or(|v| v > layout.side)
    {
        return Err(AtlasError::InvalidRectangle);
    }
    if width == 0 || height == 0 {
        return Ok(());
    }
    if source_pitch < width
        || (height - 1)
            .checked_mul(source_pitch)
            .and_then(|v| v.checked_add(width))
            .is_none_or(|v| v > source.len())
    {
        return Err(AtlasError::ShortSource);
    }
    for row in 0..height {
        let current_y = y + row;
        let alpha = &source[row * source_pitch..][..width];
        let base = (current_y / 4) * layout.row_pitch + x * 4 + (current_y & 3) * 2;
        for (group, values) in alpha.chunks(4).enumerate() {
            let mut packed = 0u16;
            for (column, value) in values.iter().enumerate() {
                packed |= u16::from(*value >> 4) << (column * 4);
            }
            let offset = base + group * 16;
            destination[offset..offset + 2].copy_from_slice(&packed.to_le_bytes());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn white_clear_preserves_padding_and_partial_blocks() {
        let layout = Dxt3Layout {
            side: 5,
            row_pitch: 48,
        };
        let mut page = [0xa5; 96];
        clear_dxt3(&mut page, layout).unwrap();
        for base in [0, 16, 48, 64] {
            assert_eq!(
                &page[base..base + 16],
                &[0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0]
            );
        }
        assert_eq!(&page[32..48], &[0xa5; 16]);
        assert_eq!(&page[80..], &[0xa5; 16]);
    }
    #[test]
    fn alpha_uses_high_nibble_and_crosses_block_rows_without_rgb_changes() {
        let layout = Dxt3Layout {
            side: 8,
            row_pitch: 32,
        };
        let mut page = [0xa5; 64];
        let source = [
            0x0f, 0x10, 0x7f, 0xff, 0xbf, 99, 0x01, 0x2f, 0x80, 0x90, 0xff,
        ];
        write_alpha8_dxt3(
            &mut page,
            layout,
            GlyphRectangle {
                x: 0,
                y: 3,
                width: 5,
                height: 2,
            },
            &source,
            6,
        )
        .unwrap();
        assert_eq!(&page[6..8], &0xf710_u16.to_le_bytes());
        assert_eq!(&page[22..24], &0x000b_u16.to_le_bytes());
        assert_eq!(&page[32..34], &0x9820_u16.to_le_bytes());
        assert_eq!(&page[48..50], &0x000f_u16.to_le_bytes());
        for base in [0, 16, 32, 48] {
            assert_eq!(&page[base + 8..base + 16], &[0xa5; 8]);
        }
        assert_eq!(&page[..6], &[0xa5; 6]);
    }
    #[test]
    fn invalid_rectangles_are_atomic() {
        let layout = Dxt3Layout {
            side: 8,
            row_pitch: 32,
        };
        let mut page = [0x65; 64];
        assert_eq!(
            write_alpha8_dxt3(
                &mut page,
                layout,
                GlyphRectangle {
                    x: 1,
                    y: 0,
                    width: 1,
                    height: 1
                },
                &[5],
                1
            ),
            Err(AtlasError::UnalignedX)
        );
        assert_eq!(
            write_alpha8_dxt3(
                &mut page,
                layout,
                GlyphRectangle {
                    x: 0,
                    y: 7,
                    width: 5,
                    height: 2
                },
                &[5; 10],
                5
            ),
            Err(AtlasError::InvalidRectangle)
        );
        assert_eq!(
            write_alpha8_dxt3(
                &mut page,
                layout,
                GlyphRectangle {
                    x: 0,
                    y: 0,
                    width: 5,
                    height: 2
                },
                &[5; 9],
                5
            ),
            Err(AtlasError::ShortSource)
        );
        assert_eq!(page, [0x65; 64]);
    }
}
