//! Fixed nondegenerate contact corpus adapter; no original code execution.
#[path = "../src/cell_source_feature_geometry_v1.rs"] mod feature;
use std::{env,fs,io::{self,Write}};
fn w(s:&[u8],o:usize)->u32{u32::from_le_bytes(s[o..o+4].try_into().unwrap())}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=env::args_os().collect();if a.len()!=2{return Err("expected corpus".into())}let data=fs::read(&a[1])?;if data.len()!=6*52{return Err("wrong corpus length".into())}
 let mut out=Vec::new();for c in data.chunks_exact(52){
  let first=feature::Projected{center:[w(c,0),w(c,4),w(c,8)],radius:w(c,12)};let second=feature::Projected{center:[w(c,16),w(c,20),w(c,24)],radius:w(c,28)};
  let prefix=feature::pair_prefix(first,second).map_err(|e|format!("prefix: {e:?}"))?;
  if prefix.small_distance || prefix.distance_squared!=w(c,32) || prefix.delta!=[w(c,36),w(c,40),w(c,44)] || prefix.radius_sum!=w(c,48) {return Err("corpus prefix mismatch".into())}
  let result=feature::continue_pair(first,second,None).map_err(|e|format!("geometry: {e:?}"))?;
  let words=match result {feature::Geometry::Separated{distance,depth}=>[1,distance,depth,0xcccccccc,0xcccccccc,0xcccccccc,0xcccccccc,0xcccccccc,0xcccccccc],
   feature::Geometry::Nondegenerate{distance,depth,normal,point}=>[0,distance,depth,normal[0],normal[1],normal[2],point[0],point[1],point[2]],
   feature::Geometry::Small{..}=>return Err("unexpected small case".into())};
  for v in words{out.extend(v.to_le_bytes());}
 }io::stdout().write_all(&out)?;Ok(())
}
