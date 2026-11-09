//! Conditional bounded multi-group composition of source-derived slices.
//! The outer orchestration is modelled, not compared as a complete original
//! loop. Inputs, initial group count and empty owned storage are explicit;
//! supplied orientations, successful preallocation and MXCSR policy are
//! admissions. Errors exclude original partial-failure order and live identity.
use crate::{creature_skin_edge_continuity_v1 as edge,
 creature_skin_group_cachehit_v1::{self as hit,CacheEntry},
 creature_skin_group_seed_neighbors_v1 as neighbors,
 creature_skin_group_selector_v1 as selector,
 creature_skin_uv_miss_v1::{self as miss,Group,Ordinal}, native_sse::Policy};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidInput, Capacity, CacheNotInitialized, Component }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputGroup { pub group: Group, pub seed:u32, pub uv_end:u32, pub faces:Vec<u32> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output { pub groups:Vec<OutputGroup>, pub uv:Vec<[u32;2]>, pub metadata:Vec<[u32;2]>, pub cache:Vec<CacheEntry>, pub corners:Vec<u32>, pub states:Vec<u32> }
fn project(indices:&[u32],coords:&[[u32;3]],o:&mut Output,g:&mut Group,c:usize,policy:Policy)->Result<(),Error>{
 miss::project(indices,coords,&mut o.uv,&mut o.metadata,&mut o.corners,&mut o.cache,g,c,Ordinal::Append,policy).map_err(|_|Error::Component)?;Ok(())
}
fn visit(indices:&[u32],coords:&[[u32;3]],o:&mut Output,g:&mut Group,faces:&mut Vec<u32>,face:usize,lane:usize,policy:Policy)->Result<(),Error>{
 if faces.len()>=o.states.len(){return Err(Error::Capacity)}
 let base=face*3;let other=base+(lane+2)%3;
 if o.cache[indices[other] as usize].group!=g.id {project(indices,coords,o,g,other,policy)?;}
 for &position in &indices[base..base+3] {if o.cache[position as usize].uv_index as usize>=o.uv.len(){return Err(Error::CacheNotInitialized)}}
 hit::append_hit(indices,&o.cache,&mut o.corners,&mut o.states,faces,face as u32,lane as u32,g.id).map_err(|_|Error::Component)
}
/// Start with explicitly empty UV/cache, zero face states and `initial_group`
/// previously existing groups. Supplied adjacency must be in-range reciprocal.
pub fn build(indices:&[u32],links:&[u32],orientations:&[u32],coords:&[[u32;3]],initial_group:u32,policy:Policy)->Result<Output,Error>{
 if indices.is_empty() || indices.len()%3!=0 || indices.len()!=links.len() || orientations.len()!=indices.len()/3 || indices.len()>u32::MAX as usize || initial_group.checked_add(orientations.len() as u32).filter(|&x|x<0x7fffffff).is_none() {return Err(Error::InvalidInput)}
 if indices.iter().any(|&x|x as usize>=coords.len()) || links.iter().enumerate().any(|(i,&x)|x as usize>=links.len() || links[x as usize] as usize!=i) || orientations.iter().any(|&x|x>=6) || coords.iter().flatten().any(|&x|!f32::from_bits(x).is_finite()){return Err(Error::InvalidInput)}
 let n=orientations.len();let mut o=Output{groups:Vec::with_capacity(n),uv:Vec::with_capacity(indices.len()),metadata:Vec::with_capacity(indices.len()),cache:vec![CacheEntry{group:u32::MAX,uv_index:u32::MAX};coords.len()],corners:vec![u32::MAX;indices.len()],states:vec![0;n]};
 let mut group=initial_group;let mut next=0u32;
 loop {
  let Some(seed)=selector::select(&o.states,next,group).map_err(|_|Error::Component)? else {break};
  if o.groups.len()>=n{return Err(Error::Capacity)}
  let mut g=Group{id:group,axis:orientations[seed as usize],bounds:[0x7f7fffff,0xff7fffff,0x7f7fffff,0xff7fffff]};let mut faces=Vec::with_capacity(n);
  let base=seed as usize*3;project(indices,coords,&mut o,&mut g,base,policy)?;project(indices,coords,&mut o,&mut g,base+1,policy)?;visit(indices,coords,&mut o,&mut g,&mut faces,seed as usize,0,policy)?;
  let mut queue=neighbors::expand(indices,links,orientations,&mut o.states,seed,group).map_err(|_|Error::Component)?.queue;
  let mut cursor=0;
  while cursor<queue.len(){
   if queue.len()>n{return Err(Error::Capacity)}
   let [face,entry]=queue[cursor];cursor+=1;visit(indices,coords,&mut o,&mut g,&mut faces,face as usize,entry as usize,policy)?;
   for lane in 0..3u32 {if lane==entry{continue}let link=links[face as usize*3+lane as usize];let nf=link/3;let nl=link%3;
    if o.states[nf as usize]>group{continue}
    let accept=orientations[nf as usize]==g.axis && edge::matches(indices,face,lane,nf,nl).map_err(|_|Error::Component)?;
    if accept {if queue.len()>=n{return Err(Error::Capacity)}o.states[nf as usize]=0x7fffffff;queue.push([nf,nl]);}
    else{o.states[nf as usize]=group+1;next=nf;}
   }
  }
  o.groups.push(OutputGroup{group:g,seed,uv_end:o.uv.len() as u32,faces});group+=1;
 }
 Ok(o)
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
 use super::*;
 #[test] fn disconnected_faces_keep_shared_uv_and_group_state(){
  let idx=[0,1,2,3,4,5];let links=[0,1,2,3,4,5];let ori=[0,1];let coords=[[0,0,0];6];
  let out=build(&idx,&links,&ori,&coords,4,Policy::Mxcsr1f80).unwrap();
  assert_eq!(out.groups.iter().map(|g|(g.group.id,g.seed,g.faces.clone())).collect::<Vec<_>>(),vec![(4,0,vec![0]),(5,1,vec![1])]);
  assert_eq!(out.uv.len(),6);assert_eq!(out.states,[!4,!5]);
 }
 #[test] fn rejects_bad_links_and_group_overflow(){
  assert_eq!(build(&[0,0,0],&[1,1,2],&[0],&[[0,0,0]],0,Policy::Mxcsr1f80),Err(Error::InvalidInput));
  assert_eq!(build(&[0,0,0],&[0,1,2],&[0],&[[0,0,0]],0x7ffffffe,Policy::Mxcsr1f80),Err(Error::InvalidInput));
 }
}
