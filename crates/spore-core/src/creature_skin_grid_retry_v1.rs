//! Controlled 414270 retry sequencing on an explicitly admitted query snapshot.
//! Downstream construction and output callbacks are supplied outcomes.
use crate::{creature_skin_grid_geometry_v1::{self as geometry,ArithmeticAdmission},native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)] pub struct Admission;
#[derive(Clone,Copy,Debug)] pub struct Gates {pub setup_ok:bool,pub mesh_ok:bool,pub retry_flag:bool,pub grid_policy:Policy,pub retry_policy:Policy}
#[derive(Clone,Copy,Debug,PartialEq)] pub struct Attempt {pub step:f32,pub lower_before:[f32;3],pub lower_after:[f32;3],pub grid_ok:bool}
#[derive(Debug,PartialEq)] pub struct Output {pub succeeded:bool,pub last_step:f32,pub lower:[f32;3],pub attempts:Vec<Attempt>}
#[derive(Debug,PartialEq,Eq)] pub enum Error {InvalidInput,Geometry(geometry::Error),MissingGates,ExcessAttempts,AllocationFailed,UnsupportedArchitecture}
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
/// Each gate record admits the source outcomes for one successfully constructed
/// grid and the FP policy at both the grid call and subsequent retry multiply.
/// The current implementation supports only declared Policy::Mxcsr1f80; it
/// cannot claim parity with FTZ/global-modified runtime retries. It omits native
/// side effects and cleanup, returns fresh results, and caps attempts at 128.
pub fn run(mut lower:[f32;3],upper:[f32;3],mut step:f32,gates:&[Gates],_:Admission)->Result<Output,Error>{
 if lower.into_iter().chain(upper).chain([step]).any(|v|!v.is_finite()) || step<=0. || (0..3).any(|i|lower[i]>upper[i]) {return Err(Error::InvalidInput);}
 let mut attempts=Vec::new();attempts.try_reserve_exact(gates.len().min(128)).map_err(|_|Error::AllocationFailed)?;
 for i in 0..128 {
  let gate=gates.get(i).ok_or(Error::MissingGates)?;let before=lower;
  let aligned=geometry::align_lower(before,step,gate.grid_policy,ArithmeticAdmission).map_err(Error::Geometry)?;
  match geometry::derive(before,upper,step,gate.grid_policy,ArithmeticAdmission) {
   Ok(_)=>lower=aligned,
   Err(geometry::Error::UnsupportedDimension)=>{lower=aligned;attempts.push(Attempt{step,lower_before:before,lower_after:lower,grid_ok:false});return Ok(Output{succeeded:false,last_step:step,lower,attempts});}
   Err(e)=>return Err(Error::Geometry(e)),
  }
  attempts.push(Attempt{step,lower_before:before,lower_after:lower,grid_ok:true});
  if !gate.setup_ok {return Ok(Output{succeeded:false,last_step:step,lower,attempts});}
  if gate.mesh_ok {return Ok(Output{succeeded:true,last_step:step,lower,attempts});}
  if !gate.retry_flag || step>=0.5 {return Ok(Output{succeeded:false,last_step:step,lower,attempts});}
  step=native_sse::scoped(gate.retry_policy,||Ok::<_,Error>(step*f32::from_bits(0x3f99999a)))?;
  if !step.is_finite(){return Err(Error::InvalidInput);}
 }
 Err(Error::ExcessAttempts)
}
#[cfg(test)] mod tests {
 use super::*;fn g(setup_ok:bool,mesh_ok:bool,retry_flag:bool)->Gates{Gates{setup_ok,mesh_ok,retry_flag,grid_policy:Policy::Mxcsr1f80,retry_policy:Policy::Mxcsr1f80}}
 #[test] fn retry_carries_mutated_lower(){let o=run([-1.;3],[1.;3],0.25,&[g(true,false,true),g(true,true,false)],Admission).unwrap();assert!(o.succeeded);assert_eq!(o.attempts.len(),2);assert_eq!(o.attempts[0].lower_after,[-1.25;3]);assert_eq!(o.attempts[1].lower_before,o.attempts[0].lower_after);assert_eq!(o.last_step.to_bits(),(0.25f32*f32::from_bits(0x3f99999a)).to_bits());}
 #[test]fn gate_failure_and_limit_preserve_last_step(){let o=run([0.;3],[1.;3],0.5,&[g(true,false,true)],Admission).unwrap();assert!(!o.succeeded);assert_eq!(o.attempts.len(),1);assert_eq!(o.last_step,0.5);let o=run([0.;3],[1.;3],0.1,&[g(false,false,true)],Admission).unwrap();assert!(!o.succeeded);assert_eq!(o.last_step,0.1);}
 #[test]fn failed_dimension_still_updates_lower(){let o=run([-1.;3],[300.;3],1.,&[g(true,true,false)],Admission).unwrap();assert!(!o.succeeded);assert_eq!(o.lower,[-2.;3]);assert!(!o.attempts[0].grid_ok);}
 #[test]fn missing_outcome_is_not_a_guessed_success(){assert_eq!(run([0.;3],[1.;3],0.1,&[g(true,false,true)],Admission),Err(Error::MissingGates));}
}
