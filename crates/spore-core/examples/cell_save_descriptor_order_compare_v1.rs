//! Compare synthetic original initializer/builder/search retained rows.
use spore_core::cell_save_descriptor_order_v1::{lower_bound,KEYS};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),192);for i in 0..2{for (j,key) in KEYS.iter().enumerate(){assert_eq!(w(&b,i*96+j*4),lower_bound(*key).0 as u32)}let fp=w(&b,i*96+88);assert_eq!(fp&0xffff,if i==0{0x27f}else{0x37f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*96+92)&!0x3f,0x1f80);}println!("matched 22 source-order lower-bound indices at both control words; no live table or stream");}
