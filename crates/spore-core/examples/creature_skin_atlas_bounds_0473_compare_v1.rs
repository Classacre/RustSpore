//! Caller-supplied atlas bounds records; no live identity or full packing claim.
use spore_core::{creature_skin_atlas_bounds_v1::project,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let data=fs::read(&a[1]).unwrap();assert_eq!(data.len(),11*28);let mut out=Vec::new();
 for row in data.chunks_exact(28){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let v=project([w[0],w[1],w[2],w[3]],w[4],w[5],w[6],Policy::Mxcsr1f80).unwrap();for x in v.bounds.into_iter().chain([v.flags,v.area]){out.extend(x.to_le_bytes());}}
 io::stdout().write_all(&out).unwrap();}
