//! Bounded E4ECC0 / E51E50 tier and helper-return projection.
//! Caller supplies table snapshot, progress and level; no live object identity.
use crate::cell_effect_placement_prefix_v1 as placement;
use crate::native_sse::Policy;
use crate::native_x87::Context;
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct TierRow{pub tier:u32,pub threshold:i32}
#[derive(Debug,PartialEq)] pub enum Error{Unstable,Input,Table,Placement(placement::Error)}
impl From<placement::Error> for Error{fn from(e:placement::Error)->Self{Self::Placement(e)}}
/// The checked domain is exactly the independently compared corpus. Source
/// itself has an unchecked scan and wrapping subtraction outside this domain.
pub fn helper(progress:i32,level:u32,rows:&[TierRow],special_tier:u32,stable:bool)->Result<u32,Error>{
 if !stable{return Err(Error::Unstable)}
 if !(0..=1000).contains(&progress)||!matches!(level,3|11){return Err(Error::Input)}
 let tier=if progress==1000{special_tier}else{let mut i=1usize;loop{let next=rows.get(i+1).ok_or(Error::Table)?;if progress<next.threshold{break rows.get(i).ok_or(Error::Table)?.tier}i=i.checked_add(1).ok_or(Error::Table)?;}};
 let diff=i64::from(level)-i64::from(tier);if diff<i64::from(i32::MIN)||diff>i64::from(i32::MAX){return Err(Error::Input)}
 Ok(match diff{..=-1=>1f32.to_bits(),0=>0.5f32.to_bits(),1=>0.25f32.to_bits(),2=>0.125f32.to_bits(),3=>0.0625f32.to_bits(),_=>0.03125f32.to_bits()})
}
/// Explicitly composes separately validated helper and placement prefix. The
/// supplied progress, tier table and +514C snapshots must be stable independently;
/// this does not infer their identity or that an effect instance exists.
pub fn placement_with_helper(mut input:placement::Input<'_>,progress:i32,rows:&[TierRow],special_tier:u32,helper_state_stable:bool,policy:Policy,ctx:Context)->Result<placement::Projection,Error>{
 input.helper_return_bits=helper(progress,input.level,rows,special_tier,helper_state_stable)?;
 Ok(placement::project(input,policy,ctx)?)
}
#[cfg(test)] mod tests{
 use super::*;
 fn rows()->Vec<TierRow>{(0..22).map(|i|TierRow{tier:if i==0{0}else{((i+1)/2) as u32},threshold:if i<2{0}else{(i as i32-1)*50}}).collect()}
 #[test] fn boundaries_and_rejection(){let r=rows();assert_eq!(helper(0,3,&r,10,true),Ok(0.125f32.to_bits()));assert_eq!(helper(200,3,&r,10,true),Ok(0.5f32.to_bits()));assert_eq!(helper(1000,11,&r,10,true),Ok(0.25f32.to_bits()));assert_eq!(helper(1001,3,&r,10,true),Err(Error::Input));assert_eq!(helper(0,3,&r,10,false),Err(Error::Unstable));assert_eq!(helper(999,3,&r[..3],10,true),Err(Error::Table));}
}
