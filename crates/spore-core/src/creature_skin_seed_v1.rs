//! Controlled 53EDE0 inverse and 53E8D0 seed search on explicit global lanes.
use crate::{creature_skin_lattice_world_v1 as world,native_sse::{self,Policy}};
#[derive(Debug,PartialEq,Eq)]pub enum Error{InvalidInput,NonFinite,UnsupportedConversion,NoSeed,SearchLimit,Field,UnsupportedArchitecture}
impl From<native_sse::SseError> for Error{fn from(_:native_sse::SseError)->Self{Self::UnsupportedArchitecture}}
/// Explicit constructor-stored reciprocal; no inference that it equals a fresh
/// reciprocal under a different FP state. Admits finite SSE and CVTT conversion.
pub fn inverse(point:[f32;3],origin:[f32;3],reciprocal:f32,policy:Policy)->Result<u32,Error>{
 if point.into_iter().chain(origin).chain([reciprocal]).any(|v|!v.is_finite())||reciprocal<=0.{return Err(Error::InvalidInput)}
 native_sse::scoped(policy,||{let mut out=[0u8;3];for i in 0..3{let d=point[i]-origin[i];let x=d*reciprocal;if !d.is_finite()||!x.is_finite(){return Err(Error::NonFinite)}if x< -2147483648.||x>=2147483648.{return Err(Error::UnsupportedConversion)}out[i]=(x as i32) as u8;}Ok(((out[0] as u32)<<16)|((out[1] as u32)<<8)|(out[2] as u32))})
}
fn offset(p:u32,corner:u8)->u32{let mut result=0;for axis in 0..3{let byte=((p>>(8*axis))&255).wrapping_add(((corner as u32)>>axis)&1)&255;result|=byte<<(8*axis);}result}
/// `groups` is the exact global query +1C four-wide center buffer, including
/// padding slots, not the per-cell grid. The declared count controls rounded-up
/// groups but all four slots in each group are scanned in source order. Stable
/// successful finite field calls and callback FP behavior are admitted. Output
/// is a packed coordinate only, not a queue, surface or runtime identity.
pub fn seed<F:FnMut([f32;3])->f32>(extent:[f32;3],origin:[f32;3],step:f32,stored_reciprocal:f32,groups:&[[[f32;3];4]],logical_count:usize,policy:Policy,mut field:F)->Result<u32,Error>{
 if extent.into_iter().chain(origin).chain([step,stored_reciprocal]).any(|v|!v.is_finite())||step<=0.||stored_reciprocal<=0.||logical_count.checked_add(3).is_none(){return Err(Error::InvalidInput)}
 let count=(logical_count+3)/4;if groups.len()!=count{return Err(Error::InvalidInput)}
 let acceptable=native_sse::scoped(policy,||{let inv=1.0f32/step;if !inv.is_finite(){return Err(Error::NonFinite)}for v in extent {let q=v*inv;if !q.is_finite(){return Err(Error::NonFinite)}if q<=0.||q>=255.{return Ok(false)}}Ok(true)})?;if !acceptable{return Err(Error::NoSeed)}
 let mut found=None;
 'scan:for group in groups {for &lane in group {let base=inverse(lane,origin,stored_reciprocal,policy)?;for c in 0..8u8{let p=offset(base,c);let pos=world::world(p,origin,step,policy,world::Admission).map_err(|_|Error::NonFinite)?;let value=field(pos);if !value.is_finite(){return Err(Error::Field)}if value>=0.{found=Some(p);break 'scan}}}}
 let mut last=found.ok_or(Error::NoSeed)?;
 for count in 1..=129 {let p=offset(last,1);let pos=world::world(p,origin,step,policy,world::Admission).map_err(|_|Error::NonFinite)?;let value=field(pos);if !value.is_finite(){return Err(Error::Field)}if value>=0.{last=p;}if count>128{return Err(Error::SearchLimit)}if value<0.{return Ok(last)}}
 unreachable!()
}
#[cfg(test)]mod tests{use super::*;fn g()->[[[f32;3];4];1]{[[[0.;3];4]]}
 #[test]fn inverse_truncates_and_wraps(){assert_eq!(inverse([-1.9,256.9,1.9],[0.;3],1.,Policy::Mxcsr1f80),Ok(0xff0001));assert_eq!(inverse([2147483648.,0.,0.],[0.;3],1.,Policy::Mxcsr1f80),Err(Error::UnsupportedConversion));}
 #[test]fn scan_and_forward_order(){let mut seen=Vec::new();let seed=seed([2.;3],[0.;3],1.,1.,&g(),1,Policy::Mxcsr1f80,|p|{seen.push(p);1.5-p[2]}).unwrap();assert_eq!(seed,1);assert_eq!(seen,vec![[0.,0.,0.],[0.,0.,1.],[0.,0.,2.]]);}
 #[test]fn padding_slots_are_scanned(){let mut groups=g();groups[0][1]=[2.,0.,0.];let mut calls=0;let result=seed([4.;3],[0.;3],1.,1.,&groups,1,Policy::Mxcsr1f80,|p|{calls+=1;if p[0]==2.&&p[2]==0.{0.}else{-1.}});assert_eq!(result,Ok(0x020000));assert_eq!(calls,10);}
 #[test]fn limits_and_129th_negative_still_fails(){assert_eq!(seed([255.,1.,1.],[0.;3],1.,1.,&g(),1,Policy::Mxcsr1f80,|_|0.),Err(Error::NoSeed));let mut n=0;assert_eq!(seed([1.;3],[0.;3],1.,1.,&g(),1,Policy::Mxcsr1f80,|_|{n+=1;if n==130{-1.}else{0.}}),Err(Error::SearchLimit));assert_eq!(n,130);}
}
