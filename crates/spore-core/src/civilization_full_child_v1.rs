//! Narrow successful FF07F0/FF32D0 snapshot projection: one child, all 19
//! source descriptor keys, empty counted strings and either zero-count
//! collections or one explicitly admitted wall, ornament or building entry.
//! Scripted storage and cleanup do not establish live allocation or identity.
use crate::{civilization_first_entry_v1, descriptor_frame_v1};

pub const KEYS: [u32; 19] = [
    0x03221494, 0x032214b1, 0x032214b6, 0x032214bc, 0x032214c2,
    0x032214c5, 0x034da69b, 0x034da69c, 0x034da69e, 0x034da69f,
    0x03e44212, 0x0510c373, 0x054ff019, 0x055a554d, 0x055ba598,
    0x055ba59e, 0x0689db9f, 0x0689dba2, 0x06cb88ca,
];
const WIDTHS: [usize; 19] = [12, 4, 4, 4, 4, 4, 4, 4, 4, 4, 16, 4, 4, 1, 4, 4, 4, 4, 4];

#[derive(Debug, PartialEq, Eq)]
pub struct FullChild<'a> {
    pub words: [u16; 14],
    pub fields: Vec<descriptor_frame_v1::Field<'a>>,
    pub wall: Option<[u16; 2]>,
    /// Ornament words in source destination order, including reversed key words.
    pub ornament: Option<[u32; 13]>,
    pub building: Option<Building>,
    pub parent: civilization_first_entry_v1::Entry<'a>,
}

/// Logical fields copied by FF14A0. Destination padding is not modeled.
#[derive(Debug, PartialEq, Eq)]
pub struct Building {
    pub float_bits: [u32; 8],
    pub flags: [bool; 2],
    pub values: [u32; 5],
}

/// `scratch` owns the reconstructed two-zero-count parent prefix.
pub fn read<'a>(bytes: &'a [u8], scratch: &'a mut Vec<u8>, max_bytes: usize,
                max_keys: usize) -> Result<FullChild<'a>, &'static str> {
    read_inner(bytes, scratch, max_bytes, max_keys, 0)
}

/// Admits exactly one wall containing the two source descriptor keys.
pub fn read_one_wall<'a>(bytes: &'a [u8], scratch: &'a mut Vec<u8>, max_bytes: usize,
                         max_keys: usize) -> Result<FullChild<'a>, &'static str> {
    read_inner(bytes, scratch, max_bytes, max_keys, 1)
}

/// Admits exactly one ornament with six source descriptor keys.
pub fn read_one_ornament<'a>(bytes: &'a [u8], scratch: &'a mut Vec<u8>, max_bytes: usize,
                             max_keys: usize) -> Result<FullChild<'a>, &'static str> {
    read_inner(bytes, scratch, max_bytes, max_keys, 2)
}

/// Admits one building and finite normal-or-zero words for its x87 copies.
pub fn read_one_building<'a>(bytes: &'a [u8], scratch: &'a mut Vec<u8>, max_bytes: usize,
                             max_keys: usize) -> Result<FullChild<'a>, &'static str> {
    read_inner(bytes, scratch, max_bytes, max_keys, 3)
}

fn normal_or_zero(bits: u32) -> bool {
    let magnitude = bits & 0x7fff_ffff;
    magnitude == 0 || (magnitude >= 0x0080_0000 && magnitude < 0x7f80_0000)
}

fn read_inner<'a>(bytes: &'a [u8], scratch: &'a mut Vec<u8>, max_bytes: usize,
                  max_keys: usize, mode: u8) -> Result<FullChild<'a>, &'static str> {
    if bytes.len() > max_bytes { return Err("limit"); }
    if bytes.get(..4) != Some(&[0, 0, 0, 1]) { return Err("child count"); }
    let mut words = [0; 14];
    for (i, x) in bytes.get(4..32).ok_or("short words")?.chunks_exact(2).enumerate() {
        words[i] = u16::from_be_bytes([x[0], x[1]]);
    }
    let size = u32::from_be_bytes(bytes.get(36..40).ok_or("short frame")?
        .try_into().unwrap()) as usize;
    let end = 40usize.checked_add(size).ok_or("overflow")?;
    let fields = descriptor_frame_v1::parse(bytes.get(32..end).ok_or("short frame")?,
        0x01a80d26, &KEYS, 19, max_bytes).map_err(|_| "child frame")?;
    let mut wall = None;
    let mut ornament = None;
    let mut building = None;
    for (f, width) in fields.iter().zip(WIDTHS) {
        if f.key == 0x034da69b && mode == 3 {
            if f.bytes.get(..4) != Some(&[0, 0, 0, 1]) { return Err("building count"); }
            let nested = descriptor_frame_v1::parse(&f.bytes[4..], 0x01a80d26,
                &[0x034da7b6, 0x034da7b7, 0x034da7b8, 0x03def5f4, 0x03df010e,
                  0x04bee126, 0x04bee127, 0x04bee12a, 0x05761ca2, 0x05e4ee09],
                10, max_bytes).map_err(|_| "building frame")?;
            if nested.iter().zip([12, 16, 4, 4, 4, 1, 4, 4, 4, 1])
                .any(|(v, n)| v.bytes.len() != n) { return Err("building width"); }
            let word = |b: &[u8]| u32::from_be_bytes(b.try_into().unwrap());
            let mut float_bits = [0u32; 8];
            for (i, x) in nested[0].bytes.chunks_exact(4).enumerate() { float_bits[i] = word(x); }
            for (i, x) in nested[1].bytes.chunks_exact(4).enumerate() { float_bits[i + 3] = word(x); }
            float_bits[7] = word(nested[3].bytes);
            if !float_bits.iter().copied().all(normal_or_zero) { return Err("building float"); }
            building = Some(Building {
                float_bits,
                flags: [nested[5].bytes[0] != 0, nested[9].bytes[0] != 0],
                values: [word(nested[2].bytes), word(nested[4].bytes), word(nested[6].bytes),
                    word(nested[7].bytes), word(nested[8].bytes)],
            });
            continue;
        }
        if f.key == 0x034da69c && mode == 1 {
            if f.bytes.get(..4) != Some(&[0, 0, 0, 1]) { return Err("wall count"); }
            let nested = descriptor_frame_v1::parse(&f.bytes[4..], 0x01a80d26,
                &[0x034da8da, 0x034da8db], 2, max_bytes).map_err(|_| "wall frame")?;
            let word = |b: &[u8]| -> Result<u16, &'static str> {
                Ok(u16::from_be_bytes(b.try_into().map_err(|_| "wall width")?))
            };
            wall = Some([word(nested[0].bytes)?, word(nested[1].bytes)?]);
            continue;
        }
        if f.key == 0x034da69f && mode == 2 {
            if f.bytes.get(..4) != Some(&[0, 0, 0, 1]) { return Err("ornament count"); }
            let nested = descriptor_frame_v1::parse(&f.bytes[4..], 0x01a80d26,
                &[0x04acb9a9, 0x04acb9b7, 0x04acb9ba, 0x04acb9bc, 0x04acb9bf, 0x04acb9c1],
                6, max_bytes).map_err(|_| "ornament frame")?;
            if nested.iter().zip([12, 16, 12, 4, 4, 4]).any(|(v, n)| v.bytes.len() != n) {
                return Err("ornament width");
            }
            let mut out = [0u32; 13];
            let word = |b: &[u8]| u32::from_be_bytes(b.try_into().unwrap());
            for (i, x) in nested[0].bytes.chunks_exact(4).enumerate() { out[i] = word(x); }
            for (i, x) in nested[1].bytes.chunks_exact(4).enumerate() { out[i + 3] = word(x); }
            for (i, x) in nested[2].bytes.chunks_exact(4).enumerate() { out[11 - i] = word(x); }
            out[12] = word(nested[3].bytes);
            out[7] = word(nested[4].bytes);
            out[8] = word(nested[5].bytes);
            ornament = Some(out);
            continue;
        }
        if f.bytes.len() != width { return Err("field width"); }
        if matches!(f.key, 0x034da69b | 0x034da69c | 0x034da69f | 0x054ff019 | 0x06cb88ca)
            && f.bytes != [0, 0, 0, 0] { return Err("nonempty variable field"); }
    }
    let after_count = end.checked_add(4).ok_or("overflow")?;
    if bytes.get(end..after_count) != Some(&[0, 0, 0, 0]) { return Err("second count"); }
    let rest = bytes.get(after_count..).ok_or("parent")?;
    scratch.clear();
    scratch.extend_from_slice(&[0; 8]);
    scratch.extend_from_slice(rest);
    let parent = civilization_first_entry_v1::entry(scratch, max_bytes, max_keys)?;
    Ok(FullChild { words, fields, wall, ornament, building, parent })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unadmitted_and_short() {
        let mut scratch = Vec::new();
        assert!(read(&[0; 4], &mut scratch, 100, 16).is_err());
        assert!(read(&[0, 0, 0, 1], &mut scratch, 100, 16).is_err());
        assert!(read(&[0, 0, 0, 1], &mut scratch, 3, 16).is_err());
        assert!(normal_or_zero(0));
        assert!(normal_or_zero(0x8000_0000));
        assert!(normal_or_zero(0x3f80_0000));
        assert!(!normal_or_zero(1));
        assert!(!normal_or_zero(0x7f80_0000));
    }

    #[test]
    fn rejects_wrong_wall_count() {
        let mut bytes = vec![0, 0, 0, 1];
        bytes.extend([0; 28]);
        let total = 4 + 19 * 8 + WIDTHS.iter().sum::<usize>() + 32;
        for word in [0x01a80d26u32, total as u32, 19] { bytes.extend(word.to_be_bytes()); }
        for (key, width) in KEYS.into_iter().zip(WIDTHS) {
            bytes.extend(key.to_be_bytes());
            bytes.extend((if key == 0x034da69c { 36 } else { width } as u32).to_be_bytes());
        }
        for (key, width) in KEYS.into_iter().zip(WIDTHS) {
            bytes.extend(vec![0; if key == 0x034da69c { 36 } else { width }]);
        }
        let mut scratch = Vec::new();
        assert_eq!(read_one_wall(&bytes, &mut scratch, 1000, 16), Err("wall count"));
    }

    #[test]
    fn rejects_wrong_ornament_count() {
        let mut bytes = vec![0, 0, 0, 1];
        bytes.extend([0; 28]);
        let total = 4 + 19 * 8 + WIDTHS.iter().sum::<usize>();
        for word in [0x01a80d26u32, total as u32, 19] { bytes.extend(word.to_be_bytes()); }
        for (key, width) in KEYS.into_iter().zip(WIDTHS) {
            bytes.extend(key.to_be_bytes());
            bytes.extend((width as u32).to_be_bytes());
        }
        for width in WIDTHS { bytes.extend(vec![0; width]); }
        let mut scratch = Vec::new();
        assert_eq!(read_one_ornament(&bytes, &mut scratch, 1000, 16), Err("ornament count"));
    }
}
