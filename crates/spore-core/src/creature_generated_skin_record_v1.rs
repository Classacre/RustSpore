//! Controlled final-record slice of the 52B700 generated-skin producer.
//!
//! The caller supplies four already selected native eight-byte candidate records
//! in their source order. 52C310..52C3D7 adds their weights, copies each
//! index's low byte, and stores four separately divided binary32 values. This
//! models that slice for a nonpositive smoothing-iteration admission only;
//! earlier candidate production and any positive-iteration 52B270 passes are
//! not modeled. No CreatureData-row ownership, palette identity, generated
//! surface, live runtime, or baked-model regeneration follows from this API.
//! Finite/error checks and transactional return are stricter than native code;
//! caller FP sticky status is restored instead of reproduced.
use crate::native_sse::{self, Policy, SseError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    /// Low byte is copied; its relation to a finalized palette is not proven.
    pub index_word: u32,
    pub weight: f32,
}

/// Explicitly admit that the upstream records are stable and correspond to
/// this position, and that the source smoothing iteration count is <= 0.
#[derive(Clone, Copy, Debug)]
pub struct DirectOutputAdmission;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    NonFiniteSum,
    ZeroSum,
    NonFiniteResult,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "generated-skin record: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {a}, {b}", a=inout(xmm_reg) a, b=in(xmm_reg) b, options(nostack,preserves_flags));
    }
    a
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn div(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("divss {a}, {b}", a=inout(xmm_reg) a, b=in(xmm_reg) b, options(nostack,preserves_flags));
    }
    a
}

/// Return the 20 native-layout bytes for one admitted position.
pub fn record(
    candidates: [Candidate; 4],
    policy: Policy,
    _admission: DirectOutputAdmission,
) -> Result<[u8; 20], Error> {
    if candidates.iter().any(|c| !c.weight.is_finite()) {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut sum = candidates[0].weight;
            for c in &candidates[1..] {
                sum = add(sum, c.weight);
                if !sum.is_finite() {
                    return Err(Error::NonFiniteSum);
                }
            }
            if sum == 0.0 {
                return Err(Error::ZeroSum);
            }
            let mut bytes = [0u8; 20];
            for (i, c) in candidates.iter().enumerate() {
                let normalized = div(c.weight, sum);
                if !normalized.is_finite() {
                    return Err(Error::NonFiniteResult);
                }
                bytes[i] = c.index_word as u8;
                bytes[4 + i * 4..8 + i * 4].copy_from_slice(&normalized.to_bits().to_le_bytes());
            }
            Ok(bytes)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = candidates;
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn r(w: [f32; 4]) -> Result<[u8; 20], Error> {
        record(
            std::array::from_fn(|i| Candidate {
                index_word: [0, 255, 256, 0x12345678][i],
                weight: w[i],
            }),
            Policy::Mxcsr1f80,
            DirectOutputAdmission,
        )
    }
    fn bits(b: &[u8; 20], i: usize) -> u32 {
        u32::from_le_bytes(b[4 + i * 4..8 + i * 4].try_into().unwrap())
    }
    #[test]
    fn low_byte_and_weight_order_are_preserved() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let b = r([1., 2., 0., 1.]).unwrap();
        assert_eq!(&b[..4], &[0, 255, 0, 0x78]);
        assert_eq!(
            (0..4).map(|i| bits(&b, i)).collect::<Vec<_>>(),
            vec![0x3e800000, 0x3f000000, 0, 0x3e800000]
        );
        let b = r([1e20, -1e20, 3., 1.]).unwrap();
        assert_eq!(bits(&b, 2), 0x3f400000);
        assert_eq!(
            r([1e20, 3., -1e20, 1.]).unwrap()[4..8],
            1e20f32.to_bits().to_le_bytes()
        );
    }
    #[test]
    fn rejection_and_signed_zero() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        assert_eq!(r([0., -0., 0., 0.]), Err(Error::ZeroSum));
        assert_eq!(r([f32::NAN, 1., 1., 1.]), Err(Error::NonFiniteInput));
        assert_eq!(
            r([f32::MAX, f32::MAX, -f32::MAX, -f32::MAX]),
            Err(Error::NonFiniteSum)
        );
        assert_eq!(
            r([1., -1., f32::from_bits(1), 0.]),
            Err(Error::NonFiniteResult)
        );
        assert_eq!(bits(&r([-0., 1., 0., 0.]).unwrap(), 0), 0x80000000);
    }
}

#[cfg(all(test, not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[test]
fn unsupported_architecture_is_explicit() {
    let item = Candidate {
        index_word: 0,
        weight: 1.0,
    };
    assert_eq!(
        record([item; 4], Policy::Mxcsr1f80, DirectOutputAdmission),
        Err(Error::UnsupportedArchitecture)
    );
}
