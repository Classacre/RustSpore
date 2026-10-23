//! Explicit scoped scalar-SSE fixture policy, retaining the caller's full state.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Nearest-even, masked exceptions, gradual underflow and no DAZ.
    Mxcsr1f80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseError {
    UnsupportedArchitecture,
}
impl fmt::Display for SseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "explicit native SSE policy: {self:?}")
    }
}
impl std::error::Error for SseError {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct MxcsrGuard(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MxcsrGuard {
    unsafe fn install(policy: Policy) -> Self {
        let requested = match policy {
            Policy::Mxcsr1f80 => 0x1f80u32,
        };
        let mut previous = 0u32;
        // SAFETY: bounded locals, checked SSE2, same-thread scoped restoration.
        unsafe {
            std::arch::asm!("stmxcsr [{previous}]", "ldmxcsr [{requested}]",
                previous = in(reg) &mut previous, requested = in(reg) &requested,
                options(nostack, preserves_flags));
        }
        Self(previous)
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for MxcsrGuard {
    fn drop(&mut self) {
        // SAFETY: exact state saved on the same thread at scope entry.
        unsafe {
            std::arch::asm!("ldmxcsr [{saved}]", saved = in(reg) &self.0,
                options(nostack, preserves_flags));
        }
    }
}

/// Run under an explicit policy and restore the full incoming MXCSR, including
/// sticky flags, on success, checked error or unwinding.
pub(crate) fn scoped<T, E: From<SseError>>(
    policy: Policy,
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if !std::is_x86_feature_detected!("sse2") {
            return Err(SseError::UnsupportedArchitecture.into());
        }
        // SAFETY: checked SSE2 and a lexical guard on the calling thread.
        let _guard = unsafe { MxcsrGuard::install(policy) };
        operation()
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (policy, operation);
        Err(SseError::UnsupportedArchitecture.into())
    }
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn current() -> u32 {
        let mut value = 0u32;
        // SAFETY: bounded local and this test executes on an SSE2 target.
        unsafe {
            std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value,
                options(nostack, preserves_flags));
        }
        value
    }
    #[test]
    fn explicit_policy_restores_hostile_control_and_sticky_flags_on_every_exit() {
        let _restore = MxcsrGuard(current());
        let hostile = 0xdf81u32; // FTZ, upward rounding, masks, invalid sticky.
                                 // SAFETY: only defined control/status bits; no DAZ requirement.
        unsafe {
            std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &hostile,
                options(nostack, preserves_flags));
        }
        let success = scoped(Policy::Mxcsr1f80, || Ok::<_, SseError>(current()));
        assert_eq!(success, Ok(0x1f80));
        assert_eq!(current(), hostile);
        let failure = scoped(Policy::Mxcsr1f80, || {
            assert_eq!(current(), 0x1f80);
            Err::<(), _>(SseError::UnsupportedArchitecture)
        });
        assert_eq!(failure, Err(SseError::UnsupportedArchitecture));
        assert_eq!(current(), hostile);
        let unwind = std::panic::catch_unwind(|| {
            let _ = scoped::<(), SseError>(Policy::Mxcsr1f80, || panic!("guard test"));
        });
        assert!(unwind.is_err());
        assert_eq!(current(), hostile);
    }
}
