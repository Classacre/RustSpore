//! Compare the bounded source startup bounds against four retained native rows.
use spore_core::{cell_start_bounds_v1::project, native_sse::Policy};
fn word(bytes: &[u8], offset: usize) -> u32 { u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap()) }
fn main() {
    let bytes = std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();
    assert_eq!(bytes.len(), 192);
    for i in 0..4 {
        let value = if i % 2 == 0 { 0 } else { 15f32.to_bits() };
        let expected = project(value, Policy::Mxcsr1f80, true).unwrap();
        for (j, x) in expected.iter().enumerate() { assert_eq!(word(&bytes, i*48+j*4), *x); }
        let fp = word(&bytes, i*48+40);
        assert_eq!(fp & 0xffff, if i < 2 { 0x27f } else { 0x37f });
        assert_eq!((fp >> 16) & 0x3840, 0);
        assert_eq!(word(&bytes, i*48+44) & !0x3f, 0x1f80);
    }
    println!("matched four startup bounds rows; synthetic resolver and supplied +68");
}
