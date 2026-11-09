//! Controlled finite 4F9910/4F9A20 scalar gradient on an explicit grid snapshot.
use crate::{creature_generated_skin_grid_query_v1::{self as grid,CELL_COUNT,MAX_RECORDS_PER_CELL},creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)]pub struct GradientLane{pub lane:Lane,/// Source lane +60, explicit rather than inferred from the six-word scalar lane.
 pub derivative:f32}
#[derive(Debug,PartialEq)]pub struct Output{pub gradient:[f32;3],pub usable:bool}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFinite,TooManyRecords,Grid(grid::Error),UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// Caller admits stable native-compatible grid and +60 words, masked exceptions,
/// finite arithmetic and SSE controls. No native dispatch, correction, normal
/// normalization, sticky flags or live identity is inferred.
pub fn evaluate(point:[f32;3],origin:[f32;3],scale:[f32;3],threshold:f32,cells:&[Option<&[[GradientLane;4]]>;CELL_COUNT],policy:Policy)->Result<Output,Error>{
 if !threshold.is_finite(){return Err(Error::InvalidInput)}let a=grid::address(point,origin,scale,policy).map_err(Error::Grid)?;let records=a.cell.and_then(|i|cells[i]);
 if records.is_some_and(|r|r.len()>MAX_RECORDS_PER_CELL){return Err(Error::TooManyRecords)}
 if let Some(r)=records{if r.iter().flatten().any(|x|x.lane.center.into_iter().chain([x.lane.cutoff,x.lane.scale,x.lane.coefficient,x.derivative]).any(|v|!v.is_finite())){return Err(Error::InvalidInput)}}
 native_sse::scoped(policy,||{let mut gradient=[0f32;3];let mut value=f32::from_bits(threshold.to_bits()^0x80000000);if let Some(records)=records{for record in records{for item in record{let l=item.lane;if l.cutoff==0.{continue}let d=[point[0]-l.center[0],point[1]-l.center[1],point[2]-l.center[2]];let xx=d[0]*d[0];let yy=d[1]*d[1];let zz=d[2]*d[2];let distance=(xx+yy)+zz;if d.into_iter().chain([xx,yy,zz,distance]).any(|v|!v.is_finite()){return Err(Error::NonFinite)}if distance>=l.cutoff{continue}let t=distance*l.scale-1.;let q=t*t;let slope=((8.*item.derivative)*q)*t;let weight=(l.coefficient*q)*q;value=weight+value;for i in 0..3{gradient[i]=slope*d[i]+gradient[i];}if [t,q,slope,weight,value].into_iter().chain(gradient).any(|v|!v.is_finite()){return Err(Error::NonFinite)}}}}
 let xx=gradient[0]*gradient[0];let yy=gradient[1]*gradient[1];let zz=gradient[2]*gradient[2];let magnitude=(xx+yy)+zz;if !magnitude.is_finite(){return Err(Error::NonFinite)}Ok(Output{gradient,usable:magnitude>f32::from_bits(0x322bcc77)})})
}
#[cfg(test)]mod tests{use super::*;fn lane()->GradientLane{GradientLane{lane:Lane{center:[0.;3],cutoff:4.,scale:0.25,coefficient:1.,key:0},derivative:0.25}}
 #[test]fn gradient_and_cutoff(){let r=[[lane();4]];let mut cells:[Option<&[[GradientLane;4]]>;CELL_COUNT]=[None;CELL_COUNT];cells[0]=Some(&r);let x=evaluate([1.,0.,0.],[0.;3],[0.;3],0.01,&cells,Policy::Mxcsr1f80).unwrap();assert_eq!(x.gradient,[-3.375,0.,0.]);assert!(x.usable);let x=evaluate([2.,0.,0.],[0.;3],[0.;3],0.01,&cells,Policy::Mxcsr1f80).unwrap();assert_eq!(x.gradient,[0.;3]);assert!(!x.usable);}
 #[test]fn null_and_nonfinite(){let cells=[None;CELL_COUNT];assert!(!evaluate([0.;3],[0.;3],[1.;3],0.01,&cells,Policy::Mxcsr1f80).unwrap().usable);assert_eq!(evaluate([0.;3],[0.;3],[1.;3],f32::NAN,&cells,Policy::Mxcsr1f80),Err(Error::InvalidInput));}
}
