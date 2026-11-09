//! Admitted `E67543..E67550` comparison only, from the pinned x86 PE.
//!
//! The caller supplies the already-committed distance and globals +70 bits.
//! This does not establish the earlier bounds/+6C gates, record insertion,
//! actor identity, provider, scheduling, or an actual gameplay contact.
//! MXCSR status from COMISS is discarded; caller state is restored.
use crate::native_sse::{self, Policy, SseError};

/// One means ordered distance > threshold; unordered and equality yield zero.
/// The explicit masked-exception SSE policy controls denormals and COMISS.
pub fn impact_flag(
    distance_bits: u32,
    threshold_bits: u32,
    policy: Policy,
) -> Result<u8, SseError> {
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let distance = f32::from_bits(distance_bits);
            let threshold = f32::from_bits(threshold_bits);
            let result: u8;
            // SAFETY: SSE2 was checked by scoped; operands and byte output are
            // registers, and the asm intentionally declares flags modified.
            unsafe {
                std::arch::asm!("comiss {distance}, {threshold}", "seta {result}",
                    distance = in(xmm_reg) distance, threshold = in(xmm_reg) threshold,
                    result = lateout(reg_byte) result, options(nostack));
            }
            Ok(result)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (distance_bits, threshold_bits);
            Err(SseError::UnsupportedArchitecture)
        }
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    #[test]
    fn ordered_unordered_and_subnormal_inputs() {
        let run = |a, b| impact_flag(a, b, Policy::Mxcsr1f80).unwrap();
        assert_eq!(run(5f32.to_bits(), 5f32.to_bits()), 0);
        assert_eq!(run(5f32.to_bits() + 1, 5f32.to_bits()), 1);
        assert_eq!(run(5f32.to_bits() - 1, 5f32.to_bits()), 0);
        assert_eq!(run(0x8000_0000, 0), 0);
        assert_eq!(run(1, 0), 1);
        assert_eq!(run(0x7f80_0000, 5f32.to_bits()), 1);
        assert_eq!(run(0x7fc0_0001, 5f32.to_bits()), 0);
        assert_eq!(run(5f32.to_bits(), 0x7f80_0001), 0);
    }
    #[test]
    fn hostile_mxcsr_is_restored_and_subnormal_is_not_flushed() {
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &self.0, options(nostack, preserves_flags));
                }
            }
        }
        let mut old = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p=in(reg) &mut old, options(nostack, preserves_flags));
        }
        let _restore = Restore(old);
        let hostile = 0x9fc1u32; // DAZ, FTZ, masked exceptions and invalid sticky.
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &hostile, options(nostack, preserves_flags));
        }
        assert_eq!(impact_flag(1, 0, Policy::Mxcsr1f80).unwrap(), 1);
        let mut after = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p=in(reg) &mut after, options(nostack, preserves_flags));
        }
        assert_eq!(after, hostile);
    }
}
