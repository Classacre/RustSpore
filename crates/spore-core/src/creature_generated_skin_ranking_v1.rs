//! Controlled 52C1A5..52C3DF ranking and generated-skin record composition.
//!
//! The caller supplies a stable post-accumulation weight snapshot and admits
//! successful nonaliasing source-vector operations and nonpositive smoothing
//! iterations. No query, accumulation, topology, palette identity, surface,
//! live editor or baked-model regeneration is established. Fixed capacity and
//! finite/transactional errors are stricter than native; FP status is restored.
use crate::{
    creature_generated_skin_record_v1::{self as record, Candidate, DirectOutputAdmission},
    native_sse::{self, Policy, SseError},
};

/// Deliberate model bound, not a native maximum.
pub const MAX_WEIGHTS: usize = 4096;
/// Caller admits stable indexed accumulation storage, successful nonaliasing
/// source-vector insertions and nonpositive subsequent smoothing iterations.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotAdmission;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyWeights,
    NonFiniteWeight,
    Record(record::Error),
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "generated skin ranking: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Rank in descending order, keeping earlier indices first on equal weights.
/// Four `(index 0, +0)` placeholders are retained where no positive value is
/// inserted, exactly as in the finite source path. Indices are not palette IDs.
pub fn rank(
    weights: &[f32],
    policy: Policy,
    _admit: SnapshotAdmission,
) -> Result<[Candidate; 4], Error> {
    if weights.len() > MAX_WEIGHTS {
        return Err(Error::TooManyWeights);
    }
    if weights.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFiniteWeight);
    }
    native_sse::scoped(policy, || {
        let mut out = [Candidate {
            index_word: 0,
            weight: 0.0,
        }; 4];
        for (index, &weight) in weights.iter().enumerate() {
            if let Some(pos) = out.iter().position(|old| weight > old.weight) {
                for j in (pos + 1..4).rev() {
                    out[j] = out[j - 1];
                }
                out[pos] = Candidate {
                    index_word: index as u32,
                    weight,
                };
            }
        }
        Ok(out)
    })
}

/// Rank then form one record. Zero/nonfinite sums and outputs are rejected by
/// the record writer rather than reproducing native exceptional values.
pub fn rank_record(
    weights: &[f32],
    policy: Policy,
    admit: SnapshotAdmission,
) -> Result<[u8; 20], Error> {
    let candidates = rank(weights, policy, admit)?;
    record::record(candidates, policy, DirectOutputAdmission).map_err(Error::Record)
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn rank_test(w: &[f32]) -> Result<[Candidate; 4], Error> {
        rank(w, Policy::Mxcsr1f80, SnapshotAdmission)
    }
    fn run(w: &[f32]) -> Result<[u8; 20], Error> {
        rank_record(w, Policy::Mxcsr1f80, SnapshotAdmission)
    }
    #[test]
    fn descending_order_ties_and_truncation() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let r = rank_test(&[3., 5., 5., 1., 9., 5.]).unwrap();
        assert_eq!(r.map(|x| x.index_word), [4, 1, 2, 5]);
        assert_eq!(r.map(|x| x.weight), [9., 5., 5., 5.]);
        let mut many = vec![0.; 258];
        many[257] = 4.;
        many[2] = 2.;
        let r = rank_test(&many).unwrap();
        assert_eq!(r.map(|x| x.index_word), [257, 2, 0, 0]);
        let b = run(&many).unwrap();
        assert_eq!(&b[..4], &[1, 2, 0, 0]);
    }
    #[test]
    fn zero_negative_and_rounding_loss() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let r = rank_test(&[-0., -1., 0., 4.]).unwrap();
        assert_eq!(r.map(|x| x.index_word), [3, 0, 0, 0]);
        assert_eq!(r[1].weight.to_bits(), 0);
        assert_eq!(
            run(&[-1., -0., 0.]),
            Err(Error::Record(record::Error::ZeroSum))
        );
        let b = run(&[1e20, 3., -1e20, 2., 1.]).unwrap();
        assert_eq!(&b[..4], &[0, 1, 3, 4]);
        assert_eq!(&b[4..8], &1.0f32.to_bits().to_le_bytes());
    }
    #[test]
    fn bounds_nonfinite_and_overflow_errors() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        assert_eq!(
            rank_test(&vec![0.; MAX_WEIGHTS + 1]),
            Err(Error::TooManyWeights)
        );
        assert_eq!(rank_test(&[f32::NAN]), Err(Error::NonFiniteWeight));
        assert_eq!(
            run(&[f32::MAX, f32::MAX]),
            Err(Error::Record(record::Error::NonFiniteSum))
        );
        let mut boundary = vec![0.; MAX_WEIGHTS];
        boundary[MAX_WEIGHTS - 1] = 1.;
        assert_eq!(
            rank_test(&boundary).unwrap()[0].index_word,
            (MAX_WEIGHTS - 1) as u32
        );
    }
    #[test]
    fn hostile_mxcsr_subnormal_and_restoration() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        fn read() -> u32 {
            let mut x = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]",p=in(reg) &mut x,options(nostack,preserves_flags));
            }
            x
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &self.0,options(nostack,preserves_flags));
                }
            }
        }
        let _guard = Restore(read());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &hostile,options(nostack,preserves_flags));
        }
        let r = rank_test(&[f32::from_bits(1)]).unwrap();
        assert_eq!(r[0].weight.to_bits(), 1);
        assert_eq!(read(), hostile);
        assert_eq!(run(&[0.]), Err(Error::Record(record::Error::ZeroSum)));
        assert_eq!(read(), hostile);
    }
}
#[cfg(all(test, not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[test]
fn unsupported_architecture_is_explicit() {
    assert_eq!(
        rank(&[1.], Policy::Mxcsr1f80, SnapshotAdmission),
        Err(Error::UnsupportedArchitecture)
    );
}
