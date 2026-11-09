//! Checked 50ADA0 queue consumer with an explicit face-mark/visit stub.
//! This mirrors the bounded native comparison, not 50A840 UV/cache mutations,
//! source allocation, actual group state, runtime identity or partial failures.
use crate::creature_skin_edge_continuity_v1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { StreamLength, LinkOutOfRange, QueueOutOfRange, UnsupportedGroup, TooManyFaces }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
 pub queue:Vec<[u32;2]>, pub cursor:usize, pub next_seed:u32, pub visited:Vec<u32>,
 pub states:Vec<u32>, pub orientation_calls:u32, pub edge_calls:u32,
}
/// Consume an admitted initial queue. On a visit the explicit stub marks the
/// face with !group; it does not append source face/UV data.
pub fn consume(indices:&[u32],links:&[u32],orientations:&[u32],states:&[u32],initial_queue:&[[u32;2]],group:u32,group_axis:u32,next_seed:u32)->Result<Snapshot,Error>{
 if indices.len()%3!=0 || indices.len()!=links.len() || orientations.len()!=indices.len()/3 || states.len()!=orientations.len(){return Err(Error::StreamLength)}
 if orientations.len()>(u32::MAX/4) as usize || initial_queue.len()>orientations.len(){return Err(Error::TooManyFaces)}
 if group>=0x7fffffff{return Err(Error::UnsupportedGroup)}
 if links.iter().any(|&x|x as usize>=indices.len()){return Err(Error::LinkOutOfRange)}
 if initial_queue.iter().any(|p|p[0] as usize>=states.len() || p[1]>=3){return Err(Error::QueueOutOfRange)}
 let mut out=Snapshot{queue:initial_queue.to_vec(),cursor:0,next_seed,visited:Vec::new(),states:states.to_vec(),orientation_calls:0,edge_calls:0};
 while out.cursor<out.queue.len(){
  let [face,entry]=out.queue[out.cursor];out.cursor+=1;out.visited.push(face);out.states[face as usize]=!group;
  for lane in 0..3u32 {
   if lane==entry{continue}
   let link=links[face as usize*3+lane as usize];let neighbor=link/3;let neighbor_lane=link%3;
   if out.states[neighbor as usize]>group {continue}
   out.orientation_calls+=1;
   let mut accept=orientations[neighbor as usize]==group_axis;
   if accept {out.edge_calls+=1;accept=creature_skin_edge_continuity_v1::matches(indices,face,lane,neighbor,neighbor_lane).map_err(|_|Error::LinkOutOfRange)?;}
   if accept {out.states[neighbor as usize]=0x7fffffff;out.queue.push([neighbor,neighbor_lane]);}
   else {out.states[neighbor as usize]=group+1;out.next_seed=neighbor;}
  }
 }
 Ok(out)
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn excludes_entry_and_marks_rejected_neighbor(){
  let indices=[0,1,2,1,0,3];let links=[3,3,3,0,0,0];let orient=[4,5];let states=[0x7fffffff,0];
  let r=consume(&indices,&links,&orient,&states,&[[0,0]],0,4,u32::MAX).unwrap();
  assert_eq!(r.visited,[0]);assert_eq!(r.states,[u32::MAX,1]);assert_eq!(r.next_seed,1);assert_eq!((r.orientation_calls,r.edge_calls),(1,0));
  assert_eq!(consume(&indices,&links,&orient,&states,&[[0,3]],0,4,0),Err(Error::QueueOutOfRange));
  assert_eq!(consume(&indices,&links,&orient,&states,&[],0x7fffffff,4,0),Err(Error::UnsupportedGroup));
 }
}
