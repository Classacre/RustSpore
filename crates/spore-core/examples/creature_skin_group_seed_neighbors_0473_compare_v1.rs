//! Conditional retained input words with synthetic initial group/face state.
use spore_core::creature_skin_group_seed_neighbors_v1::expand;
use std::{env,fs,io::{self,Write}};
fn words(path:&str)->Vec<u32>{let b=fs::read(path).unwrap();assert_eq!(b.len()%4,0);b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),4);let indices=words(&a[1]);let links=words(&a[2]);let orientation=words(&a[3]);assert_eq!((indices.len(),links.len(),orientation.len()),(252,252,84));
 let mut out=Vec::new();for case in 0..86usize{let face=if case<84{case}else{0};let mut state=vec![0u32;84];state[face]=u32::MAX;if case==84{state[7]=u32::MAX}let mut data=indices.clone();if case==85{data[22]^=1}let r=expand(&data,&links,&orientation,&mut state,face as u32,0).unwrap();assert!(r.queue.len()<=3);for w in [r.queue.len() as u32,r.orientation_calls,r.edge_calls]{out.extend(w.to_le_bytes())}let mut queue=[u32::MAX;6];for (i,pair) in r.queue.iter().enumerate(){queue[i*2]=pair[0];queue[i*2+1]=pair[1]}for w in queue{out.extend(w.to_le_bytes())}for w in state{out.extend(w.to_le_bytes())}}
 io::stdout().write_all(&out).unwrap();}
