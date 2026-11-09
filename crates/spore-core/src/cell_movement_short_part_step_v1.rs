//! Bounded `E7C0B0` no-flow short-part arithmetic, not a live actor tick.
//!
//! Installed base SporeApp.exe SHA-256
//! `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`:
//! `E7C0B0..E7C109` copies the old position; with actor +248 zero,
//! `E7C213..E7C239` requires definition +B4 nonzero and zeros +98/+1CC.
//! `E7C2F2..E7C3B3` accumulates, writes position and updates the two words.
//! This does not model later callbacks, scheduling, vector reset, other branches,
//! collision, source object validity, or original game runtime state.

use crate::native_sse::{self, Policy, SseError};

/// The actor fields touched by this bounded branch. The old previous-position
/// value is irrelevant: the branch replaces it with the old position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub position_4c: [f32; 3],
    pub previous_84: [f32; 3],
    pub accumulator_90: [f32; 3],
    pub vector_1c4: [f32; 3],
    pub flags_48: u16,
    pub counter_4a: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedFlowOrMultipart,
    DefinitionGateClosed,
    NonFiniteInput,
    NonFiniteIntermediate,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self { Self::UnsupportedArchitecture }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "bounded movement step: {self:?}") }
}
impl std::error::Error for Error {}

// These individual instructions prevent contraction and preserve the source's
// operand order. Volatile, memory-clobbering asm keeps operations ordered with
// the enclosing native_sse scope that sets and restores MXCSR.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut left: f32, right: f32) -> f32 {
    // SAFETY: scalar XMM registers only; native_sse checks SSE2 before calling.
    unsafe { std::arch::asm!("mulss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags)); }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut left: f32, right: f32) -> f32 {
    // SAFETY: scalar XMM registers only; native_sse checks SSE2 before calling.
    unsafe { std::arch::asm!("addss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags)); }
    left
}

/// Apply only actor +248 == 0 and definition +B4 != 0 under MXCSR 1F80.
/// The policy is an explicit fixture admission, not a measurement of live
/// game MXCSR. Finite input/intermediate checks are model bounds, not native
/// branches. An error returns no updated state; no partial native writes are
/// represented. Old +84 values are overwritten and need not be finite.
pub fn step(state: State, actor_248: u32, definition_b4: u32, scalar: f32, policy: Policy) -> Result<State, Error> {
    if actor_248 != 0 { return Err(Error::UnsupportedFlowOrMultipart); }
    if definition_b4 == 0 { return Err(Error::DefinitionGateClosed); }
    if !state.position_4c.into_iter().chain(state.accumulator_90).chain(state.vector_1c4)
        .chain([scalar]).all(f32::is_finite) { return Err(Error::NonFiniteInput); }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // E7C224..E7C239 zero Z before any multiplication.
            let mut result = state;
            result.previous_84 = state.position_4c;
            result.accumulator_90[2] = 0.0;
            result.vector_1c4[2] = 0.0;
            // E7C2F2..E7C350: source uses accumulator as ADDSS destination.
            let vx = mul(result.vector_1c4[0], scalar);
            let ax = add(result.accumulator_90[0], vx);
            let vy = mul(result.vector_1c4[1], scalar);
            let ay = add(result.accumulator_90[1], vy);
            let vz = mul(result.vector_1c4[2], scalar);
            let az = add(result.accumulator_90[2], vz);
            // E7C36B..E7C3A7: product is ADDSS destination for X and Y;
            // position Z is the product alone, without previous Z.
            let px_product = mul(ax, scalar);
            let px = add(px_product, state.position_4c[0]);
            let py_product = mul(ay, scalar);
            let py = add(py_product, state.position_4c[1]);
            let pz = mul(az, scalar);
            if ![vx, ax, vy, ay, vz, az, px_product, px, py_product, py, pz]
                .into_iter().all(f32::is_finite) { return Err(Error::NonFiniteIntermediate); }
            result.accumulator_90 = [ax, ay, az];
            result.position_4c = [px, py, pz];
            result.flags_48 |= 0x0004;
            result.counter_4a = result.counter_4a.wrapping_add(1);
            Ok(result)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        { Err(Error::UnsupportedArchitecture) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> State {
        State { position_4c: [10.0, 20.0, 30.0], previous_84: [99.0; 3],
            accumulator_90: [2.0, -4.0, 999.0], vector_1c4: [4.0, 6.0, 11.0],
            flags_48: 0x8001, counter_4a: 0xffff }
    }
    #[test]
    fn writes_short_part_fields_in_order_and_wraps_counter() {
        let out = step(sample(), 0, 7, 0.5, Policy::Mxcsr1f80).unwrap();
        assert_eq!(out.previous_84, [10.0, 20.0, 30.0]);
        assert_eq!(out.accumulator_90, [4.0, -1.0, 0.0]);
        assert_eq!(out.position_4c, [12.0, 19.5, 0.0]);
        assert_eq!(out.vector_1c4, [4.0, 6.0, 0.0]);
        assert_eq!(out.flags_48, 0x8005);
        assert_eq!(out.counter_4a, 0);
    }
    #[test]
    fn rejects_unmodeled_gates_nonfinite_and_overflow_without_state_mutation() {
        let state = sample();
        assert_eq!(step(state, 1, 1, 1.0, Policy::Mxcsr1f80), Err(Error::UnsupportedFlowOrMultipart));
        assert_eq!(step(state, 0, 0, 1.0, Policy::Mxcsr1f80), Err(Error::DefinitionGateClosed));
        assert_eq!(step(state, 0, 1, f32::NAN, Policy::Mxcsr1f80), Err(Error::NonFiniteInput));
        let mut huge = state; huge.vector_1c4[0] = f32::MAX;
        assert_eq!(step(huge, 0, 1, 2.0, Policy::Mxcsr1f80), Err(Error::NonFiniteIntermediate));
        assert_eq!(state, sample());
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn hostile_mxcsr_preserves_subnormals_and_restores_complete_caller_state() {
        if !std::is_x86_feature_detected!("sse2") { return; }
        fn current() -> u32 {
            let mut value = 0u32;
            // SAFETY: bounded local and checked SSE2.
            unsafe { std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value, options(nostack, preserves_flags)); }
            value
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                // SAFETY: restores the value captured on this thread.
                unsafe { std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &self.0, options(nostack, preserves_flags)); }
            }
        }
        let _restore = Restore(current());
        let hostile = 0xdf81u32; // FTZ, upward rounding, masked exceptions, sticky invalid.
        // SAFETY: these MXCSR bits are defined; SSE2 was checked.
        unsafe { std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &hostile, options(nostack, preserves_flags)); }
        let mut state = sample();
        state.position_4c = [0.0; 3];
        state.accumulator_90 = [f32::from_bits(1), 0.0, 0.0];
        state.vector_1c4 = [0.0; 3];
        let result = step(state, 0, 1, 1.0, Policy::Mxcsr1f80).unwrap();
        assert_eq!(result.accumulator_90[0].to_bits(), 1);
        assert_eq!(result.position_4c[0].to_bits(), 1);
        assert_eq!(current(), hostile);
        let mut overflow = state;
        overflow.vector_1c4[0] = f32::MAX;
        assert_eq!(step(overflow, 0, 1, 2.0, Policy::Mxcsr1f80), Err(Error::NonFiniteIntermediate));
        assert_eq!(current(), hostile);
    }

    #[test]
    fn nearest_even_subnormals_and_zero_sign_follow_admitted_sse_policy() {
        let mut state = sample();
        state.position_4c = [1.0, 0.0, 5.0];
        state.accumulator_90 = [f32::from_bits(0x33800000), f32::from_bits(1), 3.0];
        state.vector_1c4 = [0.0; 3];
        let out = step(state, 0, 1, 1.0, Policy::Mxcsr1f80).unwrap();
        assert_eq!(out.position_4c[0].to_bits(), 1.0f32.to_bits());
        assert_eq!(out.position_4c[1].to_bits(), 1);
        assert_eq!(out.position_4c[2].to_bits(), 0);
        let negative = step(state, 0, 1, -1.0, Policy::Mxcsr1f80).unwrap();
        assert_eq!(negative.position_4c[2].to_bits(), (-0.0f32).to_bits());
    }
}
