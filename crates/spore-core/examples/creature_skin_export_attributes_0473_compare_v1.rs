//! Conditional supplied snapshots, including only the tangent half of each pair.
use spore_core::creature_skin_export_attributes_v1::project;
use std::{env,fs,io::{self,Write}};
fn rows<const N:usize>(path:&str)->Vec<[u32;N]>{let b=fs::read(path).unwrap();assert_eq!(b.len()%(4*N),0);b.chunks_exact(4*N).map(|r|std::array::from_fn(|i|u32::from_le_bytes(r[i*4..i*4+4].try_into().unwrap()))).collect()}
fn main(){let a:Vec<_>=env::args().collect();assert_eq!(a.len(),6);let out=project(&rows::<3>(&a[1]),&rows::<3>(&a[2]),&rows::<3>(&a[3]),&rows::<2>(&a[4]),&rows::<6>(&a[5])).unwrap();let mut w=io::stdout().lock();for x in out.positions.iter().flatten().chain(out.normals.iter().flatten()).chain(out.uvs.iter().flatten()).chain(out.tangents.iter().flatten()){w.write_all(&x.to_le_bytes()).unwrap();}}
