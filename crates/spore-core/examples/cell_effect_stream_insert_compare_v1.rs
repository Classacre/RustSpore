//! Compare pinned export-record bytes to retained synthetic-stream source rows.
use spore_core::cell_effect_map_insert_v1::insert;
use spore_core::cell_effect_map_lookup_v1::{project,Table};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn be(b:&[u8],o:usize)->u32{u32::from_be_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let mut a=std::env::args().skip(1);let out=std::fs::read(a.next().expect("native rows")).unwrap();let records=std::fs::read(a.next().expect("admitted exact record bytes")).unwrap();assert_eq!((out.len(),records.len()),(96,24));
 for i in 0..4 {let off=(i%2)*12;let index=be(&records,off);let group=be(&records,off+4);let id=be(&records,off+8);assert_eq!((index,group,id),if i%2==0{(4809,0,0x5a314ebd)}else{(4810,0,0x5a314eb9)});let mut t=Table{buckets:vec![None],nodes:vec![]};assert!(insert(&mut t,[id,group],index as i32,1,true).unwrap());let value=project(&[Some(t)],[id,group],true).unwrap().unwrap();let o=i*24;assert_eq!((w(&out,o),w(&out,o+4),w(&out,o+8),w(&out,o+12)),(value,1,1,2));let fp=w(&out,o+16);assert_eq!(fp&0xffff,if i<2{0x27f}else{0x37f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&out,o+20)&!0x3f,0x1f80);}
 println!("matched four original key-read/insertion/lookup rows; synthetic stream and provider");}
