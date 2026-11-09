//! Controlled FIFO over native-matched seed snapshots; face/UV work is stubbed.
use spore_core::creature_skin_group_queue_consumer_v1::consume;
use std::{env,fs,io::{self,Write}};
fn words(path:&str)->Vec<u32>{let b=fs::read(path).unwrap();assert_eq!(b.len()%4,0);b.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),5);let indices=words(&a[1]);let links=words(&a[2]);let orientations=words(&a[3]);let seeds=words(&a[4]);assert_eq!((indices.len(),links.len(),orientations.len(),seeds.len()),(252,252,84,86*93));let mut out=Vec::new();
 for case in 0..86usize{let row=&seeds[case*93..(case+1)*93];let n=row[0] as usize;assert!(n<=3);let queue:Vec<[u32;2]>=(0..n).map(|i|[row[3+i*2],row[4+i*2]]).collect();let face=if case<84{case}else{0};let mut pos=indices.clone();if case==85{pos[22]^=1}let r=consume(&pos,&links,&orientations,&row[9..93],&queue,0,orientations[face],u32::MAX).unwrap();assert!(r.queue.len()<=84 && r.visited.len()<=84);
 for w in [r.queue.len() as u32,r.cursor as u32,r.next_seed,r.visited.len() as u32,r.orientation_calls,r.edge_calls]{out.extend(w.to_le_bytes())}let mut q=[u32::MAX;168];for (i,p) in r.queue.iter().enumerate(){q[i*2]=p[0];q[i*2+1]=p[1]}for w in q{out.extend(w.to_le_bytes())}for w in r.states{out.extend(w.to_le_bytes())}let mut v=[u32::MAX;84];v[..r.visited.len()].copy_from_slice(&r.visited);for w in v{out.extend(w.to_le_bytes())}}
 io::stdout().write_all(&out).unwrap();}
