//! Conditional single-group composition of source grouping/UV slices.
//!
//! Uses caller-supplied topology, orientations, seed and group ID. It starts
//! with empty cache/UV state, bounded preallocated vectors and finite binary32
//! values under MXCSR 1F80. Unknown cache indices reject. Owned output and
//! prevalidation exclude native partial failure and intermediate observation;
//! outer group selection, native allocation and live identity are not modeled.
use crate::{creature_skin_edge_continuity_v1, creature_skin_group_seed_neighbors_v1,
 creature_skin_group_cachehit_v1::{self,CacheEntry}, creature_skin_uv_miss_v1::{self,Group,Ordinal}, native_sse::Policy};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidInput, Capacity, CacheNotInitialized, Component }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
 pub group:Group, pub uv:Vec<[u32;2]>, pub metadata:Vec<[u32;2]>, pub cache:Vec<CacheEntry>,
 pub corners:Vec<u32>, pub states:Vec<u32>, pub faces:Vec<u32>, pub queue:Vec<[u32;2]>,
 pub cursor:usize, pub next_seed:u32, pub orientation_calls:u32, pub edge_calls:u32,
}

fn project(corner:usize,indices:&[u32],coords:&[[u32;3]],s:&mut Snapshot,policy:Policy)->Result<(),Error>{
 creature_skin_uv_miss_v1::project(indices,coords,&mut s.uv,&mut s.metadata,&mut s.corners,&mut s.cache,&mut s.group,corner,Ordinal::Append,policy).map_err(|_|Error::Component)?;Ok(())
}
fn visit(face:usize,lane:usize,indices:&[u32],coords:&[[u32;3]],s:&mut Snapshot,policy:Policy)->Result<(),Error>{
 if s.faces.len()>=s.faces.capacity(){return Err(Error::Capacity)}
 let base=face*3;let other=base+(lane+2)%3;let pos=indices[other] as usize;
 if s.cache[pos].group!=s.group.id {project(other,indices,coords,s,policy)?;}
 for &position in &indices[base..base+3] {if s.cache[position as usize].uv_index as usize>=s.uv.len(){return Err(Error::CacheNotInitialized)}}
 creature_skin_group_cachehit_v1::append_hit(indices,&s.cache,&mut s.corners,&mut s.states,&mut s.faces,face as u32,lane as u32,s.group.id).map_err(|_|Error::Component)
}
/// Build one admitted group from an explicitly supplied seed and empty state.
pub fn build(indices:&[u32],links:&[u32],orientations:&[u32],coords:&[[u32;3]],seed:u32,group:u32,policy:Policy)->Result<Snapshot,Error>{
 if indices.is_empty() || indices.len()%3!=0 || indices.len()!=links.len() || orientations.len()!=indices.len()/3 || seed as usize>=orientations.len() || group>=0x7fffffff || indices.len()>u32::MAX as usize || orientations.len()>(u32::MAX/4) as usize {return Err(Error::InvalidInput)}
 if indices.iter().any(|&x|x as usize>=coords.len()) || links.iter().any(|&x|x as usize>=indices.len()) || orientations.iter().any(|&x|x>=6) || coords.iter().flatten().any(|&x|!f32::from_bits(x).is_finite()) {return Err(Error::InvalidInput)}
 let n=orientations.len();let mut s=Snapshot{group:Group{id:group,axis:orientations[seed as usize],bounds:[0x7f7fffff,0xff7fffff,0x7f7fffff,0xff7fffff]},uv:Vec::with_capacity(indices.len()),metadata:Vec::with_capacity(indices.len()),cache:vec![CacheEntry{group:u32::MAX,uv_index:u32::MAX};coords.len()],corners:vec![0xcccccccc;indices.len()],states:vec![0;n],faces:Vec::with_capacity(n),queue:Vec::with_capacity(n),cursor:0,next_seed:u32::MAX,orientation_calls:0,edge_calls:0};
 let first=seed as usize*3;project(first,indices,coords,&mut s,policy)?;project(first+1,indices,coords,&mut s,policy)?;visit(seed as usize,0,indices,coords,&mut s,policy)?;
 let initial=creature_skin_group_seed_neighbors_v1::expand(indices,links,orientations,&mut s.states,seed,group).map_err(|_|Error::Component)?;s.orientation_calls=initial.orientation_calls;s.edge_calls=initial.edge_calls;s.queue.extend(initial.queue);
 while s.cursor<s.queue.len(){
  let [face,entry]=s.queue[s.cursor];s.cursor+=1;visit(face as usize,entry as usize,indices,coords,&mut s,policy)?;
  for lane in 0..3u32 {if lane==entry{continue}let link=links[face as usize*3+lane as usize];let neighbor=link/3;let neighbor_lane=link%3;
   if s.states[neighbor as usize]>group{continue}s.orientation_calls+=1;
   let mut accept=orientations[neighbor as usize]==s.group.axis;
   if accept{s.edge_calls+=1;accept=creature_skin_edge_continuity_v1::matches(indices,face,lane,neighbor,neighbor_lane).map_err(|_|Error::Component)?;}
   if accept {if s.queue.len()>=n{return Err(Error::Capacity)}s.states[neighbor as usize]=0x7fffffff;s.queue.push([neighbor,neighbor_lane]);}
   else{s.states[neighbor as usize]=group+1;s.next_seed=neighbor;}
  }
 }
 Ok(s)
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn seed_only_and_validation(){
  let coords=[[0,0,0],[0x3f800000,0,0],[0,0x3f800000,0]];
  let r=build(&[0,1,2],&[0,1,2],&[4],&coords,0,0,Policy::Mxcsr1f80).unwrap();
  assert_eq!(r.faces,[0]);assert_eq!(r.uv.len(),3);assert!(r.queue.is_empty());assert_eq!(r.states,[u32::MAX]);
  assert_eq!(build(&[0,1,3],&[0,1,2],&[4],&coords,0,0,Policy::Mxcsr1f80),Err(Error::InvalidInput));
 }
}
