//! Conditional retained positions with explicitly supplied synthetic group values.
use spore_core::creature_skin_uv_select_v1::select;
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),3);let p=fs::read(&a[1]).unwrap();let g=fs::read(&a[2]).unwrap();assert_eq!(p.len(),564);assert_eq!(g.len(),32);let groups:Vec<u32>=g.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let mut out=Vec::new();for row in p.chunks_exact(12){let pos=std::array::from_fn(|i|u32::from_le_bytes(row[i*4..i*4+4].try_into().unwrap()));for &group in &groups{for word in select(pos,group){out.extend(word.to_le_bytes());}}}io::stdout().write_all(&out).unwrap();}
