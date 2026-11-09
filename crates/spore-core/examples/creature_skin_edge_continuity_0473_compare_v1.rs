//! Conditional retained +6C copy and synthetic endpoint mutation controls.
use spore_core::creature_skin_edge_continuity_v1::matches;
use std::{env, fs, io::{self, Write}};
fn words(path: &str) -> Vec<u32> {
    let bytes=fs::read(path).unwrap(); assert_eq!(bytes.len(), 1008);
    bytes.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect()
}
fn main() {
    let a:Vec<_>=env::args().collect(); assert_eq!(a.len(),3);
    let mut indices=words(&a[1]); let links=words(&a[2]); assert_eq!(links[0],22);
    let mut out=Vec::new();
    for (i,&j) in links.iter().enumerate() {
        assert!(j<252 && links[j as usize]==i as u32);
        let v=matches(&indices,(i/3) as u32,(i%3) as u32,j/3,j%3).unwrap() as u32;
        out.extend(v.to_le_bytes());
    }
    indices[22]^=1;
    out.extend((matches(&indices,0,0,7,1).unwrap() as u32).to_le_bytes());
    indices[22]^=1; indices[23]^=1;
    out.extend((matches(&indices,0,0,7,1).unwrap() as u32).to_le_bytes());
    io::stdout().write_all(&out).unwrap();
}
