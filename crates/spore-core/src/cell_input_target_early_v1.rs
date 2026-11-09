//! Checked finite early-return portion of native E5AF20.
//!
//! Pinned SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a,
//! E5AF20..E5AFF5. A is the caller's current state vector and B the supplied
//! candidate. The longer angle path is not executed or approximated. This is
//! a controlled fixture, not source actor, input, runtime or gameplay evidence.
//! FP sticky flags, native stack writes and partial writes are not outputs.
use crate::native_sse::{self, Policy, SseError};

/// Admit masked native x87 exceptions, sufficient stack, valid nonaliasing
/// pointers and finite binary32 copy semantics. Native x87 status is excluded.
#[derive(Debug, Clone, Copy)]
pub struct X87CopyAdmission;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarlyBranch {
    SmallCurrent,
    SmallDifference,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Early {
        branch: EarlyBranch,
        candidate: [f32; 3],
    },
    /// The native routine would enter its separate angle-related path.
    NotEarly,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    NonFiniteIntermediate,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bounded E5AF20 early return: {self:?}")
    }
}
impl std::error::Error for Error {}
const THRESHOLD: f32 = f32::from_bits(0x37800000);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn sub(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("mulss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}

/// Compute the two finite early branches under explicit SSE policy. All A/B
/// words are admitted finite even if an early branch would not consume them;
/// this is a stricter checked domain. Successful copies retain B's exact words,
/// subject to the native x87-copy admission. Errors return no partial output.
pub fn early(
    current: [f32; 3],
    candidate: [f32; 3],
    policy: Policy,
    _x87: X87CopyAdmission,
) -> Result<Outcome, Error> {
    if !current.into_iter().chain(candidate).all(f32::is_finite) {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // E5AF35..56: x*x + y*y, then z*z added to that sum.
            let yy = mul(current[1], current[1]);
            let xx = mul(current[0], current[0]);
            let xy = add(xx, yy);
            let zz = mul(current[2], current[2]);
            let magnitude = add(xy, zz);
            if ![yy, xx, xy, zz, magnitude].into_iter().all(f32::is_finite) {
                return Err(Error::NonFiniteIntermediate);
            }
            if magnitude < THRESHOLD {
                return Ok(Outcome::Early {
                    branch: EarlyBranch::SmallCurrent,
                    candidate,
                });
            }
            // E5AF8F..CE: dx and dy, dy squared, dz, then dy²+dx²+dz².
            let dx = sub(current[0], candidate[0]);
            let dy = sub(current[1], candidate[1]);
            let dy2 = mul(dy, dy);
            let dz = sub(current[2], candidate[2]);
            let dx2 = mul(dx, dx);
            let dyx = add(dy2, dx2);
            let dz2 = mul(dz, dz);
            let difference = add(dyx, dz2);
            if ![dx, dy, dy2, dz, dx2, dyx, dz2, difference]
                .into_iter()
                .all(f32::is_finite)
            {
                return Err(Error::NonFiniteIntermediate);
            }
            if difference < THRESHOLD {
                return Ok(Outcome::Early {
                    branch: EarlyBranch::SmallDifference,
                    candidate,
                });
            }
            Ok(Outcome::NotEarly)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (current, candidate);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(a: [f32; 3], b: [f32; 3]) -> Result<Outcome, Error> {
        early(a, b, Policy::Mxcsr1f80, X87CopyAdmission)
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn current_threshold_is_strict_and_candidate_bits_are_preserved() {
        let below = f32::from_bits(0x3b800000 - 1); // immediately below 1/256
        let b = [-0.0, f32::from_bits(1), -f32::from_bits(1)];
        match run([below, 0., 0.], b).unwrap() {
            Outcome::Early {
                branch: EarlyBranch::SmallCurrent,
                candidate,
            } => assert_eq!(candidate.map(f32::to_bits), b.map(f32::to_bits)),
            _ => panic!("expected first early branch"),
        }
        assert_eq!(
            run([1.0 / 256.0, 0., 0.], [1., 0., 0.]),
            Ok(Outcome::NotEarly)
        );
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn difference_threshold_is_strict_and_long_path_is_not_implemented() {
        let a = [1., 0., 0.];
        let below = f32::from_bits((1.0f32 + 1.0 / 256.0).to_bits() - 1);
        assert_eq!(
            run(a, [below, 0., -0.0]),
            Ok(Outcome::Early {
                branch: EarlyBranch::SmallDifference,
                candidate: [below, 0., -0.0]
            })
        );
        if let Outcome::Early { candidate, .. } = run(a, [below, 0., -0.0]).unwrap() {
            assert_eq!(
                candidate.map(f32::to_bits),
                [below.to_bits(), 0, 0x8000_0000]
            );
        } else {
            panic!("expected small-difference branch");
        }
        assert_eq!(run(a, [1.0 + 1.0 / 256.0, 0., 0.]), Ok(Outcome::NotEarly));
        assert_eq!(run(a, [-1., 0., 0.]), Ok(Outcome::NotEarly));
    }
    #[test]
    fn rejects_nonfinite_inputs_without_mutation() {
        let a = [f32::NAN, 0., 0.];
        let b = [1., 0., 0.];
        assert_eq!(run(a, b), Err(Error::NonFiniteInput));
        assert!(a[0].is_nan());
        assert_eq!(b, [1., 0., 0.]);
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn rejects_overflow_in_either_reached_stage() {
        assert_eq!(
            run([f32::MAX, 0., 0.], [0.; 3]),
            Err(Error::NonFiniteIntermediate)
        );
        assert_eq!(
            run([1., 0., 0.], [-f32::MAX, 0., 0.]),
            Err(Error::NonFiniteIntermediate)
        );
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[test]
    fn non_x86_finite_input_is_unsupported() {
        assert_eq!(run([0.; 3], [0.; 3]), Err(Error::UnsupportedArchitecture));
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn hostile_mxcsr_is_restored_and_copied_subnormals_remain_exact() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        fn mxcsr() -> u32 {
            let mut x = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]", p=in(reg) &mut x, options(nostack,preserves_flags));
            }
            x
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &self.0, options(nostack,preserves_flags));
                }
            }
        }
        let _restore = Restore(mxcsr());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &hostile, options(nostack,preserves_flags));
        }
        let b = [f32::from_bits(1), -0.0, -f32::from_bits(1)];
        match run([0.; 3], b).unwrap() {
            Outcome::Early { candidate, .. } => {
                assert_eq!(candidate.map(f32::to_bits), b.map(f32::to_bits))
            }
            _ => panic!("early"),
        }
        assert_eq!(mxcsr(), hostile);
        assert_eq!(
            run([f32::MAX, 0., 0.], [0.; 3]),
            Err(Error::NonFiniteIntermediate)
        );
        assert_eq!(mxcsr(), hostile);
    }
}
