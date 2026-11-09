//! Bounded E6A2B0 tail after caller-admitted external calls and snapshots.
//!
//! The caller must obtain the relation-five branch outcome in source order,
//! including every external call and any intervening mutations. Actor IDs,
//! geometry and output must be freshly supplied at the final boundary and must
//! not alias. No callback, source object, floating exception or event is modeled.
use crate::cell_feeding_contact_arbitration_e59b40_v1::ContactOutput;

/// Result of the ordered relation-five tests, supplied after their effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationFiveOutcome {
    /// E6A250 returned a nonzero low byte, or a later identity/kind test passed.
    DirectPriority22,
    /// All direct-path tests failed; E59B40 is the next external call.
    Submit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailInput {
    pub actor_ids: [u32; 2],
    pub point_words: [u32; 3],
    pub normal_words: [u32; 3],
    pub depth_bits: u32,
    pub relation: i32,
    pub word_20: u32,
    /// Required for relation 5; ignored for other relations.
    pub relation_five: Option<RelationFiveOutcome>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TailResult {
    DirectWrite,
    DirectSkipped,
    /// Caller must invoke the separately admitted E59B40 boundary; output has
    /// not been changed by this model and the returned record is only input.
    PendingSubmission(ContactOutput),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TailError { MissingRelationFiveOutcome, NonFiniteDepth }

/// Finite binary32, nearest/masked environment with subnormals preserved is an
/// explicit admission. Comparisons are integer based to avoid host FP modes.
/// Errors have no effect. Nonfinite operands are rejected even when a priority
/// decision would skip comparison; this is a conservative checked restriction.
pub fn evaluate_tail(output: &mut ContactOutput, input: TailInput) -> Result<TailResult, TailError> {
    if !f32::from_bits(input.depth_bits).is_finite() || !f32::from_bits(output.depth_bits).is_finite() {
        return Err(TailError::NonFiniteDepth);
    }
    let selector = if input.relation != 5 { 1 } else {
        match input.relation_five.ok_or(TailError::MissingRelationFiveOutcome)? {
            RelationFiveOutcome::DirectPriority22 => 0x16,
            RelationFiveOutcome::Submit => if input.word_20 == 0 { 0x16 } else { 0x11 },
        }
    };
    let candidate = ContactOutput { actor_ids: input.actor_ids, selector,
        feature_indices: [u32::MAX; 2], depth_bits: input.depth_bits,
        point_words: input.point_words, normal_words: input.normal_words };
    if input.relation == 5 && input.relation_five == Some(RelationFiveOutcome::Submit) {
        return Ok(TailResult::PendingSubmission(candidate));
    }
    let greater = finite_greater(input.depth_bits, output.depth_bits);
    if output.selector < selector || (output.selector == selector && greater) {
        *output = candidate;
        Ok(TailResult::DirectWrite)
    } else { Ok(TailResult::DirectSkipped) }
}

fn finite_greater(a: u32, b: u32) -> bool {
    if a & 0x7fff_ffff == 0 && b & 0x7fff_ffff == 0 { return false; }
    if (a ^ b) >> 31 != 0 { return a >> 31 == 0; }
    if a >> 31 != 0 { a < b } else { a > b }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> TailInput { TailInput { actor_ids:[8,9], point_words:[1,2,3], normal_words:[4,5,6], depth_bits:1f32.to_bits(), relation:0, word_20:0, relation_five:None } }
    fn output(selector:i32, depth:f32) -> ContactOutput { ContactOutput { actor_ids:[0,0], selector, feature_indices:[0,0], depth_bits:depth.to_bits(), point_words:[0;3], normal_words:[0;3] } }
    #[test]
    fn direct_priority_and_depth_preserve_words() {
        let mut o=output(0, 9.); let i=input();
        assert_eq!(evaluate_tail(&mut o,i),Ok(TailResult::DirectWrite));
        assert_eq!(o.actor_ids,[8,9]); assert_eq!(o.feature_indices,[u32::MAX;2]);
        assert_eq!(o.point_words,[1,2,3]); assert_eq!(o.normal_words,[4,5,6]);
        let old=o; assert_eq!(evaluate_tail(&mut o,i),Ok(TailResult::DirectSkipped)); assert_eq!(o,old);
        let mut deeper=i; deeper.depth_bits=2f32.to_bits();
        assert_eq!(evaluate_tail(&mut o,deeper),Ok(TailResult::DirectWrite));
        o.selector=2; assert_eq!(evaluate_tail(&mut o,deeper),Ok(TailResult::DirectSkipped));
    }
    #[test]
    fn relation_five_direct_and_pending_submission_are_distinct() {
        let mut o=output(10, 0.); let mut i=input(); i.relation=5;
        let old=o; assert_eq!(evaluate_tail(&mut o,i),Err(TailError::MissingRelationFiveOutcome)); assert_eq!(o,old);
        i.relation_five=Some(RelationFiveOutcome::Submit);
        let TailResult::PendingSubmission(c)=evaluate_tail(&mut o,i).unwrap() else { panic!() };
        assert_eq!(c.selector,0x16); assert_eq!(o,old);
        i.word_20=1; let TailResult::PendingSubmission(c)=evaluate_tail(&mut o,i).unwrap() else { panic!() };
        assert_eq!(c.selector,0x11); assert_eq!(o,old);
        i.relation_five=Some(RelationFiveOutcome::DirectPriority22);
        assert_eq!(evaluate_tail(&mut o,i),Ok(TailResult::DirectWrite)); assert_eq!(o.selector,0x16);
    }
    #[test]
    fn signed_zero_subnormals_and_rejected_nonfinite() {
        let mut i=input(); let mut o=output(1,-0.); i.depth_bits=0;
        assert_eq!(evaluate_tail(&mut o,i),Ok(TailResult::DirectSkipped));
        i.depth_bits=1; assert_eq!(evaluate_tail(&mut o,i),Ok(TailResult::DirectWrite));
        let old=o; i.depth_bits=f32::NAN.to_bits();
        assert_eq!(evaluate_tail(&mut o,i),Err(TailError::NonFiniteDepth)); assert_eq!(o,old);
    }
}
