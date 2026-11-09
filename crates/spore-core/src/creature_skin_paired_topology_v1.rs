//! Checked paired-incidence subset of the 509D30 directed-link builder.
//!
//! Inputs are explicit triangle position indices. This accepts only referenced
//! vertices and exactly two oppositely directed, nondegenerate incidences per
//! edge. It does not model source hashing, allocation, nonmanifold repair,
//! partial writes, runtime object identity, or smoothing configuration.
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Topology {
    /// Source +94: linked corner index for each directed triangle edge.
    pub links: Vec<u32>,
    /// Source +A8: last source corner index visited for each vertex.
    pub starts: Vec<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidLength, InvalidVertex, DegenerateEdge, NonPairedEdge, UnreferencedVertex, TooManyCorners }

/// Construct source-ordered links for a closed, consistently oriented paired mesh.
/// This checked transaction rejects states that native code may partially mutate.
pub fn build(indices: &[u32], vertex_count: usize) -> Result<Topology, Error> {
    if indices.is_empty() || indices.len() % 3 != 0 || vertex_count == 0 { return Err(Error::InvalidLength); }
    if indices.len() > u32::MAX as usize || vertex_count > u32::MAX as usize { return Err(Error::TooManyCorners); }
    if vertex_count > indices.len() { return Err(Error::UnreferencedVertex); }
    if indices.iter().any(|&x| x as usize >= vertex_count) { return Err(Error::InvalidVertex); }
    let mut links = vec![u32::MAX; indices.len()];
    let mut starts = vec![u32::MAX; vertex_count];
    let mut edges: BTreeMap<(u32,u32), (usize,u32,u32)> = BTreeMap::new();
    for base in (0..indices.len()).step_by(3) {
        for (from,to) in [(base+2,base),(base,base+1),(base+1,base+2)] {
            let a=indices[from]; let b=indices[to];
            if a==b { return Err(Error::DegenerateEdge); }
            starts[a as usize]=from as u32;
            let key=(a.min(b),a.max(b));
            if let Some(&(prior, old_a, old_b))=edges.get(&key) {
                if (a,b)!=(old_b,old_a) || links[prior] != prior as u32 { return Err(Error::NonPairedEdge); }
                links[prior]=from as u32;links[from]=prior as u32;
            } else { links[from]=from as u32;edges.insert(key,(from,a,b)); }
        }
    }
    if starts.contains(&u32::MAX) { return Err(Error::UnreferencedVertex); }
    if links.iter().enumerate().any(|(i,&n)|n==u32::MAX || n==i as u32) { return Err(Error::NonPairedEdge); }
    Ok(Topology { links, starts })
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn oriented_tetrahedron_pairs_and_preserves_latest_source_corner() {
  let t=build(&[0,2,1, 0,1,3, 1,2,3, 2,0,3],4).unwrap();
  assert_eq!(t.starts,vec![10,6,9,11]);
  for (i,&j) in t.links.iter().enumerate() { assert_ne!(i,j as usize);assert_eq!(t.links[j as usize],i as u32); }
 }
 #[test] fn rejects_boundaries_nonmanifold_and_unused_vertices() {
  assert_eq!(build(&[0,1,2],usize::MAX),Err(Error::TooManyCorners));
  assert_eq!(build(&[0,1,2],4),Err(Error::UnreferencedVertex));
  assert_eq!(build(&[0,1,2],3),Err(Error::NonPairedEdge));
  assert_eq!(build(&[0,1,1],2),Err(Error::DegenerateEdge));
  assert_eq!(build(&[0,1,3],3),Err(Error::InvalidVertex));
  assert_eq!(build(&[0,2,1,0,1,3,1,2,3,2,0,3],5),Err(Error::UnreferencedVertex));
  assert_eq!(build(&[0,2,1,0,1,3,1,2,3,2,0,3,0,1,2],4),Err(Error::NonPairedEdge));
 }
}
