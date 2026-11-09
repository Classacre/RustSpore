//! Conditional fixture; sqrt callback assumes compatible binary32 CRT spills.
use spore_core::{creature_skin_uv_tangent_final_v1::project,creature_skin_normal_v1::SqrtSpill,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
struct Math;impl SqrtSpill for Math{fn sqrt_spill(&mut self,x:f32)->Option<f32>{Some(x.sqrt())}}
fn words(v:&[u8])->Vec<u32>{v.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),5);let t=fs::read(&a[1]).unwrap();let n=fs::read(&a[2]).unwrap();let ix=fs::read(&a[3]).unwrap();let ni=fs::read(&a[4]).unwrap();assert_eq!((t.len(),n.len(),ix.len(),ni.len()),(85*12,44*12,85*4,252*4));let tw=words(&t);let nw=words(&n);let indices=words(&ix);let normal_indices=words(&ni);let mut out=Vec::new();for (i,k) in indices.into_iter().enumerate(){let k=k as usize;let id=*normal_indices.get(k).unwrap() as usize;assert!(id<44);let q=project(tw[i*3..i*3+3].try_into().unwrap(),nw[id*3..id*3+3].try_into().unwrap(),Policy::Mxcsr1f80,&mut Math).unwrap();for v in q{for w in v{out.extend(w.to_le_bytes());}}}io::stdout().write_all(&out).unwrap();}
