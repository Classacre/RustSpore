//! Supplied shelf order and sqrt(area), not observed runtime atlas state.
use spore_core::{creature_skin_shelf_select_v1::select,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let data=fs::read(&a[1]).unwrap();assert_eq!(data.len(),400);let mut out=Vec::new();
 for row in data.chunks_exact(200){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let n=w[0] as usize;assert!((1..=12).contains(&n));let bounds:Vec<[u32;4]>=w[2..2+n*4].chunks_exact(4).map(|p|[p[0],p[1],p[2],p[3]]).collect();let o=select(&bounds,w[1],Policy::Mxcsr1f80).unwrap();let mut v=vec![o.shelves.len() as u32];for i in 0..12{v.push(o.remaining.get(i).copied().flatten().unwrap_or(u32::MAX))}for i in 0..12{if let Some(s)=o.shelves.get(i){v.extend([s.groups.len() as u32,s.width,s.height]);for k in 0..12{v.push(s.groups.get(k).copied().unwrap_or(u32::MAX))}}else{v.extend([u32::MAX;15])}}assert_eq!(v.len(),193);for x in v{out.extend(x.to_le_bytes());}}
 io::stdout().write_all(&out).unwrap();}
