//! Candidate v2 full first-channel sampler after private fresh clock advancement.
//! No native admission yet. Original defaults derive from99EA40/9A1450;
//! caller frames/counters/payload defaults are not accepted. Full binding and
//! allocator/actor/registry admission remain separate from borrowed source data.
use crate::{anim::{Anim,AnimComponent},anim_curve::{self,CurveError},
 anim_rotation_curve::{sample_rotation,RotationBlendPolicy,RotationCurveError},
 anim_quaternion::{ArithmeticContext,SsePolicy},native_crt_acos::ConstantWindow,
 cell_runtime_curves::{sample_component,CellRuntimeCurveContext,CellRuntimeCurveError},
 cell_runtime_fresh_aid::FreshAidClockState,cell_runtime_fresh_loop::{activate_fresh_loop,FreshLoopState,FreshLoopProjection,FreshLoopError,LoopStep,SourceGeneration,ZeroMaskBoundary},
 cell_runtime_source_clock_root_data::SourceMetadataAdmission,cell_runtime_controller_loop::PropertyLoopMode,
 cell_runtime_loop_clock::LoopArithmeticContext,cell_runtime_contributions::CellRuntimeScalarSlot,
 native_sse,native_x87::{Extended,X87Error}};
use std::{cmp::Ordering,fmt};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum WrapperSerialSeed{Unestablished,ExplicitIsolatedSnapshot(u32)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum SamplingBoundary{Unestablished,SourceFreshFirstChannelBothMasks}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum RotationDispatch{Unestablished,ExplicitSourceImageInitialOne}
#[derive(Clone,Copy)]pub struct FreshSampleContext<'a>{pub clock:LoopArithmeticContext,pub rotation_dispatch:RotationDispatch,pub acos_constants:ConstantWindow<'a>}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum FreshSampleError{UnestablishedWrapperSerial,UnestablishedSamplingBoundary,UnestablishedRotationDispatch,UnsupportedChannel,UnsupportedDescriptors,InvalidRows,UnsupportedTimestamp,EventCapacity,Sse(native_sse::SseError),Clock(FreshLoopError),Position(CurveError),Rotation(RotationCurveError),Scalar(CellRuntimeCurveError),X87(X87Error)}
impl fmt::Display for FreshSampleError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source fresh sampled channel: {self:?}")}}
impl std::error::Error for FreshSampleError{}
impl From<X87Error>for FreshSampleError{fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<native_sse::SseError>for FreshSampleError{fn from(e:native_sse::SseError)->Self{Self::Sse(e)}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ChannelEvent{pub channel:u16,pub key:u32}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct FreshSampleProjection{pub clock:FreshLoopProjection,pub payload:[u8;440],pub events:Vec<ChannelEvent>}
impl FreshSampleProjection{
 /// Event paddingU16 is deliberately absent; source initializes only6of8bytes.
 pub fn to_le_bytes(&self)->[u8;544]{let mut b=[0;544];b[..88].copy_from_slice(&self.clock.to_le_bytes());b[56..60].copy_from_slice(&(self.events.len()as u32).to_le_bytes());b[88..528].copy_from_slice(&self.payload);b[528..532].copy_from_slice(&(self.events.len()as u32).to_le_bytes());for(i,e)in self.events.iter().enumerate(){let at=532+i*6;b[at..at+2].copy_from_slice(&e.channel.to_le_bytes());b[at+2..at+6].copy_from_slice(&e.key.to_le_bytes());}b}
 pub fn scalar_slots(&self)->[CellRuntimeScalarSlot;6]{std::array::from_fn(|i|{let at=200+40*i;CellRuntimeScalarSlot{value_bits:u32::from_le_bytes(self.payload[at..at+4].try_into().unwrap()),weight_bits:u32::from_le_bytes(self.payload[at+4..at+8].try_into().unwrap())}})}
}
#[derive(Clone)]pub struct FreshSampleState<'a>{clock:FreshLoopState,animation:&'a Anim,payload:[u8;440],fresh_payload:[u8;440],next_wrapper_serial:u32,event_capacity:u32}
impl FreshSampleState<'_>{
 pub fn source_constructor_payload(&self)->[u8;440]{self.fresh_payload}
 pub fn next_wrapper_serial(&self)->u32{self.next_wrapper_serial}
 pub fn step(&mut self,input:LoopStep,pending_events:u32,context:FreshSampleContext<'_>)->Result<FreshSampleProjection,FreshSampleError>{
  native_sse::scoped(context.clock.sse,||{validate_dispatch(context)?;let mut next_clock=self.clock;let clock=next_clock.step(input,pending_events,context.clock).map_err(FreshSampleError::Clock)?;let result=sample(self.animation,self.payload,clock,self.event_capacity,context)?;self.clock=next_clock;self.payload=result.payload;Ok(result)})
 }
}
fn validate_dispatch(context:FreshSampleContext<'_>)->Result<(),FreshSampleError>{if context.rotation_dispatch!=RotationDispatch::ExplicitSourceImageInitialOne{Err(FreshSampleError::UnestablishedRotationDispatch)}else{Ok(())}}
/// Original source constructor zeroes440bytes, then sets INFOword+4 to its old
/// globalserial. Everyactualchannel wrapper is initialized; sampledchannel0
/// keeps the first serial and global advances by the actualchannelcount.
#[allow(clippy::too_many_arguments)]
pub fn activate_fresh_sampled_channel<'a>(fresh:FreshAidClockState,animation:&'a Anim,admission:SourceMetadataAdmission,generation:SourceGeneration,wrapper_serial:WrapperSerialSeed,boundary:SamplingBoundary,context:FreshSampleContext<'_>)->Result<(FreshSampleState<'a>,FreshSampleProjection),FreshSampleError>{
 native_sse::scoped(context.clock.sse,||{validate_dispatch(context)?;if boundary!=SamplingBoundary::SourceFreshFirstChannelBothMasks{return Err(FreshSampleError::UnestablishedSamplingBoundary);}let seed=match wrapper_serial{WrapperSerialSeed::Unestablished=>return Err(FreshSampleError::UnestablishedWrapperSerial),WrapperSerialSeed::ExplicitIsolatedSnapshot(v)=>v};validate_source(animation)?;if fresh.event_capacity()<2{return Err(FreshSampleError::EventCapacity);}
 let mut payload=[0;440];payload[4..8].copy_from_slice(&seed.to_le_bytes());
 // Reuse the accepted privateclock scalar-state submodel, then apply the new
 // complete sampling/event model. Its zero-mask projection's vector length is
 // replaced by the actual event count in this new projection; old gate scope
 // does not validate the new nativecallchain.
 let(clock,initial)=activate_fresh_loop(fresh,animation,admission,generation,PropertyLoopMode::Repeat,ZeroMaskBoundary::UnboundFreshFourZeroMasks,context.clock).map_err(FreshSampleError::Clock)?;
 let result=sample(animation,payload,initial,fresh.event_capacity(),context)?;let state=FreshSampleState{clock,animation,payload:result.payload,fresh_payload:payload,next_wrapper_serial:seed.wrapping_add(animation.bone_count),event_capacity:fresh.event_capacity()};Ok((state,result))})
}
fn validate_source(animation:&Anim)->Result<(),FreshSampleError>{
 if animation.bone_count==0||animation.bone_count>16||animation.bones.len()!=animation.bone_count as usize{return Err(FreshSampleError::UnsupportedChannel);}let bone=&animation.bones[0];
 if bone.keyframe_count==0||bone.keyframe_count>46||bone.keyframes.len()!=bone.keyframe_count as usize||bone.selector_words[9]&1==0||bone.components.len()<3||bone.components.len()>6{return Err(FreshSampleError::UnsupportedChannel);}
 let mut counts=[0;4];for c in &bone.components{let kind=(c.flags&15)as usize;if kind>3||c.stride!=bone.keyframe_stride||c.slot>=9||kind<3&&c.slot!=kind as u32||kind==3&&c.slot<3{return Err(FreshSampleError::UnsupportedDescriptors);}counts[kind]+=1;}
 if counts[0]!=1||counts[1]!=1||counts[2]!=1||bone.components[0].flags&15!=0||bone.components[0].tag!=*b"INFO"||bone.components[0].offset!=0{return Err(FreshSampleError::UnsupportedDescriptors);}
 if bone.raw_keyframes.len()!=bone.keyframes.len()*bone.keyframe_stride as usize{return Err(FreshSampleError::InvalidRows);}
 let info=&bone.components[0];let mut events=0;let mut previous=0;
 for(i,key)in bone.keyframes.iter().enumerate(){if key.frame>=1<<24||i>0&&key.frame<previous{return Err(FreshSampleError::UnsupportedTimestamp);}previous=key.frame;let row=row(animation,info,i,20)?;if u32::from_le_bytes(row[..4].try_into().unwrap())!=key.frame{return Err(FreshSampleError::InvalidRows);}events+=usize::from(row[10]!=0);}
 if events>2{return Err(FreshSampleError::EventCapacity);}Ok(())
}
fn row<'a>(animation:&'a Anim,c:&AnimComponent,index:usize,n:usize)->Result<&'a[u8],FreshSampleError>{let bone=&animation.bones[0];let offset=c.offset as usize;let stride=c.stride as usize;if offset.checked_add(n).is_none_or(|end|end>stride){return Err(FreshSampleError::InvalidRows);}let start=index.checked_mul(stride).and_then(|v|v.checked_add(offset)).ok_or(FreshSampleError::InvalidRows)?;bone.raw_keyframes.get(start..start+n).ok_or(FreshSampleError::InvalidRows)}
fn sample(animation:&Anim,mut payload:[u8;440],clock:FreshLoopProjection,event_capacity:u32,context:FreshSampleContext<'_>)->Result<FreshSampleProjection,FreshSampleError>{
 let bone=&animation.bones[0];let prepared=&bone.components;let frame=f32::from_bits(clock.frame_bits);let f=Extended::from_f32(frame)?;let info=&prepared[0];let mut upper=0;let mut events=Vec::new();
 for(i,key)in bone.keyframes.iter().enumerate(){let timestamp=anim_curve::unsigned_word(key.frame,context.clock.x87).map_err(FreshSampleError::Position)?;if context.clock.x87.compare(f,timestamp)?==Ordering::Less{break;}upper+=1;
  // Original9A3140 zeroes the privatekeyepoch table. This gate doesnotdispatch
  // events or stamp it externally; literal0<privateepoch is the actualpredicate.
  if row(animation,info,i,20)?[10]!=0&&0<clock.epoch{events.push(ChannelEvent{channel:0,key:i as u32});}
 }
 if events.len()>2||events.len()>event_capacity as usize{return Err(FreshSampleError::EventCapacity);}
 let neighbors=anim_curve::key_neighbors(bone.keyframes.len(),upper,animation.header_flags_280&8!=0).map_err(FreshSampleError::Position)?;
 if neighbors[1]==neighbors[2]{for c in prepared{let n=match c.flags&15{0=>20,1=>80,2=>100,3=>40,_=>return Err(FreshSampleError::UnsupportedDescriptors)};let dst=slot(c.slot);payload[dst..dst+n].copy_from_slice(row(animation,c,neighbors[1],n)?);}}
 else{
  if let Some(p)=anim_curve::sample_position(animation,0,prepared,frame,context.clock.x87).map_err(FreshSampleError::Position)?{for(i,v)in p.position.into_iter().chain([p.weight]).enumerate(){payload[20+i*4..24+i*4].copy_from_slice(&v.to_bits().to_le_bytes());}}
  if let Some(r)=sample_rotation(animation,0,prepared,frame,ArithmeticContext{x87:context.clock.x87,sse:SsePolicy::Mxcsr1f80},RotationBlendPolicy::NormalizedHemisphere,context.acos_constants).map_err(FreshSampleError::Rotation)?{for(i,v)in r.rotation_xyzw.into_iter().chain([r.weight]).enumerate(){payload[100+i*4..104+i*4].copy_from_slice(&v.to_bits().to_le_bytes());}}
  for(i,c)in prepared.iter().enumerate(){if c.flags&15==3{if let Some(s)=sample_component(animation,0,prepared,i,frame,CellRuntimeCurveContext{x87:context.clock.x87,sse:context.clock.sse}).map_err(FreshSampleError::Scalar)?{let dst=slot(c.slot);payload[dst..dst+4].copy_from_slice(&s.value.to_bits().to_le_bytes());payload[dst+4..dst+8].copy_from_slice(&s.weight.to_bits().to_le_bytes());}}}
  payload[..20].copy_from_slice(row(animation,info,neighbors[1],20)?);
 }
 Ok(FreshSampleProjection{clock,payload,events})
}
fn slot(slot:u32)->usize{match slot{0=>0,1=>20,2=>100,3..=8=>200+(slot as usize-3)*40,_=>unreachable!("validatedsource slot")}}
