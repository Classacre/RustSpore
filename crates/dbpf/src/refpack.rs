//! RefPack decompression, the codec SPORE uses for compressed DBPF payloads.
//!
//! RefPack is an LZ77 variant with a variable-length command byte. The four
//! command encodings are documented in `docs/dbpf-format.md` section 5 and in
//! `tools/refpack.py`, which this module mirrors.
//!
//! Validated against real game data: 800 coded resources across two packages
//! decompressed to exactly the size declared by the DBPF index, using only the
//! size declared inside the RefPack stream itself. See
//! `tools/validate_refpack.py`.

use std::fmt;

/// The compressed stream carries its own size header, so the caller does not
/// have to supply one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefPackInfo {
    /// Compressed size declared inside the stream, when the stream declares one.
    pub declared_compressed: Option<u32>,
    /// Decompressed size declared inside the stream.
    pub declared_decompressed: Option<u32>,
}

/// A decompressed payload and what was learned while producing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefPackOutput {
    /// The decompressed bytes.
    pub data: Vec<u8>,
    /// How many input bytes were consumed.
    pub consumed: usize,
    /// Size declarations read out of the stream header.
    pub info: RefPackInfo,
}

/// Everything that can go wrong while decompressing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefPackError {
    /// The stream ended in the middle of a command or a literal run.
    #[error("truncated RefPack stream at input offset {at}")]
    Truncated { at: usize },

    /// A back-reference pointed before the start of the output.
    #[error("invalid RefPack copy distance {dist} at output offset {at}")]
    BadDistance { dist: usize, at: usize },

    /// The stream declares no size and the caller supplied none.
    #[error("RefPack stream has no size header and no output size was supplied")]
    NoOutputSize,

    /// The stream declares an output size the caller did not expect.
    #[error("RefPack stream declares {declared} output bytes, caller expected {expected}")]
    SizeMismatch { declared: usize, expected: usize },

    /// The stream ended (end-of-stream marker or input exhausted) before
    /// producing the number of bytes the stream declared.
    #[error("RefPack stream produced {produced} of {declared} declared output bytes")]
    Incomplete { produced: usize, declared: usize },
}

/// Decompress a RefPack stream.
///
/// `out_size` is the expected decompressed size. When `has_header` is true the
/// stream's own declaration is used and `out_size` only acts as a check; when
/// it is false, `out_size` is required.
pub fn decompress(
    data: &[u8],
    out_size: Option<usize>,
    has_header: bool,
) -> Result<RefPackOutput, RefPackError> {
    let mut info = RefPackInfo::default();
    let mut pos = 0usize;

    let limit = if has_header {
        let (next, csize, dsize) = read_header(data)?;
        pos = next;
        info.declared_compressed = csize;
        info.declared_decompressed = Some(dsize);
        let declared = dsize as usize;
        if let Some(expected) = out_size {
            if expected != declared {
                return Err(RefPackError::SizeMismatch { declared, expected });
            }
        }
        declared
    } else {
        out_size.ok_or(RefPackError::NoOutputSize)?
    };

    let mut out: Vec<u8> = Vec::with_capacity(limit);

    while out.len() < limit {
        if pos >= data.len() {
            return Err(RefPackError::Truncated { at: pos });
        }
        let b0 = data[pos];
        pos += 1;

        if b0 & 0x80 == 0 {
            // 2-byte command: 0DDRRRPP DDDDDDDD
            let b1 = *data.get(pos).ok_or(RefPackError::Truncated { at: pos })?;
            pos += 1;
            let n_lit = (b0 & 0x03) as usize;
            pos = copy_literals(data, pos, n_lit, &mut out, limit)?;
            let dist = (((b0 & 0x60) as usize) << 3) + b1 as usize + 1;
            let n_cpy = (((b0 >> 2) & 0x07) as usize) + 3;
            copy_match(&mut out, dist, n_cpy, limit)?;
        } else if b0 & 0x40 == 0 {
            // 3-byte command: 10RRRRRR PPDDDDDD DDDDDDDD
            if pos + 1 >= data.len() {
                return Err(RefPackError::Truncated { at: pos });
            }
            let b1 = data[pos];
            let b2 = data[pos + 1];
            pos += 2;
            let n_lit = (b1 >> 6) as usize;
            pos = copy_literals(data, pos, n_lit, &mut out, limit)?;
            let dist = (((b1 & 0x3F) as usize) << 8) + b2 as usize + 1;
            let n_cpy = (b0 & 0x3F) as usize + 4;
            copy_match(&mut out, dist, n_cpy, limit)?;
        } else if b0 & 0x20 == 0 {
            // 4-byte command: 110DRRPP DDDDDDDD DDDDDDDD RRRRRRRR
            if pos + 2 >= data.len() {
                return Err(RefPackError::Truncated { at: pos });
            }
            let b1 = data[pos];
            let b2 = data[pos + 1];
            let b3 = data[pos + 2];
            pos += 3;
            let n_lit = (b0 & 0x03) as usize;
            pos = copy_literals(data, pos, n_lit, &mut out, limit)?;
            let dist = (((b0 & 0x10) as usize) << 12) + ((b1 as usize) << 8) + b2 as usize + 1;
            let n_cpy = (((b0 & 0x0C) as usize) << 6) + b3 as usize + 5;
            copy_match(&mut out, dist, n_cpy, limit)?;
        } else {
            // 1-byte command: 111PPPPP
            let n_lit = ((b0 & 0x1F) as usize) * 4 + 4;
            if n_lit <= 0x70 {
                pos = copy_literals(data, pos, n_lit, &mut out, limit)?;
            } else {
                // End-of-stream marker: emit the trailing literals, then stop.
                let tail = (b0 & 0x03) as usize;
                pos = copy_literals(data, pos, tail, &mut out, limit)?;
                break;
            }
        }
    }

    // The declared size is the contract this codec is validated against: every
    // real resource decompresses to exactly what its stream declares. An end
    // marker that arrives early must not be reported as success.
    if out.len() != limit {
        return Err(RefPackError::Incomplete {
            produced: out.len(),
            declared: limit,
        });
    }

    Ok(RefPackOutput {
        data: out,
        consumed: pos,
        info,
    })
}

/// Read the optional in-stream size header. Returns `(next_pos, compressed, decompressed)`.
fn read_header(data: &[u8]) -> Result<(usize, Option<u32>, u32), RefPackError> {
    if data.len() < 2 {
        return Err(RefPackError::Truncated { at: 0 });
    }
    let sig = ((data[0] as u16) << 8) | data[1] as u16;
    let mut pos = 2usize;
    let has_compressed_size = sig & 0x0100 != 0;
    let large = sig & 0x8000 != 0;

    let mut compressed = None;
    if has_compressed_size {
        if large {
            let v = read_be_u32(data, pos)?;
            pos += 4;
            compressed = Some(v);
        } else {
            let v = read_be_u24(data, pos)?;
            pos += 3;
            compressed = Some(v);
        }
    }

    let decompressed = if large {
        let v = read_be_u32(data, pos)?;
        pos += 4;
        v
    } else {
        let v = read_be_u24(data, pos)?;
        pos += 3;
        v
    };

    Ok((pos, compressed, decompressed))
}

fn read_be_u32(data: &[u8], pos: usize) -> Result<u32, RefPackError> {
    if pos + 4 > data.len() {
        return Err(RefPackError::Truncated { at: pos });
    }
    Ok(u32::from_be_bytes([
        data[pos],
        data[pos + 1],
        data[pos + 2],
        data[pos + 3],
    ]))
}

fn read_be_u24(data: &[u8], pos: usize) -> Result<u32, RefPackError> {
    if pos + 3 > data.len() {
        return Err(RefPackError::Truncated { at: pos });
    }
    Ok(((data[pos] as u32) << 16) | ((data[pos + 1] as u32) << 8) | data[pos + 2] as u32)
}

fn copy_literals(
    data: &[u8],
    mut pos: usize,
    count: usize,
    out: &mut Vec<u8>,
    limit: usize,
) -> Result<usize, RefPackError> {
    for _ in 0..count {
        if out.len() >= limit {
            return Ok(pos);
        }
        let b = *data.get(pos).ok_or(RefPackError::Truncated { at: pos })?;
        out.push(b);
        pos += 1;
    }
    Ok(pos)
}

/// Copy `count` bytes from `dist` bytes back in the output.
///
/// Overlapping copies are byte-by-byte on purpose: that is how the format
/// encodes runs, so reading and writing one byte at a time is correct.
fn copy_match(out: &mut Vec<u8>, dist: usize, count: usize, limit: usize) -> Result<(), RefPackError> {
    if dist == 0 || dist > out.len() {
        return Err(RefPackError::BadDistance {
            dist,
            at: out.len(),
        });
    }
    for _ in 0..count {
        if out.len() >= limit {
            return Ok(());
        }
        let b = out[out.len() - dist];
        out.push(b);
    }
    Ok(())
}

impl fmt::Display for RefPackInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "compressed={:?} decompressed={:?}",
            self.declared_compressed, self.declared_decompressed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_of_stream_marker_emits_its_trailing_literals() {
        // 0xFF: literal run length = (0x1F * 4) + 4 = 128, which exceeds 0x70, so
        // this is the end-of-stream marker. Its low two bits (3) request 3 more
        // literals, which the marker emits before stopping.
        let data = [0xFFu8, b'a', b'b', b'c'];
        let out = decompress(&data, Some(3), false).expect("should decode");
        assert_eq!(out.data, b"abc");
    }

    #[test]
    fn end_of_stream_marker_with_no_trailing_literals_produces_nothing() {
        // 0xFC has low two bits 0, so the marker emits no literals.
        let data = [0xFCu8];
        let out = decompress(&data, Some(0), false).expect("should decode");
        assert!(out.data.is_empty());
    }

    #[test]
    fn end_marker_before_declared_size_is_incomplete() {
        // 0xFC ends the stream immediately, but the caller expects 3 bytes.
        // Reporting success here would hide a truncated resource.
        let data = [0xFCu8, b'a', b'b', b'c'];
        match decompress(&data, Some(3), false) {
            Err(RefPackError::Incomplete {
                produced: 0,
                declared: 3,
            }) => {}
            other => panic!("expected Incomplete, got {other:?}"),
        }
    }

    #[test]
    fn missing_output_size_without_header_is_an_error() {
        let data = [0xFCu8];
        assert_eq!(
            decompress(&data, None, false),
            Err(RefPackError::NoOutputSize)
        );
    }

    #[test]
    fn distance_beyond_output_start_is_rejected() {
        // 0x00 is a 2-byte command with 0 literals and a copy that points back
        // before the start of the output, which is invalid as the first command.
        let data = [0x00u8, 0x00];
        match decompress(&data, Some(4), false) {
            Err(RefPackError::BadDistance { .. }) => {}
            other => panic!("expected BadDistance, got {other:?}"),
        }
    }

    #[test]
    fn declared_size_mismatch_is_reported() {
        // Build a stream with a small header: signature 0x0000 means no
        // compressed size and a 3-byte decompressed size, then a stop marker.
        let mut data = vec![0x00u8, 0x00];
        data.extend_from_slice(&[0x00, 0x00, 0x04]); // declares 4 bytes
        data.push(0xFC); // end of stream, 3 trailing literals expected
        data.extend_from_slice(b"abcd");
        match decompress(&data, Some(99), true) {
            Err(RefPackError::SizeMismatch { declared, expected }) => {
                assert_eq!(declared, 4);
                assert_eq!(expected, 99);
            }
            other => panic!("expected SizeMismatch, got {other:?}"),
        }
    }
}
