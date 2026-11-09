//! Emitted counters/FP are compared; child and parent values are host guards.
use spore_core::civilization_one_child_v1::read;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let stream = std::fs::read(args.next().expect("synthetic stream")).unwrap();
    let output = std::fs::read(args.next().expect("native stdout")).unwrap();
    assert_eq!(stream.len(), 336);
    let mut scratch = Vec::new();
    let modeled = read(&stream, &mut scratch, 336, 16).unwrap();
    for (i, value) in modeled.words.iter().enumerate() {
        assert_eq!(*value, 0x100 + i as u16);
    }
    assert_eq!(modeled.child_scalar, 77);
    assert_eq!(modeled.parent.words, [11, 22, 33, 44, 55]);
    assert!(modeled.parent.trailing.is_empty());
    assert_eq!(modeled.parent.model_keys.len(), 16);
    for (i, triple) in modeled.parent.model_keys.iter().enumerate() {
        let n = 100 + 3 * i as u32;
        assert_eq!(*triple, [n, n + 1, n + 2]);
    }
    assert_eq!(output.len(), 32);
    let w: Vec<u32> = output.chunks_exact(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap())).collect();
    assert_eq!(&w[..3], &[336, 91, 10]);
    assert_eq!(w[3] & 0xffff, 0x027f);
    assert_eq!((w[3] >> 16) & 0x3840, 0);
    assert_eq!(w[4] & !0x3f, 0x1f80);
    assert!(w[5..].iter().all(|&x| x == 0));
    println!("matched emitted counters/FP; modeled one-child stream");
}
