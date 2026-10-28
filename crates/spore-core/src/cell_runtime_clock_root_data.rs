//! Composition candidate: accepted first-duration clock -> explicit-frame root.
//!
//! Duration, source resource, prepared binding/slot state and actor association
//! remain caller inputs. This does not enable the older root adapter's reserved
//! clock variant, choose active clips or claim live Cell registry routing.
use crate::{anim::Anim,
    cell_runtime_clock::{evaluate_first_duration,ClockContext,ClockError,ClockInput,ClockProjection},
    cell_runtime_data::AGGREGATE_SIZE,
    cell_runtime_root_data::{evaluate_single_clip_root_data,ConsumerContext,FrameInput,
        PreparedChannel,RootBinding,RootDataContext,RootDataError,RootDataEvaluation}};
use std::fmt;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ClockRootDataError {ResourceFrameWordMismatch,Clock(ClockError),Root(RootDataError)}
impl fmt::Display for ClockRootDataError {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {write!(f,"explicit clock/root-data composition: {self:?}")}
}
impl std::error::Error for ClockRootDataError {}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ClockRootDataEvaluation {pub clock:ClockProjection,pub root:RootDataEvaluation}

/// Join explicit source resource/frame context through the two admitted kernels.
/// The literal frame-count input must agree with this parsed resource's retained
/// frame-count word; no duration setter or resource-loading policy is inferred.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_clocked_root_data(animation:&Anim,prepared:&[PreparedChannel],bindings:&[RootBinding],
    sample_mask:u64,interpolation_mask:u64,selected_ordinal:u8,clock_input:ClockInput,
    prior_root_aggregate:[u8;AGGREGATE_SIZE],consumer_context:ConsumerContext,context:RootDataContext,
)->Result<ClockRootDataEvaluation,ClockRootDataError> {
    if clock_input.resource_frames_bits!=animation.duration.to_bits() {
        return Err(ClockRootDataError::ResourceFrameWordMismatch);
    }
    let clock=evaluate_first_duration(clock_input,ClockContext {x87:context.x87,sse:context.sse})
        .map_err(ClockRootDataError::Clock)?;
    let root=evaluate_single_clip_root_data(animation,prepared,bindings,sample_mask,interpolation_mask,
        selected_ordinal,FrameInput::ExplicitSourceFrame(clock.frame_bits),prior_root_aggregate,
        consumer_context,context).map_err(ClockRootDataError::Root)?;
    Ok(ClockRootDataEvaluation {clock,root})
}
