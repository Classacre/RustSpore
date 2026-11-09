//! Controlled finite E5AE70 angle-to-vector fixture.
//!
//! Pinned SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.
//! A supplied baseline and angle do not establish live globals, actor state,
//! source helper execution, input routing or gameplay. Native FP status and
//! partial writes are excluded. Host x87 FSIN is hardware-dependent.
//!
//! x86-64 only. FXSAVE64/FXRSTOR64 reconstruct abridged x87 tags; on older
//! AMD64 CPUs diagnostic FIP/FDP/FOP fields may not be preserved when ES is
//! clear. The actual native caller's FP state and stack remain unobserved.
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;

const PI: f32 = f32::from_bits(0x4049_0fdb);
const HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    PositiveOuter,
    NegativeOuter,
    Middle,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    pub vector: [f32; 3],
    pub branch: Branch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    AngleOutsideBound,
    NonFiniteIntermediate,
    SinIncomplete,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bounded angle vector: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);

#[cfg(target_arch = "x86_64")]
fn greater(left: f32, right: f32) -> bool {
    let result: u8;
    // COMISS is kept inside the scoped MXCSR policy, with the same operand
    // order as the source's two ordered finite comparisons.
    unsafe {
        std::arch::asm!("comiss {left}, {right}", "seta {result}",
            left=in(xmm_reg) left, right=in(xmm_reg) right,
            result=lateout(reg_byte) result, options(nostack));
    }
    result != 0
}

#[cfg(target_arch = "x86_64")]
fn xy(angle: f32, branch: Branch, context: Context) -> Result<[f32; 2], Error> {
    let mut env = Environment([0; 512]);
    let control = context.control_word();
    let pi = PI;
    let selector: u32 = match branch {
        Branch::PositiveOuter => 0,
        Branch::NegativeOuter => 1,
        Branch::Middle => 2,
    };
    let (mut x, mut y) = (0.0f32, 0.0f32);
    let (mut sin_status, mut final_status) = (0u16, 0u16);
    // SAFETY: bounded aligned storage; an isolated, masked x87 stack is
    // restored in the same block. Integer labels select the exact source x87
    // sequence; FNSTSW reads C2 immediately after FSIN without popping it.
    // LLVM encodes the ST0 self-multiply as D8 C8; the source uses DC C8.
    // Both multiply ST0 by itself, but opcode identity is not modeled.
    unsafe {
        std::arch::asm!(
            "fxsave64 [{env}]", "fninit", "fldcw [{control}]",
            "cmp {selector:e}, 0", "je 2f", "cmp {selector:e}, 1", "je 3f",
            "fld dword ptr [{angle}]", "jmp 4f",
            "2:", "fld dword ptr [{pi}]", "fsub dword ptr [{angle}]", "jmp 4f",
            "3:", "fld dword ptr [{angle}]", "fsub dword ptr [{pi}]",
            "4:", "fsin", "fnstsw word ptr [{sin_status}]",
            "cmp {selector:e}, 1", "jne 5f", "fchs",
            "5:", "fst dword ptr [{x}]", "fmul st(0), st(0)", "fld1", "fsubrp st(1), st(0)", "fsqrt",
            "cmp {selector:e}, 2", "je 6f", "fchs",
            "6:", "fstp dword ptr [{y}]", "fnstsw word ptr [{final_status}]", "fxrstor64 [{env}]",
            env=in(reg) &mut env, control=in(reg) &control, angle=in(reg) &angle, pi=in(reg) &pi,
            selector=in(reg) selector, x=in(reg) &mut x, y=in(reg) &mut y,
            sin_status=in(reg) &mut sin_status, final_status=in(reg) &mut final_status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    if sin_status & 0x0400 != 0 {
        return Err(Error::SinIncomplete);
    }
    if final_status & 0x004d != 0 || !x.is_finite() || !y.is_finite() {
        return Err(Error::NonFiniteIntermediate);
    }
    Ok([x, y])
}

/// Model finite |angle| <= the image binary32 pi, with explicit FP policy.
/// A finite baseline is admitted even where the native routine overwrites it.
/// The positive outer branch retains baseline Z; the other branches write +0.
pub fn angle_vector(
    angle: f32,
    baseline: [f32; 3],
    policy: Policy,
    context: Context,
) -> Result<Output, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (angle, baseline, policy, context);
        return Err(Error::UnsupportedArchitecture);
    }
    #[cfg(target_arch = "x86_64")]
    {
        if !angle.is_finite() || !baseline.into_iter().all(f32::is_finite) {
            return Err(Error::NonFiniteInput);
        }
        if angle.to_bits() & 0x7fff_ffff > PI.to_bits() {
            return Err(Error::AngleOutsideBound);
        }
        native_sse::scoped(policy, || {
            let branch = if greater(angle, HALF_PI) {
                Branch::PositiveOuter
            } else if greater(-HALF_PI, angle) {
                Branch::NegativeOuter
            } else {
                Branch::Middle
            };
            let [x, y] = xy(angle, branch, context)?;
            let z = if branch == Branch::PositiveOuter {
                baseline[2]
            } else {
                0.0
            };
            Ok(Output {
                vector: [x, y, z],
                branch,
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
    fn run(a: f32) -> Result<Output, Error> {
        angle_vector(a, [2., 3., -7.], Policy::Mxcsr1f80, ctx())
    }
    #[test]
    fn boundaries_and_asymmetric_z() {
        for a in [HALF_PI, -HALF_PI] {
            let out = run(a).unwrap();
            assert_eq!(out.branch, Branch::Middle);
            assert_eq!(out.vector[2].to_bits(), 0);
        }
        let positive = run(f32::from_bits(HALF_PI.to_bits() + 1)).unwrap();
        assert_eq!(positive.branch, Branch::PositiveOuter);
        assert_eq!(positive.vector[2], -7.);
        assert!(positive.vector[1].is_sign_negative());
        let negative = run(f32::from_bits((-HALF_PI).to_bits() + 1)).unwrap();
        assert_eq!(negative.branch, Branch::NegativeOuter);
        assert_eq!(negative.vector[2].to_bits(), 0);
        assert!(negative.vector[1].is_sign_negative());
    }
    #[test]
    fn signed_zero_and_quadrants() {
        for a in [0.0, -0.0] {
            let o = run(a).unwrap();
            assert_eq!(o.vector[0].to_bits(), a.to_bits());
            assert_eq!(o.vector[1], 1.0);
        }
        for (a, x_negative, y_negative) in [
            (0.75, false, false),
            (2.25, false, true),
            (-0.75, true, false),
            (-2.25, true, true),
        ] {
            let o = run(a).unwrap();
            assert_eq!(o.vector[0].is_sign_negative(), x_negative);
            assert_eq!(o.vector[1].is_sign_negative(), y_negative);
        }
        assert!(run(PI).is_ok());
        assert!(run(-PI).is_ok());
    }
    #[test]
    fn invalid_inputs_are_rejected() {
        assert_eq!(run(f32::NAN), Err(Error::NonFiniteInput));
        assert_eq!(
            run(f32::from_bits(PI.to_bits() + 1)),
            Err(Error::AngleOutsideBound)
        );
        assert_eq!(
            angle_vector(0., [0., 0., f32::NAN], Policy::Mxcsr1f80, ctx()),
            Err(Error::NonFiniteInput)
        );
    }
    #[test]
    fn hostile_mxcsr_and_x87_control_are_restored() {
        fn mxcsr() -> u32 {
            let mut x = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]",p=in(reg) &mut x,options(nostack,preserves_flags));
            }
            x
        }
        fn cw() -> u16 {
            let mut x = 0u16;
            unsafe {
                std::arch::asm!("fnstcw [{p}]",p=in(reg) &mut x,options(nostack,preserves_flags));
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
        let _restore = Restore(mxcsr());
        let old_cw = cw();
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &hostile,options(nostack,preserves_flags));
        }
        let o = run(f32::from_bits(1)).unwrap();
        assert_eq!(o.vector[0].to_bits(), 1);
        assert_eq!(mxcsr(), hostile);
        assert_eq!(cw(), old_cw);
        assert_eq!(run(f32::NAN), Err(Error::NonFiniteInput));
        assert_eq!(mxcsr(), hostile);
        assert_eq!(cw(), old_cw);
    }
}

#[cfg(all(test, not(target_arch = "x86_64")))]
#[test]
fn unsupported_architecture_is_explicit() {
    use crate::native_x87::{Precision, Rounding};
    let c = Context {
        precision: Precision::Bits53,
        rounding: Rounding::NearestEven,
    };
    assert_eq!(
        angle_vector(0., [0.; 3], Policy::Mxcsr1f80, c),
        Err(Error::UnsupportedArchitecture)
    );
}
