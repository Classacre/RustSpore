//! Bounded FF32D0 entry projection with two explicitly zero subcollection counts.
//! Complete supplied snapshots and successful callbacks are admitted; this does
//! not establish original entry execution, runtime ownership or vehicle identity.
//! The five words are in sorted descriptor-key order, excluding mModelKeys.
use crate::civilization_vehicle_selector_v1;
use crate::descriptor_frame_v1;
const CLASS: u32 = 0x01a80d26;
const KEYS: [u32; 6] = [
    0x032218ad, 0x032218d3, 0x03718b19, 0x03718b1a, 0x04acca33, 0x05d6ab6b,
];
#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    pub words: [u32; 5],
    pub model_keys: Vec<[u32; 3]>,
    pub trailing: &'a [u8],
}
pub fn entry(bytes: &[u8], max_bytes: usize, max_keys: usize) -> Result<Entry<'_>, &'static str> {
    if bytes.len() > max_bytes {
        return Err("limit");
    }
    let prefix = bytes.get(..8).ok_or("short prefix")?;
    if prefix.iter().any(|&x| x != 0) {
        return Err("unsupported nested counts");
    }
    let size =
        u32::from_be_bytes(bytes.get(12..16).ok_or("short header")?.try_into().unwrap()) as usize;
    let end = 16usize.checked_add(size).ok_or("overflow")?;
    let frame = bytes.get(8..end).ok_or("short frame")?;
    let fields =
        descriptor_frame_v1::parse(frame, CLASS, &KEYS, 6, max_bytes).map_err(|_| "frame")?;
    let mut words = [0u32; 5];
    let mut at = 0;
    for f in &fields {
        if f.key != 0x04acca33 {
            let b: [u8; 4] = f.bytes.try_into().map_err(|_| "word")?;
            words[at] = u32::from_be_bytes(b);
            at += 1;
        }
    }
    let model_keys =
        civilization_vehicle_selector_v1::decode_model_keys(fields[4].bytes, max_keys)?;
    Ok(Entry {
        words,
        model_keys,
        trailing: &bytes[end..],
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn build() -> Vec<u8> {
        let mut vals = vec![vec![0, 0, 0, 7]; 6];
        let mut keys = Vec::new();
        keys.extend(16u32.to_be_bytes());
        for i in 0..16u32 {
            for j in [i + 200, i + 100, i] {
                keys.extend(j.to_be_bytes())
            }
        }
        vals[4] = keys;
        let size = 4 + 48 + vals.iter().map(Vec::len).sum::<usize>();
        let mut b = vec![0; 8];
        for x in [CLASS, size as u32, 6] {
            b.extend(x.to_be_bytes())
        }
        for (k, v) in KEYS.iter().zip(&vals) {
            b.extend(k.to_be_bytes());
            b.extend((v.len() as u32).to_be_bytes())
        }
        for v in vals {
            b.extend(v)
        }
        b.extend([9, 8]);
        b
    }
    #[test]
    fn fixed_entry_and_selection() {
        let b = build();
        let e = entry(&b, 1024, 16).unwrap();
        assert_eq!(e.model_keys.len(), 16);
        assert_eq!(e.model_keys[5], [5, 105, 205]);
        assert_eq!(e.trailing, [9, 8]);
        let mut keys = [0u32; 16];
        keys[5] = 0x99e92f05;
        let triples: [[u32; 3]; 16] = e.model_keys.try_into().unwrap();
        assert_eq!(
            civilization_vehicle_selector_v1::select(0x018ea1eb, &keys, &triples),
            (5, &[5, 105, 205])
        );
    }
    #[test]
    fn rejects_out_of_scope() {
        let b = build();
        let mut x = b.clone();
        x[3] = 1;
        assert!(entry(&x, 1024, 16).is_err());
        assert!(entry(&b, 1024, 15).is_err());
        assert!(entry(&b[..20], 1024, 16).is_err());
        let mut x = b.clone();
        x[19] ^= 1;
        assert!(entry(&x, 1024, 16).is_err());
        assert!(entry(&b, 10, 16).is_err());
    }
}
