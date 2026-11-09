//! Experimental host-math adapter on admitted 0473 rows; not a CRT binding.
use sha2::{Digest,Sha256};use spore_core::{creature_data,creature_skin_class5_prefix_v1::{self as prefix,FrozenRowsAdmission},creature_skin_class5_segments_v1,creature_skin_segment_crt_admitted_v1::{self as sampler,CrtMath,MathAdmission},native_sse::Policy,native_x87::{Context,Precision,Rounding}};
use std::{error::Error,fs::File,io::Read};
struct HostMath;impl CrtMath for HostMath {fn pow_spill(&mut self,b:f64,e:f64)->Result<f32,()>{Ok(b.powf(e) as f32)}fn log_spill(&mut self,v:f32)->Result<f32,()>{Ok((v as f64).ln() as f32)}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args().skip(1).collect();if a.len()!=3||a[2]!="--assume-frozen-and-threshold-0.01" {return Err("usage: <decoded-creature> <sha256> --assume-frozen-and-threshold-0.01".into());}
 let mut bytes=Vec::new();File::open(&a[0])?.take(64*1024*1024+1).read_to_end(&mut bytes)?;if bytes.len()>64*1024*1024{return Err("too large".into());}
 let hash=format!("{:x}",Sha256::digest(&bytes));if !hash.eq_ignore_ascii_case(&a[1]){return Err("sha mismatch".into());}
 let data=creature_data::parse(&bytes)?;if !data.exact||data.consumed!=bytes.len()||data.data0.len()!=data.parts.len()*140{return Err("inexact".into());}
 let rows:Vec<[u8;140]>=data.data0.chunks_exact(140).map(|r|r.try_into().unwrap()).collect();
 let prefix=prefix::build(&rows,Policy::Mxcsr1f80,FrozenRowsAdmission).map_err(|e|format!("{e:?}"))?;
 let segs=creature_skin_class5_segments_v1::build(&rows,creature_skin_class5_segments_v1::FrozenRowsAdmission).map_err(|e|format!("{e:?}"))?;
 let radii:Vec<f32>=rows.iter().take(prefix.lanes.len()).map(|r|{let x=f32::from_le_bytes(r[0x2c..0x30].try_into().unwrap());let y=(x*f32::from_bits(0x3ec18cd8))*f32::from_bits(0x3f333333);if f32::from_bits(0x3d23d70a)>y {f32::from_bits(0x3d23d70a)}else{y}}).collect();
 let mut lower=prefix.lower;let mut upper=prefix.upper;let mut lanes=prefix.lanes.clone();let mut counts=vec![lanes.len()];
 let ctx=Context{precision:Precision::Bits53,rounding:Rounding::NearestEven};
 for s in segs {let out=sampler::sample(prefix.lanes[s.from_node].center,prefix.lanes[s.to_node].center,radii[s.from_node],radii[s.to_node],s.spacing,f32::from_bits(0x3c23d70a),s.key,Policy::Mxcsr1f80,ctx,&mut HostMath,MathAdmission).map_err(|e|format!("{e:?}"))?;
  for x in out {for i in 0..3 {let lo=x.lane.center[i]-x.radius;let hi=x.lane.center[i]+x.radius;if lo<lower[i]{lower[i]=lo}if hi>upper[i]{upper[i]=hi}}lanes.push(x.lane);}counts.push(lanes.len());}
 println!("{}",serde_json::json!({"schema":"experimental-hostmath-nonzero-segments-v1","source_sha256":hash,"counts":counts,"lanes":lanes.iter().map(|l|l.center.map(f32::to_bits).into_iter().chain([l.cutoff.to_bits(),l.scale.to_bits(),l.coefficient.to_bits(),l.key]).collect::<Vec<_>>()).collect::<Vec<_>>(),"lower_bits":lower.map(f32::to_bits),"upper_bits":upper.map(f32::to_bits),"limits":"frozen rows and fresh threshold admitted; host powf/ln adapter is not verified generally against MSVCR90; partial class-5 nodes and segments only"}));Ok(())
}
