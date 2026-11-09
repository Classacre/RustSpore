//! Controlled finite E5B160 indexed-state vector assembly.
//!
//! Source PE SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a;
//! E5B185..E5B3D5. Inputs are a frozen caller snapshot, not live input, actor
//! identity or callback ordering. The four E844A0 calls themselves are not run.
//! This does not normalize, call E5AF20, update an actor or execute side effects.
//! SSE sticky flags and native partial writes are not output; finite rejection
//! and transactional return are checked-model bounds, not native branches.
use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResultVector {
    pub vector: [f32; 3],
    /// For each reached group, native code requests state +C4 = 0 and writes
    /// byte 1 to the independently resolved +5190 object's +E0. This model
    /// reports the requests; it does not resolve or mutate either object.
    pub side_effect_requests: [bool; 4],
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
        write!(f, "bounded indexed input: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn sub(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
fn bit(words: &[u32; 8], code: usize) -> bool {
    words[code >> 5] & (1u32 << (code & 31)) != 0
}

/// Assemble one admitted, frozen indexed-state snapshot. The four bytes are
/// the separately supplied AL returns of E844A0; only indices 0 and 2 select
/// codes, but callbacks and their effects are not executed. All relevant
/// globals, pointers and state must remain stable for a source comparison.
/// Policy declares masked SSE exceptions, rounding and subnormal behavior;
/// nonfinite baseline or intermediate results return an error without output.
pub fn assemble(
    words: [u32; 8],
    layout_al: [u8; 4],
    baseline: [f32; 3],
    policy: Policy,
) -> Result<ResultVector, Error> {
    if !baseline.into_iter().all(f32::is_finite) {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let groups = [
                bit(&words, if layout_al[0] == 0 { 0x57 } else { 0x5a })
                    || bit(&words, 0x26)
                    || bit(&words, 0x68),
                bit(&words, 0x53) || bit(&words, 0x28) || bit(&words, 0x62),
                bit(&words, if layout_al[2] == 0 { 0x41 } else { 0x51 })
                    || bit(&words, 0x25)
                    || bit(&words, 0x64),
                bit(&words, 0x44) || bit(&words, 0x27) || bit(&words, 0x66),
            ];
            let mut v = baseline;
            if groups[0] {
                v = [add(0.0, v[0]), add(v[1], 1.0), add(v[2], 0.0)];
            }
            if groups[1] {
                v = [sub(v[0], 0.0), sub(v[1], 1.0), sub(v[2], 0.0)];
            }
            if groups[2] {
                v = [sub(v[0], 1.0), sub(v[1], 0.0), sub(v[2], 0.0)];
            }
            if groups[3] {
                v = [add(1.0, v[0]), add(v[1], 0.0), add(v[2], 0.0)];
            }
            if !v.into_iter().all(f32::is_finite) {
                return Err(Error::NonFiniteIntermediate);
            }
            Ok(ResultVector {
                vector: v,
                side_effect_requests: groups,
            })
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (words, layout_al, baseline);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn keys(codes: &[usize]) -> [u32; 8] {
        let mut words = [0; 8];
        for &code in codes {
            words[code >> 5] |= 1 << (code & 31);
        }
        words
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn run(codes: &[usize], al: [u8; 4], base: [f32; 3]) -> ResultVector {
        assemble(keys(codes), al, base, Policy::Mxcsr1f80).unwrap()
    }
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn simultaneous_opposed_groups_preserve_order_and_report_each_effect() {
        let r = run(&[0x57, 0x53, 0x41, 0x44], [0; 4], [0.; 3]);
        assert_eq!(r.vector, [0.; 3]);
        assert_eq!(r.side_effect_requests, [true; 4]);
        let r = run(&[0x26, 0x68, 0x27], [0; 4], [2., 3., 4.]);
        assert_eq!(r.vector, [3., 4., 4.]);
        assert_eq!(r.side_effect_requests, [true, false, false, true]);
    }
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn only_the_first_and_third_low_byte_values_select_layout() {
        let a = run(&[0x5a, 0x51], [0, 9, 0, 4], [0.; 3]);
        assert_eq!(a.side_effect_requests, [false; 4]);
        let b = run(&[0x5a, 0x51], [2, 0, 255, 0], [0.; 3]);
        assert_eq!(b.vector, [-1., 1., 0.]);
        assert_eq!(b.side_effect_requests, [true, false, true, false]);
    }
    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn signed_zero_is_preserved_without_groups_and_committed_on_addition() {
        let untouched = run(&[], [0; 4], [-0.0; 3]);
        assert_eq!(untouched.vector.map(f32::to_bits), [0x80000000; 3]);
        let active = run(&[0x57], [0; 4], [-0.0; 3]);
        assert_eq!(active.vector.map(f32::to_bits), [0, 1.0f32.to_bits(), 0]);
    }
    #[test]
    fn rejects_nonfinite_inputs_without_mutating_caller() {
        let words = keys(&[0x57]);
        let baseline = [f32::NAN, 0., 0.];
        assert_eq!(
            assemble(words, [0; 4], baseline, Policy::Mxcsr1f80),
            Err(Error::NonFiniteInput)
        );
        assert!(baseline[0].is_nan());
        assert_eq!(words, keys(&[0x57]));
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[test]
    fn finite_inputs_reject_unsupported_architecture() {
        assert_eq!(
            assemble(keys(&[0x57]), [0; 4], [0.; 3], Policy::Mxcsr1f80),
            Err(Error::UnsupportedArchitecture)
        );
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn each_alternative_code_selects_only_its_group() {
        let cases: &[(usize, [u8; 4], usize, [f32; 3])] = &[
            (0x57, [0; 4], 0, [0., 1., 0.]),
            (0x5a, [1, 0, 0, 0], 0, [0., 1., 0.]),
            (0x26, [0; 4], 0, [0., 1., 0.]),
            (0x68, [0; 4], 0, [0., 1., 0.]),
            (0x53, [0; 4], 1, [0., -1., 0.]),
            (0x28, [0; 4], 1, [0., -1., 0.]),
            (0x62, [0; 4], 1, [0., -1., 0.]),
            (0x41, [0; 4], 2, [-1., 0., 0.]),
            (0x51, [0, 0, 1, 0], 2, [-1., 0., 0.]),
            (0x25, [0; 4], 2, [-1., 0., 0.]),
            (0x64, [0; 4], 2, [-1., 0., 0.]),
            (0x44, [0; 4], 3, [1., 0., 0.]),
            (0x27, [0; 4], 3, [1., 0., 0.]),
            (0x66, [0; 4], 3, [1., 0., 0.]),
        ];
        for &(code, al, group, expected) in cases {
            let result = run(&[code], al, [0.; 3]);
            let mut requests = [false; 4];
            requests[group] = true;
            assert_eq!(result.vector, expected, "code {code:02x}");
            assert_eq!(result.side_effect_requests, requests, "code {code:02x}");
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn hostile_mxcsr_is_restored_and_subnormal_is_not_flushed() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        fn current() -> u32 {
            let mut value = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value, options(nostack, preserves_flags));
            }
            value
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &self.0, options(nostack, preserves_flags));
                }
            }
        }
        let _restore = Restore(current());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &hostile, options(nostack, preserves_flags));
        }
        let result = assemble(
            keys(&[0x57]),
            [0; 4],
            [f32::from_bits(1), 0., 0.],
            Policy::Mxcsr1f80,
        )
        .unwrap();
        assert_eq!(result.vector[0].to_bits(), 1);
        assert_eq!(current(), hostile);
        assert_eq!(
            assemble(
                keys(&[]),
                [0; 4],
                [f32::INFINITY, 0., 0.],
                Policy::Mxcsr1f80
            ),
            Err(Error::NonFiniteInput)
        );
        assert_eq!(current(), hostile);
    }
}
