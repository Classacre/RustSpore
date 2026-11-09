//! Inspect only the conditional direct class-5 lane prefix of frozen source rows.
use sha2::{Digest,Sha256};
use spore_core::{creature_data,creature_skin_class5_prefix_v1::{self as prefix,FrozenRowsAdmission},native_sse::Policy};
use std::{error::Error,fs::File,io::Read};
fn main()->Result<(),Box<dyn Error>> {
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.len()!=3 || args[2]!="--assume-frozen-post-mutation" {return Err("usage: <decoded-creature> <sha256> --assume-frozen-post-mutation".into());}
 if args[1].len()!=64 || !args[1].bytes().all(|x|x.is_ascii_hexdigit()) {return Err("invalid SHA-256".into());}
 let mut bytes=Vec::new();File::open(&args[0])?.take(64*1024*1024+1).read_to_end(&mut bytes)?;
 if bytes.len()>64*1024*1024 {return Err("input too large".into());}
 let digest=format!("{:x}",Sha256::digest(&bytes));if !digest.eq_ignore_ascii_case(&args[1]) {return Err("SHA-256 mismatch".into());}
 let data=creature_data::parse(&bytes)?;if !data.exact || data.consumed!=bytes.len() || data.data0.len()!=data.parts.len()*0x8c {return Err("inexact rows".into());}
 let rows:Vec<[u8;0x8c]>=data.data0.chunks_exact(0x8c).map(|r|r.try_into().unwrap()).collect();
 let result=prefix::build(&rows,Policy::Mxcsr1f80,FrozenRowsAdmission).map_err(|e|format!("{e:?}"))?;
 println!("{}",serde_json::json!({"schema":"class5-prefix-source-inspection-v1","source_sha256":digest,"rows":rows.len(),"prefix_lanes":result.lanes.iter().map(|l|serde_json::json!({"center_bits":l.center.map(f32::to_bits),"cutoff_bits":l.cutoff.to_bits(),"scale_bits":l.scale.to_bits(),"key":l.key})).collect::<Vec<_>>(),"lower_bits":result.lower.map(f32::to_bits),"upper_bits":result.upper.map(f32::to_bits),"admission":"frozen post-mutation rows and successful native preparation/storage","limitations":"partial class-5 direct nodes only; no segment, class-3, complete bounds, runtime identity or grid"}));
 Ok(())
}
