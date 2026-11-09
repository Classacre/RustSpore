//! Conditional eight generated-subobject composition of 509B22..509C57.
//! The caller explicitly admits stable same-object normal indices equal to the
//! copied position-index stream and successful preallocated source lifecycle.
//! First four row-derived groups, source allocation and live identity excluded.
use crate::creature_skin_corner_triples_v1::{self,Triples};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct CopiedNormalIndices;
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidGroupCount,InvalidFacePartition,InvalidStreams,Triple(creature_skin_corner_triples_v1::Error)}
/// The fixed fixture has eight nonempty, disjoint groups covering 84 faces.
/// Each group uses an independent position-key cache, as in 5098B0.
pub fn build(positions:&[u32],uv_indices:&[u32],groups:&[&[u32]],_:CopiedNormalIndices)->Result<Vec<Triples>,Error>{
 if groups.len()!=8{return Err(Error::InvalidGroupCount)}
 if positions.len()!=252||uv_indices.len()!=252{return Err(Error::InvalidStreams)}
 let mut seen=[false;84];let mut result=Vec::with_capacity(8);
 for faces in groups{
  if faces.is_empty(){return Err(Error::InvalidFacePartition)}
  for &face in *faces{let slot=seen.get_mut(face as usize).ok_or(Error::InvalidFacePartition)?;if *slot{return Err(Error::InvalidFacePartition)}*slot=true;}
  result.push(creature_skin_corner_triples_v1::build(positions,positions,uv_indices,faces,44).map_err(Error::Triple)?);
 }
 if seen.contains(&false){return Err(Error::InvalidFacePartition)}Ok(result)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn groups_have_independent_caches(){let pos=vec![0;252];let mut uv=vec![0;252];uv[3]=1;let mut gs:Vec<Vec<u32>>=(0..8).map(|_|Vec::new()).collect();for f in 0..84{gs[f%8].push(f as u32);}let refs:Vec<&[u32]>=gs.iter().map(Vec::as_slice).collect();let out=build(&pos,&uv,&refs,CopiedNormalIndices).unwrap();assert_eq!(out.len(),8);assert_eq!(out[0].triples,vec![[0,0,0]]);assert_eq!(out[1].triples,vec![[0,0,1]]);assert_eq!(out[0].corners.len(),33);}
 #[test]fn rejects_bad_partition(){let p=[0;252];let mut gs:Vec<Vec<u32>>=(0..8).map(|_|Vec::new()).collect();for f in 0..84{gs[f%8].push(f as u32);}gs[1][0]=gs[0][0];let refs:Vec<&[u32]>=gs.iter().map(Vec::as_slice).collect();assert_eq!(build(&p,&p,&refs,CopiedNormalIndices),Err(Error::InvalidFacePartition));}
}
