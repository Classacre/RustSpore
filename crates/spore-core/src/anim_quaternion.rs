//! Bounded native quaternion primitives from explicit arithmetic state.
//!
//! `99C810` normalizes, `99C8D0` adds within a common hemisphere, and `99C9A0`
//! interpolates without normalizing its inputs/output. The spherical path
//! explicitly borrows the installed finite CRT SSE2 acos constant window.
//! This does not select the original process's dispatch or FP environment.

use crate::native_crt_acos::{self, AcosError, AcosPolicy, ConstantWindow};
use crate::native_x87::{Context, Extended, X87Error};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsePolicy {
    /// Nearest-even binary32, masked exceptions, gradual underflow, no DAZ.
    Mxcsr1f80,
}

#[derive(Debug, Clone, Copy)]
pub struct ArithmeticContext {
    pub x87: Context,
    pub sse: SsePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuaternionError {
    NonFiniteInput {
        operand: &'static str,
        component: usize,
    },
    FractionOutsideUnitInterval,
    NonFiniteDot,
    NonFiniteOutput,
    X87(X87Error),
    Acos(AcosError),
    UnsupportedArchitecture,
}

impl fmt::Display for QuaternionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded native quaternion arithmetic: {self:?}")
    }
}

impl std::error::Error for QuaternionError {}
impl From<X87Error> for QuaternionError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}
impl From<AcosError> for QuaternionError {
    fn from(value: AcosError) -> Self {
        Self::Acos(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedQuaternion {
    pub quaternion: [f32; 4],
    /// The native optional length output, committed before SSE division.
    pub length: f32,
}

fn validate(value: [f32; 4], operand: &'static str) -> Result<(), QuaternionError> {
    if let Some(component) = value.iter().position(|v| !v.is_finite()) {
        return Err(QuaternionError::NonFiniteInput { operand, component });
    }
    Ok(())
}

fn finite_output(value: [f32; 4]) -> Result<[f32; 4], QuaternionError> {
    if value.iter().all(|v| v.is_finite()) {
        Ok(value)
    } else {
        Err(QuaternionError::NonFiniteOutput)
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct MxcsrGuard(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MxcsrGuard {
    unsafe fn install(_: SsePolicy) -> Self {
        let mut previous = 0u32;
        let requested = 0x1f80u32;
        // SAFETY: local bounded words; caller checks SSE2. Drop restores the
        // entire incoming MXCSR, including sticky flags, on either return path.
        unsafe {
            std::arch::asm!("stmxcsr [{old}]", "ldmxcsr [{new}]",
                old = in(reg) &mut previous, new = in(reg) &requested,
                options(nostack, preserves_flags));
        }
        Self(previous)
    }
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl Drop for MxcsrGuard {
    fn drop(&mut self) {
        // SAFETY: exact saved MXCSR from this same thread and scope.
        unsafe {
            std::arch::asm!("ldmxcsr [{saved}]", saved = in(reg) &self.0,
                options(nostack, preserves_flags));
        }
    }
}

pub(crate) fn scoped<T, E: From<QuaternionError>>(
    policy: SsePolicy,
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if !std::is_x86_feature_detected!("sse2") {
            return Err(QuaternionError::UnsupportedArchitecture.into());
        }
        // SAFETY: checked feature and lexical guard on the calling thread.
        let _guard = unsafe { MxcsrGuard::install(policy) };
        operation()
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (policy, operation);
        Err(QuaternionError::UnsupportedArchitecture.into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DotPolicy {
    FiniteOnly,
    SourceBranches,
}

fn dot(left: [f32; 4], right: [f32; 4], policy: DotPolicy) -> Result<f32, QuaternionError> {
    let value =
        ((right[0] * left[0] + right[1] * left[1]) + right[2] * left[2]) + right[3] * left[3];
    if policy == DotPolicy::SourceBranches || value.is_finite() {
        Ok(value)
    } else {
        Err(QuaternionError::NonFiniteDot)
    }
}

/// `99C810`: retained x87 squared length/sqrt, binary32 commit, SSE reciprocal.
///
/// A zero committed length copies every original component unchanged, including
/// signed zero. This is the only primitive here that normalizes. Nonfinite
/// inputs/intermediates/results are defensive errors at this bounded interface.
pub fn normalize(
    context: ArithmeticContext,
    value: [f32; 4],
) -> Result<NormalizedQuaternion, QuaternionError> {
    validate(value, "value")?;
    scoped(context.sse, || {
        let [x, y, z, w] = value.map(Extended::from_f32);
        let (x, y, z, w) = (x?, y?, z?, w?);
        let ctx = context.x87;
        let ww = ctx.mul(w, w)?;
        let xx = ctx.mul(x, x)?;
        let sum_xw = ctx.add(xx, ww)?;
        let yy = ctx.mul(y, y)?;
        let sum_yxw = ctx.add(yy, sum_xw)?;
        let zz = ctx.mul(z, z)?;
        let sum_zyxw = ctx.add(zz, sum_yxw)?;
        let length = ctx.store_f32(ctx.sqrt(sum_zyxw)?)?;
        let quaternion = if length == 0.0 {
            value
        } else {
            let reciprocal = 1.0 / length;
            [
                reciprocal * value[0],
                reciprocal * value[1],
                reciprocal * value[2],
                value[3] * reciprocal,
            ]
        };
        Ok(NormalizedQuaternion {
            quaternion: finite_output(quaternion)?,
            length,
        })
    })
}

/// `99C8D0`: dot<0 selects left-right; otherwise right+left. No normalization.
pub fn hemisphere_add(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
) -> Result<[f32; 4], QuaternionError> {
    hemisphere_add_with_policy(context, left, right, DotPolicy::FiniteOnly)
}

/// `99C8D0` for finite vector inputs, including overflow/unordered dot results.
///
/// COMISS zero,dot followed by JBE selects right+left for an unordered dot;
/// only an ordered negative dot selects left-right. Nonfinite vector inputs
/// and nonfinite output components remain errors at this bounded interface.
pub fn hemisphere_add_source(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
) -> Result<[f32; 4], QuaternionError> {
    hemisphere_add_with_policy(context, left, right, DotPolicy::SourceBranches)
}

fn hemisphere_add_with_policy(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    policy: DotPolicy,
) -> Result<[f32; 4], QuaternionError> {
    validate(left, "left")?;
    validate(right, "right")?;
    scoped(context.sse, || {
        let negative = dot(left, right, policy)? < 0.0;
        finite_output(std::array::from_fn(|i| {
            if negative {
                left[i] - right[i]
            } else {
                right[i] + left[i]
            }
        }))
    })
}

/// `99C9A0`: clamped shortest-hemisphere interpolation under explicit state.
///
/// The hemisphere sign multiplies the first input's weight, as in the original;
/// no generic library slerp, quaternion-quality filter or normalization is used.
/// Fraction is finite [0,1], with either sign of zero allowed. The supplied CRT
/// window's owner must verify its source/window digest. Spherical interpolation
/// uses the explicitly scoped SSE2/MXCSR1F80 finite CRT policy; legacy dispatch
/// and the retail choice of dispatch/state remain outside this operation.
pub fn slerp(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    fraction: f32,
    constants: ConstantWindow<'_>,
) -> Result<[f32; 4], QuaternionError> {
    slerp_bounded(
        context,
        left,
        right,
        fraction,
        constants,
        false,
        DotPolicy::FiniteOnly,
    )
}

/// The source COMISS/JA/JBE dot clamp for finite inputs and fraction [0,1].
///
/// An unordered dot and negative infinity clamp to -1; positive infinity
/// clamps to +1. These select the native near-dot branch. Other arithmetic,
/// finite-output bounds and explicit CRT/state policies match `slerp`.
pub fn slerp_source(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    fraction: f32,
    constants: ConstantWindow<'_>,
) -> Result<[f32; 4], QuaternionError> {
    slerp_bounded(
        context,
        left,
        right,
        fraction,
        constants,
        false,
        DotPolicy::SourceBranches,
    )
}

/// The same source primitive for the additional [-1,2] fractions used by
/// 99F6A0's authored rotation curve. All sine arguments are bounded by pi<4
/// after its finite clamped-dot acos. The unit-interval entry stays separate.
pub fn slerp_extrapolated(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    fraction: f32,
    constants: ConstantWindow<'_>,
) -> Result<[f32; 4], QuaternionError> {
    slerp_bounded(
        context,
        left,
        right,
        fraction,
        constants,
        true,
        DotPolicy::FiniteOnly,
    )
}

/// The source finite-input dot branches with the authored curve's [-1,2]
/// fraction domain. Unordered/overflow dots are handled before any CRT call;
/// finite vector/output bounds and the four-radian sine bound remain explicit.
pub fn slerp_extrapolated_source(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    fraction: f32,
    constants: ConstantWindow<'_>,
) -> Result<[f32; 4], QuaternionError> {
    slerp_bounded(
        context,
        left,
        right,
        fraction,
        constants,
        true,
        DotPolicy::SourceBranches,
    )
}

fn slerp_bounded(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    fraction: f32,
    constants: ConstantWindow<'_>,
    extrapolate: bool,
    dot_policy: DotPolicy,
) -> Result<[f32; 4], QuaternionError> {
    validate(left, "left")?;
    validate(right, "right")?;
    // Integer admission avoids dependence on an incoming caller's DAZ mode.
    let fraction_bits = fraction.to_bits();
    let fraction_magnitude = fraction_bits & 0x7fff_ffff;
    let outside = if extrapolate {
        fraction_magnitude
            > if fraction_bits >> 31 == 0 {
                0x4000_0000
            } else {
                0x3f80_0000
            }
    } else {
        fraction_magnitude > 0x3f80_0000 || fraction_bits >> 31 != 0 && fraction_magnitude != 0
    };
    if outside {
        return Err(QuaternionError::FractionOutsideUnitInterval);
    }
    scoped(context.sse, || {
        let raw_dot = dot(left, right, dot_policy)?;
        // Source99CA1B/99CA41 first admits only ordered raw>-1. Unordered
        // falls through to MOVAPS -1 at99CA47, rather than propagating NaN.
        let clamped = if raw_dot > -1.0 {
            if raw_dot <= 1.0 {
                raw_dot
            } else {
                1.0
            }
        } else {
            -1.0
        };
        let (absolute_dot, sign) = if clamped < 0.0 {
            (-clamped, -1.0)
        } else {
            (clamped, 1.0)
        };
        let (first, second) = if 1.0 - absolute_dot <= f32::from_bits(0x38d1_b717) {
            (1.0 - fraction, fraction)
        } else {
            let theta = native_crt_acos::finite_nonnegative(
                f64::from(absolute_dot),
                constants,
                AcosPolicy::Sse2Mxcsr1f80,
            )?;
            let theta = Extended::from_f64(theta)?;
            let ctx = context.x87;
            let sine_of = |argument| {
                if extrapolate {
                    ctx.sin_magnitude_at_most_four(argument)
                } else {
                    ctx.sin_magnitude_at_most_two(argument)
                }
            };
            let sine = sine_of(theta)?;
            let reciprocal = ctx.div(Extended::from_i32(1), sine)?;
            let amount = Extended::from_f32(fraction)?;
            let complement = ctx.sub(Extended::from_i32(1), amount)?;
            let first_angle = ctx.mul(complement, theta)?;
            let first_sine = sine_of(first_angle)?;
            let first = ctx.store_f32(ctx.mul(first_sine, reciprocal)?)?;
            let second_angle = ctx.mul(theta, amount)?;
            let second_sine = sine_of(second_angle)?;
            let second = ctx.store_f32(ctx.mul(reciprocal, second_sine)?)?;
            (first, second)
        };
        let first = first * sign;
        finite_output(std::array::from_fn(|i| left[i] * first + right[i] * second))
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn context() -> ArithmeticContext {
        ArithmeticContext {
            x87: Context {
                precision: Precision::Bits64,
                rounding: Rounding::NearestEven,
            },
            sse: SsePolicy::Mxcsr1f80,
        }
    }

    #[test]
    fn normalize_reports_length_and_copies_zero_bits() {
        let result = normalize(context(), [0., 0., 0., 2.]).unwrap();
        assert_eq!(result.length, 2.);
        assert_eq!(result.quaternion, [0., 0., 0., 1.]);
        let value = [-0., 0., -0., 0.];
        let result = normalize(context(), value).unwrap();
        assert_eq!(result.quaternion.map(f32::to_bits), value.map(f32::to_bits));
    }

    #[test]
    fn hemisphere_add_keeps_magnitude_and_flips_only_when_dot_is_negative() {
        assert_eq!(
            hemisphere_add(context(), [0., 0., 0., 2.], [0., 0., 0., 3.]).unwrap(),
            [0., 0., 0., 5.]
        );
        assert_eq!(
            hemisphere_add(context(), [0., 0., 0., 2.], [0., 0., 0., -3.]).unwrap(),
            [0., 0., 0., 5.]
        );
    }

    #[test]
    fn close_dot_slerp_uses_native_first_weight_sign_without_normalization() {
        // Linear branch never reads the borrowed constant bytes.
        let bytes = vec![0; native_crt_acos::CONSTANT_WINDOW_LENGTH];
        let constants = ConstantWindow::new(&bytes).unwrap();
        assert_eq!(
            slerp(
                context(),
                [0., 0., 0., 2.],
                [0., 0., 0., 3.],
                0.5,
                constants
            )
            .unwrap(),
            [0., 0., 0., 2.5]
        );
        assert_eq!(
            slerp(
                context(),
                [0., 0., 0., 1.],
                [0., 0., 0., -1.],
                0.25,
                constants
            )
            .unwrap(),
            [0., 0., 0., -1.]
        );
    }

    #[test]
    fn invalid_operands_and_fractions_are_explicit_errors() {
        assert!(matches!(
            normalize(context(), [f32::NAN; 4]),
            Err(QuaternionError::NonFiniteInput { .. })
        ));
        let bytes = vec![0; native_crt_acos::CONSTANT_WINDOW_LENGTH];
        let constants = ConstantWindow::new(&bytes).unwrap();
        for fraction in [f32::NAN, f32::INFINITY, -0.1, 1.0001] {
            assert_eq!(
                slerp(
                    context(),
                    [0., 0., 0., 1.],
                    [0., 0., 0., 1.],
                    fraction,
                    constants
                ),
                Err(QuaternionError::FractionOutsideUnitInterval)
            );
        }
    }

    #[test]
    fn extrapolation_has_a_separate_native_curve_domain() {
        let bytes = vec![0; native_crt_acos::CONSTANT_WINDOW_LENGTH];
        let constants = ConstantWindow::new(&bytes).unwrap();
        let left = [0., 0., 0., 2.];
        let right = [0., 0., 0., 3.];
        for (fraction, expected) in [(-1., 1.), (2., 4.)] {
            assert_eq!(
                slerp(context(), left, right, fraction, constants),
                Err(QuaternionError::FractionOutsideUnitInterval)
            );
            assert_eq!(
                slerp_extrapolated(context(), left, right, fraction, constants).unwrap()[3],
                expected
            );
        }
        for fraction in [-1.0001, 2.0001, f32::NAN, f32::INFINITY] {
            assert_eq!(
                slerp_extrapolated(context(), left, right, fraction, constants),
                Err(QuaternionError::FractionOutsideUnitInterval)
            );
        }
    }

    #[test]
    fn source_hemisphere_uses_original_overflow_and_unordered_predicates() {
        let h = f32::from_bits(0x6e00_0000);
        let left = [h; 4];
        for (right, expected) in [
            ([h; 4], [2. * h; 4]),
            ([-h; 4], [2. * h; 4]),
            ([h, -h, h, -h], [2. * h, 0., 2. * h, 0.]),
        ] {
            assert_eq!(
                hemisphere_add(context(), left, right),
                Err(QuaternionError::NonFiniteDot)
            );
            assert_eq!(
                hemisphere_add_source(context(), left, right)
                    .unwrap()
                    .map(f32::to_bits),
                expected.map(f32::to_bits)
            );
        }
        assert_eq!(
            hemisphere_add_source(context(), [f32::MAX; 4], [f32::MAX; 4]),
            Err(QuaternionError::NonFiniteOutput)
        );
    }

    #[test]
    fn source_slerp_clamps_unordered_to_negative_one_before_near_branch() {
        // The original clamp bypasses acos for all three dot outcomes, so
        // this test's borrowed bytes cannot supply a fallback angle/result.
        let bytes = vec![0; native_crt_acos::CONSTANT_WINDOW_LENGTH];
        let constants = ConstantWindow::new(&bytes).unwrap();
        let h = f32::from_bits(0x6e00_0000);
        let left = [h; 4];
        for (right, expected) in [
            ([h; 4], [h; 4]),
            ([-h; 4], [-h; 4]),
            ([h, -h, h, -h], [0., -h, 0., -h]),
        ] {
            assert_eq!(
                slerp(context(), left, right, 0.5, constants),
                Err(QuaternionError::NonFiniteDot)
            );
            assert_eq!(
                slerp_source(context(), left, right, 0.5, constants)
                    .unwrap()
                    .map(f32::to_bits),
                expected.map(f32::to_bits)
            );
        }
        assert_eq!(
            slerp_extrapolated_source(context(), left, [h, -h, h, -h], -1., constants).unwrap(),
            [-3. * h, -h, -3. * h, -h]
        );
    }

    #[test]
    fn scoped_sse_gradual_underflow_restores_caller_state_on_success_and_error() {
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                // SAFETY: the exact incoming MXCSR from this same thread.
                unsafe {
                    std::arch::asm!("ldmxcsr [{saved}]", saved = in(reg) &self.0,
                        options(nostack, preserves_flags));
                }
            }
        }
        fn read() -> u32 {
            let mut value = 0;
            // SAFETY: bounded local output word; no arithmetic-state change.
            unsafe {
                std::arch::asm!("stmxcsr [{out}]", out = in(reg) &mut value,
                    options(nostack, preserves_flags));
            }
            value
        }
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let _restore = Restore(read());
        // Masked exceptions, toward-zero, FTZ, existing sticky flags. No DAZ
        // bit is needed, so this test does not assume optional DAZ support.
        let caller = 0xFFA1u32;
        // SAFETY: only defined SSE2 MXCSR control/status bits are set.
        unsafe {
            std::arch::asm!("ldmxcsr [{requested}]", requested = in(reg) &caller,
                options(nostack, preserves_flags));
        }
        let result =
            hemisphere_add(context(), [f32::from_bits(1), 0., 0., 0.], [0., 0., 0., 0.]).unwrap();
        assert_eq!(result[0].to_bits(), 1);
        assert_eq!(read(), caller);
        assert_eq!(
            hemisphere_add(context(), [f32::MAX; 4], [f32::MAX; 4]),
            Err(QuaternionError::NonFiniteDot)
        );
        assert_eq!(read(), caller);
    }
}
