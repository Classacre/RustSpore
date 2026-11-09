//! Bounded scalar 4F9440/4F94E0 signed field on supplied grid records.
//! This is separate from positive influence appends and SIMD field variants.
use crate::{creature_generated_skin_grid_query_v1::{self as grid,CELL_COUNT,MAX_RECORDS_PER_CELL},creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
#[derive(Clone,Copy,Debug)] pub struct SnapshotAdmission;
#[derive(Debug,PartialEq)] pub struct Output {pub address:grid::Address,pub cell_present:bool,pub value:f32}
#[derive(Debug,PartialEq,Eq)] pub enum Error {InvalidInput,NonFiniteIntermediate,TooManyRecords,Grid(grid::Error),X87(X87Error),UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
impl From<X87Error> for Error{fn from(x:X87Error)->Self{Self::X87(x)}}
/// Caller admits valid stable pointers/record count, scalar dispatch, masked FP
/// exceptions and x87 stack. No native status flags or partial effects modeled.
/// Each record accumulates its four lanes with SSE, then x87 adds that record
/// sum to the sign-bit-flipped threshold and stores binary32 before the next.
pub fn evaluate(point:[f32;3],origin:[f32;3],scale:[f32;3],threshold:f32,cells:&[Option<&[[Lane;4]]>;CELL_COUNT],sse:Policy,x87:Context,_:SnapshotAdmission)->Result<Output,Error>{
 if !threshold.is_finite(){return Err(Error::InvalidInput);}
 let address=grid::address(point,origin,scale,sse).map_err(Error::Grid)?;
 let records=address.cell.and_then(|i|cells[i]);let mut value=f32::from_bits(threshold.to_bits()^0x8000_0000);
 let Some(records)=records else{return Ok(Output{address,cell_present:false,value});};
 if records.len()>MAX_RECORDS_PER_CELL{return Err(Error::TooManyRecords);}
 if records.iter().flatten().any(|l|l.center.into_iter().chain([l.cutoff,l.scale,l.coefficient]).any(|x|!x.is_finite())){return Err(Error::InvalidInput);}
 native_sse::scoped(sse,||{
  for record in records {
   let mut sum=0f32;
   for lane in record {
    if lane.cutoff.to_bits()&0x7fff_ffff==0{continue;}
    let dx=point[0]-lane.center[0];let dy=point[1]-lane.center[1];let dz=point[2]-lane.center[2];let xx=dx*dx;let yy=dy*dy;let zz=dz*dz;let xy=xx+yy;let distance=xy+zz;
    if [dx,dy,dz,xx,yy,zz,xy,distance].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate);}
    if lane.cutoff<=distance{continue;}
    let t=distance*lane.scale-1.;let q=t*t;let first=lane.coefficient*q;let weight=first*q;sum=weight+sum;
    if [t,q,first,weight,sum].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate);}
   }
   value=x87.store_f32(x87.add(Extended::from_f32(sum)?,Extended::from_f32(value)?)?)?;
  }
  Ok(Output{address,cell_present:true,value})
 })
}
#[cfg(test)]mod tests{
 use super::*;use crate::native_x87::{Precision,Rounding};fn ctx()->Context{Context{precision:Precision::Bits53,rounding:Rounding::NearestEven}}
 #[test]fn signed_field_is_not_positive_only_query(){let pos=Lane{center:[0.;3],cutoff:1.,scale:0.,coefficient:1.,key:0};let neg=Lane{coefficient:-0.25,..pos};let zero=Lane{cutoff:0.,..pos};let record=[[pos,neg,zero,zero]];let mut cells:[Option<&[[Lane;4]]>;CELL_COUNT]=[None;CELL_COUNT];cells[0]=Some(&record);let o=evaluate([0.;3],[0.;3],[1.;3],0.5,&cells,Policy::Mxcsr1f80,ctx(),SnapshotAdmission).unwrap();assert_eq!(o.value,0.25);}
 #[test]fn strict_cutoff_and_null_sign(){let lane=Lane{center:[0.;3],cutoff:1.,scale:0.,coefficient:1.,key:0};let r=[[lane;4]];let mut cells:[Option<&[[Lane;4]]>;CELL_COUNT]=[None;CELL_COUNT];cells[0]=Some(&r);let o=evaluate([1.,0.,0.],[-1.,0.,0.],[0.;3],0.5,&cells,Policy::Mxcsr1f80,ctx(),SnapshotAdmission).unwrap();assert_eq!(o.value,-0.5);let o=evaluate([0.;3],[0.;3],[1.;3],0.,&[None;CELL_COUNT],Policy::Mxcsr1f80,ctx(),SnapshotAdmission).unwrap();assert_eq!(o.value.to_bits(),0x8000_0000);}
}
