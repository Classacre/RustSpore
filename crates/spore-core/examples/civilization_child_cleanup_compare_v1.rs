//! Compare emitted counters/FP; callback order and state are native-host guards.
use spore_core::civilization_one_child_v1::{fixed_cleanup_events, read};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let stream = std::fs::read(args.next().expect("synthetic stream")).unwrap();
    let output = std::fs::read(args.next().expect("native stdout")).unwrap();
    let mut scratch = Vec::new();
    let model = read(&stream, &mut scratch, 336, 16).unwrap();
    assert_eq!(model.child_scalar, 77);
    assert_eq!(fixed_cleanup_events().len(), 5);
    assert_eq!(output.len(), 32);
    let w: Vec<u32> = output.chunks_exact(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap())).collect();
    assert_eq!(&w[..3], &[336, 91, 10]);
    assert_eq!(w[3] & 0xffff, 0x027f);
    assert_eq!((w[3] >> 16) & 0x3840, 0);
    assert_eq!(w[4] & !0x3f, 0x1f80);
    assert_eq!(&w[5..], &[3, 2, 0]);
    println!("matched emitted read/cleanup counters and FP framing");
}
