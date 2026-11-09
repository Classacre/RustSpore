//! Fixed E673 record-phase corpus adapter; no original byte execution.
#[path = "../src/cell_source_contact_record_v1.rs"] mod record;
use std::{env,fs,io::{self,Write}};
fn w(s:&[u8],o:usize)->u32{u32::from_le_bytes(s[o..o+4].try_into().unwrap())}
fn put(s:&mut[u8],o:usize,v:u32){s[o..o+4].copy_from_slice(&v.to_le_bytes())}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=env::args_os().collect();if a.len()!=2{return Err("expected corpus".into())}let data=fs::read(&a[1])?;if data.len()!=224{return Err("wrong corpus size".into())}
 let keys=[w(&data,0),w(&data,4),w(&data,8)];let mut out=Vec::new();
 for i in 0..6{let index=record::lookup(&keys,w(&data,12+4*i)).map_err(|e|format!("lookup {e:?}"))?.map(|v|v as u32).unwrap_or(u32::MAX);out.extend(index.to_le_bytes());}
 let raw=&data[36..80];let mut entry=record::Entry{identity:7,key:w(raw,0),ids:[w(raw,4),w(raw,8)],created:w(raw,12),updated:w(raw,16),point:[w(raw,20),w(raw,24),w(raw,28)],impact:raw[32],handles:[w(raw,36),w(raw,40)]};
 let ticket=record::begin_existing(&mut entry,entry_key(raw),w(&data,80),9).map_err(|e|format!("begin {e:?}"))?;
 record::finish_existing(ticket,&mut entry,9,[w(&data,84),w(&data,88),w(&data,92)]).map_err(|e|format!("finish {e:?}"))?;
 let mut existing=raw.to_vec();put(&mut existing,16,entry.updated);for i in 0..3{put(&mut existing,20+i*4,entry.point[i]);}out.extend(existing);
 for c in data[96..].chunks_exact(32){let ids=[w(c,0),w(c,4)];let point=[w(c,8),w(c,12),w(c,16)];let input=record::Input{ids,point_id:9,point,distance:w(c,24),current:w(c,20)};let key=record::packed_key(ids);
  let e=record::new_entry(1,input,key,w(c,28),9,point).map_err(|e|format!("new {e:?}"))?;let mut r=[0xa5;44];
  for (i,v) in [e.key,e.ids[0],e.ids[1],e.created,e.updated,e.point[0],e.point[1],e.point[2]].into_iter().enumerate(){put(&mut r,i*4,v)}r[32]=e.impact;put(&mut r,36,e.handles[0]);put(&mut r,40,e.handles[1]);out.extend(r);
 }io::stdout().write_all(&out)?;Ok(())
}
fn entry_key(raw:&[u8])->u32{w(raw,0)}
