//! Fixed post-arbitration x87 distance corpus adapter; no original code execution.
#[path = "../src/cell_source_contact_call_v1.rs"] mod contact;
use std::{env,fs,io::{self,Write}};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=env::args_os().collect();if a.len()!=2{return Err("expected corpus".into())}let data=fs::read(&a[1])?;if data.len()!=6*24{return Err("wrong corpus size".into())}
 let mut out=Vec::new();for c in data.chunks_exact(24){let mut p=[[0u32;3];2];for i in 0..6{p[i/3][i%3]=u32::from_le_bytes(c[i*4..i*4+4].try_into()?)}let d=contact::distance(p,0x027f).map_err(|e|format!("distance: {e}"))?;out.extend(d.to_le_bytes());}io::stdout().write_all(&out)?;Ok(())
}
