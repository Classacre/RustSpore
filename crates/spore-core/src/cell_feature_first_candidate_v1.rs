//! Controlled one-row-each E6ADE0 composition, stopping before E6A450.
//! Row/cache identities, actor/definition ownership and callback effects are
//! supplied admissions; no native contact or live gameplay is implied.
use crate::{cell_part_geometry::NativeCellPart,cell_feature_projection_v1::{self as projection,Transform},native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unstable,Projection(projection::Error),NonFinite,NegativeRadius,Sse(SseError),X87(X87Error)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}impl From<X87Error> for Error{fn from(e:X87Error)->Self{Self::X87(e)}}
#[derive(Debug,PartialEq,Eq)]pub enum LocalGeometry{OpaqueSmall,Nondegenerate{point:[u32;3],normal:[u32;3]}}
#[derive(Debug,PartialEq,Eq)]pub struct PendingCandidate{pub actor_tokens:[u32;2],pub definition_b4:[u32;2],pub second_definition_b0:u32,pub relation:i32,pub classes:[u32;2],pub indices:[u32;2],pub depth:u32,pub geometry:LocalGeometry}
#[derive(Debug,PartialEq,Eq)]pub enum Outcome{SkippedClassOne,Separated,Pending(PendingCandidate)}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]macro_rules! op{($name:ident,$ins:literal)=>{fn $name(a:u32,b:u32)->Result<u32,Error>{let r:u32;unsafe{std::arch::asm!("movd {x}, {a:e}","movd {y}, {b:e}",$ins,"movd {r:e}, {x}",x=out(xmm_reg) _,y=out(xmm_reg) _,a=in(reg)a,b=in(reg)b,r=lateout(reg)r,options(nostack,preserves_flags));}if !f32::from_bits(r).is_finite(){return Err(Error::NonFinite)}Ok(r)}}}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]op!(mul,"mulss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]op!(add,"addss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]op!(sub,"subss {x}, {y}");
#[cfg(any(target_arch="x86",target_arch="x86_64"))]op!(div,"divss {x}, {y}");
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn mul(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn add(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn sub(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn div(_:u32,_:u32)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
/// Both rows and transforms are caller-admitted stable, disjoint snapshots;
/// flags/definition_b4/relations/identities are synthetic inputs. Finite results,
/// nonnegative projected radii, MXCSR 1F80 and x87 027F are checked/modelled
/// safeguards. The small-distance branch exposes no point or normal locals.
/// Every successful candidate is pending E6A450; no callback is executed.
pub fn first(rows:[&NativeCellPart;2],transforms:[Transform;2],actor_tokens:[u32;2],definition_b4:[u32;2],second_definition_b0:u32,relation:i32,stable:bool)->Result<Outcome,Error>{
 if !stable{return Err(Error::Unstable)}if rows[0].class==1||rows[1].class==1{return Ok(Outcome::SkippedClassOne)}
 let a=projection::project(rows[0],transforms[0],false).map_err(Error::Projection)?;let b=projection::project(rows[1],transforms[1],true).map_err(Error::Projection)?;
 if f32::from_bits(a.radius)<0.||f32::from_bits(b.radius)<0.{return Err(Error::NegativeRadius)}
 let (delta,dsq,sum)=native_sse::scoped(Policy::Mxcsr1f80,||{let x=sub(b.center[0],a.center[0])?;let y=sub(b.center[1],a.center[1])?;let yy=mul(y,y)?;let xx=mul(x,x)?;let yx=add(yy,xx)?;let z=sub(b.center[2],a.center[2])?;let zz=mul(z,z)?;Ok::<_,Error>(([x,y,z],add(yx,zz)?,add(b.radius,a.radius)?))})?;
 let (depth,geometry)=if f32::from_bits(dsq)<f32::from_bits(0x37800000){(sum,LocalGeometry::OpaqueSmall)}else{
  let ctx=Context::from_control_word(0x027f)?;let distance=ctx.store_f32(ctx.sqrt(Extended::from_f32(f32::from_bits(dsq))?)?)?.to_bits();
  let value=native_sse::scoped(Policy::Mxcsr1f80,||{let depth=sub(sum,distance)?;if f32::from_bits(depth)<0.{return Ok::<_,Error>(None)}let inv=div(1f32.to_bits(),distance)?;let nx=mul(inv,delta[0])?;let ny=mul(inv,delta[1])?;let nz=mul(delta[2],inv)?;let dr=sub(a.radius,b.radius)?;let px=mul(add(add(b.center[0],a.center[0])?,mul(nx,dr)?)?,0.5f32.to_bits())?;let py=mul(add(add(a.center[1],b.center[1])?,mul(ny,dr)?)?,0.5f32.to_bits())?;let pz=mul(add(add(a.center[2],b.center[2])?,mul(nz,dr)?)?,0.5f32.to_bits())?;Ok(Some((depth,LocalGeometry::Nondegenerate{point:[px,py,pz],normal:[nx,ny,nz]})))})?;
  match value{Some(v)=>v,None=>return Ok(Outcome::Separated)}
 };
 Ok(Outcome::Pending(PendingCandidate{actor_tokens,definition_b4,second_definition_b0,relation,classes:[rows[0].class,rows[1].class],indices:[0,0],depth,geometry}))
}
#[cfg(test)]mod tests{use super::*;fn row()->NativeCellPart{NativeCellPart{word_06_is_ffff:false,class:2,center:[0.;3],matrix_bits:[0;9],radius:1.,tail_bits:[0;3]}}fn transform(x:f32)->Transform{Transform{position:[x.to_bits(),0,0],scale:1f32.to_bits(),flags:0,matrix:[0;9]}}
 #[test]fn separates_and_keeps_small_locals_opaque(){let a=row();let b=row();assert_eq!(first([&a,&b],[transform(0.),transform(3.)],[1,2],[2,3],4,0,true).unwrap(),Outcome::Separated);let out=first([&a,&b],[transform(0.),transform(0.)],[1,2],[2,3],4,0,true).unwrap();assert!(matches!(out,Outcome::Pending(PendingCandidate{geometry:LocalGeometry::OpaqueSmall,..})));}
 #[test]fn rejects_unstable_and_skips_class_one(){let a=row();let mut b=row();assert_eq!(first([&a,&b],[transform(0.),transform(1.)],[1,2],[2,3],4,0,false),Err(Error::Unstable));b.class=1;assert_eq!(first([&a,&b],[transform(0.),transform(1.)],[1,2],[2,3],4,0,true),Ok(Outcome::SkippedClassOne));}
}
