//! E5E100 per-candidate route, with optional bounded E50080 curve composition.
//! Candidate rows and effect-pool lifetime are supplied snapshots, not a live
//! iterator, allocation, callback or placement binding.
use crate::{cell_effect_curve_v1::{self as curve,Curve},cell_start_table_v1::Row,native_sse::Policy,native_x87::Context};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Route { NoMatch, PendingZeroEffectInspection, PendingNonzeroPlacement }
#[derive(Debug,PartialEq)]
pub enum Error { UnstableCandidate, NonfiniteWeight, UnsupportedInstalledLevel, Curve(curve::Error) }

/// Finite weight and stable candidate words are extra checked admissions. The
/// native prefix tests kind and level first, then routes both signed zeros to
/// cleanup inspection and all finite nonzero values to placement. No +300 field
/// is read here, and neither pending operation is called.
pub fn route(kind:u32,candidate_level:u32,row_level:u32,weight_bits:u32,candidate_stable:bool)->Result<Route,Error>{
 if !candidate_stable{return Err(Error::UnstableCandidate)}
 if !f32::from_bits(weight_bits).is_finite(){return Err(Error::NonfiniteWeight)}
 if kind!=1||candidate_level!=row_level{return Ok(Route::NoMatch)}
 Ok(if weight_bits&0x7fffffff==0{Route::PendingZeroEffectInspection}else{Route::PendingNonzeroPlacement})
}

/// The caller admits the same authenticated installed row and stable scalar
/// used by the separately bounded curve model. Near and Far are the two
/// admitted type-5 rows whose +18 levels are 3 and 11 respectively. This does
/// not discover a row, progress value, effect-pool slot or live provider.
pub fn from_installed_curve(curve_id:Curve,row_level:u32,scalar:f32,ctx:Context,kind:u32,candidate_level:u32,candidate_stable:bool)->Result<(u32,Route),Error>{
 let expected=match curve_id{Curve::Near=>3,Curve::Far=>11};
 if row_level!=expected{return Err(Error::UnsupportedInstalledLevel)}
 let weight=curve::evaluate(curve_id,scalar,ctx).map_err(Error::Curve)?.to_bits();
 Ok((weight,route(kind,candidate_level,row_level,weight,candidate_stable)?))
}

/// Compose the separately checked start-table progress projection, authenticated
/// curve and caller-supplied candidate. The caller admits the same stable table,
/// row, global words and candidate snapshot; no current runtime state is read.
pub fn from_progress(progress:i32,rows:&[Row],special_bits:u32,numerator_bits:u32,policy:Policy,ctx:Context,curve_id:Curve,row_level:u32,kind:u32,candidate_level:u32,candidate_stable:bool)->Result<(u32,u32,Route),Error>{
 let expected=match curve_id{Curve::Near=>3,Curve::Far=>11};if row_level!=expected{return Err(Error::UnsupportedInstalledLevel)}
 let (scalar,near,far)=curve::from_start_table(progress,rows,special_bits,numerator_bits,policy,ctx).map_err(Error::Curve)?;
 let weight=match curve_id{Curve::Near=>near,Curve::Far=>far}.to_bits();
 Ok((scalar.to_bits(),weight,route(kind,candidate_level,row_level,weight,candidate_stable)?))
}

#[cfg(test)] mod tests{
 use super::*;
 #[test] fn gates_and_signed_zero_route_to_pending_boundaries(){
  for w in [0,0x80000000]{assert_eq!(route(1,3,3,w,true),Ok(Route::PendingZeroEffectInspection));}
  for w in [1f32.to_bits(),(-1f32).to_bits()]{assert_eq!(route(1,3,3,w,true),Ok(Route::PendingNonzeroPlacement));}
  assert_eq!(route(0,3,3,1f32.to_bits(),true),Ok(Route::NoMatch));assert_eq!(route(1,4,3,1f32.to_bits(),true),Ok(Route::NoMatch));
  assert_eq!(route(1,3,3,f32::NAN.to_bits(),true),Err(Error::NonfiniteWeight));assert_eq!(route(1,3,3,0,false),Err(Error::UnstableCandidate));
 }
 #[test] fn admitted_installed_curve_composes_without_selecting_pool(){let c=Context::from_control_word(0x027f).unwrap();assert_eq!(from_installed_curve(Curve::Near,3,0.,c,1,3,true),Ok((1f32.to_bits(),Route::PendingNonzeroPlacement)));assert_eq!(from_installed_curve(Curve::Near,3,24000.,c,1,3,true),Ok((0,Route::PendingZeroEffectInspection)));assert_eq!(from_installed_curve(Curve::Far,11,14950.,c,1,11,true),Ok((1f32.to_bits(),Route::PendingNonzeroPlacement)));assert_eq!(from_installed_curve(Curve::Near,11,0.,c,1,11,true),Err(Error::UnsupportedInstalledLevel));}
 #[test] fn admitted_progress_composes_start_curve_and_candidate(){let c=Context::from_control_word(0x027f).unwrap();let out=from_progress(1000,&[],2f32.to_bits(),4f32.to_bits(),Policy::Mxcsr1f80,c,Curve::Near,3,1,3,true).unwrap();assert_eq!(out,(1f32.to_bits(),1f32.to_bits(),Route::PendingNonzeroPlacement));}
}
