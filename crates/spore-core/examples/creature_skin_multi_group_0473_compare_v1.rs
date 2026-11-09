//! Caller-supplied fixture; conditional inspection, not live creature state.
use spore_core::{creature_skin_multi_group_v1::build,native_sse::Policy};
use std::{env,fs,io::{self,Write}};
fn words(p:&str)->Vec<u32>{let b=fs::read(p).unwrap();assert_eq!(b.len()%4,0);b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),5);let p=words(&a[1]);let idx=words(&a[2]);let links=words(&a[3]);let ori=words(&a[4]);assert_eq!((p.len(),idx.len(),links.len(),ori.len()),(132,252,252,84));let coords:Vec<[u32;3]>=p.chunks_exact(3).map(|x|[x[0],x[1],x[2]]).collect();let o=build(&idx,&links,&ori,&coords,4,Policy::Mxcsr1f80).unwrap();
 let mut w=vec![o.groups.len() as u32,o.uv.len() as u32,o.cache.len() as u32,o.corners.len() as u32];
 for g in &o.groups{w.extend([g.group.id,g.seed,g.group.axis,g.uv_end,g.faces.len() as u32]);w.extend(g.group.bounds);w.extend(&g.faces);}
 for pair in &o.uv{w.extend(pair)}for pair in &o.metadata{w.extend(pair)}for c in &o.cache{w.extend([c.group,c.uv_index])}w.extend(&o.corners);w.extend(&o.states);
 let mut out=Vec::with_capacity(w.len()*4);for v in w{out.extend(v.to_le_bytes())}io::stdout().write_all(&out).unwrap();}
