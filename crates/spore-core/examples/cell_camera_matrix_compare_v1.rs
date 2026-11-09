//! Compare bounded matrix projection to three retained original-helper rows.
use spore_core::{cell_camera_handoff_v1::project as handoff,cell_camera_matrix_v1::project};
fn w(b:&[u8],o:usize)->u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main() {
    let b=std::fs::read(std::env::args().nth(1).expect("retained native stdout")).unwrap(); assert_eq!(b.len(),312);
    let cases=[([0x3f800000,0x40000000,0x40400000],[0x40800000,0x40a00000,0x40c00000],[0x40e00000,0x41000000,0x41100000]),
        ([0x60ad78ec;3],[0xe0ad78ec;3],[0x40400000,0x40a00000,0x40e00000]),
        ([0x80000000,0,0x80000000],[0,0x80000000,0x80000000],[0x80000000,0,0x80000000])];
    for (i,(a,v,c)) in cases.into_iter().enumerate() {
        let up=[0,0x3f800000,0]; let h=handoff(a,v,c,up,0x1f80,true).unwrap();
        let m=project(h.second,h.first,up,0x027f,0x1f80,true).unwrap();
        for j in 0..16 { assert_eq!(m[j],w(&b,i*104+40+j*4),"matrix case {i} word {j}"); }
        let fp=w(&b,i*104+24); assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*104+28)&!0x3f,0x1f80);
    }
    println!("matched three retained original matrix-helper rows; camera and setters scripted");
}
