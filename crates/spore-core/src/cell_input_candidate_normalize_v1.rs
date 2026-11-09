//! Controlled finite candidate normalization from E5B3D5..E5B446.
//!
//! Pinned SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! The vector is explicitly supplied; global inputs, callback state, the source
//! input helper, actor identity, scheduling and FP status are not established.
//! This is not a complete target update or a live player step.
//!
//! On x86-64 the x87 operation uses an isolated FXSAVE64/FNINIT/FXRSTOR64
//! sequence with a caller-selected masked-exception context. The saved image
//! reconstructs abridged tags; older AMD64 hardware can omit diagnostic x87
//! FIP/FDP/FOP fields when ES is clear. Other architectures are unsupported.
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;

const THRESHOLD: f32 = f32::from_bits(0x3780_0000);
const EPSILON: f32 = f32::from_bits(0x322b_cc77);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Inactive {
        magnitude_squared: f32,
    },
    Active {
        magnitude_squared: f32,
        reciprocal: f32,
        candidate: [f32; 3],
    },
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
        write!(f, "bounded candidate normalization: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);

#[cfg(target_arch = "x86_64")]
fn reciprocal(magnitude: f32, context: Context) -> f32 {
    let mut env = Environment([0; 512]);
    let control = context.control_word();
    let epsilon = EPSILON;
    let mut result = 0.0f32;
    // SAFETY: live aligned FXSAVE image and bounded scalar operands. The
    // isolated stack has masked exceptions and is restored in this block.
    unsafe {
        std::arch::asm!(
            "fxsave64 [{env}]", "fninit", "fldcw [{control}]",
            "fld dword ptr [{magnitude}]", "fadd dword ptr [{epsilon}]",
            "fsqrt", "fld1", "fdivrp st(1), st(0)",
            "fstp dword ptr [{result}]", "fxrstor64 [{env}]",
            env = in(reg) &mut env, control = in(reg) &control,
            magnitude = in(reg) &magnitude, epsilon = in(reg) &epsilon,
            result = in(reg) &mut result,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    result
}

#[cfg(target_arch = "x86_64")]
fn mul(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("mulss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}
#[cfg(target_arch = "x86_64")]
fn add(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
}

/// Compute one finite frozen-vector fixture under explicit SSE/x87 controls.
/// The inactive branch does not call the x87 operation. Nonfinite input or a
/// reached nonfinite arithmetic intermediate returns an error with no output.
pub fn normalize(vector: [f32; 3], policy: Policy, context: Context) -> Result<Outcome, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (vector, policy, context);
        return Err(Error::UnsupportedArchitecture);
    }
    #[cfg(target_arch = "x86_64")]
    {
        if !vector.into_iter().all(f32::is_finite) {
            return Err(Error::NonFiniteInput);
        }
        native_sse::scoped(policy, || {
            let zz = mul(vector[2], vector[2]);
            let yy = mul(vector[1], vector[1]);
            let zy = add(zz, yy);
            let xx = mul(vector[0], vector[0]);
            let magnitude = add(zy, xx);
            if ![zz, yy, zy, xx, magnitude].into_iter().all(f32::is_finite) {
                return Err(Error::NonFiniteIntermediate);
            }
            if magnitude <= THRESHOLD {
                return Ok(Outcome::Inactive {
                    magnitude_squared: magnitude,
                });
            }
            let inv = reciprocal(magnitude, context);
            if !inv.is_finite() {
                return Err(Error::NonFiniteIntermediate);
            }
            // Native X is reciprocal*X; Y and Z are component*reciprocal.
            let x = mul(inv, vector[0]);
            let y = mul(vector[1], inv);
            let z = mul(vector[2], inv);
            if ![x, y, z].into_iter().all(f32::is_finite) {
                return Err(Error::NonFiniteIntermediate);
            }
            Ok(Outcome::Active {
                magnitude_squared: magnitude,
                reciprocal: inv,
                candidate: [x, y, z],
            })
        })
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn ctx() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }
    fn run(v: [f32; 3]) -> Result<Outcome, Error> {
        normalize(v, Policy::Mxcsr1f80, ctx())
    }

    #[test]
    fn strict_threshold_and_neighbors() {
        let root = 0x3b80_0000u32; // 1/256 squared equals the threshold.
        for bits in [root - 1, root] {
            assert!(matches!(
                run([f32::from_bits(bits), 0., 0.]),
                Ok(Outcome::Inactive { .. })
            ));
        }
        assert!(matches!(
            run([f32::from_bits(root + 1), 0., 0.]),
            Ok(Outcome::Active { .. })
        ));
    }

    #[test]
    fn active_signed_zeros_and_subnormal_components() {
        let tiny = f32::from_bits(1);
        match run([1., tiny, -tiny]).unwrap() {
            Outcome::Active {
                reciprocal,
                candidate,
                ..
            } => {
                assert_eq!(reciprocal.to_bits(), 1.0f32.to_bits());
                assert_eq!(
                    candidate.map(f32::to_bits),
                    [1.0f32.to_bits(), 1, 0x8000_0001]
                );
            }
            _ => panic!("expected active"),
        }
        match run([-0., 1., -0.]).unwrap() {
            Outcome::Active { candidate, .. } => assert_eq!(
                candidate.map(f32::to_bits),
                [0x8000_0000, 1.0f32.to_bits(), 0x8000_0000]
            ),
            _ => panic!("expected active"),
        }
    }

    #[test]
    fn invalid_and_overflow_are_transactional() {
        assert_eq!(run([f32::NAN, 0., 0.]), Err(Error::NonFiniteInput));
        assert_eq!(run([f32::MAX, 0., 0.]), Err(Error::NonFiniteIntermediate));
        match run([f32::from_bits(1), 0., 0.]).unwrap() {
            Outcome::Inactive { magnitude_squared } => assert_eq!(magnitude_squared.to_bits(), 0),
            _ => panic!("expected inactive"),
        }
    }

    #[test]
    fn hostile_mxcsr_restored_on_success_and_error() {
        fn read() -> u32 {
            let mut v = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]", p=in(reg) &mut v, options(nostack,preserves_flags));
            }
            v
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &self.0, options(nostack,preserves_flags));
                }
            }
        }
        let _restore = Restore(read());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p=in(reg) &hostile, options(nostack,preserves_flags));
        }
        let tiny = f32::from_bits(1);
        match run([1., tiny, -tiny]).unwrap() {
            Outcome::Active { candidate, .. } => assert_eq!(candidate[1].to_bits(), 1),
            _ => panic!("expected active"),
        }
        assert_eq!(read(), hostile);
        assert_eq!(run([f32::MAX, 0., 0.]), Err(Error::NonFiniteIntermediate));
        assert_eq!(read(), hostile);
    }
}

#[cfg(all(test, not(target_arch = "x86_64")))]
#[test]
fn unsupported_architecture_is_explicit() {
    use crate::native_x87::{Precision, Rounding};
    let context = Context {
        precision: Precision::Bits53,
        rounding: Rounding::NearestEven,
    };
    assert_eq!(
        normalize([0.; 3], Policy::Mxcsr1f80, context),
        Err(Error::UnsupportedArchitecture)
    );
}
