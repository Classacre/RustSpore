//! Conditional retained topology/positions with explicitly synthetic group seed.
use spore_core::{creature_skin_single_group_v1::build,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn words(path:&str)->Vec<u32>{let b=fs::read(path).unwrap();assert_eq!(b.len()%4,0);b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),5);let p=words(&a[1]);let idx=words(&a[2]);let links=words(&a[3]);let ori=words(&a[4]);assert_eq!((p.len(),idx.len(),links.len(),ori.len()),(132,252,252,84));let coords:Vec<[u32;3]>=p.chunks_exact(3).map(|x|[x[0],x[1],x[2]]).collect();let s=build(&idx,&links,&ori,&coords,0,0,Policy::Mxcsr1f80).unwrap();
 assert!(s.uv.len()<=252 && s.metadata.len()<=252 && s.faces.len()<=84 && s.queue.len()<=84);let mut out=Vec::new();
 for w in [s.group.id,s.group.axis,s.uv.len() as u32,s.faces.len() as u32,s.queue.len() as u32,s.cursor as u32,s.next_seed,s.orientation_calls,s.edge_calls]{out.extend(w.to_le_bytes())}
 for i in 0..252{let pair=s.uv.get(i).copied().unwrap_or([0xcccccccc;2]);for w in pair{out.extend(w.to_le_bytes())}}
 for i in 0..252{let pair=s.metadata.get(i).copied().unwrap_or([0xcccccccc;2]);for w in pair{out.extend(w.to_le_bytes())}}
 for c in s.cache{out.extend(c.group.to_le_bytes());out.extend(c.uv_index.to_le_bytes())}
 for w in s.corners{out.extend(w.to_le_bytes())}for w in s.states{out.extend(w.to_le_bytes())}
 for i in 0..84{out.extend(s.faces.get(i).copied().unwrap_or(u32::MAX).to_le_bytes())}
 for i in 0..84{for w in s.queue.get(i).copied().unwrap_or([u32::MAX;2]){out.extend(w.to_le_bytes())}}
 for w in s.group.bounds{out.extend(w.to_le_bytes())}assert_eq!(out.len(),6788);io::stdout().write_all(&out).unwrap();}
