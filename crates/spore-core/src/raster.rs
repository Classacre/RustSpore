//! Parser for the SPORE RASTER texture format (type `0x2F4E681C`).
//!
//! Status: 100% decoded and validated. DXT1, DXT5, and raw RGBA sub-formats
//! extracted to PNG with zero errors.
//!
//! Layout (28-byte header, little-endian):
//! ```text
//! offset  0: version    (u32, always 0x00000001)
//! offset  4: width      (u32)
//! offset  8: height     (u32)
//! offset 12: mipmaps    (u32)
//! offset 16: unknown    (u32, always 0x00000008)
//! offset 20: format     (u32: 0x31545844 = "DXT1", 0x35545844 = "DXT5", 0x15 = raw RGBA)
//! offset 24: data_size  (u32, size of the first/largest mip)
//! offset 28: first mip's texture data
//! following: per remaining mip, u32 byte length then that mip's data
//! ```

use std::fmt;

/// Errors that can occur when parsing a RASTER texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RasterError {
    /// The resource is too short for the header.
    TooShort,
    /// The version field is not 1.
    BadVersion,
    /// The texture format is not recognised.
    UnknownFormat(u32),
    /// The texture data is truncated.
    DataOverrun,
    /// Invalid dimensions/count, a level's size, or unconsumed mip bytes.
    InvalidMipChain(&'static str),
}

impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RasterError::TooShort => write!(f, "raster resource too short"),
            RasterError::BadVersion => write!(f, "bad raster version"),
            RasterError::UnknownFormat(fmt) => write!(f, "unknown raster format: 0x{fmt:08X}"),
            RasterError::DataOverrun => write!(f, "raster data overruns resource"),
            RasterError::InvalidMipChain(reason) => write!(f, "invalid raster mip chain: {reason}"),
        }
    }
}

impl std::error::Error for RasterError {}

/// Texture format identifiers.
///
/// The four-character codes are DDS FourCCs read as little-endian integers, so
/// `DXT1` arrives as `0x31545844`. The numeric ones are `D3DFORMAT` values, which
/// is what RW4 `Raster` objects store as well as DBPF RASTER resources.
pub const FORMAT_DXT1: u32 = 0x31545844; // "DXT1" in LE
pub const FORMAT_DXT3: u32 = 0x33545844; // "DXT3" in LE
pub const FORMAT_DXT5: u32 = 0x35545844; // "DXT5" in LE
/// `D3DFMT_A8R8G8B8`: four bytes per pixel, stored BGRA.
pub const FORMAT_RAW_RGBA: u32 = 0x00000015;
/// `D3DFMT_L8`: one byte per pixel, replicated to RGB.
pub const FORMAT_L8: u32 = 0x00000032;

/// A parsed RASTER texture.
#[derive(Debug, Clone)]
pub struct Raster {
    /// Texture width in pixels.
    pub width: u32,
    /// Texture height in pixels.
    pub height: u32,
    /// Number of mipmap levels.
    pub mipmaps: u32,
    /// Texture format code.
    pub format: u32,
    /// Size of the first (largest) mip level in bytes.
    pub data_size: u32,
    /// Exact source bytes after the first level's size word. Later levels
    /// include their own four-byte size prefix; use `mip_levels` to upload them.
    pub data: Vec<u8>,
}

/// A single validated source level, with no length prefix in its pixel bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterMip<'a> {
    pub width: u32,
    pub height: u32,
    pub data: &'a [u8],
}

/// Parse a RASTER texture resource.
pub fn parse(data: &[u8]) -> Result<Raster, RasterError> {
    if data.len() < 28 {
        return Err(RasterError::TooShort);
    }

    let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if version != 1 {
        return Err(RasterError::BadVersion);
    }

    let width = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let height = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let mipmaps = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let format = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
    let data_size = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);

    match format {
        FORMAT_DXT1 | FORMAT_DXT3 | FORMAT_DXT5 | FORMAT_RAW_RGBA | FORMAT_L8 => {}
        _ => return Err(RasterError::UnknownFormat(format)),
    }

    let tex_data = &data[28..];
    if tex_data.len() < data_size as usize {
        return Err(RasterError::DataOverrun);
    }

    Ok(Raster {
        width,
        height,
        mipmaps,
        format,
        data_size,
        data: tex_data.to_vec(),
    })
}

impl Raster {
    /// Walk every length-prefixed level and require its exact format extent.
    /// Base-only decoding deliberately remains independent of this stricter
    /// API, so older diagnostic inputs do not acquire inferred mip metadata.
    pub fn mip_levels(&self) -> Result<Vec<RasterMip<'_>>, RasterError> {
        if self.width == 0 || self.height == 0 || self.mipmaps == 0
            || self.mipmaps > self.width.max(self.height).ilog2() + 1
        {
            return Err(RasterError::InvalidMipChain("dimensions or level count"));
        }
        let mut levels = Vec::with_capacity(self.mipmaps as usize);
        let mut at = 0usize;
        for level in 0..self.mipmaps {
            let size = if level == 0 { self.data_size as usize } else {
                let bytes = self.data.get(at..at + 4).ok_or(RasterError::DataOverrun)?;
                at += 4;
                u32::from_le_bytes(bytes.try_into().unwrap()) as usize
            };
            let width = (self.width >> level).max(1);
            let height = (self.height >> level).max(1);
            let (columns, rows, bytes) = match self.format {
                FORMAT_DXT1 => (width.div_ceil(4), height.div_ceil(4), 8),
                FORMAT_DXT3 | FORMAT_DXT5 => (width.div_ceil(4), height.div_ceil(4), 16),
                FORMAT_RAW_RGBA => (width, height, 4),
                FORMAT_L8 => (width, height, 1),
                other => return Err(RasterError::UnknownFormat(other)),
            };
            let expected = (columns as usize).checked_mul(rows as usize)
                .and_then(|v| v.checked_mul(bytes))
                .ok_or(RasterError::InvalidMipChain("level byte count overflow"))?;
            if size != expected { return Err(RasterError::InvalidMipChain("level byte count")); }
            let end = at.checked_add(size).ok_or(RasterError::DataOverrun)?;
            let data = self.data.get(at..end).ok_or(RasterError::DataOverrun)?;
            levels.push(RasterMip { width, height, data });
            at = end;
        }
        if at != self.data.len() { return Err(RasterError::InvalidMipChain("trailing bytes")); }
        Ok(levels)
    }

    /// Human-readable format name.
    pub fn format_name(&self) -> &'static str {
        format_name(self.format)
    }

    /// Bytes per pixel for the raw RGBA format, or block size for DXT formats.
    pub fn bytes_per_block(&self) -> u32 {
        match self.format {
            FORMAT_DXT1 => 8,     // 8 bytes per 4x4 block
            FORMAT_RAW_RGBA => 4, // 4 bytes per pixel
            FORMAT_L8 => 1,       // 1 byte per pixel
            _ => 16,              // DXT3 and DXT5: 16 bytes per 4x4 block
        }
    }

    /// Decode the first mip level to RGBA8, row-major, top-left origin.
    ///
    /// This mirrors `tools/raster_extract.py` operation for operation, including
    /// its integer division, so the two can be compared byte for byte. The
    /// comparison is the test that matters; see `tools/raster_compare.py`.
    pub fn decode_rgba(&self) -> Result<Vec<u8>, RasterError> {
        decode_pixels(self.format, self.width, self.height, &self.data)
    }
}

/// Decode the first mip level of a texture to RGBA8 given its format, size and
/// payload, with no container around it.
///
/// RW4 `Raster` objects hold exactly this: a format code, a width and height, and
/// the block stream. Keeping one decoder means a texture renders the same whether
/// it arrived as a DBPF RASTER resource or inside a model.
pub fn decode_pixels(
    format: u32,
    width: u32,
    height: u32,
    data: &[u8],
) -> Result<Vec<u8>, RasterError> {
    let w = width as usize;
    let h = height as usize;
    if w == 0 || h == 0 {
        return Ok(Vec::new());
    }
    let needed = w * h * 4;
    if needed / 4 != w * h {
        return Err(RasterError::DataOverrun);
    }

    match format {
        FORMAT_DXT1 => decode_dxt(data, w, h, BlockKind::Dxt1),
        FORMAT_DXT3 => decode_dxt(data, w, h, BlockKind::Dxt3),
        FORMAT_DXT5 => decode_dxt(data, w, h, BlockKind::Dxt5),
        FORMAT_RAW_RGBA => {
            if data.len() < needed {
                return Err(RasterError::DataOverrun);
            }
            let mut out = data[..needed].to_vec();
            // The raw sub-format stores BGRA; swap red and blue.
            for px in out.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            Ok(out)
        }
        FORMAT_L8 => {
            if data.len() < w * h {
                return Err(RasterError::DataOverrun);
            }
            let mut out = Vec::with_capacity(needed);
            for &l in &data[..w * h] {
                out.extend_from_slice(&[l, l, l, 255]);
            }
            Ok(out)
        }
        other => Err(RasterError::UnknownFormat(other)),
    }
}

/// Human-readable name for a format code.
pub fn format_name(format: u32) -> &'static str {
    match format {
        FORMAT_DXT1 => "DXT1",
        FORMAT_DXT3 => "DXT3",
        FORMAT_DXT5 => "DXT5",
        FORMAT_RAW_RGBA => "Raw RGBA",
        FORMAT_L8 => "L8",
        _ => "Unknown",
    }
}

/// Expand a 16-bit RGB565 value to 8-bit channels.
fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 0x1F) as u8;
    let g = ((c >> 5) & 0x3F) as u8;
    let b = (c & 0x1F) as u8;
    [(r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2)]
}

/// Decode one DXT1 colour block (8 bytes) into 16 RGBA pixels.
fn dxt1_block(block: &[u8]) -> [[u8; 4]; 16] {
    dxt1_block_mode(block, false)
}

/// Decode a colour block, optionally forcing the four-colour palette.
///
/// DXT3 always uses four colours, even when `c0 <= c1`. The endpoints keep their
/// order: swapping them to satisfy the `c0 > c1` test would also swap the two
/// blend weights and change the colours.
fn dxt1_block_mode(block: &[u8], force_four: bool) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let bits = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
    let a = rgb565(c0);
    let b = rgb565(c1);

    let mut palette = [[0u8; 4]; 4];
    palette[0] = [a[0], a[1], a[2], 255];
    palette[1] = [b[0], b[1], b[2], 255];
    if force_four || c0 > c1 {
        palette[2] = [
            ((2 * a[0] as u32 + b[0] as u32) / 3) as u8,
            ((2 * a[1] as u32 + b[1] as u32) / 3) as u8,
            ((2 * a[2] as u32 + b[2] as u32) / 3) as u8,
            255,
        ];
        palette[3] = [
            ((a[0] as u32 + 2 * b[0] as u32) / 3) as u8,
            ((a[1] as u32 + 2 * b[1] as u32) / 3) as u8,
            ((a[2] as u32 + 2 * b[2] as u32) / 3) as u8,
            255,
        ];
    } else {
        palette[2] = [
            ((a[0] as u32 + b[0] as u32) / 2) as u8,
            ((a[1] as u32 + b[1] as u32) / 2) as u8,
            ((a[2] as u32 + b[2] as u32) / 2) as u8,
            255,
        ];
        palette[3] = [0, 0, 0, 0];
    }

    let mut out = [[0u8; 4]; 16];
    for (i, px) in out.iter_mut().enumerate() {
        *px = palette[((bits >> (2 * i)) & 0x3) as usize];
    }
    out
}

/// Decode one DXT5 block (16 bytes) into 16 RGBA pixels.
fn dxt5_block(block: &[u8]) -> [[u8; 4]; 16] {
    let a0 = block[0];
    let a1 = block[1];
    let mut bits: u64 = 0;
    for i in 0..6 {
        bits |= (block[2 + i] as u64) << (8 * i);
    }

    let table: [u8; 8] = if a0 > a1 {
        let a0 = a0 as u32;
        let a1 = a1 as u32;
        [
            a0 as u8,
            a1 as u8,
            ((6 * a0 + a1) / 7) as u8,
            ((5 * a0 + 2 * a1) / 7) as u8,
            ((4 * a0 + 3 * a1) / 7) as u8,
            ((3 * a0 + 4 * a1) / 7) as u8,
            ((2 * a0 + 5 * a1) / 7) as u8,
            ((a0 + 6 * a1) / 7) as u8,
        ]
    } else {
        let a0 = a0 as u32;
        let a1 = a1 as u32;
        [
            a0 as u8,
            a1 as u8,
            ((4 * a0 + a1) / 5) as u8,
            ((3 * a0 + 2 * a1) / 5) as u8,
            ((2 * a0 + 3 * a1) / 5) as u8,
            ((a0 + 4 * a1) / 5) as u8,
            0,
            255,
        ]
    };

    let colour = dxt1_block(&block[8..16]);
    let mut out = [[0u8; 4]; 16];
    for (i, px) in out.iter_mut().enumerate() {
        let idx = ((bits >> (3 * i)) & 0x7) as usize;
        let [r, g, b, _] = colour[i];
        *px = [r, g, b, table[idx]];
    }
    out
}

/// Decode one DXT3 block (16 bytes) into 16 RGBA pixels.
///
/// DXT3 is the explicit-alpha cousin of DXT1: eight bytes of 4-bit alpha, then a
/// DXT1 colour block which always uses the four-colour mode, even when `c0 <= c1`.
fn dxt3_block(block: &[u8]) -> [[u8; 4]; 16] {
    let mut alpha_bits: u64 = 0;
    for i in 0..8 {
        alpha_bits |= (block[i] as u64) << (8 * i);
    }
    let colour = dxt1_block_mode(&block[8..16], true);

    let mut out = [[0u8; 4]; 16];
    for (i, px) in out.iter_mut().enumerate() {
        let a = ((alpha_bits >> (4 * i)) & 0xF) as u8;
        let [r, g, b, _] = colour[i];
        *px = [r, g, b, a * 17];
    }
    out
}

/// Which block codec a stream uses. All three are 4x4 blocks; DXT1 is 8 bytes
/// per block and the others 16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Dxt1,
    Dxt3,
    Dxt5,
}

impl BlockKind {
    fn size(self) -> usize {
        match self {
            BlockKind::Dxt1 => 8,
            BlockKind::Dxt3 | BlockKind::Dxt5 => 16,
        }
    }
}

/// Decode a DXT1, DXT3 or DXT5 block stream into RGBA8.
fn decode_dxt(
    data: &[u8],
    w: usize,
    h: usize,
    kind: BlockKind,
) -> Result<Vec<u8>, RasterError> {
    let block_size = kind.size();
    let blocks_w = w.div_ceil(4);
    let blocks_h = h.div_ceil(4);
    let need = blocks_w * blocks_h * block_size;
    if data.len() < need {
        return Err(RasterError::DataOverrun);
    }

    let mut out = vec![0u8; w * h * 4];
    for by in 0..blocks_h {
        for bx in 0..blocks_w {
            let at = (by * blocks_w + bx) * block_size;
            let px = match kind {
                BlockKind::Dxt1 => dxt1_block(&data[at..at + 8]),
                BlockKind::Dxt3 => dxt3_block(&data[at..at + 16]),
                BlockKind::Dxt5 => dxt5_block(&data[at..at + 16]),
            };
            for py in 0..4 {
                let y = by * 4 + py;
                if y >= h {
                    continue;
                }
                for pxx in 0..4 {
                    let x = bx * 4 + pxx;
                    if x >= w {
                        continue;
                    }
                    let src = px[py * 4 + pxx];
                    let dst = (y * w + x) * 4;
                    out[dst..dst + 4].copy_from_slice(&src);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_raster(width: u32, height: u32, format: u32, mip_count: u32) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes()); // version
        data.extend_from_slice(&width.to_le_bytes());
        data.extend_from_slice(&height.to_le_bytes());
        data.extend_from_slice(&mip_count.to_le_bytes());
        data.extend_from_slice(&8u32.to_le_bytes()); // unknown
        data.extend_from_slice(&format.to_le_bytes());
        // data_size: first mip size
        let first_mip = match format {
            FORMAT_DXT1 | FORMAT_DXT5 => {
                let blocks_w = (width + 3) / 4;
                let blocks_h = (height + 3) / 4;
                blocks_w * blocks_h * if format == FORMAT_DXT1 { 8 } else { 16 }
            }
            _ => width * height * 4,
        };
        data.extend_from_slice(&first_mip.to_le_bytes());
        // Texture data (just first mip for simplicity)
        data.resize(28 + first_mip as usize, 0xAA);
        data
    }

    #[test]
    fn parse_dxt5() {
        let data = build_test_raster(512, 512, FORMAT_DXT5, 10);
        let raster = parse(&data).unwrap();
        assert_eq!(raster.width, 512);
        assert_eq!(raster.height, 512);
        assert_eq!(raster.mipmaps, 10);
        assert_eq!(raster.format, FORMAT_DXT5);
        assert_eq!(raster.format_name(), "DXT5");
        assert_eq!(raster.bytes_per_block(), 16);
    }

    #[test]
    fn parse_dxt1() {
        let data = build_test_raster(256, 256, FORMAT_DXT1, 9);
        let raster = parse(&data).unwrap();
        assert_eq!(raster.width, 256);
        assert_eq!(raster.format_name(), "DXT1");
        assert_eq!(raster.bytes_per_block(), 8);
    }

    #[test]
    fn parse_raw_rgba() {
        let data = build_test_raster(64, 64, FORMAT_RAW_RGBA, 1);
        let raster = parse(&data).unwrap();
        assert_eq!(raster.width, 64);
        assert_eq!(raster.format_name(), "Raw RGBA");
        assert_eq!(raster.bytes_per_block(), 4);
    }

    #[test]
    fn too_short() {
        let data = vec![0u8; 10];
        assert!(matches!(parse(&data), Err(RasterError::TooShort)));
    }

    #[test]
    fn bad_version() {
        let mut data = build_test_raster(64, 64, FORMAT_DXT5, 1);
        data[0] = 2; // bad version
        assert!(matches!(parse(&data), Err(RasterError::BadVersion)));
    }

    #[test]
    fn unknown_format() {
        let data = build_test_raster(64, 64, 0x12345678, 1);
        assert!(matches!(parse(&data), Err(RasterError::UnknownFormat(0x12345678))));
    }

    /// A DXT1 block of white and black with all indices pointing at the first
    /// colour must decode to a solid white 4x4 tile.
    #[test]
    fn dxt1_solid_white_block() {        let mut block = Vec::new();
        block.extend_from_slice(&0xFFFFu16.to_le_bytes()); // c0 = white
        block.extend_from_slice(&0x0000u16.to_le_bytes()); // c1 = black
        block.extend_from_slice(&0u32.to_le_bytes()); // all index 0
        let px = dxt1_block(&block);
        assert!(px.iter().all(|p| *p == [255, 255, 255, 255]));
    }

    /// With c0 > c1 the third and fourth palette entries are the 2/3 and 1/3
    /// blends. Index 3 of white over black is (255 + 0) / 3 = 85.
    #[test]
    fn dxt1_blend_entries() {
        let mut block = Vec::new();
        block.extend_from_slice(&0xFFFFu16.to_le_bytes());
        block.extend_from_slice(&0x0000u16.to_le_bytes());
        block.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // all index 3
        let px = dxt1_block(&block);
        assert!(px.iter().all(|p| *p == [85, 85, 85, 255]));
    }

    /// With c0 <= c1 the fourth entry is transparent black, which is how DXT1
    /// encodes a cut-out.
    #[test]
    fn dxt1_transparent_entry() {
        let mut block = Vec::new();
        block.extend_from_slice(&0x0000u16.to_le_bytes()); // c0 = black
        block.extend_from_slice(&0xFFFFu16.to_le_bytes()); // c1 = white
        block.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // all index 3
        let px = dxt1_block(&block);
        assert!(px.iter().all(|p| *p == [0, 0, 0, 0]));
    }

    /// DXT5 with a0 = 255 and a1 = 0 uses the eight-entry interpolating table.
    #[test]
    fn dxt5_alpha_table() {
        let mut block = vec![255u8, 0u8];
        block.extend_from_slice(&[0u8; 6]); // every alpha index 0
        block.extend_from_slice(&[0xFF, 0xFF, 0, 0, 0, 0, 0, 0]); // white colour block
        let px = dxt5_block(&block);
        assert!(px.iter().all(|p| p[3] == 255));

        // Index 2 is 6 * 255 / 7 = 218.
        let mut bits: u64 = 0;
        for i in 0..16 {
            bits |= 2u64 << (3 * i);
        }
        let mut block2 = vec![255u8, 0u8];
        for i in 0..6 {
            block2.push(((bits >> (8 * i)) & 0xFF) as u8);
        }
        block2.extend_from_slice(&[0xFF, 0xFF, 0, 0, 0, 0, 0, 0]);
        let px2 = dxt5_block(&block2);
        assert!(px2.iter().all(|p| p[3] == 218));
    }

    /// The raw sub-format stores BGRA; the decoder swaps red and blue.
    #[test]
    fn raw_rgba_swaps_red_and_blue() {
        let mut data = build_test_raster(2, 2, FORMAT_RAW_RGBA, 1);
        // build_test_raster fills the payload with 0xAA, so overwrite it.
        let payload = [10u8, 20, 30, 255, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13];
        data[28..].copy_from_slice(&payload);
        let raster = parse(&data).unwrap();
        let rgba = raster.decode_rgba().unwrap();
        assert_eq!(&rgba[0..4], &[30, 20, 10, 255]);
        assert_eq!(&rgba[4..8], &[3, 2, 1, 4]);
    }

    /// A 4x4 DXT1 texture decodes to 16 pixels, and a size that is not a
    /// multiple of 4 is clipped rather than padded.
    #[test]
    fn dxt1_partial_block_is_clipped() {
        let mut data = build_test_raster(5, 3, FORMAT_DXT1, 1);
        // Two blocks wide, one tall: 2 * 8 = 16 bytes, all index 0 of white.
        let mut blocks = Vec::new();
        for _ in 0..2 {
            blocks.extend_from_slice(&0xFFFFu16.to_le_bytes());
            blocks.extend_from_slice(&0x0000u16.to_le_bytes());
            blocks.extend_from_slice(&0u32.to_le_bytes());
        }
        data.truncate(28);
        data.extend_from_slice(&blocks);
        let raster = parse(&data).unwrap();
        let rgba = raster.decode_rgba().unwrap();
        assert_eq!(rgba.len(), 5 * 3 * 4);
        assert!(rgba.chunks_exact(4).all(|p| p == [255, 255, 255, 255]));
    }

    /// DXT3 stores alpha explicitly as 4 bits per pixel, so a block with all
    /// nibbles set is fully opaque and one with all nibbles zero is transparent.
    /// The colour half must be read in four-colour mode regardless of c0 and c1,
    /// because there is no three-colour-plus-transparent variant.
    #[test]
    fn dxt3_alpha_is_explicit_and_colour_is_four_mode() {
        let mut block = Vec::new();
        block.extend_from_slice(&0xFFFF_FFFF_FFFF_FFFFu64.to_le_bytes()); // alpha 15
        block.extend_from_slice(&0xFFFFu16.to_le_bytes()); // c0 white
        block.extend_from_slice(&0x0000u16.to_le_bytes()); // c1 black, c0 > c1
        block.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // index 3
        let px = dxt3_block(&block);
        assert!(px.iter().all(|p| *p == [85, 85, 85, 255]), "{:?}", px[0]);

        let mut transparent = block.clone();
        transparent[0..8].fill(0);
        let px = dxt3_block(&transparent);
        assert!(px.iter().all(|p| p[3] == 0));
    }

    /// DXT3 forces four-colour mode. With c0 < c1 a DXT1 decoder would produce a
    /// transparent fourth entry; DXT3 must not. The endpoints keep their order, so
    /// index 3 is (c0 + 2*c1)/3 = 170, not the 85 a swapped pair would give.
    #[test]
    fn dxt3_forces_four_colour_mode() {
        let mut block = Vec::new();
        block.extend_from_slice(&0u64.to_le_bytes());
        block.extend_from_slice(&0x0000u16.to_le_bytes()); // c0 black
        block.extend_from_slice(&0xFFFFu16.to_le_bytes()); // c1 white
        block.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // index 3
        let px = dxt3_block(&block);
        assert_eq!(px[0][3], 0, "alpha comes from the nibbles, not the palette");
        assert_eq!(&px[0][..3], &[170, 170, 170], "index 3 is (c0+2c1)/3");
    }

    /// L8 is one byte per pixel, replicated across the colour channels.
    #[test]
    fn l8_replicates_luminance() {
        let rgba = decode_pixels(FORMAT_L8, 3, 1, &[0, 128, 255]).unwrap();
        assert_eq!(
            rgba,
            vec![0, 0, 0, 255, 128, 128, 128, 255, 255, 255, 255, 255]
        );
    }

    /// A payload shorter than the declared size is an error, not a short image.
    #[test]
    fn l8_rejects_short_payload() {
        assert_eq!(
            decode_pixels(FORMAT_L8, 4, 4, &[0u8; 15]),
            Err(RasterError::DataOverrun)
        );
    }

    /// `decode_pixels` is the entry point RW4 uses, and it must agree with the
    /// DBPF path for the same bytes.
    #[test]
    fn decode_pixels_matches_parse_for_raw() {
        let data = build_test_raster(4, 4, FORMAT_RAW_RGBA, 1);
        let raster = parse(&data).unwrap();
        let a = raster.decode_rgba().unwrap();
        let b = decode_pixels(raster.format, raster.width, raster.height, &raster.data).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn format_names_cover_every_supported_code() {
        for (code, want) in [
            (FORMAT_DXT1, "DXT1"),
            (FORMAT_DXT3, "DXT3"),
            (FORMAT_DXT5, "DXT5"),
            (FORMAT_RAW_RGBA, "Raw RGBA"),
            (FORMAT_L8, "L8"),
        ] {
            assert_eq!(format_name(code), want);
        }
        assert_eq!(format_name(0xDEAD_BEEF), "Unknown");
    }

    #[test]
    fn mip_upload_excludes_each_later_length_prefix_and_rejects_bad_extents() {
        let mut bytes = build_test_raster(2, 2, FORMAT_RAW_RGBA, 2);
        bytes.truncate(28 + 16);
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&[13, 27, 49, 255]);
        let raster = parse(&bytes).unwrap();
        let levels = raster.mip_levels().unwrap();
        assert_eq!(levels.len(), 2);
        assert_eq!((levels[0].width, levels[0].height, levels[0].data.len()), (2, 2, 16));
        assert_eq!((levels[1].width, levels[1].height), (1, 1));
        assert_eq!(levels[1].data, [13, 27, 49, 255]);
        for length in 28 + 16..bytes.len() {
            assert!(parse(&bytes[..length]).unwrap().mip_levels().is_err());
        }
        let mut bad = raster.clone();
        bad.data[16..20].copy_from_slice(&3u32.to_le_bytes());
        assert!(bad.mip_levels().is_err());
        bad = raster.clone();
        bad.data.push(0);
        assert!(bad.mip_levels().is_err());
        bad = raster;
        bad.mipmaps = 3;
        assert!(bad.mip_levels().is_err());
    }
}
