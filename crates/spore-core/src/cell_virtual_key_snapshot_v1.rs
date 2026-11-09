//! Bounded, caller-admitted indexed-key snapshot and vector projection.
//!
//! The E81920 down path calls E82930 first and only invokes 6974E0 when AL
//! is zero; E50E70 invokes 697510 on release. Their indexed bit stores accept
//! codes <= FFh. Both setters call 697450 after the store: it derives the
//! separate +20 word from five indexed bits, with no calls or bit-array writes
//! on the inspected nonaliasing path. This module models the recomputed word
//! under a stable, nonaliasing object admission. The separate E81938..E81951 H/modifier toggle is
//! also excluded. The caller admits delivery, receiver identity, ordering and
//! the gate result. It is not
//! live Windows routing, a physical-key mapper, or a gameplay controller.
//!
//! Focus loss is a caller-selected policy, not a recovered source behavior.
//! Vector projection admits a frozen state and four separately supplied layout
//! callback AL values; no callback, actor, or native side effect is executed.
use crate::{
    cell_indexed_input_vector_v1::{self, ResultVector},
    native_sse::Policy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    words: [u32; 8],
    summary: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownGate {
    Passed,
    Consumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Down { code: u32, gate: DownGate },
    Up { code: u32 },
}

/// Reports whether the modeled post-store summary update occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub summary_recomputed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusLoss {
    Keep,
    ClearAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    CodeOutOfRange,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new([0; 8], 0)
    }
}
impl Snapshot {
    pub const fn new(words: [u32; 8], summary: u32) -> Self {
        Self { words, summary }
    }
    pub const fn words(&self) -> [u32; 8] {
        self.words
    }
    pub const fn summary(&self) -> u32 {
        self.summary
    }

    /// Apply one explicitly admitted event. Values outside the original 0..FF
    /// bound return an error and leave all words unchanged, including when the
    /// down gate is consumed. Repeated downs and ups are idempotent bit stores,
    /// but each admitted setter still requests a separate +20 summary update.
    pub fn apply(&mut self, event: Event) -> Result<Outcome, Error> {
        let code = match event {
            Event::Down { code, .. } | Event::Up { code } => code,
        };
        if code > 0xff {
            return Err(Error::CodeOutOfRange);
        }
        let mask = 1u32 << (code & 31);
        let word = &mut self.words[(code >> 5) as usize];
        match event {
            Event::Down {
                gate: DownGate::Passed,
                ..
            } => *word |= mask,
            Event::Down {
                gate: DownGate::Consumed,
                ..
            } => {}
            Event::Up { .. } => *word &= !mask,
        }
        let updated = !matches!(
            event,
            Event::Down {
                gate: DownGate::Consumed,
                ..
            }
        );
        if updated {
            self.summary = [0x10u32, 0x11, 0x12, 0xe1, 0x5b]
                .into_iter()
                .enumerate()
                .fold(0, |out, (index, code)| {
                    out | (((self.words[(code >> 5) as usize] >> (code & 31)) & 1) << index)
                });
        }
        Ok(Outcome {
            summary_recomputed: updated,
        })
    }

    /// A policy chosen by the embedding application, not native focus handling.
    /// ClearAll clears both the bit words and the separately supplied summary.
    pub fn focus_lost(&mut self, policy: FocusLoss) {
        if policy == FocusLoss::ClearAll {
            self.words = [0; 8];
            self.summary = 0;
        }
    }

    /// Evaluate the existing frozen finite vector model without mutating state.
    pub fn vector(
        &self,
        layout_al: [u8; 4],
        baseline: [f32; 3],
        policy: Policy,
    ) -> Result<ResultVector, cell_indexed_input_vector_v1::Error> {
        cell_indexed_input_vector_v1::assemble(self.words, layout_al, baseline, policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn down(code: u32, gate: DownGate) -> Event {
        Event::Down { code, gate }
    }
    #[test]
    fn down_repeat_release_and_gate_preserve_other_bits() {
        let mut s = Snapshot::default();
        assert!(
            s.apply(down(0x57, DownGate::Passed))
                .unwrap()
                .summary_recomputed
        );
        assert!(
            s.apply(down(0x57, DownGate::Passed))
                .unwrap()
                .summary_recomputed
        );
        assert!(
            !s.apply(down(0x53, DownGate::Consumed))
                .unwrap()
                .summary_recomputed
        );
        assert_eq!(s.words()[2], 1 << (0x57 - 64));
        s.apply(down(0x53, DownGate::Passed)).unwrap();
        assert!(
            s.apply(Event::Up { code: 0x57 })
                .unwrap()
                .summary_recomputed
        );
        assert!(
            s.apply(Event::Up { code: 0x57 })
                .unwrap()
                .summary_recomputed
        );
        assert_eq!(s.words()[2], 1 << (0x53 - 64));
    }
    #[test]
    fn summary_is_rebuilt_after_every_admitted_store_and_preserved_by_gate() {
        let mut s = Snapshot::new([0; 8], 0xdeadbeef);
        assert!(
            !s.apply(down(0x10, DownGate::Consumed))
                .unwrap()
                .summary_recomputed
        );
        assert_eq!(s.summary(), 0xdeadbeef);
        for (code, expected) in [(0x10, 1), (0x11, 3), (0x12, 7), (0xe1, 15), (0x5b, 31)] {
            s.apply(down(code, DownGate::Passed)).unwrap();
            assert_eq!(s.summary(), expected);
        }
        s.apply(Event::Up { code: 0x12 }).unwrap();
        assert_eq!(s.summary(), 27);
        s.apply(down(0x57, DownGate::Passed)).unwrap();
        assert_eq!(s.summary(), 27);
    }
    #[test]
    fn boundary_and_invalid_codes_are_transactional() {
        let mut s = Snapshot::default();
        s.apply(down(255, DownGate::Passed)).unwrap();
        assert_eq!(s.words()[7], 0x80000000);
        let before = s;
        for event in [
            down(256, DownGate::Passed),
            down(u32::MAX, DownGate::Consumed),
            Event::Up { code: 256 },
        ] {
            assert_eq!(s.apply(event), Err(Error::CodeOutOfRange));
            assert_eq!(s, before);
        }
        s.apply(Event::Up { code: 255 }).unwrap();
        assert_eq!(s, Snapshot::default());
    }
    #[test]
    fn focus_loss_policy_is_explicit_and_preserves_or_clears_all_words() {
        let mut s = Snapshot::new([u32::MAX; 8], 0xdeadbeef);
        s.focus_lost(FocusLoss::Keep);
        assert_eq!(s.words(), [u32::MAX; 8]);
        assert_eq!(s.summary(), 0xdeadbeef);
        s.focus_lost(FocusLoss::ClearAll);
        assert_eq!(s.words(), [0; 8]);
        assert_eq!(s.summary(), 0);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn frozen_layout_results_and_opposed_keys_reach_existing_projection() {
        let mut s = Snapshot::default();
        for c in [0x5a, 0x51, 0x28, 0x27] {
            s.apply(down(c, DownGate::Passed)).unwrap();
        }
        let normal = s.vector([0; 4], [0.; 3], Policy::Mxcsr1f80).unwrap();
        assert_eq!(normal.vector, [1., -1., 0.]);
        let french = s.vector([1, 0, 1, 0], [0.; 3], Policy::Mxcsr1f80).unwrap();
        assert_eq!(french.vector, [0.; 3]);
        assert_eq!(french.side_effect_requests, [true; 4]);
        s.apply(Event::Up { code: 0x28 }).unwrap();
        assert_eq!(
            s.vector([1, 0, 1, 0], [0.; 3], Policy::Mxcsr1f80)
                .unwrap()
                .vector,
            [0., 1., 0.]
        );
    }
}
