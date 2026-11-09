//! Controlled legacy `_CIasin` interior arithmetic at 7859A8A3..7859A8B1.
//!
//! The caller admits this exact branch and supplies an effective masked x87
//! context and finite binary32 input strictly inside (-1, 1). This models the
//! original x87 sequence followed by the Cell caller's binary32 FSTP; it does
//! not reproduce CRT dispatch, classification, control-word adjustment, return
//! policy, MSVCR90 selection, sticky flags, or live input. FPATAN precision is
//! hardware-dependent. Checked errors and state restoration differ from native.
//!
//! x86_64 only. FXSAVE64/FXRSTOR64 use abridged x87 tags; older AMD64 CPUs can
//! leave FIP/FDP/FOP diagnostic fields changed when ES was clear. A real native
//! call additionally requires adequate x87 stack space; this fixture isolates it.
use crate::native_x87::Context;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    OutsideInterior,
    NonFiniteOrInvalidResult,
    UnsupportedArchitecture,
    UnsupportedControlWord,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "controlled legacy CIasin interior: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Caller admits that native dispatch reached the finite legacy interior and
/// that its effective x87 control word has the supplied precision and rounding.
/// This does not establish native stack capacity, CRT identity or dispatch.
#[derive(Clone, Copy, Debug)]
pub struct InteriorAdmission;

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);

/// Return the caller-committed binary32 result for the controlled interior.
pub fn calculate(
    input: f32,
    context: Context,
    _admission: InteriorAdmission,
) -> Result<f32, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (input, context);
        return Err(Error::UnsupportedArchitecture);
    }
    #[cfg(target_arch = "x86_64")]
    {
        let magnitude = input.to_bits() & 0x7fff_ffff;
        if magnitude >= 0x7f80_0000 {
            return Err(Error::NonFiniteInput);
        }
        if magnitude >= 0x3f80_0000 {
            return Err(Error::OutsideInterior);
        }
        let mut env = Environment([0; 512]);
        let control = context.control_word();
        let mut output = 0.0f32;
        let mut status = 0u16;
        // The source slice leaves the original x in ST(1) before FPATAN.
        // Exact instruction encoding was independently checked against the DLL.
        // SAFETY: bounded, aligned storage and same-block save/restore; all x87
        // registers are declared clobbered and the isolated working stack is empty.
        unsafe {
            std::arch::asm!(
                "fxsave64 [{env}]", "fninit", "fldcw [{control}]",
                "fld dword ptr [{input}]",
                "fld1", "fadd st(0), st(1)", "fld1", "fsub st(0), st(2)",
                "fmulp st(1), st(0)", "fsqrt", "fpatan",
                "fstp dword ptr [{output}]", "fnstsw word ptr [{status}]",
                "fxrstor64 [{env}]",
                env=in(reg) &mut env, control=in(reg) &control,
                input=in(reg) &input, output=in(reg) &mut output, status=in(reg) &mut status,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        if status & 0x004d != 0 || !output.is_finite() {
            return Err(Error::NonFiniteOrInvalidResult);
        }
        Ok(output)
    }
}

/// Caller admits the legacy SSE2-disabled path and CW 027F, stable dispatch and
/// return globals, a valid source x87 stack, and no pending unmasked x87
/// exception, including for the endpoint constant/return path. This does not
/// establish the live CRT dispatch.
#[derive(Clone, Copy, Debug)]
pub struct FiniteClosedAdmission;

/// The separately admitted finite closed interval [-1, 1]. For an exact
/// endpoint, the DLL loads an extended pi/2 constant and conditionally applies
/// FCHS before the caller commits binary32. Under CW 027F the committed bits
/// are +/-3FC90FDB. This checked model does not reproduce native FP status.
/// Interior values use the existing isolated original-operation sequence.
pub fn calculate_finite_closed(
    input: f32,
    context: Context,
    _admission: FiniteClosedAdmission,
) -> Result<f32, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (input, context);
        return Err(Error::UnsupportedArchitecture);
    }
    #[cfg(target_arch = "x86_64")]
    {
        if context.control_word() != 0x027f {
            return Err(Error::UnsupportedControlWord);
        }
        let bits = input.to_bits();
        let magnitude = bits & 0x7fff_ffff;
        if magnitude >= 0x7f80_0000 {
            return Err(Error::NonFiniteInput);
        }
        if magnitude > 0x3f80_0000 {
            return Err(Error::OutsideInterior);
        }
        if magnitude == 0x3f80_0000 {
            return Ok(f32::from_bits(0x3fc9_0fdb | (bits & 0x8000_0000)));
        }
        calculate(input, context, InteriorAdmission)
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
    fn run(x: f32) -> Result<f32, Error> {
        calculate(x, ctx(), InteriorAdmission)
    }
    #[test]
    fn zeros_quarters_and_interior_edges() {
        assert_eq!(run(0.0).unwrap().to_bits(), 0);
        assert_eq!(run(-0.0).unwrap().to_bits(), 0x80000000);
        assert_eq!(run(0.5).unwrap().to_bits(), 0x3f060a92);
        assert_eq!(run(-0.5).unwrap().to_bits(), 0xbf060a92);
        let edge = f32::from_bits(0x3f7fffff);
        assert!((run(edge).unwrap() - edge.asin()).abs() < 0.000001);
        assert!((run(-edge).unwrap() + edge.asin()).abs() < 0.000001);
    }
    #[test]
    fn rejects_endpoints_and_nonfinite() {
        for x in [-1., 1., -2., 2.] {
            assert_eq!(run(x), Err(Error::OutsideInterior));
        }
        for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(run(x), Err(Error::NonFiniteInput));
        }
    }
    #[test]
    fn finite_closed_handles_exact_endpoints_with_bounded_control() {
        assert_eq!(
            calculate_finite_closed(1.0, ctx(), FiniteClosedAdmission)
                .unwrap()
                .to_bits(),
            0x3fc90fdb
        );
        assert_eq!(
            calculate_finite_closed(-1.0, ctx(), FiniteClosedAdmission)
                .unwrap()
                .to_bits(),
            0xbfc90fdb
        );
        assert_eq!(
            calculate_finite_closed(f32::from_bits(0x3f800001), ctx(), FiniteClosedAdmission),
            Err(Error::OutsideInterior)
        );
        assert_eq!(
            calculate_finite_closed(f32::NAN, ctx(), FiniteClosedAdmission),
            Err(Error::NonFiniteInput)
        );
        let wrong = Context {
            precision: Precision::Bits64,
            ..ctx()
        };
        assert_eq!(
            calculate_finite_closed(1.0, wrong, FiniteClosedAdmission),
            Err(Error::UnsupportedControlWord)
        );
    }
    #[test]
    fn preserves_caller_controls() {
        let mut before_mx = 0u32;
        let mut before_cw = 0u16;
        unsafe {
            std::arch::asm!("stmxcsr [{mx}]","fnstcw [{cw}]",mx=in(reg) &mut before_mx,cw=in(reg) &mut before_cw,options(nostack,preserves_flags));
        }
        let _ = run(f32::from_bits(1)).unwrap();
        let mut after_mx = 0u32;
        let mut after_cw = 0u16;
        unsafe {
            std::arch::asm!("stmxcsr [{mx}]","fnstcw [{cw}]",mx=in(reg) &mut after_mx,cw=in(reg) &mut after_cw,options(nostack,preserves_flags));
        }
        assert_eq!((after_mx, after_cw), (before_mx, before_cw));
    }
}
#[cfg(all(test, not(target_arch = "x86_64")))]
#[test]
fn unsupported_architecture_is_explicit() {
    assert_eq!(
        calculate(
            0.0,
            Context {
                precision: crate::native_x87::Precision::Bits53,
                rounding: crate::native_x87::Rounding::NearestEven
            },
            InteriorAdmission
        ),
        Err(Error::UnsupportedArchitecture)
    );
}
