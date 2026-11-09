//! Compare earlier B/C producer to four retained original rows.
use spore_core::cell_camera_earlier_vector_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native stdout")).unwrap();assert_eq!(b.len(),192);
 let state_b=[10f32.to_bits(),20f32.to_bits(),30f32.to_bits()];let state_c=[3f32.to_bits(),4f32.to_bits(),5f32.to_bits()];let actor=[2f32.to_bits(),4f32.to_bits()];
 for (i,a) in [0f32,0.5,0.75,1.].into_iter().enumerate(){let x=project(state_b,state_c,a.to_bits(),if i==0{None}else{Some(actor)},0x027f,0x1f80,true).unwrap();let o=i*48;for j in 0..3{assert_eq!(w(&b,o+j*4),x.b[j],"B {i}/{j}");assert_eq!(w(&b,o+12+j*4),x.c[j],"C {i}/{j}");}assert_eq!(w(&b,o+24),x.ramp_or_alpha);let fp=w(&b,o+28);assert_eq!(fp&0xffff,0x027f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+32)&!0x3f,0x1f80);assert_eq!(w(&b,o+36),a.to_bits());assert_eq!(w(&b,o+40),0xc0dec0de);assert_eq!(w(&b,o+44),0xa5a5a5a5);}
 println!("matched four retained earlier camera-vector rows; actor and state supplied");}
