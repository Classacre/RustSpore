//! Controlled supplied vertical bounds; no runtime group-order assertion.
use spore_core::{creature_skin_atlas_height_sort_v1::sort,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let data=fs::read(&a[1]).unwrap();assert_eq!(data.len(),400);let mut out=Vec::new();
 for row in data.chunks_exact(100){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let n=w[0] as usize;assert!((2..=12).contains(&n));let bounds:Vec<[u32;2]>=w[1..1+n*2].chunks_exact(2).map(|p|[p[0],p[1]]).collect();let order=sort(&bounds,Policy::Mxcsr1f80).unwrap();for i in 0..12{out.extend(order.get(i).copied().unwrap_or(u32::MAX).to_le_bytes());}}
 io::stdout().write_all(&out).unwrap();}
