//! Offline comparison for the separately reviewed synthetic E6B560 host.
use std::{env,fs};use spore_core::{cell_definition::{self,DEFINITION_SIZE},cell_pick_filter_v1::{self,Input,CacheBytes}};
fn u(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i*4..i*4+4].try_into().unwrap())}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=env::args_os().skip(1).collect();if a.len()!=2{return Err("usage: INPUT OUTPUT".into());}
 let input=fs::read(&a[0])?;let output=fs::read(&a[1])?;if input.len()%32!=0||output.len()!=input.len()/32*24{return Err("bad extent".into());}
 for(i,(p,r)) in input.chunks_exact(32).zip(output.chunks_exact(24)).enumerate(){
  let mut def=[0u8;DEFINITION_SIZE];def[0xb4..0xb8].copy_from_slice(&u(p,6).to_le_bytes());let d=cell_definition::parse(&def)?;
  let x=Input{player_handle:u(p,0),candidate_id:u(p,1),player_size:Some(u(p,2)),candidate_size:u(p,3),
    player_cache:Some(CacheBytes{flag_1250:u(p,4) as u8,flag_1251:u(p,5) as u8}),candidate_definition:Some(d),progress:Some(u(p,7) as i32)};
  let t=cell_pick_filter_v1::evaluate(x).map_err(|e|format!("case {i}: {e:?}"))?;
  let expected=[t.accepted as u32,t.player_resolved as u32,t.cache_read as u32,t.definition_read as u32,t.definition_read as u32,t.definition_read as u32];
  for(j,&v) in expected.iter().enumerate(){let n=u(r,j);if n!=v{return Err(format!("case {i}, field {j}: native {n} Rust {v}").into());}}
 }
 println!("matched {} predicate and provider-call replies",input.len()/32);Ok(())
}
