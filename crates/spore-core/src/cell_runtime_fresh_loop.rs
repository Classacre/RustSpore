//! Candidate fresh private state -> full source activation -> crossing lifecycle.
//! Native admission is separate. Explicit unbound fresh ownership, fourzero masks,
//! selectedv25resource and isolatedglobal counters exclude livechannel/actor/event
//! scheduling. No caller C0/C4/time-history/active/loop snapshots are accepted.
use crate::{anim::Anim,cell_runtime_fresh_aid::FreshAidClockState,
 cell_runtime_clip_metadata::{prepare_metadata,ResourceMetadata,MetadataContext,MetadataError},
 cell_runtime_controller_loop::{apply_property_loop,ControllerLoopSnapshot,ControllerSlot,PropertyLoopMode,ControllerLoopError},
 cell_runtime_source_clock_root_data::SourceMetadataAdmission,
 cell_runtime_loop_clock::{evaluate_loop_arithmetic,LoopArithmeticInput,LoopArithmeticProjection,LoopArithmeticContext,LoopArithmeticError,LoopStateBoundary},
 native_sse::{self,SseError},native_x87::{Extended,X87Error}};
use std::{cmp::Ordering,fmt};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum SourceGeneration{Unestablished,ExplicitIsolatedSnapshot(u32)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum ZeroMaskBoundary{Unestablished,UnboundFreshFourZeroMasks}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum LoopStep{Absolute{time_bits:u32},Advance{delta_bits:u32}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum FreshLoopError{UnestablishedResourceMapping,UnestablishedGeneration,UnestablishedZeroMaskBoundary,UnsupportedPropertyMode,UnsupportedChannelCount,PendingEventsExceedCapacity,NonFiniteTime,NegativeTime,UnsupportedCounterQuotient,Metadata(MetadataError),Controller(ControllerLoopError),Loop(LoopArithmeticError),X87(X87Error),Sse(SseError)}
impl From<X87Error>for FreshLoopError{fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<SseError>for FreshLoopError{fn from(e:SseError)->Self{Self::Sse(e)}}
impl fmt::Display for FreshLoopError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source fresh isolated loop session: {self:?}")}}
impl std::error::Error for FreshLoopError{}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct FreshLoopState{frames:u32,duration:u64,channels:u32,time:u64,epoch:u32,crossing:u32,resource_generation:u32,next_generation:u32,serial:u32,next_serial:u32,event_capacity:u32}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct FreshLoopProjection{pub returned_fraction:[u8;10],pub fraction_bits:u32,pub frame_bits:u32,pub time_argument_bits:u32,pub stored_time_bits:u64,pub epoch:u32,pub crossing:u32,pub resource_generation:u32,pub next_generation:u32,pub serial:u32,pub next_serial:u32,pub event_capacity:u32,pub resource_frames_bits:u32,pub duration_bits:u64}
impl FreshLoopProjection{pub fn to_le_bytes(self)->[u8;88]{let mut b=[0;88];b[..10].copy_from_slice(&self.returned_fraction);for(at,v)in[(12,self.fraction_bits),(16,self.frame_bits),(20,self.time_argument_bits),(32,self.epoch),(36,self.crossing),(40,self.resource_generation),(44,self.next_generation),(48,self.serial),(52,self.next_serial),(60,self.event_capacity),(64,1),(68,1),(72,0),(76,self.resource_frames_bits)]{b[at..at+4].copy_from_slice(&v.to_le_bytes());}b[24..32].copy_from_slice(&self.stored_time_bits.to_le_bytes());b[80..88].copy_from_slice(&self.duration_bits.to_le_bytes());b}}
impl FreshLoopState{
 fn arithmetic(self,time:u32,context:LoopArithmeticContext)->Result<LoopArithmeticProjection,FreshLoopError>{evaluate_loop_arithmetic(LoopArithmeticInput{time_bits:time,duration_bits:self.duration,resource_frames_bits:self.frames,state_boundary:LoopStateBoundary::CallerSuppliedArithmeticSnapshot},context).map_err(FreshLoopError::Loop)}
 fn projection(self,time:u32,a:LoopArithmeticProjection)->Result<FreshLoopProjection,FreshLoopError>{Ok(FreshLoopProjection{returned_fraction:Extended::from_f32(f32::from_bits(a.normalized_fraction_bits))?.to_le_bytes(),fraction_bits:a.normalized_fraction_bits,frame_bits:a.frame_bits,time_argument_bits:time,stored_time_bits:a.stored_time_bits,epoch:self.epoch,crossing:self.crossing,resource_generation:self.resource_generation,next_generation:self.next_generation,serial:self.serial,next_serial:self.next_serial,event_capacity:self.event_capacity,resource_frames_bits:self.frames,duration_bits:self.duration})}
 /// Full9A2BB0 derives the next argument from this private source time history.
 /// A bounded pending8B-record vector is cleared by the complete source reset;
 /// records are opaque and no eventdispatch or sampling result is inferred.
 pub fn step(&mut self,input:LoopStep,pending_events:u32,context:LoopArithmeticContext)->Result<FreshLoopProjection,FreshLoopError>{
  native_sse::scoped(context.sse,||{
   if pending_events>self.event_capacity{return Err(FreshLoopError::PendingEventsExceedCapacity);}
   let time=match input{LoopStep::Absolute{time_bits}=>{let t=f32::from_bits(time_bits);if !t.is_finite(){return Err(FreshLoopError::NonFiniteTime);}t},LoopStep::Advance{delta_bits}=>{let d=f32::from_bits(delta_bits);if !d.is_finite(){return Err(FreshLoopError::NonFiniteTime);}let retained=context.x87.add(Extended::from_f32(d)?,Extended::from_f64(f64::from_bits(self.time))?)?;context.x87.store_f32(retained).map_err(|e|if e==X87Error::NonFiniteResult{FreshLoopError::NonFiniteTime}else{FreshLoopError::X87(e)})?}};
   if time<0.{return Err(FreshLoopError::NegativeTime);}let arithmetic=self.arithmetic(time.to_bits(),context)?;let mut next=*self;
   if context.x87.compare(Extended::from_f32(time)?,Extended::from_f64(f64::from_bits(self.duration))?)?!=Ordering::Less{
    // The sourcecounter uses its FIRST committed32 quotient, notretained80.
    let q=f32::from_bits(arithmetic.quotient_commit_bits);if !(0.0..2147483648.0).contains(&q){return Err(FreshLoopError::UnsupportedCounterQuotient);}
    let crossed=source_floor(arithmetic.quotient_commit_bits)?;
    if crossed>next.crossing{next.crossing=crossed;next.resource_generation=next.next_generation;next.next_generation=next.next_generation.wrapping_add(1);next.epoch=next.epoch.wrapping_add(1);}
   }
   next.time=arithmetic.stored_time_bits;let projection=next.projection(time.to_bits(),arithmetic)?;*self=next;Ok(projection)
  })
 }
 pub fn channel_count(self)->u32{self.channels}
}
/// Models the full9A2BE0 source boundary, including its initial9A1CB0 call.
/// Native gate must run the completeclock+zero-mask channel loop, not skip it.
/// Sourceglobals are isolated perexplicit session; otheractors cannot interleave.
#[allow(clippy::too_many_arguments)]
pub fn activate_fresh_loop(fresh:FreshAidClockState,animation:&Anim,admission:SourceMetadataAdmission,generation:SourceGeneration,property:PropertyLoopMode,boundary:ZeroMaskBoundary,context:LoopArithmeticContext)->Result<(FreshLoopState,FreshLoopProjection),FreshLoopError>{
 if admission!=SourceMetadataAdmission::ExplicitValidatedV25Resource{return Err(FreshLoopError::UnestablishedResourceMapping);}
 if boundary!=ZeroMaskBoundary::UnboundFreshFourZeroMasks{return Err(FreshLoopError::UnestablishedZeroMaskBoundary);}
 let generation=match generation{SourceGeneration::Unestablished=>return Err(FreshLoopError::UnestablishedGeneration),SourceGeneration::ExplicitIsolatedSnapshot(v)=>v};
 if property!=PropertyLoopMode::Repeat{return Err(FreshLoopError::UnsupportedPropertyMode);}
 let channels=animation.bone_count;if channels==0||channels>16||channels as usize!=animation.bones.len(){return Err(FreshLoopError::UnsupportedChannelCount);}
 let metadata=prepare_metadata(ResourceMetadata{flags:animation.header_flags_280,frames_bits:animation.duration.to_bits(),frame_interval_bits:animation.frame_duration.to_bits()},MetadataContext{x87:context.x87,sse:context.sse}).map_err(FreshLoopError::Metadata)?;
 let controller=ControllerLoopSnapshot{primary_index:0,secondary_index:1,primary:ControllerSlot{epoch:0,linked_handle:0,has_aid:true,loop_request:0},secondary:ControllerSlot{epoch:0,linked_handle:0,has_aid:false,loop_request:0},aid_loop_bytes:[u8::from(metadata.constructor_default_loop),0]};
 let selected=apply_property_loop(controller,1,property).map_err(FreshLoopError::Controller)?;debug_assert!(selected.source_success&&selected.resolved_primary&&selected.snapshot.aid_loop_bytes[0]==1);
 let state=FreshLoopState{frames:animation.duration.to_bits(),duration:metadata.duration_bits,channels,time:0,epoch:fresh.cycle_epoch().wrapping_add(1),crossing:0,resource_generation:generation,next_generation:generation.wrapping_add(1),serial:fresh.next_global_serial(),next_serial:fresh.next_global_serial().wrapping_add(1),event_capacity:fresh.event_capacity()};
 let a=state.arithmetic(0,context)?;let projection=state.projection(0,a)?;Ok((state,projection))
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn source_floor(bits:u32)->Result<u32,SseError>{let out:u32;
 // SAFETY: literalsourceinstructions under scopedMXCSR1F80,finite0<=q<2^31.
 unsafe{std::arch::asm!("movd xmm0,{bits:e}","cvtss2si eax,xmm0","cvtsi2ss xmm1,eax","mov ecx,eax","sub ecx,1","ucomiss xmm0,xmm1","cmovb eax,ecx",bits=in(reg)bits,out("eax")out,out("ecx")_,out("xmm0")_,out("xmm1")_,options(nostack));}Ok(out)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn source_floor(_:u32)->Result<u32,SseError>{Err(SseError::UnsupportedArchitecture)}
