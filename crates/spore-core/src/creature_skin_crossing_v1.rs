//! Controlled 4FC7C0 crossing refinement with a caller-supplied signed field.
//! No native queue, field dispatch, surface topology or source identity implied.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug)]pub struct Admission;
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Sample{pub t:f32,pub point:[f32;3],pub field:f32}
#[derive(Debug,PartialEq)]pub struct Output{pub point:[f32;3],pub t:f32,pub samples:Vec<Sample>}
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFiniteIntermediate,Callback,AllocationFailed,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// Caller's callback must reproduce the chosen query +08 field and preserve the
/// admitted FP controls; it may have side effects even if this wrapper fails.
/// The endpoints are ordered as the native caller supplies them. This function
/// does not test or enforce a valid sign bracket. Native pointer/stack validity,
/// masked exceptions and exact finite x87 comparison behavior are admissions.
pub fn refine<F>(start:[f32;3],end:[f32;3],threshold:f32,limit:f32,policy:Policy,mut field:F,_:Admission)->Result<Output,Error>
where F:FnMut([f32;3])->Option<f32>{
 if start.into_iter().chain(end).chain([threshold,limit]).any(|v|!v.is_finite()){return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{
  let mut delta=[0.;3];for i in 0..3{delta[i]=end[i]-start[i];if !delta[i].is_finite(){return Err(Error::NonFiniteIntermediate)}}
  let scaled=f32::from_bits(0x3dcccccd)*threshold;if !scaled.is_finite(){return Err(Error::NonFiniteIntermediate)}let tolerance=if scaled<=limit{scaled}else{limit};
  let mut lo=0f32;let mut hi=1f32;let mut samples=Vec::new();samples.try_reserve_exact(20).map_err(|_|Error::AllocationFailed)?;
  for _ in 0..20 {let t=(lo+hi)*0.5;let mut point=[0.;3];for i in 0..3 {let p=delta[i]*t;point[i]=start[i]+p;if !p.is_finite()||!point[i].is_finite(){return Err(Error::NonFiniteIntermediate)}}
   let v=field(point).ok_or(Error::Callback)?;if !v.is_finite(){return Err(Error::NonFiniteIntermediate)}samples.push(Sample{t,point,field:v});if v<0.{hi=t}else{lo=t}if v.abs()<=tolerance{return Ok(Output{point,t,samples})}
  }
  let last=*samples.last().expect("20 iterations");Ok(Output{point:last.point,t:last.t,samples})
 })
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn exact_midpoint_stops_after_one(){let o=refine([0.;3],[1.,0.,0.],0.01,0.001,Policy::Mxcsr1f80,|p|Some(0.5-p[0]),Admission).unwrap();assert_eq!(o.t,0.5);assert_eq!(o.samples.len(),1);assert_eq!(o.point,[0.5,0.,0.]);}
 #[test]fn sign_moves_correct_bound_and_zero_limit_exhausts(){let o=refine([0.;3],[1.,0.,0.],0.,0.,Policy::Mxcsr1f80,|p|Some(0.3-p[0]),Admission).unwrap();assert_eq!(o.samples.len(),20);assert_eq!(o.samples[0].t,0.5);assert_eq!(o.samples[1].t,0.25);assert!((o.t-0.3).abs()<0.000002);}
 #[test]fn callback_failure_propagates(){assert_eq!(refine([0.;3],[1.;3],1.,1.,Policy::Mxcsr1f80,|_|None,Admission),Err(Error::Callback));}
}
