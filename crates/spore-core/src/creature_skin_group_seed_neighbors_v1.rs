//! Checked first-neighbor expansion from 50AC9A..50AD9F.
//! Orientations, edge links, +6C values, group id and prior face states are
//! supplied explicitly. The model prevalidates before mutation and excludes
//! native allocation, UV projection, the later flood and live group identity.
use crate::creature_skin_edge_continuity_v1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { StreamLength, SeedOutOfRange, LinkOutOfRange }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expansion { pub queue: Vec<[u32;2]>, pub orientation_calls: u32, pub edge_calls: u32 }

pub fn expand(indices:&[u32], links:&[u32], orientations:&[u32], states:&mut [u32], seed:u32, group:u32)->Result<Expansion,Error>{
 if indices.len()%3!=0 || indices.len()!=links.len() || orientations.len()!=indices.len()/3 || states.len()!=orientations.len(){return Err(Error::StreamLength)}
 let seed=seed as usize;if seed>=states.len(){return Err(Error::SeedOutOfRange)}
 let base=seed.checked_mul(3).ok_or(Error::SeedOutOfRange)?;
 for &link in &links[base..base+3] {if link as usize>=indices.len(){return Err(Error::LinkOutOfRange)}}
 let mut result=Expansion{queue:Vec::new(),orientation_calls:0,edge_calls:0};
 for lane in 0..3usize {
  let link=links[base+lane];let neighbor=(link/3) as usize;let neighbor_lane=link%3;
  if states[neighbor]>group {continue}
  result.orientation_calls+=1;
  if orientations[neighbor]!=orientations[seed] {continue}
  result.edge_calls+=1;
  if !creature_skin_edge_continuity_v1::matches(indices,seed as u32,lane as u32,neighbor as u32,neighbor_lane).map_err(|_|Error::LinkOutOfRange)? {continue}
  states[neighbor]=0x7fffffff;
  result.queue.push([neighbor as u32,neighbor_lane]);
 }
 Ok(result)
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn admits_once_and_checks_mismatch(){
  let indices=[0,1,2,1,0,3];let links=[3,3,3,0,0,0];let orientation=[4,4];
  let mut states=[u32::MAX,0];let got=expand(&indices,&links,&orientation,&mut states,0,0).unwrap();
  assert_eq!(got.queue,vec![[1,0]]);assert_eq!((got.orientation_calls,got.edge_calls),(1,1));assert_eq!(states,[u32::MAX,0x7fffffff]);
  let mut states=[u32::MAX,0];let mut altered=indices;altered[3]=9;assert!(expand(&altered,&links,&orientation,&mut states,0,0).unwrap().queue.is_empty());
  let mut states=[u32::MAX,0];assert_eq!(expand(&indices,&[9,3,3,0,0,0],&orientation,&mut states,0,0),Err(Error::LinkOutOfRange));assert_eq!(states,[u32::MAX,0]);
 }
}
