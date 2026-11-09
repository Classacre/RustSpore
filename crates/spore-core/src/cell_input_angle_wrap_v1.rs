//! Bounded finite E50AB0/E50B10 angle-wrap arithmetic.
//!
//! Pinned SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! The period is an explicit caller input, not a proven live global value.
//! This does not run the CRT, callbacks, source helper, actor or gameplay.
//! Iteration and nonprogress checks are model restrictions; native sticky FP
//! flags, traps and partial writes are outside the result.
use crate::native_sse::{self, Policy, SseError};

const PI: f32 = f32::from_bits(0x4049_0fdb);
const NEG_PI: f32 = f32::from_bits(0xc049_0fdb);

/// Admit valid nonaliasing storage, a sufficiently free native x87 stack,
/// masked exceptions and exact finite FLD/FABS/FSTP binary32 copy semantics.
/// This marker does not execute x87 instructions or establish runtime state.
#[derive(Debug, Clone, Copy)]
pub struct X87CopyAdmission;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Difference {
    pub delta: f32,
    pub magnitude: f32,
    pub sign: f32,
    pub iterations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wrapped {
    pub value: f32,
    pub iterations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    InvalidPeriod,
    NonFiniteIntermediate,
    NonProgress,
    IterationLimit,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bounded source angle wrap: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {left}, {right}", left=inout(xmm_reg) left, right=in(xmm_reg) right, options(nostack,preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn sub(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {left}, {right}", left=inout(xmm_reg) left, right=in(xmm_reg) right, options(nostack,preserves_flags));
    }
    left
}

fn validate(period: f32) -> Result<(), Error> {
    if !period.is_finite() {
        return Err(Error::NonFiniteInput);
    }
    let bits = period.to_bits();
    if bits & 0x8000_0000 != 0 || bits & 0x7fff_ffff == 0 {
        return Err(Error::InvalidPeriod);
    }
    Ok(())
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn inner(mut value: f32, period: f32, limit: usize) -> Result<Wrapped, Error> {
    let mut iterations = 0usize;
    // The native first loop uses period + value and includes equality at -pi.
    while NEG_PI >= value {
        if iterations == limit {
            return Err(Error::IterationLimit);
        }
        let next = add(period, value);
        if !next.is_finite() {
            return Err(Error::NonFiniteIntermediate);
        }
        if next.to_bits() == value.to_bits() {
            return Err(Error::NonProgress);
        }
        value = next;
        iterations += 1;
    }
    // The second loop uses value - period and only enters above +pi. Native
    // does not restart the first loop if an unusually large period overshoots.
    while value > PI {
        if iterations == limit {
            return Err(Error::IterationLimit);
        }
        let next = sub(value, period);
        if !next.is_finite() {
            return Err(Error::NonFiniteIntermediate);
        }
        if next.to_bits() == value.to_bits() {
            return Err(Error::NonProgress);
        }
        value = next;
        iterations += 1;
    }
    Ok(Wrapped { value, iterations })
}

/// Model E50AB0 for finite inputs and a strictly positive explicit period.
/// The bound applies across both loops; zero permits already-in-range inputs.
pub fn wrap(
    angle: f32,
    period: f32,
    limit: usize,
    policy: Policy,
    _copy: X87CopyAdmission,
) -> Result<Wrapped, Error> {
    if !angle.is_finite() {
        return Err(Error::NonFiniteInput);
    }
    validate(period)?;
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            inner(angle, period, limit)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (angle, period, limit);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

/// Model E50B10's `b - a`, the same two loops, FABS-to-binary32 magnitude,
/// and -1 only for ordered negative delta. Both signs of zero return +1.
pub fn difference(
    a: f32,
    b: f32,
    period: f32,
    limit: usize,
    policy: Policy,
    _copy: X87CopyAdmission,
) -> Result<Difference, Error> {
    if !a.is_finite() || !b.is_finite() {
        return Err(Error::NonFiniteInput);
    }
    validate(period)?;
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let delta = sub(b, a);
            if !delta.is_finite() {
                return Err(Error::NonFiniteIntermediate);
            }
            let wrapped = inner(delta, period, limit)?;
            let magnitude = f32::from_bits(wrapped.value.to_bits() & 0x7fff_ffff);
            let sign = if wrapped.value < 0.0 { -1.0 } else { 1.0 };
            Ok(Difference {
                delta: wrapped.value,
                magnitude,
                sign,
                iterations: wrapped.iterations,
            })
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            let _ = (a, b, period, limit);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    const PERIOD: f32 = f32::from_bits(0x40c9_0fdb);
    fn w(a: f32, p: f32, n: usize) -> Result<Wrapped, Error> {
        wrap(a, p, n, Policy::Mxcsr1f80, X87CopyAdmission)
    }
    fn d(a: f32, b: f32) -> Result<Difference, Error> {
        difference(a, b, PERIOD, 8, Policy::Mxcsr1f80, X87CopyAdmission)
    }
    #[test]
    fn asymmetric_pi_boundaries_and_neighbors() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        assert_eq!(w(PI, PERIOD, 0).unwrap().value.to_bits(), PI.to_bits());
        assert_eq!(w(NEG_PI, PERIOD, 1).unwrap().value.to_bits(), PI.to_bits());
        assert_eq!(
            w(f32::from_bits(PI.to_bits() + 1), PERIOD, 0),
            Err(Error::IterationLimit)
        );
        assert_eq!(
            w(f32::from_bits(NEG_PI.to_bits() - 1), PERIOD, 0)
                .unwrap()
                .iterations,
            0
        );
        let above = w(f32::from_bits(PI.to_bits() + 1), PERIOD, 1).unwrap();
        assert_eq!((above.value.to_bits(), above.iterations), (0xc049_0fda, 1));
        let below = w(f32::from_bits(NEG_PI.to_bits() + 1), PERIOD, 1).unwrap();
        assert_eq!((below.value.to_bits(), below.iterations), (0x4049_0fda, 1));
    }
    #[test]
    fn source_loop_order_preserves_large_period_overshoot() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let out = w(4.0, 10.0, 2).unwrap();
        assert_eq!((out.value, out.iterations), (-6.0, 1));
        let out = w(-4.0, 10.0, 2).unwrap();
        assert_eq!((out.value, out.iterations), (-4.0, 2));
    }
    #[test]
    fn difference_is_b_minus_a_and_signed_zero_is_positive_sign() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let x = d(0.25, 1.25).unwrap();
        assert_eq!((x.delta, x.magnitude, x.sign), (1., 1., 1.));
        let x = d(1.25, 0.25).unwrap();
        assert_eq!((x.delta, x.magnitude, x.sign), (-1., 1., -1.));
        let x = d(0.0, -0.0).unwrap();
        assert_eq!(x.delta.to_bits(), 0x8000_0000);
        assert_eq!(x.magnitude.to_bits(), 0);
        assert_eq!(x.sign, 1.);
        assert_eq!(w(-0.0, PERIOD, 0).unwrap().value.to_bits(), 0x8000_0000);
    }
    #[test]
    fn invalid_nonprogress_overflow_and_limit_are_explicit() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        assert_eq!(w(f32::NAN, PERIOD, 1), Err(Error::NonFiniteInput));
        assert_eq!(w(0., f32::INFINITY, 1), Err(Error::NonFiniteInput));
        for p in [0., -0., -1.] {
            assert_eq!(w(0., p, 1), Err(Error::InvalidPeriod));
        }
        assert_eq!(w(-f32::MAX, 1., 1), Err(Error::NonProgress));
        assert_eq!(w(10., PERIOD, 0), Err(Error::IterationLimit));
        assert_eq!(w(-10., PERIOD, 0), Err(Error::IterationLimit));
        assert_eq!(w(-f32::MAX, f32::from_bits(1), 1), Err(Error::NonProgress));
        assert_eq!(w(f32::MAX, f32::from_bits(1), 1), Err(Error::NonProgress));
        assert_eq!(d(-f32::MAX, f32::MAX), Err(Error::NonFiniteIntermediate));
    }
    #[test]
    fn hostile_mxcsr_restored_on_success_and_error() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        fn read() -> u32 {
            let mut v = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]",p=in(reg) &mut v,options(nostack,preserves_flags));
            }
            v
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &self.0,options(nostack,preserves_flags));
                }
            }
        }
        let _restore = Restore(read());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &hostile,options(nostack,preserves_flags));
        }
        let tiny = f32::from_bits(1);
        let result = d(0., tiny).unwrap();
        assert_eq!(result.delta.to_bits(), 1);
        assert_eq!(result.magnitude.to_bits(), 1);
        assert_eq!(read(), hostile);
        assert_eq!(w(-f32::MAX, 1., 1), Err(Error::NonProgress));
        assert_eq!(read(), hostile);
    }
}

#[cfg(all(test, not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[test]
fn unsupported_architecture_is_explicit() {
    assert_eq!(
        wrap(0., 1., 1, Policy::Mxcsr1f80, X87CopyAdmission),
        Err(Error::UnsupportedArchitecture)
    );
}
