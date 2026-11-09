//! Compare the retained fixed synthetic initializer/sort/builder/search rows.
use spore_core::cell_save_descriptor_order_v1::{sorted_fixture_lower_bound, KEYS};

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn main() {
    let bytes = std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();
    assert_eq!(bytes.len(), 192);
    for row in 0..2 {
        for (ordinal, key) in KEYS.iter().enumerate() {
            let (index, matched) = sorted_fixture_lower_bound(*key);
            assert!(matched);
            assert_eq!(word(&bytes, row * 96 + ordinal * 4), index as u32);
        }
        let fp = word(&bytes, row * 96 + 88);
        assert_eq!(fp & 0xffff, if row == 0 { 0x27f } else { 0x37f });
        assert_eq!((fp >> 16) & 0x3840, 0);
        assert_eq!(word(&bytes, row * 96 + 92) & !0x3f, 0x1f80);
    }
    println!("matched all 22 sorted synthetic descriptor lookups at both control words; live initialization and streams unverified");
}
