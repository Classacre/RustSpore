//! Bounded original clock arithmetic, separate from admitted root-frame joins.
//!
//! Only finite, nonnegative committed time below a positive first duration and
//! a finite frame count >=1 are supported. Looping, crossing counters, event
//! reset/callbacks and actual clip admission are checked exclusions. This module
//! uses a separate original-byte comparison before root-frame integration.
use crate::{native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
use std::{cmp::Ordering,fmt};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ClockContext {pub x87:Context,pub sse:Policy}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ClockTime {
    Absolute {time_bits:u32},
    /// Original FLD32 delta; FADD64 previous time; FSTP32 next argument.
    Advance {previous_time_bits:u64,delta_bits:u32},
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ClockInput {
    pub time:ClockTime,
    pub duration_bits:u64,
    pub resource_frames_bits:u32,
    pub looping:bool,
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ClockProjection {
    pub time_argument_bits:u32,
    /// 9A1CDF FST64 of the exact binary32 argument.
    pub stored_time_bits:u64,
    /// 9A1CED committed x87 division result, reused at9A1D69.
    pub normalized_bits:u32,
    /// Original SUBSS frames,1 then MULSS normalized.
    pub frame_bits:u32,
}
impl ClockProjection {
    pub fn to_le_bytes(self)->[u8;20] {
        let mut raw=[0;20];raw[..4].copy_from_slice(&self.time_argument_bits.to_le_bytes());
        raw[4..12].copy_from_slice(&self.stored_time_bits.to_le_bytes());
        raw[12..16].copy_from_slice(&self.normalized_bits.to_le_bytes());
        raw[16..].copy_from_slice(&self.frame_bits.to_le_bytes());raw
    }
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ClockError {UnsupportedLoop,NonFiniteInput,NonFiniteResult,NonPositiveDuration,
    NegativeTime,DurationCrossing,UnsupportedFrameCount,X87(X87Error),Sse(SseError)}
impl From<X87Error>for ClockError {fn from(e:X87Error)->Self {Self::X87(e)}}
impl From<SseError>for ClockError {fn from(e:SseError)->Self {Self::Sse(e)}}
impl fmt::Display for ClockError {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {write!(f,"bounded source clock: {self:?}")}}
impl std::error::Error for ClockError {}

/// Evaluate only the source branch whose crossing and looping edges are absent.
/// No native root-data adapter is automatically enabled by this separate module.
pub fn evaluate_first_duration(input:ClockInput,context:ClockContext)->Result<ClockProjection,ClockError> {
    native_sse::scoped(context.sse,|| {
        if input.looping {return Err(ClockError::UnsupportedLoop);}
        let duration=f64::from_bits(input.duration_bits);
        let frames=f32::from_bits(input.resource_frames_bits);
        if !duration.is_finite() || !frames.is_finite() {return Err(ClockError::NonFiniteInput);}
        if duration<=0. {return Err(ClockError::NonPositiveDuration);}
        if frames<1. {return Err(ClockError::UnsupportedFrameCount);}
        let time=match input.time {
            ClockTime::Absolute {time_bits}=>{
                let time=f32::from_bits(time_bits);
                if !time.is_finite() {return Err(ClockError::NonFiniteInput);}time
            }
            ClockTime::Advance {previous_time_bits,delta_bits}=>{
                let previous=f64::from_bits(previous_time_bits);
                let delta=f32::from_bits(delta_bits);
                if !previous.is_finite() || !delta.is_finite() {return Err(ClockError::NonFiniteInput);}
                let value=context.x87.add(Extended::from_f32(delta)?,Extended::from_f64(previous)?)?;
                context.x87.store_f32(value).map_err(|e|if e==X87Error::NonFiniteResult {ClockError::NonFiniteResult}else{ClockError::X87(e)})?
            }
        };
        if time<0. {return Err(ClockError::NegativeTime);}
        let time_extended=Extended::from_f32(time)?;
        let duration_extended=Extended::from_f64(duration)?;
        if context.x87.compare(time_extended,duration_extended)?!=Ordering::Less {
            return Err(ClockError::DurationCrossing);
        }
        let normalized=context.x87.store_f32(context.x87.div(time_extended,duration_extended)?)?;
        let frame_bits=frame_products(input.resource_frames_bits,normalized.to_bits())?;
        if !f32::from_bits(frame_bits).is_finite() {return Err(ClockError::NonFiniteResult);}
        Ok(ClockProjection {time_argument_bits:time.to_bits(),
            // Binary32 to binary64 is exact, preserving signed zero; this is
            // the FST64 output encoding, not a replacement arithmetic step.
            stored_time_bits:f64::from(time).to_bits(),normalized_bits:normalized.to_bits(),frame_bits})
    })
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn frame_products(frames:u32,ratio:u32)->Result<u32,SseError> {
    let result:u32;
    // SAFETY: bounded register-only source instructions under scoped MXCSR.
    unsafe {std::arch::asm!("movd {value},{frames:e}","movd {one},{one_bits:e}",
        "subss {value},{one}","movd {factor},{ratio:e}","mulss {value},{factor}","movd {result:e},{value}",
        value=out(xmm_reg)_,one=out(xmm_reg)_,factor=out(xmm_reg)_,
        frames=in(reg)frames,one_bits=in(reg)0x3f800000u32,ratio=in(reg)ratio,
        result=out(reg)result,options(nostack,preserves_flags));}
    Ok(result)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn frame_products(_:u32,_:u32)->Result<u32,SseError> {Err(SseError::UnsupportedArchitecture)}
