//! Compare checked vector handoff with three retained scripted original rows.
use spore_core::cell_camera_handoff_v1::project;
fn word(b: &[u8], o: usize) -> u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main() {
    let b=std::fs::read(std::env::args().nth(1).expect("retained native stdout")).unwrap();
    assert_eq!(b.len(),120);
    let cases=[
        ([0x3f800000,0x40000000,0x40400000],[0x40800000,0x40a00000,0x40c00000],[0x40e00000,0x41000000,0x41100000]),
        ([0x60ad78ec;3],[0xe0ad78ec;3],[0x40400000,0x40a00000,0x40e00000]),
        ([0x80000000,0,0x80000000],[0,0x80000000,0x80000000],[0x80000000,0,0x80000000]),
    ];
    for (i,(a,bits,c)) in cases.into_iter().enumerate() {
        let x=project(a,bits,c,[0,0x3f800000,0],0x1f80,true).unwrap();
        for j in 0..3 { assert_eq!(x.first[j],word(&b,i*40+j*4),"first case {i}/{j}"); assert_eq!(x.second[j],word(&b,i*40+12+j*4),"second case {i}/{j}"); }
        let fp=word(&b,i*40+24); assert_eq!(fp&0xffff,0x27f); assert_eq!((fp>>16)&0x3840,0);
        assert_eq!(word(&b,i*40+28)&!0x3f,0x1f80); assert_eq!(word(&b,i*40+32),6); assert_eq!(word(&b,i*40+36),0xc0dec0de);
    }
    println!("matched three retained camera vector handoffs; helper, setters and camera are scripted");
}
