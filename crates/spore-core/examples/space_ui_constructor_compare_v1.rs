//! Compare emitted counts and FP state from the one retained guarded host.
use spore_core::space_ui_constructor_v1::initialize;

fn main() {
    let path = std::env::args_os().nth(1).expect("native stdout path");
    let bytes = std::fs::read(path).expect("retained native stdout");
    assert_eq!(bytes.len(), 32);
    let words: Vec<u32> = bytes.chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap())).collect();
    let first: Vec<_> = "Tooltips\0".encode_utf16().collect();
    let locale: Vec<_> = "All\0".encode_utf16().collect();
    let projection = initialize([&first, &locale, &[0]], 16, 0x03754e6c,
                                [0, 0xc2480000]).unwrap();
    assert_eq!(projection.capacities, [9, 4, 1]);
    assert_eq!(projection.strings.iter().map(Vec::len).collect::<Vec<_>>(), [8, 3, 0]);
    assert_eq!(words[..2], [3, 3]);
    assert_eq!(words[2] & 0xffff, 0x027f);
    assert_eq!((words[2] >> 16) & 0x3840, 0);
    assert_eq!(words[3] & !0x3f, 0x1f80);
    assert!(words[4..].iter().all(|&word| word == 0));
    println!("matched retained reserve/copy counts and FP framing");
}
