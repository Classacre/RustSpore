//! Checked E60750 earlier camera vector and optional actor interpolation.
use crate::{cell_swimming::{retained_ramp,CellSwimmingError},native_x87::Context};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedControl, UnsupportedFloat, Helper(CellSwimmingError) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub b:[u32;3], pub c:[u32;3], pub ramp_or_alpha:u32, pub next_dispatch_pending:bool }
/// The caller supplies stable, disjoint current state and an optional resolved
/// actor position. No live player identity or subsequent dispatch is inferred.
/// CW 027F/MXCSR 1F80 and finite binary32 normal/zero reached operands and
/// intermediate results are required. Actor Z is not read on this slice.
pub fn project(state_b:[u32;3],state_c:[u32;3],alpha:u32,actor_xy:Option<[u32;2]>,cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}if cw!=0x027f||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 fn valid(x:f32)->bool{x.is_finite()&&(x==0.0||x.is_normal())}fn op(x:f32)->Result<f32,Error>{if valid(x){Ok(x)}else{Err(Error::UnsupportedFloat)}}
 let b=state_b.map(f32::from_bits);let c=state_c.map(f32::from_bits);let a=f32::from_bits(alpha);
 if !b.iter().chain(c.iter()).all(|&x|valid(x))||!valid(a){return Err(Error::UnsupportedFloat)}
 let cz=op(c[2]*a)?;let mut out=state_b;let mut q=a;
 if let Some(actor)=actor_xy{let v=actor.map(f32::from_bits);if !v.iter().all(|&x|valid(x)){return Err(Error::UnsupportedFloat)}
  let ctx=Context::from_control_word(cw).map_err(|e|Error::Helper(CellSwimmingError::X87(e)))?;
  let retained=retained_ramp(ctx,0.5,1.0,a,0.5,1.0).map_err(Error::Helper)?;
  q=ctx.store_f32(retained).map_err(|e|Error::Helper(CellSwimmingError::X87(e)))?;if !valid(q){return Err(Error::UnsupportedFloat)}
  let dx=op(b[0]-v[0])?;let dy=op(b[1]-v[1])?;let sx=op(dx*q)?;let sy=op(dy*q)?;
  out[0]=op(sx+v[0])?.to_bits();out[1]=op(sy+v[1])?.to_bits();
 }
 Ok(Outcome{b:out,c:[state_c[0],state_c[1],cz.to_bits()],ramp_or_alpha:q.to_bits(),next_dispatch_pending:true})
}
#[cfg(test)]mod tests{use super::*;#[test]fn paths_and_admission(){let b=[10f32.to_bits(),20f32.to_bits(),30f32.to_bits()];let c=[3f32.to_bits(),4f32.to_bits(),5f32.to_bits()];let actor=[2f32.to_bits(),4f32.to_bits()];
 let x=project(b,c,0,None,0x027f,0x1f80,true).unwrap();assert_eq!(x.b,b);assert_eq!(x.c[2],0);
 assert!(project(b,c,0.75f32.to_bits(),Some(actor),0x027f,0x1f80,true).unwrap().b[0]!=b[0]);
 assert_eq!(project(b,c,0,None,0x027f,0x1f80,false),Err(Error::Unadmitted));assert_eq!(project(b,c,0,None,0x037f,0x1f80,true),Err(Error::UnsupportedControl));assert_eq!(project(b,c,1,None,0x027f,0x1f80,true),Err(Error::UnsupportedFloat));
}}
