//! Conditional frozen rows/caps; the sqrt callback assumes compatible binary32 CRT spills.
use spore_core::{creature_skin_row_group_v1::{project,SqrtSpill},native_sse::Policy};
use std::{env,fs,io::{self,Write}};
struct Math;impl SqrtSpill for Math {fn sqrt_spill(&mut self,x:f32)->Option<f32>{Some(x.sqrt())}}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),3);let rows=fs::read(&a[1]).unwrap();let caps=fs::read(&a[2]).unwrap();assert_eq!(rows.len(),12*140);assert_eq!(caps.len(),64);let caps:Vec<_>=caps.chunks_exact(4).map(|v|u32::from_le_bytes(v.try_into().unwrap())).collect();let mut out=Vec::new();for i in 0..6{let row: &[u8;140]=rows[(i+6)*140..(i+7)*140].try_into().unwrap();let g=project(row,&caps,i as u32,Policy::Mxcsr1f80,&mut Math).unwrap().unwrap();for w in [g.id,0,g.bounds[0],g.bounds[1],g.bounds[2],g.bounds[3],g.flags]{out.extend(w.to_le_bytes());}}io::stdout().write_all(&out).unwrap();}
