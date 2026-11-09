//! Inspect only source-backed class-5 builder segment descriptors.
use sha2::{Digest,Sha256};
use spore_core::{creature_data,creature_skin_class5_segments_v1::{self as segments,FrozenRowsAdmission}};
use std::{error::Error,fs::File,io::Read};
fn main()->Result<(),Box<dyn Error>> {
 let a:Vec<_>=std::env::args().skip(1).collect();
 if a.len()!=3 || a[2]!="--assume-frozen-post-mutation" {return Err("usage: <decoded-creature> <sha256> --assume-frozen-post-mutation".into());}
 if a[1].len()!=64 || !a[1].bytes().all(|x|x.is_ascii_hexdigit()) {return Err("invalid SHA-256".into());}
 let mut bytes=Vec::new();File::open(&a[0])?.take(64*1024*1024+1).read_to_end(&mut bytes)?;
 if bytes.len()>64*1024*1024 {return Err("input too large".into());}
 let hash=format!("{:x}",Sha256::digest(&bytes));if !hash.eq_ignore_ascii_case(&a[1]) {return Err("SHA-256 mismatch".into());}
 let data=creature_data::parse(&bytes)?;if !data.exact || data.consumed!=bytes.len() || data.data0.len()!=data.parts.len()*0x8c {return Err("inexact rows".into());}
 let rows:Vec<[u8;0x8c]>=data.data0.chunks_exact(0x8c).map(|r|r.try_into().unwrap()).collect();
 let result=segments::build(&rows,FrozenRowsAdmission).map_err(|e|format!("{e:?}"))?;
 println!("{}",serde_json::json!({"schema":"class5-segments-source-inspection-v1","source_sha256":hash,"rows":rows.len(),"segments":result.iter().map(|s|serde_json::json!({"from_node":s.from_node,"to_node":s.to_node,"spacing_bits":s.spacing.to_bits(),"key":s.key})).collect::<Vec<_>>(),"admission":"frozen post-mutation rows, empty builder and successful vector operations","limitations":"class-5 segment descriptors only; no sampled interior lanes, complete bounds or grid"}));
 Ok(())
}
