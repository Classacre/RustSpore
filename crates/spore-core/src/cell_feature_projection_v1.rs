//! Checked E6ADE0 feature projection over explicit row/actor snapshots.
//! These variants do not resolve caches, actors, geometry acceptance or contact.
use crate::{cell_part_geometry::NativeCellPart,native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug)]pub struct Transform{pub position:[u32;3],pub scale:u32,pub flags:u8,pub matrix:[u32;9]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Projected{pub center:[u32;3],pub radius:u32}
#[derive(Debug,PartialEq,Eq)]pub enum Error{NonFinite,Sse(SseError)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
macro_rules! operation{($name:ident,$ins:literal)=>{fn $name(a:u32,b:u32)->Result<u32,Error>{let r:u32;unsafe{std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$ins,"movd {r:e}, {x}",x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg)a,b=in(reg)b,r=lateout(reg)r,options(nostack,preserves_flags));}if !f32::from_bits(r).is_finite(){return Err(Error::NonFinite)}Ok(r)}}}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]operation!(mul,"mulss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]operation!(add,"addss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn mul(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn add(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
/// Actor/row snapshots are explicitly stable and disjoint, inputs/intermediates
/// finite, and MXCSR is scoped to 1F80. Independent operation order and sticky
/// flags/native partial writes are not modeled. `second` preserves the distinct
/// source operand order of the second-side X matrix product.
pub fn project(row:&NativeCellPart,actor:Transform,second:bool)->Result<Projected,Error>{
 let c=row.center.map(f32::to_bits);let r=row.radius.to_bits();let finite=|v:u32|f32::from_bits(v).is_finite();
 if c.iter().any(|&v|!finite(v))||!finite(r)||!finite(actor.scale)||actor.position.iter().any(|&v|!finite(v))||(actor.flags&2!=0&&actor.matrix.iter().any(|&v|!finite(v))){return Err(Error::NonFinite)}
 native_sse::scoped(Policy::Mxcsr1f80,||{let [cx,cy,cz]=c;let [m0,m1,m2,m3,m4,m5,m6,m7,m8]=actor.matrix;let [x,y,z]=if actor.flags&2!=0{let last_x=if second{mul(cx,m0)?}else{mul(m0,cx)?};let x=add(add(mul(m3,cy)?,mul(m6,cz)?)?,last_x)?;let y=add(add(mul(m4,cy)?,mul(m7,cz)?)?,mul(m1,cx)?)?;let z=add(add(mul(m5,cy)?,mul(m8,cz)?)?,mul(m2,cx)?)?;[x,y,z]}else{c};Ok(Projected{center:[add(actor.position[0],mul(actor.scale,x)?)?,add(actor.position[1],mul(actor.scale,y)?)?,add(actor.position[2],mul(actor.scale,z)?)?],radius:mul(r,actor.scale)?})})
}
#[cfg(test)]mod tests{use super::*;fn row()->NativeCellPart{NativeCellPart{word_06_is_ffff:false,class:2,center:[1.,2.,3.],matrix_bits:[0;9],radius:2.,tail_bits:[0;3]}}
 #[test]fn matrix_grouping_and_finite_checks(){let t=Transform{position:[0;3],scale:1f32.to_bits(),flags:2,matrix:[1.,2.,3.,4.,5.,6.,7.,8.,9.].map(f32::to_bits)};let p=project(&row(),t,false).unwrap();assert_eq!(p.center,[30f32.to_bits(),36f32.to_bits(),42f32.to_bits()]);let mut r=row();r.radius=f32::NAN;assert_eq!(project(&r,t,true),Err(Error::NonFinite));}
}
