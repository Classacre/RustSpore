//! Conditional averaged geometry and synthetic control faces; not live identity.
use spore_core::{creature_skin_face_orientation_v1::classify,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){
 let a:Vec<_>=env::args().collect();assert_eq!(a.len(),3);
 let p=fs::read(&a[1]).unwrap();let i=fs::read(&a[2]).unwrap();assert_eq!(p.len(),636);assert_eq!(i.len(),1044);
 let pos:Vec<[f32;3]>=p.chunks_exact(12).map(|b|std::array::from_fn(|k|f32::from_le_bytes(b[k*4..k*4+4].try_into().unwrap()))).collect();
 let idx:Vec<u32>=i.chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap())).collect();
 let mut out=Vec::new();for face in 0..87 {out.extend(classify(&pos,&idx,face,Policy::Mxcsr1f80).unwrap().to_le_bytes());}
 io::stdout().write_all(&out).unwrap();
}
