//! Compare a full pinned export slice with retained synthetic-map source rows.
use spore_core::cell_effect_map_insert_v1::insert;
use spore_core::cell_effect_map_lookup_v1::{project,Table};
fn le(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn be(b:&[u8],o:usize)->u32{u32::from_be_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let mut args=std::env::args().skip(1);let out=std::fs::read(args.next().expect("native rows")).unwrap();let records=std::fs::read(args.next().expect("admitted pinned export slice")).unwrap();assert_eq!(out.len(),144);assert_eq!(records.len(),3049*12+4);assert_eq!(be(&records,3049*12),u32::MAX);
 let mut table=Table{buckets:vec![None;4096],nodes:Vec::new()};for i in 0..3049{let o=i*12;let index=be(&records,o);let group=be(&records,o+4);let id=be(&records,o+8);assert!(index<=i32::MAX as u32);assert!(insert(&mut table,[id,group],index as i32,3049,true).unwrap());}assert_eq!(table.nodes.len(),3049);
 let queries=[0x5a314ebd,0x5a314eb9,0xdeadbeef];for i in 0..6{let value=project(&[Some(table.clone())],[queries[i%3],0],true).unwrap().unwrap_or(u32::MAX);let o=i*24;assert_eq!((le(&out,o),le(&out,o+4),le(&out,o+8),le(&out,o+12)),(value,3049,3049,6098));let fp=le(&out,o+16);assert_eq!(fp&0xffff,if i<3{0x27f}else{0x37f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(le(&out,o+20)&!0x3f,0x1f80);}
 println!("matched six complete-export source rows; synthetic map/allocator and no live provider");}
