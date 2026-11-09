//! Compare bounded synthetic tables with retained original lookup rows.
use spore_core::cell_effect_map_lookup_v1::{project,Node,Table};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),144);
 for i in 0..12 {let mode=i%6;let id=if mode==5{0x5a314eb9}else{0x5a314ebd};let n0=Node{key:[id,0],value:if mode==2{-1}else{4809},next:None};let n1=Node{key:[0x5a314ebd,0],value:4810,next:None};let mut a=Table{buckets:vec![None],nodes:vec![n0]};let mut c=Table{buckets:vec![None],nodes:vec![n1]};
 if mode==1||mode==2||mode==5{a.buckets[0]=Some(0)}if mode==2||mode==4{c.buckets[0]=Some(0)}if mode==3{a.nodes.push(Node{key:[0xdeadbeef,0],value:7,next:Some(0)});a.buckets[0]=Some(1)}
 let tables=[if mode==4{None}else{Some(a)},Some(c)];let value=project(&tables,[id,0],true).unwrap().unwrap_or(u32::MAX);let o=i*12;assert_eq!(w(&b,o),value);let fp=w(&b,o+4);assert_eq!(fp&0xffff,if i<6{0x27f}else{0x37f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+8)&!0x3f,0x1f80);
 }println!("matched twelve original synthetic map lookup rows; no live provider claim");}
