//! Conditional fixture adapter for the checked paired topology model.
use std::{env,fs,io::{self,Write}};use spore_core::creature_skin_paired_topology_v1::build;
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let b=fs::read(&a[1]).unwrap();assert_eq!(b.len(),1008);let indices:Vec<_>=b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();let t=build(&indices,44).unwrap();let mut out=Vec::new();out.extend(0u32.to_le_bytes());for x in t.links.into_iter().chain(t.starts){out.extend(x.to_le_bytes())}io::stdout().write_all(&out).unwrap();}
