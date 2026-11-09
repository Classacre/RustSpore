//! Bounded 4F9840 SSE field on scalar-addressed supplied grid records.
//! Does not implement the separate 4F9760 packed-address SIMD variant.
use crate::{creature_generated_skin_grid_query_v1::{self as grid,CELL_COUNT,MAX_RECORDS_PER_CELL},creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)]pub struct SnapshotAdmission;
#[derive(Debug,PartialEq)]pub struct Output{pub address:grid::Address,pub cell_present:bool,pub value:f32}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFiniteIntermediate,EmptyRecordArray,TooManyRecords,Grid(grid::Error),UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// `bias` is the actual finite query +10 word. The constructor normally stores
/// a sign-bit-flipped threshold there, but caller admits its object lifetime.
/// A nonnull native cell needs a positive record count: 4F9840 uses a do-loop.
/// Source pointers/alignment, masked exceptions and finite arithmetic are
/// admitted; status flags and native partial effects are not reproduced.
pub fn evaluate(point:[f32;3],origin:[f32;3],scale:[f32;3],bias:f32,cells:&[Option<&[[Lane;4]]>;CELL_COUNT],policy:Policy,_:SnapshotAdmission)->Result<Output,Error>{
 if !bias.is_finite(){return Err(Error::InvalidInput)}
 let address=grid::address(point,origin,scale,policy).map_err(Error::Grid)?;
 let Some(records)=address.cell.and_then(|i|cells[i]) else{return Ok(Output{address,cell_present:false,value:bias})};
 if records.is_empty(){return Err(Error::EmptyRecordArray)}if records.len()>MAX_RECORDS_PER_CELL{return Err(Error::TooManyRecords)}
 if records.iter().flatten().any(|l|l.center.into_iter().chain([l.cutoff,l.scale,l.coefficient]).any(|x|!x.is_finite())){return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{
  let mut sums=[bias,0.,0.,0.];
  for rec in records {for (i,lane) in rec.iter().enumerate(){
   let dx=lane.center[0]-point[0];let dy=lane.center[1]-point[1];let dz=lane.center[2]-point[2];let xx=dx*dx;let yy=dy*dy;let zz=dz*dz;let xy=xx+yy;let dist=xy+zz;
   let t=dist*lane.scale-1.;let q=t*t;let fourth=q*q;let raw=fourth*lane.coefficient;
   if [dx,dy,dz,xx,yy,zz,xy,dist,t,q,fourth,raw].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate)}
   let weight=if dist<lane.cutoff{raw}else{0.};sums[i]+=weight;if !sums[i].is_finite(){return Err(Error::NonFiniteIntermediate)}
  }}
  let a=sums[0]+sums[2];let b=sums[1]+sums[3];let value=a+b;if [a,b,value].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate)}
  Ok(Output{address,cell_present:true,value})
 })
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn simd_reduction_pairs_lanes_zero_two(){let l=|coef|Lane{center:[0.;3],cutoff:1.,scale:0.,coefficient:coef,key:0};let records=[[l(1e20),l(1.),l(-1e20),l(1.)]];let mut cells:[Option<&[[Lane;4]]>;CELL_COUNT]=[None;CELL_COUNT];cells[0]=Some(&records);let out=evaluate([0.;3],[0.;3],[1.;3],0.,&cells,Policy::Mxcsr1f80,SnapshotAdmission).unwrap();assert_eq!(out.value,2.);}
 #[test]fn null_and_invalid_nonnull_cell(){let mut cells:[Option<&[[Lane;4]]>;CELL_COUNT]=[None;CELL_COUNT];let o=evaluate([0.;3],[0.;3],[1.;3],-0.,&cells,Policy::Mxcsr1f80,SnapshotAdmission).unwrap();assert_eq!(o.value.to_bits(),0x80000000);cells[0]=Some(&[]);assert_eq!(evaluate([0.;3],[0.;3],[1.;3],0.,&cells,Policy::Mxcsr1f80,SnapshotAdmission),Err(Error::EmptyRecordArray));}
}
