//! Checked 4F9D60 spatial-box arithmetic on explicit finite bounds and step.
//! This does not select the step, reproduce retries, or establish complete lanes.
use crate::{creature_generated_skin_grid_pack_v1::{self as pack,CellGeometry,PackedGrid,CandidateAdmission},creature_generated_skin_grid_query_v1::CELL_COUNT,creature_generated_skin_kernel_v1::Lane,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)] pub struct ArithmeticAdmission;
#[derive(Debug,PartialEq,Eq)] pub enum Error {InvalidInput,NonFinite,UnsupportedDimension,UnsupportedArchitecture,Filter(pack::FilterError)}
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
pub struct Geometry {pub origin:[f32;3],pub scale:[f32;3],pub cells:[CellGeometry;CELL_COUNT]}
/// Models the modf/ceil path for bounded binary32 operands under x87 027F and
/// MXCSR 1F80; CRT modf/ceil results and masked exceptions are admitted. Rejects
/// large decomposition operands and any inverted bounds more strictly than native.
fn split(v:f32)->Result<(f32,f32),Error>{
 if !v.is_finite() || v.abs()>1_000_000. {return Err(Error::NonFinite);}
 let x=if v<0.0 {(v as f64)-1.0}else{v as f64};let whole=x.trunc();let frac=(x-whole) as f32;
 let fraction=if v<0.0 {1.0+frac}else{frac};Ok((whole as f32,fraction))
}
/// Align the lower bound as the builder does before its dimension checks.
/// This is separately exposed for the retry controller's state mutation.
pub fn align_lower(lower:[f32;3],step:f32,policy:Policy,_:ArithmeticAdmission)->Result<[f32;3],Error>{
 if lower.into_iter().chain([step]).any(|x|!x.is_finite()) || step<=0. {return Err(Error::InvalidInput);}
 native_sse::scoped(policy,||{let inv=1.0/step;if !inv.is_finite(){return Err(Error::NonFinite);}let mut out=[0.;3];for i in 0..3{let (whole,_)=split(lower[i]*inv)?;out[i]=whole*step;if !out[i].is_finite(){return Err(Error::NonFinite);}}Ok(out)})
}
/// Return grid origin/scale and 512 x-major/z-minor boxes from admitted bounds.
/// The size words and lane threshold remain distinct from the caller's step.
pub fn derive(lower:[f32;3],upper:[f32;3],step:f32,policy:Policy,_:ArithmeticAdmission)->Result<Geometry,Error>{
 if lower.into_iter().chain(upper).chain([step]).any(|x|!x.is_finite()) || step<=0. || (0..3).any(|i|lower[i]>upper[i]) {return Err(Error::InvalidInput);}
 native_sse::scoped(policy,||{
  let inv=1.0/step;if !inv.is_finite(){return Err(Error::NonFinite);}
  let mut origin=[0.;3];let mut scale=[0.;3];let mut spacing=[0.;3];let mut first=[0.;3];let mut half=[0.;3];
  for i in 0..3 {
   let (integer,_)=split(lower[i]*inv)?;let aligned=integer*step;
   let span=(upper[i]-aligned)*inv;let (whole,frac)=split(span)?;let dim=whole+frac.ceil();
   if !dim.is_finite()||dim<1.||dim>=256. {return Err(Error::UnsupportedDimension);}
   let world=step*dim;let high=aligned+world;let delta=high-aligned;let margin=f32::from_bits(0x38d1b717)*delta;
   origin[i]=aligned-margin;let expanded=delta+2.0*margin;
   scale[i]=8.0/expanded;spacing[i]=expanded*0.125;first[i]=aligned+spacing[i]*0.5;half[i]=expanded*0.0625+f32::from_bits(0x38d1b717);
   if [aligned,span,world,high,delta,margin,origin[i],expanded,scale[i],spacing[i],first[i],half[i]].into_iter().any(|x|!x.is_finite()) || expanded<=0. {return Err(Error::NonFinite);}
  }
  let cells=std::array::from_fn(|idx| {let x=(idx/64) as f32;let y=((idx/8)%8) as f32;let z=(idx%8) as f32;CellGeometry{center:[x*spacing[0]+first[0],y*spacing[1]+first[1],z*spacing[2]+first[2]],half_extent:half}});
  if cells.iter().any(|c|c.center.into_iter().any(|x|!x.is_finite())){return Err(Error::NonFinite);}
  Ok(Geometry{origin,scale,cells})
 })
}
/// Derive, filter and pack explicit lanes. Bounds and threshold must correspond
/// to those same prepared lanes; incomplete lanes yield only a partial model.
pub fn derive_and_pack(lanes:&[Lane],lower:[f32;3],upper:[f32;3],step:f32,threshold:f32,policy:Policy,admit:ArithmeticAdmission)->Result<(Geometry,PackedGrid),Error>{
 let g=derive(lower,upper,step,policy,admit)?;let p=pack::filter_and_pack(lanes,&g.cells,threshold,policy,CandidateAdmission).map_err(Error::Filter)?;Ok((g,p))
}
#[cfg(test)] mod tests {
 use super::*;
 #[test]fn negative_integer_decomposition_preserves_native_extra_cell(){assert_eq!(split(-1.).unwrap(),(-2.,1.));assert_eq!(split(-1.25).unwrap(),(-2.,0.75));assert_eq!(split(2.25).unwrap(),(2.,0.25));}
 #[test]fn nondegenerate_bounds_form_ordered_boxes_and_pack(){let l=Lane{center:[0.5;3],cutoff:100.,scale:0.,coefficient:1.,key:7};let (g,p)=derive_and_pack(&[l],[0.;3],[1.;3],0.5,0.,Policy::Mxcsr1f80,ArithmeticAdmission).unwrap();assert!(g.origin[0]<0.);assert_eq!(g.cells[0].center[0].to_bits(),0x3d80068e);assert!(g.cells[1].center[2]>g.cells[0].center[2]);assert!(g.cells[8].center[1]>g.cells[0].center[1]);assert!(g.cells[64].center[0]>g.cells[0].center[0]);assert!(p.cells().iter().all(Option::is_some));}
 #[test]fn rejects_invalid_or_excess_dimensions(){assert!(matches!(derive([1.;3],[0.;3],1.,Policy::Mxcsr1f80,ArithmeticAdmission),Err(Error::InvalidInput)));assert!(matches!(derive([0.;3],[256.;3],1.,Policy::Mxcsr1f80,ArithmeticAdmission),Err(Error::UnsupportedDimension)));}
}
