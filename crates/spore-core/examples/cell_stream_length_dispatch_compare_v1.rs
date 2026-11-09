//! Compare length dispatch against eight retained original-instruction rows.
use spore_core::cell_stream_length_dispatch_v1::{project, Branch};
fn w(b: &[u8], o: usize) -> u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main() {
    let b = std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();
    assert_eq!(b.len(), 256);
    for i in 0..8 {
        let mode = i % 4;
        let field = if mode & 1 == 0 { 1 } else { 3 };
        let length = if mode >= 2 { 5 } else { 0 };
        // The retained fixture callback leaves stream +14 unchanged.
        let x = project(field, length, true).unwrap();
        let trap = match x.branch { Branch::PresentPending => 1, Branch::ZeroReadPending => 2, Branch::ZeroWritePending => 3 };
        let r = &b[i*32..(i+1)*32];
        // Trap stubs return AL=0; this does not model downstream returns.
        assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12)), (0,trap,x.field_14,x.length_calls));
        let fp=w(r,16); assert_eq!(fp & 0xffff, if i>=4 {0x37f} else {0x27f});
        assert_eq!((fp>>16)&0x3840,0); assert_eq!(w(r,20)&!0x3f,0x1f80);
        assert_eq!((w(r,24),w(r,28)),(0xc0dec0de,0xa5a5a5a5));
    }
    println!("matched eight source length-dispatch rows; downstream operations trapped");
}

