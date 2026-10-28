//! Candidate source resource metadata arithmetic from9A3140; not exported.
//! Partial entry requires an already selected nonnull resource. Allocation,
//! reference ownership, channel wrappers and actual actor selection are excluded.
use crate::{native_sse::{self,Policy,SseError},native_x87::{Context,X87Error}};
use std::fmt;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ResourceMetadata {pub flags:u32,pub frames_bits:u32,pub frame_interval_bits:u32}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct MetadataContext {pub x87:Context,pub sse:Policy}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ClipMetadata {pub duration_commit_bits:u32,pub duration_bits:u64,pub constructor_default_loop:bool}
impl ClipMetadata {pub fn to_le_bytes(self)->[u8;16] {let mut b=[0;16];b[..4].copy_from_slice(&self.duration_commit_bits.to_le_bytes());b[4..12].copy_from_slice(&self.duration_bits.to_le_bytes());b[12..].copy_from_slice(&u32::from(self.constructor_default_loop).to_le_bytes());b}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum MetadataError {NonFiniteInput,UnsupportedFrameCount,NonPositiveInterval,NonFiniteDuration,NonPositiveDuration,X87(X87Error),Sse(SseError)}
impl From<X87Error>for MetadataError {fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<SseError>for MetadataError {fn from(e:SseError)->Self{Self::Sse(e)}}
impl fmt::Display for MetadataError {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source clip metadata: {self:?}")}}
impl std::error::Error for MetadataError {}
/// Source9A3198 SUBSS frames,1; MULSS frameInterval; MOVSS; FLD32/FSTP64.
/// Source9A32D4 extracts bit3 of the resource flags into clip+AD.
/// FLD32/FSTP64 widens finitebinary32 exactly under every masked source CW.
pub fn prepare_metadata(input:ResourceMetadata,context:MetadataContext)->Result<ClipMetadata,MetadataError>{
 native_sse::scoped(context.sse,|| {
  let frames=f32::from_bits(input.frames_bits);let interval=f32::from_bits(input.frame_interval_bits);
  if !frames.is_finite()||!interval.is_finite(){return Err(MetadataError::NonFiniteInput);}
  if frames<=1. {return Err(MetadataError::UnsupportedFrameCount);}
  if interval<=0. {return Err(MetadataError::NonPositiveInterval);}
  let duration_bits=products(input.frames_bits,input.frame_interval_bits)?;
  let duration=f32::from_bits(duration_bits);
  if !duration.is_finite(){return Err(MetadataError::NonFiniteDuration);}
  if duration<=0. {return Err(MetadataError::NonPositiveDuration);}
  // No rounding occurs on finitebinary32 -> binary64 widening.
  // This encodes the source FLD32/FSTP64 output, not replacement arithmetic.
  let stored=f64::from(duration);
  Ok(ClipMetadata{duration_commit_bits:duration_bits,duration_bits:stored.to_bits(),constructor_default_loop:input.flags&8!=0})
 })
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn products(frames:u32,interval:u32)->Result<u32,SseError>{let out:u32;
 // SAFETY: register-only source SSE operations under scoped policy.
 unsafe{std::arch::asm!("movd {v},{frames:e}","movd {one},{one_bits:e}","subss {v},{one}","movd {factor},{interval:e}","mulss {v},{factor}","movd {out:e},{v}",v=out(xmm_reg)_,one=out(xmm_reg)_,factor=out(xmm_reg)_,frames=in(reg)frames,one_bits=in(reg)0x3f800000u32,interval=in(reg)interval,out=out(reg)out,options(nostack,preserves_flags));}Ok(out)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn products(_:u32,_:u32)->Result<u32,SseError>{Err(SseError::UnsupportedArchitecture)}
