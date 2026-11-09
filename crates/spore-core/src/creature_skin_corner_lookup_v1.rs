//! Bounded model of the original 5406B0/540780 corner-cache lookup.
//! Slots contain source key words; high byte FF is the source empty marker.
//! This does not perform the 53F540 field callback or cache mutation.

pub const SLOT_COUNT: usize = 0x8000;

pub fn corner_hash(key: u32) -> u32 {
    let mut h = 0x811c9dc5u32;
    for shift in [16, 8, 0] {
        h = h.wrapping_mul(0x01000193) ^ ((key >> shift) & 0xff);
    }
    h
}

/// Returns the source slot index, or `u32::MAX` after a full-table probe.
/// Requires exactly 32768 source key words and does not enforce key validity.
pub fn lookup(keys: &[u32], key: u32) -> Option<u32> {
    if keys.len() != SLOT_COUNT { return None; }
    let hash = corner_hash(key);
    let mut index = (hash & 0x7fff) as usize;
    let slot = keys[index];
    if slot >> 24 == 0xff || slot == key { return Some(index as u32); }
    let stride = (((hash >> 16) | (hash << 16) | 1) & 0x7fff) as usize;
    let mut count = 1u32;
    loop {
        index = (index + stride) & 0x7fff;
        count += 1;
        let slot = keys[index];
        if slot == key || slot >> 24 == 0xff { return Some(index as u32); }
        if count > 0x8000 { return Some(u32::MAX); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn byte_order_and_ignored_high_byte() {
        assert_eq!(corner_hash(0x00123456), corner_hash(0xab123456));
        assert_ne!(corner_hash(0x00123456), corner_hash(0x00563412));
    }
    #[test] fn empty_hit_and_collision() {
        let mut keys = vec![u32::MAX; SLOT_COUNT];
        let key = 0x00123456;
        let first = lookup(&keys, key).unwrap() as usize;
        keys[first] = key;
        assert_eq!(lookup(&keys, key), Some(first as u32));
        keys[first] = 0x01123456;
        let second = lookup(&keys, key).unwrap() as usize;
        assert_ne!(first, second);
        keys[second] = key;
        assert_eq!(lookup(&keys, key), Some(second as u32));
    }
    #[test] fn full_table_and_length() {
        assert_eq!(lookup(&[], 1), None);
        assert_eq!(lookup(&vec![0; SLOT_COUNT], 1), Some(u32::MAX));
    }
}
