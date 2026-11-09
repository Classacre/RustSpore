//! Controlled partial 53FBA0 edge driver. No queue, triangle mesh or live identity.
use crate::{creature_skin_corner_lookup_v1::SLOT_COUNT,creature_skin_corner_sample_v1 as corner,creature_skin_crossing_v1 as crossing,creature_skin_edge_commit_v1 as commit,creature_skin_edge_lookup_v1 as edge,creature_skin_cube_walk_v1 as cube_walk,native_sse::{self,Policy}};
#[derive(Debug)]pub enum Error{Capacity,Corner,Field,NonFinite,Status,Crossing,Commit,Walk,UnsupportedArchitecture}
impl From<native_sse::SseError> for Error {fn from(_:native_sse::SseError)->Self{Self::UnsupportedArchitecture}}
/// Owned scratch arrays represent a single synthetic worker; no source receiver
/// identity, allocator, queue, or automatic field dispatch is implied.
pub struct Diagnostic {pub corner_keys:Vec<u32>,pub corner_values:Vec<[u32;2]>,pub records:Vec<[u32;6]>,pub corner_count:u32,pub status:u32,pub edge_keys:Vec<[u32;2]>,pub edge_values:Vec<u32>,pub positions:Vec<[f32;3]>,pub counts:Vec<[u32;2]>,pub position_count:usize}
impl Diagnostic {
 pub fn new(corner_capacity:usize,position_capacity:usize)->Self {Self{corner_keys:vec![u32::MAX;SLOT_COUNT],corner_values:vec![[0;2];SLOT_COUNT],records:vec![[0;6];corner_capacity],corner_count:0,status:0,edge_keys:vec![[u32::MAX;2];SLOT_COUNT],edge_values:vec![0;SLOT_COUNT],positions:vec![[0.;3];position_capacity],counts:vec![[0;2];position_capacity],position_count:0}}
 /// Processes one explicitly classified cube through the table walk and edge
 /// diagnostic. The class, field, origin and step are caller-supplied; no queue,
 /// classification verification, finalization or complete mesh is inferred.
 /// On failure previously emitted triangles and worker mutations remain visible.
 pub fn cube<F:FnMut([f32;3])->f32>(&mut self,tables:&cube_walk::Tables,packed:u32,class:u8,triangles:&mut Vec<[u32;3]>,origin:[f32;3],step:f32,threshold:f32,limit:f32,policy:Policy,mut field:F)->Result<(),Error>{
  let mut failure=None;
  let result=cube_walk::walk(tables,packed,class,triangles,|key,corners|match self.edge(key,corners,origin,step,threshold,limit,policy,|p|field(p)){Ok(index)=>Some(index),Err(e)=>{failure=Some(e);None}});
  match result{Ok(())=>Ok(()),Err(_)=>Err(failure.unwrap_or(Error::Walk))}
 }
 /// Ordered cache key and sampled endpoints are independent explicit inputs as in
 /// 53FBA0. Successful finite callback, nonaliasing storage, valid receiver and
 /// 1F80 field policy are admissions. Errors can leave partial state/side effects.
 /// As in the source, an exact all-FF pair can hit an empty-key sentinel; the
 /// cached value then comes from caller-admitted synthetic state. This is not
 /// evidence that such a key occurs in a real source queue.
 pub fn edge<F:FnMut([f32;3])->f32>(&mut self,key:[u32;2],corners:[u32;2],origin:[f32;3],step:f32,threshold:f32,limit:f32,policy:Policy,mut field:F)->Result<u32,Error>{
  if self.edge_values.len()!=SLOT_COUNT {return Err(Error::Capacity)}
  let index=edge::lookup(&self.edge_keys,key[0],key[1]).ok_or(Error::Capacity)?;
  if index<0x8000&&self.edge_keys[index as usize]==key{return Ok(self.edge_values[index as usize]);}
  self.status|=index&0xffff8000;
  let mut samples=[[0u32;2];2];let mut invalid=false;
  for i in 0..2 {corner::sample(corners[i],&mut self.corner_keys,&mut self.corner_values,&mut self.records,&mut self.status,&mut self.corner_count,&mut samples[i],origin,step,policy,|p,out|{let v=field(p);if !v.is_finite(){invalid=true}out[0]=v.to_bits()}).map_err(|_|Error::Corner)?;if invalid{return Err(Error::Field)}}
  if self.status!=0{return Err(Error::Status)}
  let i0=samples[0][1] as usize;let i1=samples[1][1] as usize;if i0>=self.records.len()||i1>=self.records.len(){return Err(Error::Capacity)}
  let p0=[3,4,5].map(|j|f32::from_bits(self.records[i0][j]));let p1=[3,4,5].map(|j|f32::from_bits(self.records[i1][j]));
  let mid=native_sse::scoped(policy,||{let mut p=[0.;3];for i in 0..3 {p[i]=(p0[i]+p1[i])*0.5;}Ok::<_,Error>(p)})?;
  if mid.iter().any(|v|!v.is_finite()){return Err(Error::NonFinite)}let value=field(mid);if !value.is_finite(){return Err(Error::Field)}
  let (start,end,selected)=if value>=0.{(mid,p1,i1)}else{(p0,mid,i0)};
  let refined=crossing::refine(start,end,threshold,limit,policy,|p|Some(field(p)),crossing::Admission).map_err(|_|Error::Crossing)?;
  let mut slots=vec![[0;3];SLOT_COUNT];slots[index as usize][0..2].copy_from_slice(&self.edge_keys[index as usize]);slots[index as usize][2]=self.edge_values[index as usize];
  let vertex=commit::commit(&mut self.records,selected,i0 as u32,&mut self.positions,&mut self.counts,&mut self.position_count,&mut slots,index,key,refined.point,policy).map_err(|_|Error::Commit)?;
  self.edge_keys[index as usize]=key;self.edge_values[index as usize]=vertex;Ok(vertex)
 }
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn two_corners_midpoint_refine_append_then_edge_hit(){let mut d=Diagnostic::new(2,2);let mut seen=Vec::new();let i=d.edge([11,12],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,|p|{seen.push(p);0.75-p[0]}).unwrap();assert_eq!(i,0);assert_eq!(d.corner_count,2);assert_eq!(d.position_count,1);assert_eq!(d.positions[0],[0.75,0.,0.]);assert_eq!(d.counts[0],[1,1]);assert_eq!(seen,vec![[0.,0.,0.],[1.,0.,0.],[0.5,0.,0.],[0.75,0.,0.]]);let j=d.edge([11,12],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,|_|panic!("cache hit" )).unwrap();assert_eq!(j,0);assert_eq!(d.position_count,1);}
 #[test]fn shared_selected_corner_accumulates_distinct_edges(){let mut d=Diagnostic::new(3,2);let f=|p:[f32;3]|0.75-p[0];let a=d.edge([1,2],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,f).unwrap();let b=d.edge([2,3],[0x010000,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,f).unwrap();assert_eq!((a,b,d.position_count,d.counts[0][1]),(0,0,1,2));assert!((d.positions[0][0]-1.5).abs()<0.002);}
 #[test]fn negative_midpoint_selects_first_corner(){let mut d=Diagnostic::new(2,1);let a=d.edge([7,8],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,|p|0.25-p[0]).unwrap();assert_eq!(a,0);assert_eq!(d.counts[0],[0,1]);assert_eq!(d.positions[0],[0.25,0.,0.]);}
 #[test]fn sentinel_exact_match_hit_preserves_existing_state(){let mut d=Diagnostic::new(0,0);let key=[u32::MAX;2];let i=edge::lookup(&d.edge_keys,key[0],key[1]).unwrap() as usize;d.edge_values[i]=91;d.status=7;assert_eq!(d.edge(key,[0;2],[0.;3],0.,0.,0.,Policy::Mxcsr1f80,|_|panic!("hit" )).unwrap(),91);assert_eq!((d.status,d.corner_count,d.position_count),(7,0,0));}
 #[test]fn full_edge_table_samples_corners_before_status_abort(){let mut d=Diagnostic::new(2,1);d.edge_keys.fill([0,0]);let mut seen=Vec::new();assert!(matches!(d.edge([1,2],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,|p|{seen.push(p);0.75-p[0]}),Err(Error::Status)));assert_eq!(seen,vec![[0.,0.,0.],[1.,0.,0.]]);assert_eq!((d.status,d.corner_count,d.position_count),(0xffff8000,2,0));}
 #[test]fn nonfinite_midpoint_and_refinement_do_not_commit(){for bad_call in [3,4]{let mut d=Diagnostic::new(2,1);let mut n=0;assert!(d.edge([1,2],[0,0x020000],[0.;3],0.5,0.01,0.001,Policy::Mxcsr1f80,|p|{n+=1;if n==bad_call{f32::NAN}else{0.75-p[0]}}).is_err());assert_eq!((n,d.corner_count,d.position_count),(bad_call,2,0));}}
 #[test]fn malformed_edge_values_are_rejected(){let mut d=Diagnostic::new(0,0);d.edge_values.clear();assert!(matches!(d.edge([1,2],[0,0],[0.;3],1.,0.,0.,Policy::Mxcsr1f80,|_|panic!()),Err(Error::Capacity)));}
 #[test]fn cube_walk_composes_field_and_shared_edge_cache(){let mut d=Diagnostic::new(8,12);let tables=cube_walk::tables();let mut triangles=Vec::new();let f=|p:[f32;3]|p[0]+p[1]+p[2]-0.6;d.cube(&tables,0,1,&mut triangles,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,f).unwrap();assert!(d.position_count>0);assert!(!triangles.is_empty());assert!(triangles.iter().flatten().all(|&v|(v as usize)<d.position_count));let used=d.position_count;let count=d.corner_count;let previous=triangles.clone();d.cube(&tables,0,1,&mut triangles,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|_|panic!("cached edges" )).unwrap();assert_eq!(d.position_count,used);assert_eq!(d.corner_count,count);assert_eq!(&triangles[previous.len()..],previous.as_slice());}
 #[test]fn exact_midpoint_can_collapse_triangle(){let mut d=Diagnostic::new(8,12);let mut triangles=Vec::new();d.cube(&cube_walk::tables(),0,1,&mut triangles,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|p|p[0]+p[1]+p[2]-0.5).unwrap();assert_eq!(d.position_count,1);assert_eq!(d.counts[0][1],3);assert!(triangles.is_empty());}
 #[test]fn cube_failure_retains_prior_work(){let mut d=Diagnostic::new(1,1);let mut triangles=vec![[7,8,9]];let t=cube_walk::tables();assert!(d.cube(&t,0,1,&mut triangles,[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|p|p[0]+p[1]+p[2]-0.5).is_err());assert_eq!(triangles,vec![[7,8,9]]);assert_eq!(d.corner_count,1);}
 #[test]fn capacity_failure_is_partial_not_rollback(){let mut d=Diagnostic::new(1,1);assert!(d.edge([1,2],[0,0x010000],[0.;3],1.,0.01,0.001,Policy::Mxcsr1f80,|p|0.5-p[0]).is_err());assert_eq!(d.corner_count,1);}
}
