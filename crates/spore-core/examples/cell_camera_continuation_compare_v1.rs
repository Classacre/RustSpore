//! Compare checked continuation against four retained original rows.
use spore_core::cell_camera_continuation_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),192);let old=[2f32.to_bits(),(-3f32).to_bits(),4f32.to_bits()];
 for i in 0..4{let input=match i{2=>0f32,3=>20f32,_=>10f32};let x=project(if i==0{0}else{2},old,input.to_bits(),0x027f,0x1f80,true).unwrap();let o=i*48;assert_eq!(w(&b,o),x.helper_value,"helper case {i}");for j in 0..3{assert_eq!(w(&b,o+4+4*j),x.output[j],"output {i}/{j}");assert_eq!(w(&b,o+20+4*j),x.vector[j],"vector {i}/{j}");}assert_eq!(w(&b,o+16),x.flags);let fp=w(&b,o+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+36)&!0x3f,0x1f80);assert_eq!(w(&b,o+40),0xc0dec0de);assert_eq!(w(&b,o+44),0xa5a5a5a5);}
 println!("matched four retained camera continuation/helper rows; globals supplied");}
