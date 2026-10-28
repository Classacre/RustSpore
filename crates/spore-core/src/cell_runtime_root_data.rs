//! Explicit single-clip root scalar evaluation across recovered source stages.
//!
//! 9A1CB0/9A1590 sample before 9BD160 resets only pose+A0 count; 99D190
//! then consumes ordered bindings and E563F0 may read data2. Active resource,
//! actor ownership, event dispatch, POS/ROT effects and looping remain explicit
//! unresolved boundaries. Native comparisons admit the explicit-frame path.
use crate::{
    anim::{Anim, AnimComponent},
    cell_runtime_contributions::{accumulate_scalar_components, CellRuntimeContributionContext,
        CellRuntimeContributionError, CellRuntimeContributions, CellRuntimeScalarSlot},
    cell_runtime_curves::{sample_component, CellRuntimeCurveContext, CellRuntimeCurveError},
    cell_runtime_data::{CellRuntimeDataAccumulator, CellRuntimeDataError, AGGREGATE_SIZE},
    cell_runtime_modifiers::{data2_multiplier, ActorAnimationData, Data2Multiplier},
    native_sse::{self, Policy, SseError},
    native_x87::{Context, X87Error},
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootDataContext { pub x87: Context, pub sse: Policy }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameInput {
    /// Already supplied at 9A1590. No clock provenance is inferred.
    ExplicitSourceFrame(u32),
    /// Reserved source-clock boundary. Always rejected until an independent
    /// native comparison admits its x87 divide/commit and SSE frame products.
    NonLoopFirstDuration { time_bits: u32, duration_bits: u64, resource_frames_bits: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedChannel {
    pub descriptors: Vec<AnimComponent>,
    /// Unwritten/disabled/empty sample slots retain the supplied prior snapshot.
    pub prior_slots: [CellRuntimeScalarSlot; 6],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootBinding {
    /// Literal source 16-byte row. Byte+C must be zero when reached;
    /// byte+D selects the sampled wrapper, independently of component tags.
    pub record: [u32; 4],
    /// Outputs of earlier POS/ROT descriptor processing, supplied explicitly.
    pub prior_position_mask: u32,
    pub prior_rotation_mask: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerContext {
    UnresolvedActorAssociation,
    /// Caller explicitly asserts that E51760 resolves this root aggregate.
    ExplicitResolvedRoot { actor_state_18c: i32, actor_handle_248: u32, actor_word_fc: u32 },
    ExplicitMissingObject { actor_state_18c: i32, actor_handle_248: u32, actor_word_fc: u32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerOutcome { UnresolvedActorAssociation, Evaluated(Data2Multiplier) }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootDataError {
    PreparedChannelCount,
    UnsupportedClockDomain,
    NonFiniteFrame,
    ReachedNonRootPose { binding: usize, pose: u8 },
    MissingWrapper { binding: usize, channel: u8 },
    SampleSlot { channel: usize, component: usize, slot: u32 },
    Curve { channel: usize, component: usize, error: CellRuntimeCurveError },
    Contribution { binding: usize, error: CellRuntimeContributionError },
    Data(CellRuntimeDataError), X87(X87Error), Sse(SseError),
}
impl From<SseError> for RootDataError { fn from(e: SseError) -> Self { Self::Sse(e) } }
impl From<X87Error> for RootDataError { fn from(e: X87Error) -> Self { Self::X87(e) } }
impl From<CellRuntimeDataError> for RootDataError { fn from(e: CellRuntimeDataError) -> Self { Self::Data(e) } }
impl fmt::Display for RootDataError {
    fn fmt(&self,f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f,"explicit source root-data evaluation: {self:?}") }
}
impl std::error::Error for RootDataError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleWrite { pub channel: usize, pub component: usize, pub slot: u32, pub value_bits: u32, pub weight_bits: u32 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootDataEvaluation {
    pub frame_bits: u32,
    pub sampled_slots: Vec<[CellRuntimeScalarSlot; 6]>,
    pub sample_writes: Vec<SampleWrite>,
    pub aggregate: CellRuntimeDataAccumulator,
    pub binding_contributions: Vec<(usize, CellRuntimeContributions)>,
    pub consumer: ConsumerOutcome,
}

/// Validate an explicit source frame. Clock conversion remains a typed gate.
pub fn source_frame(input: FrameInput, context: RootDataContext) -> Result<u32,RootDataError> {
    native_sse::scoped(context.sse,|| {
        match input {
            FrameInput::ExplicitSourceFrame(bits) => {
                if !f32::from_bits(bits).is_finite() { return Err(RootDataError::NonFiniteFrame); }
                Ok(bits)
            }
            FrameInput::NonLoopFirstDuration { .. } => Err(RootDataError::UnsupportedClockDomain),
        }
    })
}

/// Evaluate a source-selected root snapshot, keeping live ownership unresolved.
///
/// Sampling visits every enabled kind3 descriptor in channel/descriptor order,
/// including descriptors excluded by a later binding mask. Repeated sample slots
/// overwrite in source order. Empty/disabled results preserve prior slot bits.
/// The root aggregate reset writes only count=0, retaining all eight row payloads.
/// Only ordinary root bindings are admitted; reached pose bytes other than0 are
/// errors. POS/ROT processing is represented solely by supplied prior masks.
/// Errors discard this owned result, rather than promising source partial state.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_single_clip_root_data(
    animation: &Anim, prepared: &[PreparedChannel], bindings: &[RootBinding],
    sample_mask: u64, interpolation_mask: u64, selected_ordinal: u8,
    frame_input: FrameInput, prior_root_aggregate: [u8;AGGREGATE_SIZE],
    consumer_context: ConsumerContext, context: RootDataContext,
) -> Result<RootDataEvaluation,RootDataError> {
    if prepared.len()!=animation.bones.len() { return Err(RootDataError::PreparedChannelCount); }
    let frame_bits = source_frame(frame_input,context)?;
    let mut sampled_slots: Vec<_> = prepared.iter().map(|channel|channel.prior_slots).collect();
    let mut sample_writes = Vec::new();
    for (channel, (bone, channel_prepared)) in animation.bones.iter().zip(prepared).enumerate() {
        let bit = 1u64.checked_shl(channel as u32).unwrap_or(0);
        // Original 9A1590 tests channel+D4 and channel+AC low bit.
        if sample_mask & bit == 0 || interpolation_mask & bit == 0 || bone.keyframe_count==0
            || bone.selector_words[9] & 1 == 0 { continue; }
        for (component, descriptor) in channel_prepared.descriptors.iter().enumerate() {
            if descriptor.flags & 15 != 3 { continue; }
            let sample = sample_component(animation,channel,&channel_prepared.descriptors,component,
                f32::from_bits(frame_bits),CellRuntimeCurveContext { x87:context.x87,sse:context.sse })
                .map_err(|error|RootDataError::Curve { channel,component,error })?;
            if let Some(sample)=sample {
                if !(3..9).contains(&descriptor.slot) {
                    return Err(RootDataError::SampleSlot { channel,component,slot:descriptor.slot });
                }
                let slot = CellRuntimeScalarSlot { value_bits:sample.value.to_bits(),weight_bits:sample.weight.to_bits() };
                sampled_slots[channel][(descriptor.slot-3) as usize]=slot;
                sample_writes.push(SampleWrite { channel,component,slot:descriptor.slot,
                    value_bits:slot.value_bits,weight_bits:slot.weight_bits });
            }
        }
    }
    let mut reset = prior_root_aggregate;
    reset[..4].fill(0); // Native 9BD21A; no aggregate row clearing.
    let mut aggregate = CellRuntimeDataAccumulator::from_snapshot(reset)?;
    let mut binding_contributions = Vec::new();
    let selected_bit=1u64.checked_shl(u32::from(selected_ordinal)).unwrap_or(0);
    for (binding,input) in bindings.iter().enumerate() {
        let record=input.record;
        let bytes=record[3].to_le_bytes();
        let groups=u64::from(record[0]) | (u64::from(record[1])<<32);
        if bytes[3]&9!=0 || groups&selected_bit==0 { continue; }
        if bytes[0]!=0 { return Err(RootDataError::ReachedNonRootPose { binding,pose:bytes[0] }); }
        let channel=usize::from(bytes[1]);
        let Some(channel_prepared)=prepared.get(channel) else {
            return Err(RootDataError::MissingWrapper { binding,channel:bytes[1] });
        };
        let contribution=accumulate_scalar_components(&mut aggregate,&channel_prepared.descriptors,
            &sampled_slots[channel],record[2]&0xffffff,input.prior_position_mask,input.prior_rotation_mask,
            0x3f800000,CellRuntimeContributionContext { x87:context.x87,sse:context.sse })
            .map_err(|error|RootDataError::Contribution { binding,error })?;
        binding_contributions.push((binding,contribution));
    }
    let consumer=match consumer_context {
        ConsumerContext::UnresolvedActorAssociation=>ConsumerOutcome::UnresolvedActorAssociation,
        ConsumerContext::ExplicitResolvedRoot { actor_state_18c,actor_handle_248,actor_word_fc }=>
            ConsumerOutcome::Evaluated(data2_multiplier(actor_state_18c,ActorAnimationData {
                actor_handle_248,actor_word_fc,resolved_aggregate:Some(aggregate.reader()) },context.x87)?),
        ConsumerContext::ExplicitMissingObject { actor_state_18c,actor_handle_248,actor_word_fc }=>
            ConsumerOutcome::Evaluated(data2_multiplier(actor_state_18c,ActorAnimationData {
                actor_handle_248,actor_word_fc,resolved_aggregate:None },context.x87)?),
    };
    Ok(RootDataEvaluation { frame_bits,sampled_slots,sample_writes,aggregate,binding_contributions,consumer })
}
