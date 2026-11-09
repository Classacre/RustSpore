//! Compare bounded close helper/caller with twelve retained original rows.
use spore_core::cell_stream_close_projection_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),384);
 for i in 0..12{let c=i%6;let bl=(c&1) as u8;let f264=(c&2) as u32;let x=project(bl,f264,(0x12345678,0xabcdef01),if f264==0{Some((0x99,0xabcdef01))}else{None},true).unwrap();let r=&b[i*32..(i+1)*32];assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12)),(u32::from(x.returned),x.field_14,x.field_26c,x.backing_calls));let fp=w(r,16);assert_eq!(fp&0xffff,if i>=6{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,20)&!0x3f,0x1f80);assert_eq!((w(r,24),w(r,28)),(0xc0dec0de,0xa5a5a5a5));}
 println!("matched twelve close-helper rows; backing synthetic, BL=1 control");}
