//! Offline comparator for a separately reviewed host output; never launches it.
//! Compile against the exact reviewed spore-core build, and hash both artifacts.
use std::{env,fs,io::{self,Write}};
use spore_core::{cell_actor_geometry_projection_v1::Output,
    cell_pick_tree_v1::{self,Kind,Node,Query},native_sse::Policy,native_x87::Context};
fn u(b:&[u8],i:usize)->u32 { u32::from_le_bytes(b[4*i..4*i+4].try_into().unwrap()) }
fn main()->Result<(),Box<dyn std::error::Error>> {
 let a:Vec<_>=env::args_os().skip(1).collect();
 if a.len()!=2 {return Err("usage: compare INPUT.bin NATIVE-OUTPUT.bin".into());}
 let input=fs::read(&a[0])?;let output=fs::read(&a[1])?;
 if input.len()%912!=0||output.len()!=input.len()/912*136{return Err("record size mismatch".into());}
 for (i,(p,r)) in input.chunks_exact(912).zip(output.chunks_exact(136)).enumerate() {
  let cw=u(p,0);let n=u(p,1) as usize;let cap=u(p,2) as usize;let root=u(p,3) as usize;
  if n>31||cap>31{return Err(format!("case {i}: invalid bounds").into());}
  let mut nodes=Vec::new();
  for j in 0..n {let b=11+j*7;let g=Output{center:[u(p,b),u(p,b+1),u(p,b+2)],scalar:u(p,b+3)};
   let kind=match u(p,b+4){0=>Kind::Leaf(u(p,b+5)),1=>Kind::Branch{first:u(p,b+5) as usize,second:u(p,b+6) as usize},_=>return Err(format!("case {i}: node kind").into())};
   nodes.push(Node{geometry:g,kind});
  }
  let q=Query{origin:[u(p,4),u(p,5),u(p,6)],direction:[u(p,7),u(p,8),u(p,9)],limit:u(p,10),capacity:cap,
      x87:Context::from_control_word(cw as u16)?,sse:Policy::Mxcsr1f80};
  let got=cell_pick_tree_v1::traverse(&nodes,if n==0{None}else{Some(root)},q)
      .map_err(|e|format!("case {i}: model rejected {e:?}"))?;
  let count=u(r,0) as usize;if count!=got.len(){return Err(format!("case {i}: count native={count}, Rust={}",got.len()).into());}
  for (j,&h) in got.iter().enumerate(){if u(r,1+j)!=h{return Err(format!("case {i}: handle {j}: native={:08x}, Rust={h:08x}",u(r,1+j)).into());}}
  for j in count..31 {if u(r,1+j)!=0xcccccccc{return Err(format!("case {i}: output guard {j}").into());}}
  if u(r,32)!=cw{return Err(format!("case {i}: x87 control mismatch").into());}
  if u(r,33)&0xffc0 != 0x1f80{return Err(format!("case {i}: MXCSR control mismatch").into());}
 }
 writeln!(io::stdout(),"matched {} ordered replies; metadata control checked; status flags not modeled",input.len()/912)?;Ok(())
}
