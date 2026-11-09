//! Controlled caller-supplied shelf inputs; not an atlas or live identity.
use spore_core::{creature_skin_shelf_append_v1::{Shelf,append},native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),2);let data=fs::read(&a[1]).unwrap();assert_eq!(data.len(),11*32);let mut out=Vec::new();
 for row in data.chunks_exact(32){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();assert!(w[7]<12);let mut groups=Vec::with_capacity(12);groups.resize(w[7] as usize,0);let mut shelf=Shelf{width:w[5],height:w[6],groups};append(&mut shelf,1,[w[2],w[3]],w[4],Policy::Mxcsr1f80).unwrap();for x in [shelf.width,shelf.height,shelf.groups.len() as u32,*shelf.groups.last().unwrap()]{out.extend(x.to_le_bytes());}}
 io::stdout().write_all(&out).unwrap();}
