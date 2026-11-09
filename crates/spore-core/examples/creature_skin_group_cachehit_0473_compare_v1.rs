//! Conditional source position indices with an explicitly synthetic cache.
use spore_core::creature_skin_group_cachehit_v1::{append_hit,CacheEntry};
use std::{env,fs,io::{self,Write}};
fn main(){
 let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let b=fs::read(&a[1]).unwrap();assert_eq!(b.len(),1008);
 let positions:Vec<u32>=b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();
 let cache:Vec<_>=(0..44).map(|i|CacheEntry{group:55,uv_index:1000+i}).collect();
 let mut uv=vec![0xcccccccc;252];let mut states=vec![0xaaaaaaaa;84];let mut out=Vec::new();
 for i in 0..252u32 {let mut faces=Vec::new();append_hit(&positions,&cache,&mut uv,&mut states,&mut faces,i/3,i%3,55).unwrap();assert_eq!(faces.len(),1);out.extend(faces[0].to_le_bytes());for v in &uv {out.extend(v.to_le_bytes())}for v in &states {out.extend(v.to_le_bytes())}}
 io::stdout().write_all(&out).unwrap();
}
