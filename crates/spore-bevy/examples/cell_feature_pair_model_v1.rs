//! Fixed pair-prefix corpus adapter; never executes original bytes.
#[path = "../src/cell_source_feature_geometry_v1.rs"] mod feature;
use spore_core::cell_part_geometry::NativeCellPart;
use std::{env,fs,io::{self,Write}};
fn w(s:&[u8],o:usize)->u32{u32::from_le_bytes(s[o..o+4].try_into().unwrap())}
fn main()->Result<(),Box<dyn std::error::Error>> {
 let a:Vec<_>=env::args_os().collect();if a.len()!=2{return Err("expected corpus".into())}
 let data=fs::read(&a[1])?;if data.len()!=6*216{return Err("wrong corpus length".into())}
 let mut out=Vec::new();
 for c in data.chunks_exact(216){let first=feature::Projected{center:[w(c,0),w(c,4),w(c,8)],radius:w(c,12)};let a=&c[16..144];let r=&c[144..];
  let row=NativeCellPart{word_06_is_ffff:false,class:2,center:[f32::from_bits(w(r,8)),f32::from_bits(w(r,12)),f32::from_bits(w(r,16))],matrix_bits:[0;9],radius:f32::from_bits(w(r,56)),tail_bits:[0;3]};
  let t=feature::Transform{position:[w(a,0x4c),w(a,0x50),w(a,0x54)],scale:w(a,0x58),flags:a[0x48],matrix:std::array::from_fn(|i|w(a,0x5c+i*4))};
  let second=feature::project_second(&row,t).map_err(|e|format!("second: {e:?}"))?;let p=feature::pair_prefix(first,second).map_err(|e|format!("pair: {e:?}"))?;
  for v in second.center.into_iter().chain([second.radius]).chain(p.delta).chain([p.distance_squared,p.radius_sum,u32::from(p.small_distance)]) {out.extend(v.to_le_bytes());}
 } io::stdout().write_all(&out)?;Ok(())
}
