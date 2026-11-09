//! Bounded 50A570 UV projection and cache update with caller-supplied state.
//! The append branch requires paired preallocated vector capacity; it excludes
//! native allocation, growth, partial failure and the preceding face append.
//! Existing ordinal and append both use finite binary32 bounds under MXCSR 1F80.
//! Actual group, UV ownership and live creature identity are not inferred.
use crate::{creature_skin_group_cachehit_v1::CacheEntry, creature_skin_uv_select_v1, native_sse::{self, Policy, SseError}};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group { pub id: u32, pub axis: u32, pub bounds: [u32;4] }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordinal { Append, Existing(u32) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { CornerOutOfRange, PositionOutOfRange, InvalidOrdinal, InvalidVector, Capacity, NonFinite, UnsupportedArchitecture }
impl From<SseError> for Error { fn from(_: SseError)->Self { Self::UnsupportedArchitecture } }
fn finite(v: u32)->bool { f32::from_bits(v).is_finite() }

/// Apply one valid source branch, after validating all inputs before mutation.
pub fn project(positions:&[u32], coordinates:&[[u32;3]], uv:&mut Vec<[u32;2]>, metadata:&mut Vec<[u32;2]>,
               corner_uv:&mut [u32], cache:&mut [CacheEntry], group:&mut Group, corner:usize,
               ordinal:Ordinal, policy:Policy)->Result<u32,Error> {
    let position=*positions.get(corner).ok_or(Error::CornerOutOfRange)? as usize;
    if position>=cache.len() { return Err(Error::PositionOutOfRange); }
    if uv.len()!=metadata.len() { return Err(Error::InvalidVector); }
    if group.bounds.iter().any(|&v|!finite(v)) { return Err(Error::NonFinite); }
    let (selected,index)=match ordinal {
        Ordinal::Append=>{
            let coord=*coordinates.get(position).ok_or(Error::PositionOutOfRange)?;
            if coord.iter().any(|&v|!finite(v)) { return Err(Error::NonFinite); }
            if corner>=corner_uv.len() { return Err(Error::CornerOutOfRange); }
            if uv.len()==uv.capacity() || metadata.len()==metadata.capacity() { return Err(Error::Capacity); }
            let index=u32::try_from(uv.len()).map_err(|_|Error::InvalidVector)?;
            (creature_skin_uv_select_v1::select(coord,group.axis),index)
        }
        Ordinal::Existing(index)=>{
            let selected=*uv.get(index as usize).ok_or(Error::InvalidOrdinal)?;
            if metadata.get(index as usize).is_none() { return Err(Error::InvalidOrdinal); }
            (selected,index)
        }
    };
    if selected.iter().any(|&v|!finite(v)) { return Err(Error::NonFinite); }
    let corner_word=u32::try_from(corner).map_err(|_|Error::CornerOutOfRange)?;
    native_sse::scoped(policy,||{
        let [u,v]=selected; let [lo_u,hi_u,lo_v,hi_v]=group.bounds;
        let bounds=[if f32::from_bits(lo_u)>f32::from_bits(u) {u} else {lo_u},
                    if f32::from_bits(u)>f32::from_bits(hi_u) {u} else {hi_u},
                    if f32::from_bits(lo_v)>f32::from_bits(v) {v} else {lo_v},
                    if f32::from_bits(v)>f32::from_bits(hi_v) {v} else {hi_v}];
        match ordinal {
            Ordinal::Append=>{ uv.push(selected); metadata.push([group.id,corner_word]); corner_uv[corner]=index; }
            Ordinal::Existing(_)=>{ metadata[index as usize]=[group.id,corner_word]; }
        }
        cache[position]=CacheEntry{group:group.id,uv_index:index}; group.bounds=bounds; Ok(index)
    })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests {
 use super::*;
 #[test] fn append_existing_and_reject_transactionally(){
  let mut uv=Vec::with_capacity(2);uv.push([0x3fc00000,0xbfc00000]);let mut meta=Vec::with_capacity(2);meta.push([1,2]);
  let mut corners=[u32::MAX];let mut cache=[CacheEntry{group:0,uv_index:0}];let mut group=Group{id:55,axis:0,bounds:[0xbf000000,0x3f000000,0xbf000000,0x3f000000]};
  assert_eq!(project(&[0],&[[0,0x3f800000,0xbf800000]],&mut uv,&mut meta,&mut corners,&mut cache,&mut group,0,Ordinal::Append,Policy::Mxcsr1f80),Ok(1));
  assert_eq!(uv[1],[0x3f800000,0xbf800000]);assert_eq!(meta[1],[55,0]);assert_eq!(corners,[1]);assert_eq!(group.bounds,[0xbf000000,0x3f800000,0xbf800000,0x3f000000]);
  corners[0]=u32::MAX;assert_eq!(project(&[0],&[],&mut uv,&mut meta,&mut corners,&mut cache,&mut group,0,Ordinal::Existing(0),Policy::Mxcsr1f80),Ok(0));assert_eq!(corners,[u32::MAX]);assert_eq!(meta[0],[55,0]);
  let prior=(uv.clone(),meta.clone(),corners,cache,group);assert_eq!(project(&[0],&[[0,0,0]],&mut uv,&mut meta,&mut corners,&mut cache,&mut group,0,Ordinal::Append,Policy::Mxcsr1f80),Err(Error::Capacity));assert_eq!((uv,meta,corners,cache,group),prior);
 }
}
