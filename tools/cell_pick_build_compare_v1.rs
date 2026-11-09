//! Offline comparison of supplied operation histories and separately retained replies.
use std::{env,fs};
use spore_core::{cell_actor_geometry_projection_v1::Output as Geometry,
 cell_pick_tree_build_v1::{Tree,Environment,LeafId},cell_pick_tree_v1::{Kind,Node},
 native_sse::Policy,native_x87::Context};
fn u(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i*4..i*4+4].try_into().unwrap())}
fn visit(nodes:&[Node],i:usize,rows:&mut Vec<[u32;6]>)->Result<(),Box<dyn std::error::Error>>{
 let n=nodes.get(i).ok_or("bad child")?;let (kind,handle)=match n.kind{Kind::Leaf(h)=>(0,h),Kind::Branch{..}=>(1,0)};
 rows.push([kind,handle,n.geometry.center[0],n.geometry.center[1],n.geometry.center[2],n.geometry.scalar]);
 if let Kind::Branch{first,second}=n.kind{visit(nodes,first,rows)?;visit(nodes,second,rows)?;}Ok(())
}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let args:Vec<_>=env::args_os().skip(1).collect();if args.len()!=2{return Err("usage: INPUT NATIVE-OUTPUT".into());}
 let input=fs::read(&args[0])?;let native=fs::read(&args[1])?;
 if input.len()%232!=0||native.len()!=input.len()/232*792{return Err("invalid extent".into());}
 for (case,(p,r)) in input.chunks_exact(232).zip(native.chunks_exact(792)).enumerate(){
  let cw=u(p,0);let count=u(p,1) as usize;if count>8{return Err("invalid steps".into());}
  let env=Environment{x87:Context::from_control_word(cw as u16)?,sse:Policy::Mxcsr1f80};
  let mut tree=Tree::new(64);let mut ids:[Option<LeafId>;8]=[None;8];
  for i in 0..count{let b=2+i*7;let op=u(p,b);let id=u(p,b+1) as usize;if id>=8{return Err("invalid id".into());}
   let geom=Geometry{center:[u(p,b+2),u(p,b+3),u(p,b+4)],scalar:u(p,b+5)};
   let result=match op {0=>tree.insert(geom,u(p,b+6),env).map(|v|ids[id]=Some(v)),1=>tree.flush(env),
    2=>tree.update(ids[id].ok_or("unknown update")?,geom,env),3=>tree.remove(ids[id].take().ok_or("unknown removal")?,env),_=>return Err("invalid op".into())};
   result.map_err(|e|format!("case {case} step {i}: {e:?}"))?;
  }
  let(nodes,root)=tree.snapshot().map_err(|e|format!("case {case}: snapshot {e:?}"))?;let mut rows=Vec::new();if let Some(i)=root{visit(&nodes,i,&mut rows)?;}
  if u(r,0) as usize!=rows.len(){return Err(format!("case {case}: row count native={} rust={}",u(r,0),rows.len()).into());}
  for(i,row) in rows.iter().enumerate(){for(j,&v) in row.iter().enumerate(){let observed=u(r,1+i*6+j);if v!=observed{return Err(format!("case {case}: row {i} word {j}: native={observed:08x} rust={v:08x}").into());}}}
  let pending=tree.pending_handles().map_err(|e|format!("case {case}: pending {e:?}"))?;
  if u(r,187) as usize!=pending.len(){return Err(format!("case {case}: pending count").into());}
  for(i,&v) in pending.iter().enumerate(){if u(r,188+i)!=v{return Err(format!("case {case}: pending {i}").into());}}
  if u(r,196)!=cw||u(r,197)&0xffc0!=0x1f80{return Err(format!("case {case}: control metadata").into());}
 }
 println!("matched {} tree snapshots and pending lists; FP status flags not modeled",input.len()/232);Ok(())
}
