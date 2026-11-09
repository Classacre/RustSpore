//! Bounded E6A450 prefix through the d==7 exit or d==6 submission boundary.
//!
//! This consumes caller-admitted feature and actor snapshots. It does not
//! construct geometry, resolve actors or implement the remaining producer.
//! The submission is a pending E59B40 call: this module makes no output writes
//! and does not invoke the E6A2B0 tail on any modeled exit.
use crate::cell_feeding_contact_arbitration_e59b40_v1::ContactOutput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixInput {
    pub feature_a: i32, pub feature_b: i32,
    pub definition_a: i32, pub definition_b: i32,
    pub actor_ids: [u32; 2], pub current_player_id: u32,
    pub feature_indices: [u32; 2], pub depth_bits: u32,
    pub point_words: [u32; 3], pub normal_words: [u32; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixResult {
    FeatureNineExit,
    DefinitionASevenExit,
    PendingSubmission(ContactOutput),
    /// The caller must perform this E59B40 call and explicitly admit its
    /// mutations, pointer/geometry lifetime and x87 environment/stack before
    /// continuing. The source preserves some pointers in nonvolatile registers
    /// and re-reads other memory; blindly replacing all values with new
    /// snapshots is not equivalent. See the source continuation note. This
    /// model performs neither call and does not provide a resume operation.
    PendingContinuationSubmission(ContactOutput),
    /// Stop before all unmodeled remaining operations. No effects are implied.
    UnsupportedContinuation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixError { NonFiniteDepth }

/// Finite binary32 depth and masked x87 exceptions are admitted only for the
/// modeled submission. Pointer identity, non-aliasing, fresh actor/player
/// reads and valid x87 stack are caller duties; sticky flags are not modeled.
pub fn evaluate_prefix(input: PrefixInput) -> Result<PrefixResult, PrefixError> {
    if input.feature_a == 9 || input.feature_b == 9 { return Ok(PrefixResult::FeatureNineExit); }
    if input.definition_b == 7 {
        if !f32::from_bits(input.depth_bits).is_finite() { return Err(PrefixError::NonFiniteDepth); }
        return Ok(PrefixResult::PendingSubmission(ContactOutput {
            actor_ids: input.actor_ids,
            selector: if input.actor_ids[0] == input.current_player_id { 19 } else { 0 },
            feature_indices: input.feature_indices, depth_bits: input.depth_bits,
            point_words: input.point_words, normal_words: input.normal_words,
        }));
    }
    if input.definition_a == 7 { return Ok(PrefixResult::DefinitionASevenExit); }
    if input.definition_b == 6 {
        if !f32::from_bits(input.depth_bits).is_finite() { return Err(PrefixError::NonFiniteDepth); }
        return Ok(PrefixResult::PendingContinuationSubmission(ContactOutput {
            actor_ids: input.actor_ids,
            selector: if input.actor_ids[0] == input.current_player_id { 18 } else { 1 },
            feature_indices: input.feature_indices, depth_bits: input.depth_bits,
            point_words: input.point_words, normal_words: input.normal_words,
        }));
    }
    Ok(PrefixResult::UnsupportedContinuation)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input()->PrefixInput { PrefixInput { feature_a:2,feature_b:3,definition_a:7,definition_b:7,
        actor_ids:[4,5],current_player_id:4,feature_indices:[0x8000_0000,u32::MAX],
        depth_bits:1,point_words:[1,2,3],normal_words:[4,5,6] } }
    #[test] fn feature_nine_short_circuits_even_nonfinite_depth() {
        let mut i=input();i.feature_b=9;i.depth_bits=f32::NAN.to_bits();
        assert_eq!(evaluate_prefix(i),Ok(PrefixResult::FeatureNineExit));
    }
    #[test] fn b_seven_precedes_a_seven_and_preserves_candidate_words() {
        let i=input();let Ok(PrefixResult::PendingSubmission(c))=evaluate_prefix(i) else { panic!() };
        assert_eq!(c.selector,19);assert_eq!(c.actor_ids,[4,5]);assert_eq!(c.depth_bits,1);
        assert_eq!(c.feature_indices,i.feature_indices);assert_eq!(c.point_words,i.point_words);assert_eq!(c.normal_words,i.normal_words);
        let mut other=i;other.current_player_id=99;
        let Ok(PrefixResult::PendingSubmission(c))=evaluate_prefix(other) else { panic!() };assert_eq!(c.selector,0);
    }
    #[test] fn a_seven_and_unmodeled_remainder_stop_without_submitting() {
        let mut i=input();i.definition_b=6;
        assert_eq!(evaluate_prefix(i),Ok(PrefixResult::DefinitionASevenExit));
        i.definition_a=0;i.definition_b=5;assert_eq!(evaluate_prefix(i),Ok(PrefixResult::UnsupportedContinuation));
    }
    #[test] fn six_produces_pending_submission_and_stops_before_effects() {
        let mut i=input();i.definition_a=0;i.definition_b=6;
        let Ok(PrefixResult::PendingContinuationSubmission(c))=evaluate_prefix(i) else {panic!()};
        assert_eq!(c.selector,18);assert_eq!(c.actor_ids,i.actor_ids);assert_eq!(c.feature_indices,i.feature_indices);
        assert_eq!(c.depth_bits,i.depth_bits);assert_eq!(c.point_words,i.point_words);assert_eq!(c.normal_words,i.normal_words);
        i.current_player_id=99;
        let Ok(PrefixResult::PendingContinuationSubmission(c))=evaluate_prefix(i) else {panic!()};assert_eq!(c.selector,1);
        i.depth_bits=f32::NAN.to_bits();assert_eq!(evaluate_prefix(i),Err(PrefixError::NonFiniteDepth));
    }
    #[test] fn nonfinite_depth_is_rejected_only_when_consumed() {
        let mut i=input();i.depth_bits=f32::INFINITY.to_bits();
        assert_eq!(evaluate_prefix(i),Err(PrefixError::NonFiniteDepth));
        i.definition_b=0;assert_eq!(evaluate_prefix(i),Ok(PrefixResult::DefinitionASevenExit));
    }
}
