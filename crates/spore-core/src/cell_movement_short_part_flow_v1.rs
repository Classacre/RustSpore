//! Controlled E7C0B0 short-part path with a nonzero handle and supplied sample.
//!
//! Source: SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a,
//! E7C213..E7C3B3. This does not sample a field, run callbacks, establish actor
//! identity or scheduling, or represent an original player tick. The sample is
//! caller-admitted as the completed native sampling result; the sampler's Z and
//! exceptions are outside this step. Native FP status and partial writes are not
//! returned. x87 transcendental results are hardware-dependent. Only x86-64 is supported;
//! x86 uses an x87 return ABI and is rejected to preserve caller stack state.
use crate::cell_movement_short_part_step_v1::State;
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    FlowHandleRequired,
    Multipart,
    DefinitionGateClosed,
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
        write!(f, "bounded short-part flow step: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
struct Environment([u8; 512]);

// The x87 operation order at E7C25A..E7C2A2, excluding interleaved SSE and
// stack-pointer instructions. FSTP commits the native binary32 factor.
// An explicit masked-exception CW and restored x87 state image bound this
// operation; exception flags, traps and the original argument-slot write are
// intentionally outside the returned model. FXSAVE64/FXRSTOR64 preserve the
// saved x87 register image under the stated hardware limits; FNINIT isolates
// the working stack. The 512-byte image uses abridged x87 tags, so restoration reconstructs
// full tags and cannot promise identical noncanonical tag encodings. Some older
// AMD64 processors can skip saving/restoring x87 FIP/FDP/FOP when ES is clear;
// these diagnostic fields may change. Original native behavior still requires
// its own sufficient stack capacity.
#[cfg(target_arch = "x86_64")]
fn factor(scalar: f32, context: Context) -> f32 {
    let mut environment = Environment([0; 512]);
    let control = context.control_word();
    let mut result = 0.0f32;
    unsafe {
        std::arch::asm!(
            "fxsave64 [{env}]", "fninit", "fldcw [{control}]", "fld dword ptr [{scalar}]",
            "fchs", "fldl2e", "fmulp st(1), st(0)", "fld st(0)",
            "frndint", "fxch st(1)", "fsub st(0), st(1)", "f2xm1",
            "fld1", "faddp st(1), st(0)", "fscale", "fstp st(1)",
            "fld1", "fsubrp st(1), st(0)",
            "fstp dword ptr [{result}]", "fxrstor64 [{env}]",
            env = in(reg) &mut environment, control = in(reg) &control,
            scalar = in(reg) &scalar, result = in(reg) &mut result,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    result
}
#[cfg(target_arch = "x86_64")]
fn sub(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {left}, {right}", left = inout(xmm_reg) left, right = in(xmm_reg) right, options(nostack, preserves_flags));
    }
    left
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

/// Return only a fully checked result. `part_count_250` is signed, as in the
/// branch instruction; negative counts are admitted fixtures, not actor proof.
/// The provided SSE policy masks exceptions and uses nearest/gradual arithmetic;
/// `context` selects x87 precision/rounding with masked exceptions. Caller
/// admits valid, nonaliasing native pointers and sufficient x87 stack space.
/// Old previous-position, accumulator Z, vector Z and sample Z are not read.
pub fn step(
    state: State,
    handle_248: u32,
    part_count_250: i32,
    definition_b4: u32,
    scalar: f32,
    sample: [f32; 3],
    policy: Policy,
    context: Context,
) -> Result<State, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (
            state,
            handle_248,
            part_count_250,
            definition_b4,
            scalar,
            sample,
            policy,
            context,
        );
        return Err(Error::UnsupportedArchitecture);
    }
    if handle_248 == 0 {
        return Err(Error::FlowHandleRequired);
    }
    if part_count_250 > 1 {
        return Err(Error::Multipart);
    }
    if definition_b4 == 0 {
        return Err(Error::DefinitionGateClosed);
    }
    if !state
        .position_4c
        .into_iter()
        .chain(state.accumulator_90[..2].iter().copied())
        .chain(state.vector_1c4[..2].iter().copied())
        .chain([scalar, sample[0], sample[1]])
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteInput);
    }
    native_sse::scoped(policy, || {
        #[cfg(target_arch = "x86_64")]
        {
            let mut result = state;
            result.previous_84 = state.position_4c;
            result.accumulator_90[2] = 0.0;
            result.vector_1c4[2] = 0.0;
            let dx = sub(sample[0], state.accumulator_90[0]);
            let dy = sub(sample[1], state.accumulator_90[1]);
            let k = factor(scalar, context);
            let yp = mul(dy, k);
            let xp = mul(dx, k);
            let x = add(xp, state.accumulator_90[0]);
            let zp = mul(-0.0, k);
            let y = add(state.accumulator_90[1], yp);
            let z = add(0.0, zp);
            let vx = mul(state.vector_1c4[0], scalar);
            let ax = add(x, vx);
            let vy = mul(state.vector_1c4[1], scalar);
            let ay = add(y, vy);
            let vz = mul(0.0, scalar);
            let az = add(z, vz);
            let pxp = mul(ax, scalar);
            let px = add(pxp, state.position_4c[0]);
            let pyp = mul(ay, scalar);
            let py = add(pyp, state.position_4c[1]);
            let pz = mul(az, scalar);
            if ![
                dx, dy, k, yp, xp, x, zp, y, z, vx, ax, vy, ay, vz, az, pxp, px, pyp, py, pz,
            ]
            .into_iter()
            .all(f32::is_finite)
            {
                return Err(Error::NonFiniteIntermediate);
            }
            result.accumulator_90 = [ax, ay, az];
            result.position_4c = [px, py, pz];
            result.flags_48 |= 0x0004;
            result.counter_4a = result.counter_4a.wrapping_add(1);
            Ok(result)
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            let _ = (state, scalar, sample, context);
            Err(Error::UnsupportedArchitecture)
        }
    })
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn context() -> Context {
        Context {
            precision: Precision::Bits64,
            rounding: Rounding::NearestEven,
        }
    }
    fn state() -> State {
        State {
            position_4c: [10., 20., 30.],
            previous_84: [f32::NAN; 3],
            accumulator_90: [2., -4., f32::NAN],
            vector_1c4: [4., 6., f32::NAN],
            flags_48: 0x8001,
            counter_4a: 0xffff,
        }
    }
    #[test]
    fn zero_scalar_keeps_xy_and_copies_previous_position() {
        let out = step(
            state(),
            1,
            1,
            1,
            0.,
            [8., 9., f32::NAN],
            Policy::Mxcsr1f80,
            context(),
        )
        .unwrap();
        assert_eq!(out.previous_84, [10., 20., 30.]);
        assert_eq!(out.accumulator_90, [2., -4., 0.]);
        assert_eq!(out.position_4c, [10., 20., 0.]);
        assert_eq!(out.vector_1c4[2].to_bits(), 0);
        assert_eq!((out.flags_48, out.counter_4a), (0x8005, 0));
    }
    #[test]
    fn nonzero_factor_drives_supplied_xy_and_ignores_sample_z() {
        let mut s = state();
        s.position_4c = [0.; 3];
        s.accumulator_90 = [0.; 3];
        s.vector_1c4 = [0.; 3];
        let a = step(
            s,
            7,
            -1,
            1,
            1.,
            [1., -1., f32::NAN],
            Policy::Mxcsr1f80,
            context(),
        )
        .unwrap();
        let b = step(
            s,
            7,
            -1,
            1,
            1.,
            [1., -1., 42.],
            Policy::Mxcsr1f80,
            context(),
        )
        .unwrap();
        assert_eq!(a, b);
        assert!(a.accumulator_90[0] > 0. && a.accumulator_90[0] < 1.);
        assert_eq!(a.accumulator_90[1], -a.accumulator_90[0]);
        assert_eq!(a.position_4c, a.accumulator_90);
    }
    #[test]
    fn rejects_closed_gates_nonfinite_and_overflow() {
        assert_eq!(
            step(state(), 0, 1, 1, 1., [0.; 3], Policy::Mxcsr1f80, context()),
            Err(Error::FlowHandleRequired)
        );
        assert_eq!(
            step(state(), 1, 2, 1, 1., [0.; 3], Policy::Mxcsr1f80, context()),
            Err(Error::Multipart)
        );
        assert_eq!(
            step(state(), 1, 1, 0, 1., [0.; 3], Policy::Mxcsr1f80, context()),
            Err(Error::DefinitionGateClosed)
        );
        assert_eq!(
            step(
                state(),
                1,
                1,
                1,
                f32::NAN,
                [0.; 3],
                Policy::Mxcsr1f80,
                context()
            ),
            Err(Error::NonFiniteInput)
        );
        let mut s = state();
        s.vector_1c4[0] = f32::MAX;
        assert_eq!(
            step(s, 1, 1, 1, 2., [0.; 3], Policy::Mxcsr1f80, context()),
            Err(Error::NonFiniteIntermediate)
        );
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn factor_restores_a_full_x87_stack_and_control() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let mut saved = Environment([0; 512]);
        let mut before_image = Environment([0; 512]);
        let mut after_image = Environment([0; 512]);
        let input: [i32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut output = [0.0f32; 8];
        let control = 0x0b7fu16;
        let mut after_control = 0u16;
        let mut before_status = 0u16;
        let mut after_status = 0u16;
        // Preserve the test thread's complete incoming state, then fill all
        // eight physical x87 registers under a distinct masked control word.
        unsafe {
            std::arch::asm!(
                "fxsave64 [{saved}]", "fninit", "fldcw [{control}]",
                "fild dword ptr [{input}]", "fild dword ptr [{input}+4]",
                "fild dword ptr [{input}+8]", "fild dword ptr [{input}+12]",
                "fild dword ptr [{input}+16]", "fild dword ptr [{input}+20]",
                "fild dword ptr [{input}+24]", "fild dword ptr [{input}+28]",
                "fnstsw word ptr [{status}]", "fxsave64 [{image}]",
                saved = in(reg) &mut saved, control = in(reg) &control,
                input = in(reg) input.as_ptr(), status = in(reg) &mut before_status,
                image = in(reg) &mut before_image,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        let result = factor(1.0, context());
        unsafe {
            std::arch::asm!(
                "fnstcw word ptr [{control}]", "fnstsw word ptr [{status}]", "fxsave64 [{image}]",
                "fstp dword ptr [{output}]", "fstp dword ptr [{output}+4]",
                "fstp dword ptr [{output}+8]", "fstp dword ptr [{output}+12]",
                "fstp dword ptr [{output}+16]", "fstp dword ptr [{output}+20]",
                "fstp dword ptr [{output}+24]", "fstp dword ptr [{output}+28]",
                "fxrstor64 [{saved}]",
                saved = in(reg) &saved, control = in(reg) &mut after_control,
                status = in(reg) &mut after_status, output = in(reg) output.as_mut_ptr(),
                image = in(reg) &mut after_image,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
                out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
                out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
                out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
                options(nostack),
            );
        }
        assert!(result > 0.0 && result < 1.0);
        assert_eq!(after_control, control);
        assert_eq!(after_status, before_status);
        assert_eq!(output, [8., 7., 6., 5., 4., 3., 2., 1.]);
        // Compare control/status and abridged tag fields and ten payload
        // bytes in each slot. FIP/FDP/FOP are not checked: some AMD64 CPUs
        // may omit them when ES is clear. This Rust ABI boundary is not proof
        // of arbitrary incoming x87-state preservation.
        assert_eq!(&before_image.0[..5], &after_image.0[..5]);
        for slot in 0..8 {
            let start = 32 + slot * 16;
            assert_eq!(
                &before_image.0[start..start + 10],
                &after_image.0[start..start + 10]
            );
        }
    }
}
