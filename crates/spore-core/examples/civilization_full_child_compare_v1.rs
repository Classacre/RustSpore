//! Compare emitted counters/FP; child values and cleanup order are host guards.
use spore_core::civilization_full_child_v1::read;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let stream = std::fs::read(args.next().expect("synthetic stream")).unwrap();
    let output = std::fs::read(args.next().expect("native stdout")).unwrap();
    assert_eq!(stream.len(), 569);
    let mut scratch = Vec::new();
    let model = read(&stream, &mut scratch, 569, 16).unwrap();
    for (i, value) in model.words.iter().enumerate() { assert_eq!(*value, 0x100 + i as u16); }
    let offsets = [0usize, 0x1c, 0x20, 0x24, 0x2c, 0x30, 0x7c, 0xa4, 0xb8,
        0x90, 0x0c, 0x34, 0x40, 0x28, 0x38, 0x3c, 0xbc, 0xc0, 0x50];
    for (i, f) in model.fields.iter().enumerate() {
        let expected: Vec<u8> = match i {
            0 => [1u32, 2, 3].into_iter().flat_map(u32::to_be_bytes).collect(),
            10 => [4u32, 5, 6, 7].into_iter().flat_map(u32::to_be_bytes).collect(),
            6 | 7 | 9 | 11 | 12 | 18 => vec![0; 4],
            13 => vec![0x5a],
            1 => 77u32.to_be_bytes().to_vec(),
            _ => (1000 + offsets[i] as u32).to_be_bytes().to_vec(),
        };
        assert_eq!(f.bytes, expected);
    }
    assert_eq!(model.parent.words, [11, 22, 33, 44, 55]);
    assert!(model.parent.trailing.is_empty());
    assert_eq!(model.parent.model_keys.len(), 16);
    for (i, triple) in model.parent.model_keys.iter().enumerate() {
        let n = 100 + 3 * i as u32;
        assert_eq!(*triple, [n, n + 1, n + 2]);
    }
    assert_eq!(output.len(), 32);
    let w: Vec<u32> = output.chunks_exact(4).map(|x| u32::from_le_bytes(x.try_into().unwrap())).collect();
    assert_eq!(&w[..3], &[569, 145, 31]);
    assert_eq!(w[3] & 0xffff, 0x027f);
    assert_eq!((w[3] >> 16) & 0x3840, 0);
    assert_eq!(w[4] & !0x3f, 0x1f80);
    assert_eq!(&w[5..], &[3, 2, 0]);
    println!("matched emitted counters/FP; modeled fixed 19-field stream");
}
