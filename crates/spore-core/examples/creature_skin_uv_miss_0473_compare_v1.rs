//! Conditional UV-miss fixture; group and existing-vector state are synthetic.
use spore_core::{creature_skin_uv_miss_v1::{project,Group,Ordinal},creature_skin_group_cachehit_v1::CacheEntry,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){
 let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let bytes=fs::read(&a[1]).unwrap();assert_eq!(bytes.len(),528);
 let coords:Vec<[u32;3]>=bytes.chunks_exact(12).map(|x|std::array::from_fn(|i|u32::from_le_bytes(x[i*4..i*4+4].try_into().unwrap()))).collect();
 let positions:Vec<u32>=(0..44).collect();let mut out=Vec::new();
 for i in 0..266usize {
  let old=i>=264;let corner=if old {0} else {i/6};let axis=if old {0} else {(i%6) as u32};
  let uv0=if i==265 {[0x80000000,0]} else {[0x3fc00000,0xbfc00000]};
  let bounds=if i==265 {[0,0x80000000,0x80000000,0]} else {[0xbf000000,0x3f000000,0xbf000000,0x3f000000]};
  let mut uv=Vec::with_capacity(2);uv.push(uv0);let mut meta=Vec::with_capacity(2);meta.push([0x11111111,0x22222222]);
  let mut corners=[0xcccccccc;44];let mut cache=[CacheEntry{group:u32::MAX,uv_index:u32::MAX};44];let mut group=Group{id:55,axis,bounds};
  let ret=project(&positions,&coords,&mut uv,&mut meta,&mut corners,&mut cache,&mut group,corner,if old{Ordinal::Existing(0)}else{Ordinal::Append},Policy::Mxcsr1f80).unwrap();
  let words=[ret,uv[0][0],uv[0][1],uv.get(1).map_or(0,|v|v[0]),uv.get(1).map_or(0,|v|v[1]),meta[0][0],meta[0][1],meta.get(1).map_or(0,|v|v[0]),meta.get(1).map_or(0,|v|v[1]),cache[corner].group,cache[corner].uv_index,corners[corner],group.bounds[0],group.bounds[1],group.bounds[2],group.bounds[3],(uv.len()*8) as u32,(meta.len()*8) as u32];
  for w in words {out.extend(w.to_le_bytes());}
 }
 io::stdout().write_all(&out).unwrap();
}
