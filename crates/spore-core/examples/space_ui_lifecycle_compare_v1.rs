//! Compare emitted refcount/free-count and FP only; buffer order is host guarded.
use spore_core::space_ui_lifecycle_v1::lifecycle;
fn main(){
 let p=std::env::args_os().nth(1).expect("native stdout path");
 let b=std::fs::read(p).expect("native stdout"); assert_eq!(b.len(),64);
 for (i,row) in b.chunks_exact(32).enumerate(){
  let w:Vec<u32>=row.chunks_exact(4).map(|v|u32::from_le_bytes(v.try_into().unwrap())).collect();
  let m=lifecycle(i==1); assert_eq!(w[0],m.before_source_release); assert_eq!(w[1],m.frees.len() as u32);
  assert_eq!(w[2]&0xffff,0x027f); assert_eq!((w[2]>>16)&0x3840,0); assert_eq!(w[3]&!0x3f,0x1f80); assert!(w[4..].iter().all(|&v|v==0));
 }
 println!("matched two scripted lifecycle rows");
}
