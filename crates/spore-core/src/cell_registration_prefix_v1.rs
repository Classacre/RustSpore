//! Controlled E74F93..E7501D registration prefix; no source actor or collection.
use crate::{cell_actor_geometry_projection_v1::Output,native_sse::{self,Policy,SseError}};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Error {UnstableOrAliased,NonFinite,Sse(SseError)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Input {pub original_manager:u32,pub current_manager:u32,pub argument10:u32,pub definition_b4:u32,pub actor_handle:u32,pub geometry:Output,pub post_e86960_stable_disjoint:bool}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ResultV1 {Skipped,Register{tag:u32,min:[u32;2],max:[u32;2]}}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn arithmetic(x:u32,y:u32,r:u32)->[u32;4]{
 let (xmax,ymax,xmin,ymin):(u32,u32,u32,u32);
 // SAFETY: caller holds a checked SSE2 MXCSR1F80 scope; exact original order.
 unsafe {std::arch::asm!("movd {x}, {xb:e}","movd {y}, {yb:e}","movd {r}, {rb:e}","movaps {t}, {x}","addss {t}, {r}","movd {xmax:e}, {t}","movaps {t}, {y}","addss {t}, {r}","movd {ymax:e}, {t}","subss {x}, {r}","movd {xmin:e}, {x}","subss {y}, {r}","movd {ymin:e}, {y}",x=out(xmm_reg) _,y=out(xmm_reg) _,r=out(xmm_reg) _,t=out(xmm_reg) _,xb=in(reg)x,yb=in(reg)y,rb=in(reg)r,xmax=lateout(reg)xmax,ymax=lateout(reg)ymax,xmin=lateout(reg)xmin,ymin=lateout(reg)ymin,options(nostack,preserves_flags));}
 [xmax,ymax,xmin,ymin]
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn arithmetic(_:u32,_:u32,_:u32)->[u32;4]{unreachable!("native_sse::scoped rejects unsupported architecture")}
/// Values are fresh post-E86960 snapshots; the caller admits stable, disjoint
/// locals, not native allocation success or persistent collection registration.
pub fn evaluate(input:Input)->Result<ResultV1,Error>{
 if !input.post_e86960_stable_disjoint{return Err(Error::UnstableOrAliased)}
 if input.original_manager!=input.current_manager{return Ok(ResultV1::Skipped)}
 let tag=if input.argument10==u32::MAX {if input.definition_b4==5||input.definition_b4==7 {(-5i32) as u32}else{input.actor_handle}}else{input.argument10};
 let [x,y,_]=input.geometry.center;let r=input.geometry.scalar;
 if [x,y,r].iter().any(|&v|!f32::from_bits(v).is_finite()){return Err(Error::NonFinite)}
 let [xmax,ymax,xmin,ymin]=native_sse::scoped(Policy::Mxcsr1f80,||Ok::<_,Error>(arithmetic(x,y,r)))?;
 if [xmax,ymax,xmin,ymin].iter().any(|&v|!f32::from_bits(v).is_finite()){return Err(Error::NonFinite)}
 Ok(ResultV1::Register{tag,min:[xmin,ymin],max:[xmax,ymax]})
}
#[cfg(test)] mod tests {
 use super::*;
 fn input()->Input{Input{original_manager:9,current_manager:9,argument10:0,definition_b4:2,actor_handle:0x10000,geometry:Output{center:[100f32.to_bits(),(-2f32).to_bits(),0],scalar:0.5f32.to_bits()},post_e86960_stable_disjoint:true}}
 #[test] fn tag_gate_and_source_ordered_bounds(){let mut a=input();assert_eq!(evaluate(a).unwrap(),ResultV1::Register{tag:0,min:[99.5f32.to_bits(),(-2.5f32).to_bits()],max:[100.5f32.to_bits(),(-1.5f32).to_bits()]});a.current_manager=10;assert_eq!(evaluate(a).unwrap(),ResultV1::Skipped);a.post_e86960_stable_disjoint=false;assert_eq!(evaluate(a),Err(Error::UnstableOrAliased));}
 #[test] fn fallback_tags_and_zero_width_are_not_confused(){let mut a=input();a.argument10=u32::MAX;a.definition_b4=5;let ResultV1::Register{tag,..}=evaluate(a).unwrap() else{panic!()};assert_eq!(tag,(-5i32) as u32);a.definition_b4=7;let ResultV1::Register{tag,..}=evaluate(a).unwrap() else{panic!()};assert_eq!(tag,(-5i32) as u32);a.definition_b4=2;let ResultV1::Register{tag,..}=evaluate(a).unwrap() else{panic!()};assert_eq!(tag,a.actor_handle);a.argument10=42;a.geometry.scalar=0;let ResultV1::Register{tag,min,max}=evaluate(a).unwrap() else{panic!()};assert_eq!(tag,42);assert_eq!(min,max);}
}
