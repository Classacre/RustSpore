//! Compare emitted counters/FP and model the fixed stream; decoded fields are
//! native-host guards, not separately emitted native snapshots.
use spore_core::civilization_first_entry_v1::entry;

fn main() {
    let mut args = std::env::args_os().skip(1);
    let stream = std::fs::read(args.next().expect("synthetic stream path")).unwrap();
    let output = std::fs::read(args.next().expect("native stdout path")).unwrap();
    assert_eq!(stream.len(), 284);
    let model = entry(&stream, 284, 16).unwrap();
    assert!(model.trailing.is_empty());
    assert_eq!(model.words, [11, 22, 33, 44, 55]);
    assert_eq!(model.model_keys.len(), 16);
    for (i, triple) in model.model_keys.iter().enumerate() {
        let n = 100 + 3 * i as u32;
        assert_eq!(*triple, [n, n + 1, n + 2]);
    }
    assert_eq!(output.len(), 32);
    let w: Vec<u32> = output.chunks_exact(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap())).collect();
    assert_eq!(&w[..3], &[284, 71, 8]);
    assert_eq!(w[3] & 0xffff, 0x027f);
    assert_eq!((w[3] >> 16) & 0x3840, 0);
    assert_eq!(w[4] & !0x3f, 0x1f80);
    assert!(w[5..].iter().all(|&x| x == 0));
    println!("matched emitted counters/FP; modeled fixed stream");
}
