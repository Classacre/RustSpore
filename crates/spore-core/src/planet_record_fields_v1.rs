//! Bounded observed PlanetRecord name and nested orbit field projections.
//!
//! These parse caller-supplied complete snapshots under the traced descriptor
//! and successful full-read admissions. They do not open files, invoke source
//! callbacks, or establish runtime identity. Float words remain raw bits.
use crate::descriptor_frame_v1::{self, FrameError};

pub const ORBIT_CLASS: u32 = 0x01a80d26;
pub const ORBIT_KEYS: [u32; 5] = [0x0374176b, 0x03741796, 0x03741811, 0x0374181f, 0x0374182d];

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    Frame(FrameError),
    Truncated,
    Length,
    Count,
    NonZeroVector,
}
impl From<FrameError> for DecodeError {
    fn from(e: FrameError) -> Self {
        Self::Frame(e)
    }
}

/// Returns raw UTF-16 code units, including embedded NUL or unpaired surrogates.
/// Original 693190 reads the count as BE and the units in mode 1 (raw LE).
pub fn name_units(bytes: &[u8], max_units: usize) -> Result<Vec<u16>, DecodeError> {
    let header: [u8; 4] = bytes
        .get(..4)
        .ok_or(DecodeError::Truncated)?
        .try_into()
        .unwrap();
    let count = u32::from_be_bytes(header) as usize;
    if count > max_units {
        return Err(DecodeError::Count);
    }
    let end = count
        .checked_mul(2)
        .and_then(|n| n.checked_add(4))
        .ok_or(DecodeError::Length)?;
    if end != bytes.len() {
        return Err(DecodeError::Length);
    }
    Ok(bytes[4..]
        .chunks_exact(2)
        .map(|x| u16::from_le_bytes([x[0], x[1]]))
        .collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orbit {
    pub null_orbit: bool,
    pub plane_normal_bits: [u32; 3],
    pub eccentricity_bits: u32,
    pub perihelion_bits: [u32; 3],
    pub period_bits: u32,
}

/// Decodes the five admitted fields, requiring full successful callback-sized spans.
/// Boolean conversion follows 93A8F0: zero is false, any other byte is true.
pub fn orbit(bytes: &[u8], max_bytes: usize) -> Result<Orbit, DecodeError> {
    let fields = descriptor_frame_v1::parse(bytes, ORBIT_CLASS, &ORBIT_KEYS, 5, max_bytes)?;
    if fields
        .iter()
        .zip([1usize, 12, 4, 12, 4])
        .any(|(f, n)| f.bytes.len() != n)
    {
        return Err(DecodeError::Length);
    }
    fn w(b: &[u8], i: usize) -> u32 {
        u32::from_be_bytes(b[i..i + 4].try_into().unwrap())
    }
    fn triple(b: &[u8]) -> [u32; 3] {
        [w(b, 0), w(b, 4), w(b, 8)]
    }
    Ok(Orbit {
        null_orbit: fields[0].bytes[0] != 0,
        plane_normal_bits: triple(fields[1].bytes),
        eccentricity_bits: w(fields[2].bytes, 0),
        perihelion_bits: triple(fields[3].bytes),
        period_bits: w(fields[4].bytes, 0),
    })
}

/// The source's sorted PlanetRecord descriptor keys; callers must explicitly
/// admit this snapshot and complete, successful callback reads.
pub const PLANET_CLASS: u32 = 0x01a80d26;
pub const PLANET_KEYS: [u32; 23] = [
    0x03d58b99, 0x03d58b9a, 0x03d58b9b, 0x03d58b9c, 0x03d58b9e, 0x03d58b9f, 0x03f6d514, 0x03f6d526,
    0x03f6d6ba, 0x0415fbbe, 0x0415fbcd, 0x0415fbdf, 0x04164f63, 0x04164f8d, 0x04164f8e, 0x04164f9e,
    0x0417a8ca, 0x0417a8e9, 0x0417a902, 0x05c19861, 0x05c1a01b, 0x05c1a027, 0x05c1a028,
];

/// Raw callback results, preserving word bits and source destination order.
#[derive(Debug, PartialEq, Eq)]
pub enum PlanetValue {
    Word(u32),
    Byte(u8),
    Bool(bool),
    Vector3([u32; 3]),
    ResourceKey([u32; 3]),
    Name(Vec<u16>),
    Orbit(Orbit),
    ResourceKeys(Vec<[u32; 3]>),
    Words(Vec<u32>),
}
#[derive(Debug, PartialEq, Eq)]
pub struct PlanetField {
    pub key: u32,
    pub value: PlanetValue,
}

fn exact_word(b: &[u8]) -> Result<u32, DecodeError> {
    if b.len() != 4 {
        return Err(DecodeError::Length);
    }
    Ok(u32::from_be_bytes(b.try_into().unwrap()))
}
fn exact_triple(b: &[u8], reverse: bool) -> Result<[u32; 3], DecodeError> {
    if b.len() != 12 {
        return Err(DecodeError::Length);
    }
    let mut a = [0; 3];
    for (i, chunk) in b.chunks_exact(4).enumerate() {
        a[if reverse { 2 - i } else { i }] = u32::from_be_bytes(chunk.try_into().unwrap());
    }
    Ok(a)
}
fn counted<'a>(b: &'a [u8], width: usize, max: usize) -> Result<&'a [u8], DecodeError> {
    let h: [u8; 4] = b
        .get(..4)
        .ok_or(DecodeError::Truncated)?
        .try_into()
        .unwrap();
    let n = u32::from_be_bytes(h) as usize;
    if n > max {
        return Err(DecodeError::Count);
    }
    if n.checked_mul(width).and_then(|x| x.checked_add(4)) != Some(b.len()) {
        return Err(DecodeError::Length);
    }
    Ok(&b[4..])
}

/// Decode all 23 admitted callbacks from one complete frame. Empty fields and
/// partial reads are outside this narrower model. Vector counts are capped by
/// `max_elements`; the returned values make no live identity claim.
pub fn planet(
    bytes: &[u8],
    max_bytes: usize,
    max_elements: usize,
) -> Result<Vec<PlanetField>, DecodeError> {
    let fields = descriptor_frame_v1::parse(bytes, PLANET_CLASS, &PLANET_KEYS, 23, max_bytes)?;
    let mut out = Vec::with_capacity(23);
    for f in fields {
        let b = f.bytes;
        let value = match f.key {
            0x03d58b9a => PlanetValue::Name(name_units(b, max_elements)?),
            0x03f6d6ba => PlanetValue::Orbit(orbit(b, max_bytes)?),
            0x0417a8ca | 0x05c1a028 => {
                if b.len() != 1 {
                    return Err(DecodeError::Length);
                }
                PlanetValue::Bool(b[0] != 0)
            }
            0x03f6d514 | 0x03f6d526 => {
                if b.len() != 1 {
                    return Err(DecodeError::Length);
                }
                PlanetValue::Byte(b[0])
            }
            0x0417a8e9 => PlanetValue::Vector3(exact_triple(b, false)?),
            0x03d58b9b | 0x03d58b9c => PlanetValue::ResourceKey(exact_triple(b, true)?),
            0x04164f63 | 0x04164f8d => PlanetValue::ResourceKeys(
                counted(b, 12, max_elements)?
                    .chunks_exact(12)
                    .map(|x| exact_triple(x, true))
                    .collect::<Result<_, _>>()?,
            ),
            0x04164f9e => PlanetValue::Words(
                counted(b, 4, max_elements)?
                    .chunks_exact(4)
                    .map(|x| u32::from_be_bytes(x.try_into().unwrap()))
                    .collect(),
            ),
            _ => PlanetValue::Word(exact_word(b)?),
        };
        out.push(PlanetField { key: f.key, value });
    }
    Ok(out)
}

/// A bounded result for the zero-vector prefix path of the source inner stream.
/// The source continues after the frame; remaining bytes are returned untouched.
#[derive(Debug, PartialEq, Eq)]
pub struct InnerRecord<'a> {
    pub fields: Vec<PlanetField>,
    pub trailing: &'a [u8],
}

/// Decode a complete supplied inner resource when all four leading vector
/// counts are zero. Other vector paths, short/error reads, stream ownership and
/// live factory selection are outside this narrower admission.
pub fn inner_zero_vectors(
    bytes: &[u8],
    max_bytes: usize,
    max_elements: usize,
) -> Result<InnerRecord<'_>, DecodeError> {
    if bytes.len() > max_bytes {
        return Err(DecodeError::Length);
    }
    let prefix = bytes.get(..16).ok_or(DecodeError::Truncated)?;
    if prefix.iter().any(|&b| b != 0) {
        return Err(DecodeError::NonZeroVector);
    }
    let len: [u8; 4] = bytes
        .get(20..24)
        .ok_or(DecodeError::Truncated)?
        .try_into()
        .unwrap();
    let end = 24usize
        .checked_add(u32::from_be_bytes(len) as usize)
        .ok_or(DecodeError::Length)?;
    let frame = bytes.get(16..end).ok_or(DecodeError::Truncated)?;
    Ok(InnerRecord {
        fields: planet(frame, max_bytes, max_elements)?,
        trailing: &bytes[end..],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> Vec<u8> {
        let widths = [1u32, 12, 4, 12, 4];
        let size = 4 + 5 * 8 + widths.iter().sum::<u32>();
        let mut b = Vec::new();
        for v in [ORBIT_CLASS, size, 5] {
            b.extend(v.to_be_bytes());
        }
        for (k, n) in ORBIT_KEYS.into_iter().zip(widths) {
            b.extend(k.to_be_bytes());
            b.extend(n.to_be_bytes());
        }
        b.push(2);
        for v in 1u32..=8 {
            b.extend(v.to_be_bytes());
        }
        b
    }
    fn build_frame(class: u32, keys: &[u32], fields: &[Vec<u8>]) -> Vec<u8> {
        let size = 4 + 8 * keys.len() + fields.iter().map(Vec::len).sum::<usize>();
        let mut b = Vec::new();
        for x in [class, size as u32, keys.len() as u32] {
            b.extend(x.to_be_bytes());
        }
        for (key, field) in keys.iter().zip(fields) {
            b.extend(key.to_be_bytes());
            b.extend((field.len() as u32).to_be_bytes());
        }
        for field in fields {
            b.extend(field);
        }
        b
    }
    #[test]
    fn zero_vector_inner_path_and_malformed_inputs() {
        let orbit = frame();
        let fields: Vec<Vec<u8>> = PLANET_KEYS
            .iter()
            .map(|&key| match key {
                0x03d58b9a => vec![0; 4],
                0x03f6d6ba => orbit.clone(),
                0x0417a8ca | 0x05c1a028 | 0x03f6d514 | 0x03f6d526 => vec![1],
                0x0417a8e9 | 0x03d58b9b | 0x03d58b9c => vec![0; 12],
                _ => vec![0; 4],
            })
            .collect();
        let body = build_frame(PLANET_CLASS, &PLANET_KEYS, &fields);
        let mut all = vec![0; 16];
        all.extend(&body);
        all.extend([4, 5]);
        let parsed = inner_zero_vectors(&all, 4096, 100).unwrap();
        assert_eq!(parsed.fields.len(), 23);
        assert_eq!(parsed.trailing, [4, 5]);
        let mut bad = all.clone();
        bad[7] = 1;
        assert_eq!(
            inner_zero_vectors(&bad, 4096, 100),
            Err(DecodeError::NonZeroVector)
        );
        assert_eq!(
            inner_zero_vectors(&all[..23], 4096, 100),
            Err(DecodeError::Truncated)
        );
        assert!(inner_zero_vectors(&all[..all.len() - 3], 4096, 100).is_err());
        assert_eq!(inner_zero_vectors(&all, 16, 100), Err(DecodeError::Length));
    }
    #[test]
    fn counted_vectors_and_source_key_order_are_bounded() {
        let b = [
            0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6,
        ];
        let rows: Vec<_> = counted(&b, 12, 2)
            .unwrap()
            .chunks_exact(12)
            .map(|x| exact_triple(x, true).unwrap())
            .collect();
        assert_eq!(rows, [[3, 2, 1], [6, 5, 4]]);
        assert_eq!(counted(&b, 12, 1), Err(DecodeError::Count));
        assert_eq!(counted(&b[..b.len() - 1], 12, 2), Err(DecodeError::Length));
        assert_eq!(exact_triple(&b[4..16], false).unwrap(), [1, 2, 3]);
    }
    #[test]
    fn name_is_raw_little_endian_units() {
        assert_eq!(
            name_units(&[0, 0, 0, 3, 0x41, 0, 0, 0, 0, 0xd8], 3).unwrap(),
            [0x41, 0, 0xd800]
        );
        assert_eq!(name_units(&[0, 0, 0, 2, 1, 0], 3), Err(DecodeError::Length));
        assert_eq!(name_units(&[0, 0, 0, 3], 2), Err(DecodeError::Count));
        assert_eq!(name_units(&[0, 0], 2), Err(DecodeError::Truncated));
    }
    #[test]
    fn orbit_decodes_bits_and_rejects_bad_frames() {
        let b = frame();
        let o = orbit(&b, 100).unwrap();
        assert!(o.null_orbit);
        assert_eq!(o.plane_normal_bits, [1, 2, 3]);
        assert_eq!(o.eccentricity_bits, 4);
        assert_eq!(o.perihelion_bits, [5, 6, 7]);
        assert_eq!(o.period_bits, 8);
        assert!(orbit(&b, 32).is_err());
        let mut bad = b.clone();
        bad[15] ^= 1;
        assert!(orbit(&bad, 100).is_err());
        let mut bad = b.clone();
        bad[19] = 2;
        assert!(orbit(&bad, 100).is_err());
    }
}
