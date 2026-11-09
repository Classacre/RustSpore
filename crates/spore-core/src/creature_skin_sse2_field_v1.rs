//! Bounded 4F9760 SSE2 signed field with packed/saturating table addressing.
//! Out-of-table native pointer arithmetic is rejected before dereference.
use crate::{creature_generated_skin_grid_query_v1::{CELL_COUNT,MAX_RECORDS_PER_CELL},creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)]pub struct SnapshotAdmission;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Address{pub coordinates:[i32;4],pub saturated:[i16;4],pub byte_offset:u32,pub cell:usize}
#[derive(Debug,PartialEq)]pub struct Output{pub address:Address,pub cell_present:bool,pub value:f32}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFiniteIntermediate,UnsupportedConversion,OutOfTable,EmptyRecordArray,TooManyRecords,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// The original dereferences without checking table bounds. This checked model
/// accepts only offsets within a 512-entry snapshot and rejects exceptional
/// conversions, nonfinite arithmetic and invalid record lengths. Native x86
/// 32-bit pointer wrapping, valid 16-byte point/origin/scale storage, aligned
/// records, masked exceptions and an SSE2-capable dispatch are admissions.
pub fn address(point:[f32;4],origin:[f32;4],scale:[f32;4],policy:Policy)->Result<Address,Error>{
 if point.into_iter().chain(origin).chain(scale).any(|x|!x.is_finite()){return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{
  let mut coordinates=[0i32;4];let mut saturated=[0i16;4];
  for i in 0..4{let d=point[i]-origin[i];let v=d*scale[i];if !d.is_finite()||!v.is_finite(){return Err(Error::NonFiniteIntermediate)}if v< -2147483648. || v>=2147483648. {return Err(Error::UnsupportedConversion)}coordinates[i]=v as i32;saturated[i]=coordinates[i].clamp(i16::MIN as i32,i16::MAX as i32) as i16;}
  let xy=(saturated[0] as i32)*256+(saturated[1] as i32)*32;let z=((saturated[2] as i32)*4) as u16 as u32;
  let byte_offset=(xy as u32).wrapping_add(z);if byte_offset>=2048 || byte_offset%4!=0{return Err(Error::OutOfTable)}
  Ok(Address{coordinates,saturated,byte_offset,cell:(byte_offset/4) as usize})
 })
}
/// `bias` is the query +10 word, not an inferred threshold. The source computes
/// every SIMD lane before applying cutoff masks and reduces pairwise as
/// `(sum0+sum3)+(sum1+sum2)`. It requires a positive nonnull record count.
pub fn evaluate(point:[f32;4],origin:[f32;4],scale:[f32;4],bias:f32,cells:&[Option<&[[Lane;4]]>;CELL_COUNT],policy:Policy,_:SnapshotAdmission)->Result<Output,Error>{
 if !bias.is_finite(){return Err(Error::InvalidInput)}let address=address(point,origin,scale,policy)?;
 let Some(records)=cells[address.cell] else{return Ok(Output{address,cell_present:false,value:bias})};if records.is_empty(){return Err(Error::EmptyRecordArray)}if records.len()>MAX_RECORDS_PER_CELL{return Err(Error::TooManyRecords)}
 if records.iter().flatten().any(|l|l.center.into_iter().chain([l.cutoff,l.scale,l.coefficient]).any(|x|!x.is_finite())){return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{let mut sums=[bias,0.,0.,0.];for rec in records{for (i,l) in rec.iter().enumerate(){let dx=l.center[0]-point[0];let dy=l.center[1]-point[1];let dz=l.center[2]-point[2];let xx=dx*dx;let yy=dy*dy;let zz=dz*dz;let xy=xx+yy;let dist=xy+zz;let t=dist*l.scale-1.;let q=t*t;let fourth=q*q;let raw=fourth*l.coefficient;if [dx,dy,dz,xx,yy,zz,xy,dist,t,q,fourth,raw].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate)}let w=if dist<l.cutoff{raw}else{0.};sums[i]+=w;if !sums[i].is_finite(){return Err(Error::NonFiniteIntermediate)}}}let a=sums[0]+sums[3];let b=sums[1]+sums[2];let value=a+b;if [a,b,value].into_iter().any(|x|!x.is_finite()){return Err(Error::NonFiniteIntermediate)}Ok(Output{address,cell_present:true,value})})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn aliases_preserve_extract_low_word(){let o=[0.;4];let s=[1.;4];assert_eq!(address([-1.,8.,0.,0.],o,s,Policy::Mxcsr1f80).unwrap().cell,0);assert_eq!(address([-255.,0.,-1.,0.],o,s,Policy::Mxcsr1f80).unwrap().cell,63);assert_eq!(address([0.,0.,-1.,0.],o,s,Policy::Mxcsr1f80),Err(Error::OutOfTable));}
 #[test]fn rejects_out_of_range_and_conversion(){assert_eq!(address([32768.,0.,0.,0.],[0.;4],[1.;4],Policy::Mxcsr1f80),Err(Error::OutOfTable));assert_eq!(address([2147483648.,0.,0.,0.],[0.;4],[1.;4],Policy::Mxcsr1f80),Err(Error::UnsupportedConversion));}
 #[test]fn reduction_pairs_zero_three(){let l=|c|Lane{center:[0.;3],cutoff:1.,scale:0.,coefficient:c,key:0};let r=[[l(1e20),l(1.),l(1.),l(-1e20)]];let mut cells:[Option<&[[Lane;4]]>;CELL_COUNT]=[None;CELL_COUNT];cells[0]=Some(&r);assert_eq!(evaluate([0.;4],[0.;4],[1.;4],0.,&cells,Policy::Mxcsr1f80,SnapshotAdmission).unwrap().value,2.);}
}
