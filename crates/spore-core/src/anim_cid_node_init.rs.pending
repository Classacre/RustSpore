//! Candidate bounded9C52F4..9C54B1 node producer; no full CID/factory claim.
use crate::native_x87::{Context,Extended,X87Error};
use std::fmt;
pub const NODE_BYTES:usize=700;
#[derive(Clone,Copy,Debug)]pub struct DefinitionNode{pub parent:u32,pub first_child:u32,pub child_count:u32,pub center:[u32;3],pub quaternion:[u32;4]}
#[derive(Clone,Copy,Debug)]pub enum Admission{Unestablished,SourceFactoryScaleOne{control_word:u16}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unestablished,Count,Parent,Child,NonFinite,ControlWord,UnsupportedArchitecture,X87(X87Error)}
impl fmt::Display for Error{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"bounded CID node initializer: {self:?}")}}
impl std::error::Error for Error{}
impl From<X87Error>for Error{fn from(v:X87Error)->Self{Self::X87(v)}}
fn put(b:&mut[u8;NODE_BYTES],at:usize,v:u32){b[at..at+4].copy_from_slice(&v.to_le_bytes());}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn mulss_one(bits:u32)->Result<u32,Error>{
 if !std::is_x86_feature_detected!("sse2"){return Err(Error::UnsupportedArchitecture);}
 let requested=0x1f80u32;let mut saved=0u32;let mut result=0u32;let one=0x3f800000u32;
 // SAFETY: bounded locals; save and restore the complete caller MXCSR in one
 // assembly block. Explicit source MULSS occurs before any history FLD/FSTP.
 unsafe{std::arch::asm!("stmxcsr [{saved}]","ldmxcsr [{requested}]","movss xmm0, [{value}]","mulss xmm0, [{one}]","movss [{result}], xmm0","ldmxcsr [{saved}]",saved=in(reg)&mut saved,requested=in(reg)&requested,value=in(reg)&bits,one=in(reg)&one,result=in(reg)&mut result,out("xmm0")_,options(nostack,preserves_flags));}
 Ok(result)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn mulss_one(_:u32)->Result<u32,Error>{Err(Error::UnsupportedArchitecture)}
/// Canonical pointers retain the source index: Definition ordinal, CID0,
/// parent/first-child ordinal, or FFFFFFFF for a source null node pointer.
/// Every unwritten byte retains the explicitly supplied A5 incoming state.
pub fn initialize(nodes:&[DefinitionNode],admission:Admission)->Result<Vec<[u8;NODE_BYTES]>,Error>{
 let cw=match admission{Admission::Unestablished=>return Err(Error::Unestablished),Admission::SourceFactoryScaleOne{control_word}=>control_word};
 if cw!=0x27f{return Err(Error::ControlWord);}let context=Context::from_control_word(cw)?;
 if nodes.is_empty()||nodes.len()>255{return Err(Error::Count);}
 for(i,n)in nodes.iter().enumerate(){if i!=0&&n.parent as usize>=nodes.len(){return Err(Error::Parent);}if n.child_count!=0&&n.first_child as usize>=nodes.len(){return Err(Error::Child);}if n.center.iter().chain(n.quaternion.iter()).any(|&v|!f32::from_bits(v).is_finite()){return Err(Error::NonFinite);}}
 let mut result=Vec::with_capacity(nodes.len());
 for(i,n)in nodes.iter().enumerate(){
  let mut b=[0xa5;NODE_BYTES];
  for at in(0x58..0xa4).step_by(4).chain([0x48,0x4c,0x50]).chain((0x144..0x164).step_by(4)){put(&mut b,at,0);}
  put(&mut b,0,i as u32);put(&mut b,4,0);put(&mut b,8,if i==0{u32::MAX}else{n.parent});put(&mut b,12,if n.child_count==0{u32::MAX}else{n.first_child});
  // Original FLD/FSTP quaternion sequence precedes source SSE position math.
  for(k,&v)in n.quaternion.iter().enumerate(){put(&mut b,0x1c+k*4,context.store_f32(Extended::from_f32(f32::from_bits(v))?)?.to_bits());}
  let mut position=[0;3];for(k,&v)in n.center.iter().enumerate(){position[k]=mulss_one(v)?;put(&mut b,0x10+k*4,position[k]);}
  for(k,&v)in position.iter().enumerate(){put(&mut b,0x3c+k*4,context.store_f32(Extended::from_f32(f32::from_bits(v))?)?.to_bits());}
  for at in[0x38,0x54,0x55,0x2a0]{b[at]=0;}put(&mut b,0x154,0x3f800000);result.push(b);
 }Ok(result)
}
