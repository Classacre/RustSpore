//! Compare bounded insert/lookup with retained original source rows.
use spore_core::cell_effect_map_insert_v1::insert;
use spore_core::cell_effect_map_lookup_v1::{project,Table};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),120);
 for i in 0..6{let mut t=Table{buckets:vec![None],nodes:vec![]};assert!(insert(&mut t,[0x5a314ebd,0],4809,2,true).unwrap());assert!(insert(&mut t,[0x5a314eb9,0],4810,2,true).unwrap());let keys=[0x5a314ebd,0x5a314eb9,0xdeadbeef];let result=project(&[Some(t)],[keys[i%3],0],true).unwrap().unwrap_or(u32::MAX);let o=i*20;assert_eq!((w(&b,o),w(&b,o+4),w(&b,o+8)),(result,2,2));let fp=w(&b,o+12);assert_eq!(fp&0xffff,if i<3{0x27f}else{0x37f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+16)&!0x3f,0x1f80);}
 println!("matched six source insertion/lookup rows; allocation and provider synthetic");}
