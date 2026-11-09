//! Bounded original CellSaveGame descriptor-order lower-bound projection.
/// Source initializer order; synthetic seeded descriptor snapshot, not a live table.
pub const KEYS:[u32;22]=[0xabcd0000,0xabcd0001,0xabcd0011,0xabcd0002,0xabcd0004,0xabcd0005,0x06947316,0x06947317,0x35875139,0xabcd0009,0xabcd0010,0xabcd0012,0xabcd0013,0xabcd0014,0xabcd0015,0xabcd0016,0xabcd0017,0xabcd0018,0xabcd0019,0xabcd0020,0xabcd0021,0xabcd0022];
/// Replays E0E6F0 over the supplied fixed source-order keys. These keys are
/// unsorted; an index is an observed lower-bound result, not proof of a match.
/// Stable synthetic descriptor storage is admitted; stream callbacks, runtime
/// table initialization, mutation, ownership and real file behavior are not.
pub fn lower_bound(key:u32)->(usize,bool){let mut first=0usize;let mut count=KEYS.len();while count>0{let half=count/2;let mid=first+half;if KEYS[mid]<key{first=mid+1;count-=half+1}else{count=half}}(first,KEYS.get(first)==Some(&key))}
/// Replays the fixed initializer's unsigned-key insertion sort, including the
/// PE terminator, followed by the builder's exclusion of that terminator.
/// This models the reviewed synthetic fixture, not CRT dispatch or a live table.
pub fn sorted_fixture_keys() -> [u32; 22] {
    let mut rows = [u32::MAX; 23];
    rows[..KEYS.len()].copy_from_slice(&KEYS);
    for i in 1..rows.len() {
        let value = rows[i];
        let mut j = i;
        while j > 0 && value < rows[j - 1] {
            rows[j] = rows[j - 1];
            j -= 1;
        }
        rows[j] = value;
    }
    debug_assert_eq!(rows[22], u32::MAX);
    rows[..22].try_into().unwrap()
}

/// Search the builder's fixed, sorted fixture. The boolean distinguishes a
/// returned insertion point from a matching key, including end-of-table.
pub fn sorted_fixture_lower_bound(key: u32) -> (usize, bool) {
    let rows = sorted_fixture_keys();
    let mut first = 0;
    let mut count = rows.len();
    while count > 0 {
        let half = count / 2;
        let mid = first + half;
        if rows[mid] < key {
            first = mid + 1;
            count -= half + 1;
        } else {
            count = half;
        }
    }
    (first, rows.get(first) == Some(&key))
}

#[cfg(test)]mod tests{use super::*;#[test]fn source_order_is_not_sorted(){assert!(!KEYS.windows(2).all(|w|w[0]<=w[1]));assert_eq!(lower_bound(0xabcd0001),(1,true));assert_eq!(lower_bound(0xabcd0011),(11,false));}
#[test] fn sorted_fixture_finds_all_keys_and_bounds() {
    let rows = sorted_fixture_keys();
    assert!(rows.windows(2).all(|pair| pair[0] < pair[1]));
    for key in KEYS { let (index, found) = sorted_fixture_lower_bound(key); assert!(found); assert_eq!(rows[index], key); }
    assert_eq!(sorted_fixture_lower_bound(0), (0, false));
    assert_eq!(sorted_fixture_lower_bound(u32::MAX), (22, false));
}}
