//! Checked E5E19E..E5E26E arithmetic projection. Source object lookup,
//! E51E50 effects, placement, allocation and callbacks remain outside scope.
use crate::{cell_effect_pool_route_v1::Route,native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};

#[derive(Clone,Copy,Debug)]
pub struct Input<'a>{
 pub route:Route,pub level:u32,pub grid:[i32;2],pub spacing_bits:&'a [u32;16],
 pub helper_return_bits:u32,pub global_514c_bits:u32,pub effect_present:bool,
 pub stable_disjoint:bool,
}
#[derive(Debug,PartialEq)]
pub enum Error{Unstable,WrongRoute,LevelOutOfRange,GridOverflow,NonPositiveOrNonFinite,NonFiniteIntermediate,X87(X87Error),Sse(SseError)}
impl From<X87Error> for Error{fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Projection{pub corner_words:[u32;6],pub pending_existing:bool}

/// Candidate, table, +514C and helper result are separately supplied stable
/// snapshots. Helper call is not run; its x87 return and preservation of source
/// intermediate XMM values are admitted. Positive finite values and checked
/// coordinate increments are additional safeguards. The source computes six
/// binary32 stack words then branches on the supplied +300 presence bit.
pub fn project(x:Input<'_>,policy:Policy,ctx:Context)->Result<Projection,Error>{
 if !x.stable_disjoint{return Err(Error::Unstable)}
 if x.route!=Route::PendingNonzeroPlacement{return Err(Error::WrongRoute)}
 let spacing=f32::from_bits(*x.spacing_bits.get(x.level as usize).ok_or(Error::LevelOutOfRange)?);
 let callback=f32::from_bits(x.helper_return_bits);let global=f32::from_bits(x.global_514c_bits);
 if ![spacing,callback,global].iter().all(|v|v.is_finite()&&*v>0.){return Err(Error::NonPositiveOrNonFinite)}
 let next=[x.grid[0].checked_add(1).ok_or(Error::GridOverflow)?,x.grid[1].checked_add(1).ok_or(Error::GridOverflow)?];
 native_sse::scoped(policy,||{
  use std::hint::black_box as b;
  let current_x=b(x.grid[0]) as f32;let current_y=b(x.grid[1]) as f32;
  let origin=b(x.level as f32)*b(0.0f32);
  let next_x=b(next[0]) as f32;let next_y=b(next[1]) as f32;
  let x0_product=b(current_x)*b(spacing);let y0_product=b(current_y)*b(spacing);
  let x1_product=b(next_x)*b(spacing);let y1_product=b(next_y)*b(spacing);
  let x0=b(x0_product)+b(origin);let y0=b(y0_product)+b(origin);
  let x1=b(x1_product)-b(origin);let y1=b(y1_product)-b(origin);
  let scaled_callback=ctx.store_f32(ctx.mul(Extended::from_f32(callback)?,Extended::from_f32(f32::from_bits(0x3f59999a))?)?)?;
  if !scaled_callback.is_finite()||scaled_callback<=0.{return Err(Error::NonFiniteIntermediate)}
  let ratio=b(global)/b(scaled_callback);let unit=b(1.0f32)/b(ratio);
  let scaled_y0=b(y0)*b(unit);let z0=b(unit)*b(0.0);
  let scaled_x1=b(x1)*b(unit);let scaled_y1=b(y1)*b(unit);
  let scaled_x0=b(unit)*b(x0);let z1=b(unit)*b(0.0);
  let values=[scaled_x0,scaled_y0,z0,scaled_x1,scaled_y1,z1];
  if ![origin,x0,y0,x1,y1,ratio,unit].iter().chain(values.iter()).all(|v|v.is_finite())||ratio==0.0{return Err(Error::NonFiniteIntermediate)}
  Ok(Projection{corner_words:values.map(f32::to_bits),pending_existing:x.effect_present})
 })
}

#[cfg(test)] mod tests{
 use super::*;
 fn input<'a>(t:&'a [u32;16])->Input<'a>{Input{route:Route::PendingNonzeroPlacement,level:3,grid:[2,-3],spacing_bits:t,helper_return_bits:1f32.to_bits(),global_514c_bits:1000f32.to_bits(),effect_present:false,stable_disjoint:true}}
 #[test] fn bounded_corners_and_branch(){let mut t=[1f32.to_bits();16];t[3]=300f32.to_bits();let c=Context::from_control_word(0x027f).unwrap();let a=project(input(&t),Policy::Mxcsr1f80,c).unwrap();let mut v=input(&t);v.effect_present=true;let b=project(v,Policy::Mxcsr1f80,c).unwrap();assert_eq!(a.corner_words,b.corner_words);assert!(!a.pending_existing&&b.pending_existing);assert!(f32::from_bits(a.corner_words[0])>0.&&f32::from_bits(a.corner_words[1])<0.);}
 #[test] fn rejects_unadmitted_and_invalid_inputs(){let t=[1f32.to_bits();16];let c=Context::from_control_word(0x027f).unwrap();let mut x=input(&t);x.route=Route::PendingZeroEffectInspection;assert_eq!(project(x,Policy::Mxcsr1f80,c),Err(Error::WrongRoute));x=input(&t);x.level=16;assert_eq!(project(x,Policy::Mxcsr1f80,c),Err(Error::LevelOutOfRange));x=input(&t);x.grid[0]=i32::MAX;assert_eq!(project(x,Policy::Mxcsr1f80,c),Err(Error::GridOverflow));x=input(&t);x.global_514c_bits=0;assert_eq!(project(x,Policy::Mxcsr1f80,c),Err(Error::NonPositiveOrNonFinite));x=input(&t);x.stable_disjoint=false;assert_eq!(project(x,Policy::Mxcsr1f80,c),Err(Error::Unstable));}
}
