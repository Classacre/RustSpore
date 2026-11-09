//! Bounded 50A840 cache-hit face append and UV-index state update.
//! Inputs are explicit; cache miss is rejected before any mutation. Native may
//! already have appended a face before its miss helper, which is not modeled.
//! No source UV values, grouping policy or live owner identity are inferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheEntry { pub group: u32, pub uv_index: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidLane, FaceOutOfRange, PositionOutOfRange, CacheMiss }

pub fn append_hit(positions: &[u32], cache: &[CacheEntry], uv: &mut [u32], states: &mut [u32],
                  faces: &mut Vec<u32>, face: u32, lane: u32, group: u32) -> Result<(), Error> {
    if lane >= 3 { return Err(Error::InvalidLane); }
    let first=(face as usize).checked_mul(3).ok_or(Error::FaceOutOfRange)?;
    let end=first.checked_add(3).ok_or(Error::FaceOutOfRange)?;
    let corners=positions.get(first..end).ok_or(Error::FaceOutOfRange)?;
    if uv.len()<end || face as usize>=states.len() { return Err(Error::FaceOutOfRange); }
    let mut entries=[CacheEntry{group:0,uv_index:0};3];
    for i in 0..3 { entries[i]=*cache.get(corners[i] as usize).ok_or(Error::PositionOutOfRange)?; }
    let current=lane as usize; let next=(current+1)%3; let other=(current+2)%3;
    if entries[other].group!=group { return Err(Error::CacheMiss); }
    faces.push(face);
    uv[first+other]=entries[other].uv_index;
    uv[first+current]=entries[current].uv_index;
    uv[first+next]=entries[next].uv_index;
    states[face as usize]=!group;
    Ok(())
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn writes_in_source_order_and_rejects_before_mutation() {
  let positions=[2,0,1];let cache=[CacheEntry{group:7,uv_index:100},CacheEntry{group:7,uv_index:101},CacheEntry{group:7,uv_index:102}];
  let mut uv=[0;3];let mut states=[0];let mut faces=Vec::new();
  append_hit(&positions,&cache,&mut uv,&mut states,&mut faces,0,1,7).unwrap();
  assert_eq!(uv,[102,100,101]);assert_eq!(states,[!7]);assert_eq!(faces,[0]);
  let prior=(uv,states,faces.clone());
  assert_eq!(append_hit(&positions,&cache,&mut uv,&mut states,&mut faces,0,1,8),Err(Error::CacheMiss));
  assert_eq!((uv,states,faces.clone()),prior);
  assert_eq!(append_hit(&positions,&cache,&mut uv,&mut states,&mut faces,0,3,7),Err(Error::InvalidLane));
  assert_eq!(append_hit(&positions,&cache[..2],&mut uv,&mut states,&mut faces,0,1,7),Err(Error::PositionOutOfRange));
 }
}
