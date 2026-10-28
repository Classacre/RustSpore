//! Candidate explicitv25 source metadata -> accepted clock/root-data composition.
//! This constructor-default path does not apply A00B30 queued forced-loop policy.
use crate::{anim::Anim,cell_runtime_clip_metadata::{prepare_metadata,ResourceMetadata,MetadataContext,ClipMetadata,MetadataError},cell_runtime_clock::{ClockInput,ClockTime},cell_runtime_clock_root_data::{evaluate_clocked_root_data,ClockRootDataError},cell_runtime_data::AGGREGATE_SIZE,cell_runtime_root_data::{ConsumerContext,PreparedChannel,RootBinding,RootDataContext,RootDataEvaluation},cell_runtime_clock::ClockProjection};
use std::fmt;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SourceMetadataAdmission {Unestablished,ExplicitValidatedV25Resource}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum SourceClockRootDataError {UnestablishedResourceMapping,Metadata(MetadataError),Composition(ClockRootDataError)}
impl fmt::Display for SourceClockRootDataError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"source metadata clock/root data: {self:?}")}}
impl std::error::Error for SourceClockRootDataError{}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SourceClockRootDataEvaluation{pub metadata:ClipMetadata,pub clock:ClockProjection,pub root:RootDataEvaluation}
/// Caller validates original ANIMv25 reader path, relocation/descriptor noalias
/// bounds, explicit selected resource and prepared binding. No live actor lookup.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_source_clocked_root_data(animation:&Anim,prepared:&[PreparedChannel],bindings:&[RootBinding],sample_mask:u64,interpolation_mask:u64,selected_ordinal:u8,time:ClockTime,admission:SourceMetadataAdmission,prior_root_aggregate:[u8;AGGREGATE_SIZE],consumer_context:ConsumerContext,context:RootDataContext)->Result<SourceClockRootDataEvaluation,SourceClockRootDataError>{
 if admission!=SourceMetadataAdmission::ExplicitValidatedV25Resource{return Err(SourceClockRootDataError::UnestablishedResourceMapping);}
 let metadata=prepare_metadata(ResourceMetadata{flags:animation.header_flags_280,frames_bits:animation.duration.to_bits(),frame_interval_bits:animation.frame_duration.to_bits()},MetadataContext{x87:context.x87,sse:context.sse}).map_err(SourceClockRootDataError::Metadata)?;
 let result=evaluate_clocked_root_data(animation,prepared,bindings,sample_mask,interpolation_mask,selected_ordinal,ClockInput{time,duration_bits:metadata.duration_bits,resource_frames_bits:animation.duration.to_bits(),looping:metadata.constructor_default_loop},prior_root_aggregate,consumer_context,context).map_err(SourceClockRootDataError::Composition)?;
 Ok(SourceClockRootDataEvaluation{metadata,clock:result.clock,root:result.root})
}
