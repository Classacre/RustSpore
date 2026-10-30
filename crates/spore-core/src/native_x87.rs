//! Explicit x87 arithmetic for reconstructed expressions with retained registers.
//!
//! A stored `f32` is not a substitute for an uncommitted x87 intermediate.
//! Precision and rounding are caller inputs: the original process control word
//! is not established by this module. Values stay in the 80-bit storage format
//! between operations. Each hardware operation restores the caller's x87
//! environment; it neither changes MXCSR nor leaves an x87 stack entry behind.

use std::{cmp::Ordering, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precision {
    Bits24,
    Bits53,
    Bits64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    NearestEven,
    Down,
    Up,
    TowardZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X87Error {
    NonFiniteInput,
    NonFiniteResult,
    UnsupportedArchitecture,
    ReservedPrecision,
    SinArgumentOutsideBound,
    SinArgumentReductionIncomplete,
    CosArgumentOutsideBound,
    CosArgumentReductionIncomplete,
}
impl fmt::Display for X87Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded x87 arithmetic: {self:?}")
    }
}
impl std::error::Error for X87Error {}

/// Finite canonical x87 storage, including signed zero and subnormals.
/// Raw construction is intentionally unavailable; operations check results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extended([u8; 10]);
impl Extended {
    /// FILD of a signed 32-bit integer is exact before arithmetic rounding.
    pub fn from_i32(value: i32) -> Self {
        let magnitude = i64::from(value).unsigned_abs();
        let mut bytes = [0; 10];
        if magnitude != 0 {
            let leading = 63 - magnitude.leading_zeros();
            bytes[..8].copy_from_slice(&(magnitude << (63 - leading)).to_le_bytes());
            bytes[8..].copy_from_slice(
                &((leading as u16 + 16383) | if value < 0 { 0x8000 } else { 0 }).to_le_bytes(),
            );
        }
        Self(bytes)
    }

    /// FLD of a binary32 operand is exact regardless of precision control.
    pub fn from_f32(value: f32) -> Result<Self, X87Error> {
        if !value.is_finite() {
            return Err(X87Error::NonFiniteInput);
        }
        let bits = value.to_bits();
        let fraction = bits & 0x7f_ffff;
        let exponent = (bits >> 23) & 0xff;
        let (significand, exponent) = if exponent != 0 {
            (
                (u64::from(fraction) | (1 << 23)) << 40,
                exponent as i32 - 127 + 16383,
            )
        } else if fraction != 0 {
            let leading = 31 - fraction.leading_zeros();
            (
                u64::from(fraction) << (63 - leading),
                leading as i32 - 149 + 16383,
            )
        } else {
            (0, 0)
        };
        let mut bytes = [0; 10];
        bytes[..8].copy_from_slice(&significand.to_le_bytes());
        bytes[8..]
            .copy_from_slice(&((exponent as u16) | ((bits >> 16) as u16 & 0x8000)).to_le_bytes());
        Ok(Self(bytes))
    }

    /// FLD of a finite binary64 operand, without an intermediate binary32
    /// commit. Exact for normals, subnormals and both signs of zero.
    pub fn from_f64(value: f64) -> Result<Self, X87Error> {
        if !value.is_finite() {
            return Err(X87Error::NonFiniteInput);
        }
        let bits = value.to_bits();
        let fraction = bits & 0x000f_ffff_ffff_ffff;
        let exponent = (bits >> 52) & 0x7ff;
        let (significand, exponent) = if exponent != 0 {
            ((fraction | (1 << 52)) << 11, exponent as i32 - 1023 + 16383)
        } else if fraction != 0 {
            let leading = 63 - fraction.leading_zeros();
            (fraction << (63 - leading), leading as i32 - 1074 + 16383)
        } else {
            (0, 0)
        };
        let mut bytes = [0; 10];
        bytes[..8].copy_from_slice(&significand.to_le_bytes());
        bytes[8..]
            .copy_from_slice(&((exponent as u16) | ((bits >> 48) as u16 & 0x8000)).to_le_bytes());
        Ok(Self(bytes))
    }

    pub fn to_le_bytes(self) -> [u8; 10] {
        self.0
    }

    pub fn abs(mut self) -> Self {
        self.0[9] &= 0x7f;
        self
    }

    pub fn negated(mut self) -> Self {
        self.0[9] ^= 0x80;
        self
    }

    fn checked(bytes: [u8; 10]) -> Result<Self, X87Error> {
        if u16::from_le_bytes([bytes[8], bytes[9]]) & 0x7fff == 0x7fff {
            Err(X87Error::NonFiniteResult)
        } else {
            Ok(Self(bytes))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub precision: Precision,
    pub rounding: Rounding,
}
impl Context {
    /// Select only the arithmetic fields of an explicitly supplied control
    /// word. Exceptions are masked inside each operation and finite results
    /// are checked. This does not claim the retail control word is known.
    pub fn from_control_word(word: u16) -> Result<Self, X87Error> {
        let precision = match word & 0x300 {
            0 => Precision::Bits24,
            0x200 => Precision::Bits53,
            0x300 => Precision::Bits64,
            _ => return Err(X87Error::ReservedPrecision),
        };
        let rounding = match word & 0xc00 {
            0 => Rounding::NearestEven,
            0x400 => Rounding::Down,
            0x800 => Rounding::Up,
            _ => Rounding::TowardZero,
        };
        Ok(Self {
            precision,
            rounding,
        })
    }

    pub fn control_word(self) -> u16 {
        0x007f
            | match self.precision {
                Precision::Bits24 => 0,
                Precision::Bits53 => 0x200,
                Precision::Bits64 => 0x300,
            }
            | match self.rounding {
                Rounding::NearestEven => 0,
                Rounding::Down => 0x400,
                Rounding::Up => 0x800,
                Rounding::TowardZero => 0xc00,
            }
    }
}

// FNSTENV/FLDENV use the 28-byte legacy environment in this address mode.
// An extra four bytes make the buffer naturally sized/aligned. All eight x87
// registers are declared clobbered, so Rust guarantees an empty entry stack.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[repr(C, align(16))]
struct Environment([u8; 32]);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! binary_operation {
    ($name:ident, $instruction:literal) => {
        pub fn $name(self, left: Extended, right: Extended) -> Result<Extended, X87Error> {
            let mut environment = Environment([0; 32]);
            let control = self.control_word();
            let mut result = [0u8; 10];
            // SAFETY: every pointer addresses a live bounded local buffer.
            // Both loaded stack entries are popped, and the saved environment
            // is restored before leaving this single assembly block.
            unsafe {
                std::arch::asm!(
                    "fnstenv [{environment}]",
                    "fldcw [{control}]",
                    "fld tbyte ptr [{left}]",
                    "fld tbyte ptr [{right}]",
                    $instruction,
                    "fstp tbyte ptr [{result}]",
                    "fldenv [{environment}]",
                    environment = in(reg) &mut environment,
                    control = in(reg) &control,
                    left = in(reg) left.0.as_ptr(), right = in(reg) right.0.as_ptr(),
                    result = in(reg) result.as_mut_ptr(),
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
            Extended::checked(result)
        }
    };
}

impl Context {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    binary_operation!(add, "faddp st(1), st(0)");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    binary_operation!(sub, "fsubp st(1), st(0)");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    binary_operation!(mul, "fmulp st(1), st(0)");
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    binary_operation!(div, "fdivp st(1), st(0)");

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sqrt(self, value: Extended) -> Result<Extended, X87Error> {
        let mut environment = Environment([0; 32]);
        let control = self.control_word();
        let mut result = [0u8; 10];
        // SAFETY: one bounded operand is loaded and popped. Environment and
        // register-stack obligations are identical to binary_operation.
        unsafe {
            std::arch::asm!(
                "fnstenv [{environment}]", "fldcw [{control}]",
                "fld tbyte ptr [{value}]", "fsqrt", "fstp tbyte ptr [{result}]",
                "fldenv [{environment}]",
                environment = in(reg) &mut environment, control = in(reg) &control,
                value = in(reg) value.0.as_ptr(), result = in(reg) result.as_mut_ptr(),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        Extended::checked(result)
    }

    /// Retained hardware FSIN only for finite |argument| <= 2 radians.
    ///
    /// The caller must establish this narrow range; no software range
    /// reduction or library sine is substituted. C2 is checked immediately
    /// after FSIN and an incomplete reduction is rejected even in this bound.
    /// The result remains in 80-bit storage, including signed zero. The
    /// instruction's hardware approximation is not claimed to be an exact
    /// transcendental oracle. x87 environment restoration and MXCSR behavior
    /// match the other controlled operations.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sin_magnitude_at_most_two(self, value: Extended) -> Result<Extended, X87Error> {
        self.sin_with_magnitude_bound(value, 2)
    }

    /// The same retained hardware FSIN operation, with a four-radian bound for
    /// the source quaternion extrapolation fractions [-1,2]. No extra range
    /// reduction or precision commit is introduced.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn sin_magnitude_at_most_four(self, value: Extended) -> Result<Extended, X87Error> {
        self.sin_with_magnitude_bound(value, 4)
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn sin_with_magnitude_bound(self, value: Extended, bound: i32) -> Result<Extended, X87Error> {
        if self.compare(value.abs(), Extended::from_i32(bound))? == Ordering::Greater {
            return Err(X87Error::SinArgumentOutsideBound);
        }
        let mut environment = Environment([0; 32]);
        let control = self.control_word();
        let mut result = [0u8; 10];
        let status: u16;
        // SAFETY: bounded canonical input/output storage; the narrow range
        // excludes the architectural 2^63 FSIN reduction limit. The status is
        // captured before a store/pop, and the saved environment is restored
        // before interpreting C2 or the returned finite-value storage.
        unsafe {
            std::arch::asm!(
                "fnstenv [{environment}]", "fldcw [{control}]",
                "fld tbyte ptr [{value}]", "fsin", "fnstsw ax",
                "fstp tbyte ptr [{result}]", "fldenv [{environment}]",
                environment = in(reg) &mut environment, control = in(reg) &control,
                value = in(reg) value.0.as_ptr(), result = in(reg) result.as_mut_ptr(),
                out("ax") status,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        if status & 0x0400 != 0 {
            Err(X87Error::SinArgumentReductionIncomplete)
        } else {
            Extended::checked(result)
        }
    }

    /// Retained host hardware FCOS for finite |argument| <= 4 radians.
    /// C2 is checked immediately; no library cosine or range reduction is
    /// substituted. Results depend on the processor's approximation and do
    /// not establish the original game's control word or floating-point flags.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn cos_magnitude_at_most_four(self, value: Extended) -> Result<Extended, X87Error> {
        if self.compare(value.abs(), Extended::from_i32(4))? == Ordering::Greater {
            return Err(X87Error::CosArgumentOutsideBound);
        }
        let mut environment = Environment([0; 32]);
        let control = self.control_word();
        let mut result = [0u8; 10];
        let status: u16;
        // SAFETY: bounded canonical storage; one x87 entry is loaded and
        // popped, C2 is captured immediately, and the saved environment is
        // restored within the block. All x87 registers are declared clobbered.
        unsafe {
            std::arch::asm!(
                "fnstenv [{environment}]", "fldcw [{control}]",
                "fld tbyte ptr [{value}]", "fcos", "fnstsw ax",
                "fstp tbyte ptr [{result}]", "fldenv [{environment}]",
                environment = in(reg) &mut environment, control = in(reg) &control,
                value = in(reg) value.0.as_ptr(), result = in(reg) result.as_mut_ptr(),
                out("ax") status,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        if status & 0x0400 != 0 {
            Err(X87Error::CosArgumentReductionIncomplete)
        } else {
            Extended::checked(result)
        }
    }

    /// FST/FSTP binary32 commit, leaving the supplied retained value available.
    /// Store rounding is controlled by RC, not by PC's significand size.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn store_f32(self, value: Extended) -> Result<f32, X87Error> {
        let mut environment = Environment([0; 32]);
        let control = self.control_word();
        let mut result = 0f32;
        // SAFETY: bounded local input/output; one load and pop, saved state
        // restored in the same block; no changes to the SSE environment.
        unsafe {
            std::arch::asm!(
                "fnstenv [{environment}]", "fldcw [{control}]",
                "fld tbyte ptr [{value}]", "fstp dword ptr [{result}]",
                "fldenv [{environment}]",
                environment = in(reg) &mut environment, control = in(reg) &control,
                value = in(reg) value.0.as_ptr(), result = in(reg) &mut result,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        if result.is_finite() {
            Ok(result)
        } else {
            Err(X87Error::NonFiniteResult)
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn compare(self, left: Extended, right: Extended) -> Result<Ordering, X87Error> {
        let mut environment = Environment([0; 32]);
        let control = self.control_word();
        let (less, equal): (u8, u8);
        // SAFETY: operands are finite canonical values. Load right then left
        // for ST0 vs ST1, pop both entries, and restore the environment. SETcc
        // captures the comparison before any later Rust-generated operation.
        unsafe {
            std::arch::asm!(
                "fnstenv [{environment}]", "fldcw [{control}]",
                "fld tbyte ptr [{right}]", "fld tbyte ptr [{left}]",
                "fucomip st(0), st(1)", "fstp st(0)",
                "fldenv [{environment}]", "setb {less}", "sete {equal}",
                environment = in(reg) &mut environment, control = in(reg) &control,
                left = in(reg) left.0.as_ptr(), right = in(reg) right.0.as_ptr(),
                less = lateout(reg_byte) less, equal = lateout(reg_byte) equal,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
        Ok(if equal != 0 {
            Ordering::Equal
        } else if less != 0 {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn add(self, _: Extended, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn sub(self, _: Extended, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn mul(self, _: Extended, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn div(self, _: Extended, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn sqrt(self, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn sin_magnitude_at_most_two(self, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn sin_magnitude_at_most_four(self, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn cos_magnitude_at_most_four(self, _: Extended) -> Result<Extended, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn store_f32(self, _: Extended) -> Result<f32, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn compare(self, _: Extended, _: Extended) -> Result<Ordering, X87Error> {
        Err(X87Error::UnsupportedArchitecture)
    }
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn context(precision: Precision) -> Context {
        Context {
            precision,
            rounding: Rounding::NearestEven,
        }
    }
    fn load(value: f32) -> Extended {
        Extended::from_f32(value).unwrap()
    }

    #[test]
    fn retained_precision_and_binary32_commit_have_different_branch_values() {
        let one = load(1.);
        let small = load(2f32.powi(-60));
        let wide = context(Precision::Bits64);
        let retained = wide.add(one, small).unwrap();
        assert_eq!(wide.compare(retained, one), Ok(Ordering::Greater));
        assert_eq!(wide.store_f32(retained), Ok(1.));
        assert_eq!(wide.sub(retained, one), Ok(small));
        assert_eq!(context(Precision::Bits53).add(one, small), Ok(one));
        assert_eq!(wide.compare(load(-0.), load(0.)), Ok(Ordering::Equal));
        let exact_integer = Extended::from_i32(16_777_217);
        assert_eq!(
            context(Precision::Bits24).compare(exact_integer, load(16_777_216.)),
            Ok(Ordering::Greater)
        );
        assert_eq!(
            context(Precision::Bits24).store_f32(exact_integer),
            Ok(16_777_216.)
        );
    }

    #[test]
    fn exact_loads_preserve_subnormals_signs_and_all_sampled_binary32_bits() {
        let ctx = context(Precision::Bits64);
        for bits in [
            0,
            0x8000_0000,
            1,
            0x007f_ffff,
            0x0080_0000,
            0x3f80_0000,
            0x7f7f_ffff,
            0xff7f_ffff,
        ]
        .into_iter()
        .chain(
            (0..10000u32)
                .map(|v| v.wrapping_mul(0x9e37_79b9))
                .filter(|v| (v >> 23) & 0xff != 0xff),
        ) {
            assert_eq!(
                ctx.store_f32(load(f32::from_bits(bits))).unwrap().to_bits(),
                bits
            );
        }
        assert_eq!(
            load(1.).to_le_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0x3f]
        );
        assert_eq!(Extended::from_f32(f32::NAN), Err(X87Error::NonFiniteInput));
        assert_eq!(ctx.div(load(1.), load(0.)), Err(X87Error::NonFiniteResult));
        assert_eq!(ctx.sqrt(load(-1.)), Err(X87Error::NonFiniteResult));
        assert_eq!(
            Context::from_control_word(0x017f),
            Err(X87Error::ReservedPrecision)
        );
    }

    #[test]
    fn controlled_operations_restore_the_callers_control_word() {
        fn word() -> u16 {
            let mut result = 0u16;
            // SAFETY: FNSTCW writes two bytes to the supplied local buffer.
            unsafe {
                std::arch::asm!("fnstcw [{result}]", result = in(reg) &mut result, options(nostack, preserves_flags));
            }
            result
        }
        let before = word();
        for control in [0x007f, 0x027f, 0x037f, 0x077f, 0x0b7f, 0x0f7f] {
            let ctx = Context::from_control_word(control).unwrap();
            let result = ctx.sqrt(ctx.div(load(2.), load(3.)).unwrap()).unwrap();
            let _ = ctx.store_f32(result).unwrap();
            let _ = ctx.sin_magnitude_at_most_two(result).unwrap();
            let _ = ctx.cos_magnitude_at_most_four(result).unwrap();
            assert_eq!(ctx.compare(result, load(1.)), Ok(Ordering::Less));
            assert_eq!(word(), before);
        }
    }

    #[test]
    fn binary64_load_retains_bits_below_binary32_and_subnormal_exponents() {
        let wide = context(Precision::Bits64);
        let one = Extended::from_f64(1.).unwrap();
        assert_eq!(one, load(1.));
        let next = Extended::from_f64(f64::from_bits(1f64.to_bits() + 1)).unwrap();
        assert_eq!(wide.compare(next, one), Ok(Ordering::Greater));
        assert_eq!(wide.store_f32(next), Ok(1.));
        assert_eq!(
            Extended::from_f64(f64::from_bits(1)).unwrap().to_le_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0x80, 0xcd, 0x3b]
        );
        assert_eq!(Extended::from_f64(-0.).unwrap(), load(-0.));
        assert_eq!(Extended::from_f64(f64::NAN), Err(X87Error::NonFiniteInput));
        assert_eq!(
            Extended::from_f64(f64::INFINITY),
            Err(X87Error::NonFiniteInput)
        );
    }

    #[test]
    fn retained_sine_has_explicit_range_and_preserves_zero_sign() {
        let ctx = context(Precision::Bits64);
        for zero in [0., -0.] {
            assert_eq!(ctx.sin_magnitude_at_most_two(load(zero)), Ok(load(zero)));
        }
        assert_eq!(
            ctx.sin_magnitude_at_most_two(Extended::from_i32(3)),
            Err(X87Error::SinArgumentOutsideBound)
        );
        let result = ctx.sin_magnitude_at_most_two(load(1.)).unwrap();
        assert_eq!(ctx.compare(result, load(0.)), Ok(Ordering::Greater));
        assert_eq!(ctx.compare(result, load(1.)), Ok(Ordering::Less));
        let wider = ctx
            .sin_magnitude_at_most_four(Extended::from_i32(3))
            .unwrap();
        assert_eq!(ctx.compare(wider, load(0.)), Ok(Ordering::Greater));
        assert_eq!(
            ctx.sin_magnitude_at_most_four(Extended::from_i32(5)),
            Err(X87Error::SinArgumentOutsideBound)
        );
    }

    #[test]
    fn retained_cosine_accepts_boundaries_and_rejects_values_beyond_them() {
        // Exercise all supported PC/rounding combinations; this is host
        // instruction coverage, not evidence of the game's runtime state.
        for control in [
            0x007f, 0x047f, 0x087f, 0x0c7f, // PC24
            0x027f, 0x067f, 0x0a7f, 0x0e7f, // PC53
            0x037f, 0x077f, 0x0b7f, 0x0f7f, // PC64
        ] {
            let ctx = Context::from_control_word(control).unwrap();
            for zero in [0.0, -0.0] {
                assert_eq!(ctx.cos_magnitude_at_most_four(load(zero)), Ok(load(1.0)));
            }
            let positive = ctx.cos_magnitude_at_most_four(load(4.0)).unwrap();
            let negative = ctx.cos_magnitude_at_most_four(load(-4.0)).unwrap();
            assert_eq!(positive, negative);
            assert_eq!(ctx.compare(positive, load(0.0)), Ok(Ordering::Less));
            let outside = f32::from_bits(4.0f32.to_bits() + 1);
            for value in [outside, -outside] {
                assert_eq!(ctx.cos_magnitude_at_most_four(load(value)),
                    Err(X87Error::CosArgumentOutsideBound));
            }
        }
    }
}
