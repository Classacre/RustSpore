//! Caller-supplied UV transform inputs; no actual atlas placement assertion.
use spore_core::{creature_skin_uv_transform_v1::transform,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let data=fs::read(&a[1]).unwrap();assert_eq!(data.len(),88*32);let mut out=Vec::new();
 for row in data.chunks_exact(32){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let v=transform([w[0],w[1]],[w[2],w[3]],w[4],[w[5],w[6]],w[7],Policy::Mxcsr1f80).unwrap();for x in v{out.extend(x.to_le_bytes());}}
 io::stdout().write_all(&out).unwrap();}
