//! Arithmetic-only looping slice, separate from clock/event state admission.
//!
//! Original 9A1CED retains its x87 quotient after committing binary32. The
//! looping call commits that retained quotient again, invokes99BCB0 and its
//! finite SSE2 MSVCR90 modf branch, commits the returned fraction, then computes
//! the source frame. Counter/generation/event work is deliberately not modeled.
use crate::{native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
use std::fmt;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LoopStateBoundary {Unestablished,CallerSuppliedArithmeticSnapshot}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ModfDispatch {Unestablished,ExplicitFiniteSse2}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct LoopArithmeticContext {pub x87:Context,pub sse:Policy,pub modf_dispatch:ModfDispatch}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct LoopArithmeticInput {pub time_bits:u32,pub duration_bits:u64,pub resource_frames_bits:u32,pub state_boundary:LoopStateBoundary}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LoopArithmeticError {UnestablishedStateBoundary,UnestablishedCrtDispatch,NonFiniteInput,
    NonPositiveDuration,NegativeTime,UnsupportedFrameCount,NonFiniteQuotient,X87(X87Error),Sse(SseError)}
impl From<X87Error>for LoopArithmeticError {fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<SseError>for LoopArithmeticError {fn from(e:SseError)->Self{Self::Sse(e)}}
impl fmt::Display for LoopArithmeticError {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source loop arithmetic boundary: {self:?}")}}
impl std::error::Error for LoopArithmeticError {}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct LoopArithmeticProjection {
    pub stored_time_bits:u64,pub retained_quotient:[u8;10],pub quotient_commit_bits:u32,
    pub helper_argument_bits:u32,pub modf_integer_bits:u64,pub helper_fraction:[u8;10],
    pub normalized_fraction_bits:u32,pub frame_bits:u32,
}
impl LoopArithmeticProjection {
    pub fn to_le_bytes(self)->[u8;56] {
        let mut raw=[0;56];raw[..8].copy_from_slice(&self.stored_time_bits.to_le_bytes());raw[8..18].copy_from_slice(&self.retained_quotient);
        raw[20..24].copy_from_slice(&self.quotient_commit_bits.to_le_bytes());raw[24..28].copy_from_slice(&self.helper_argument_bits.to_le_bytes());
        raw[28..36].copy_from_slice(&self.modf_integer_bits.to_le_bytes());raw[36..46].copy_from_slice(&self.helper_fraction);
        raw[48..52].copy_from_slice(&self.normalized_fraction_bits.to_le_bytes());raw[52..].copy_from_slice(&self.frame_bits.to_le_bytes());raw
    }
}
pub fn evaluate_loop_arithmetic(input:LoopArithmeticInput,context:LoopArithmeticContext)->Result<LoopArithmeticProjection,LoopArithmeticError> {
    native_sse::scoped(context.sse,|| {
        if input.state_boundary!=LoopStateBoundary::CallerSuppliedArithmeticSnapshot {return Err(LoopArithmeticError::UnestablishedStateBoundary);}
        if context.modf_dispatch!=ModfDispatch::ExplicitFiniteSse2 {return Err(LoopArithmeticError::UnestablishedCrtDispatch);}
        let time=f32::from_bits(input.time_bits);let duration=f64::from_bits(input.duration_bits);let frames=f32::from_bits(input.resource_frames_bits);
        if !time.is_finite()||!duration.is_finite()||!frames.is_finite(){return Err(LoopArithmeticError::NonFiniteInput);}
        if duration<=0.{return Err(LoopArithmeticError::NonPositiveDuration);}if time<0.{return Err(LoopArithmeticError::NegativeTime);}
        if frames<1.{return Err(LoopArithmeticError::UnsupportedFrameCount);}
        let quotient=context.x87.div(Extended::from_f32(time)?,Extended::from_f64(duration)?)?;
        let commit=|value|context.x87.store_f32(value).map_err(|e|if e==X87Error::NonFiniteResult{LoopArithmeticError::NonFiniteQuotient}else{LoopArithmeticError::X87(e)});
        let quotient_commit=commit(quotient)?;
        // The argument comes from the retained80 value, not from the preceding
        // committed32 destination, even when these two stores agree numerically.
        let helper_argument=commit(quotient)?;
        let argument64=f64::from(helper_argument).to_bits(); // exact FLD32/FST64 conversion
        let exponent=(argument64>>52)&0x7ff;let sign=argument64&0x8000_0000_0000_0000;
        let (integer,fraction)=if exponent<0x3ff {(sign,argument64)}else if exponent<=0x432 {
            // Literal PSRLQ/PSLLQ count433-exponent from the original SSE2
            // modf kernel. SUBSD and ORPD retain its signed fractional zero.
            let shift=0x433-exponent;let integer=(argument64>>shift)<<shift;
            (integer,source_subsd(argument64,integer)?|sign)
        }else{(argument64,sign)};
        let fraction=Extended::from_f64(f64::from_bits(fraction))?;
        let normalized=context.x87.store_f32(fraction)?;
        let frame_bits=source_frame(input.resource_frames_bits,normalized.to_bits())?;
        Ok(LoopArithmeticProjection {stored_time_bits:f64::from(time).to_bits(),retained_quotient:quotient.to_le_bytes(),
            quotient_commit_bits:quotient_commit.to_bits(),helper_argument_bits:helper_argument.to_bits(),
            modf_integer_bits:integer,helper_fraction:fraction.to_le_bytes(),normalized_fraction_bits:normalized.to_bits(),frame_bits})
    })
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn source_subsd(left:u64,right:u64)->Result<u64,SseError> {
    let mut result=0u64;
    unsafe{std::arch::asm!("movsd {value},qword ptr [{left}]","subsd {value},qword ptr [{right}]","movsd qword ptr [{result}],{value}",
        value=out(xmm_reg)_,left=in(reg)&left,right=in(reg)&right,result=in(reg)&mut result,options(nostack,preserves_flags));}Ok(result)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn source_subsd(_:u64,_:u64)->Result<u64,SseError>{Err(SseError::UnsupportedArchitecture)}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn source_frame(frames:u32,ratio:u32)->Result<u32,SseError> {
    let result:u32;
    unsafe{std::arch::asm!("movd {value},{frames:e}","movd {one},{one_bits:e}","subss {value},{one}",
        "movd {factor},{ratio:e}","mulss {value},{factor}","movd {result:e},{value}",
        value=out(xmm_reg)_,one=out(xmm_reg)_,factor=out(xmm_reg)_,frames=in(reg)frames,
        one_bits=in(reg)0x3f800000u32,ratio=in(reg)ratio,result=out(reg)result,options(nostack,preserves_flags));}Ok(result)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn source_frame(_:u32,_:u32)->Result<u32,SseError>{Err(SseError::UnsupportedArchitecture)}
