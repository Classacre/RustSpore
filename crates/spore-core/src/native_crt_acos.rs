//! Bounded installed-CRT SSE2 `_CIacos` arithmetic, with an explicit policy.
//!
//! This is the finite nonnegative kernel recovered from MSVCR90 9.00.21022.8.
//! It neither selects a retail dispatch mode nor supplies original CRT data.
//! The caller borrows the pinned DLL's constant window. Every floating
//! operation is SSE2 binary64 under a scoped MXCSR=0x1f80; the previous
//! thread state is restored. The distinct legacy x87 policy is unsupported.
use std::fmt;

pub const CONSTANT_WINDOW_ADDRESS: u32 = 0x7853_4400;
pub const CONSTANT_WINDOW_LENGTH: usize = 0x1c50;
pub const SOURCE_DLL_SHA256: &str =
    "ed0170d3de86da33e02bfa1605eec8ff6010583481b1c530843867c1939d2185";
pub const CONSTANT_WINDOW_SHA256: &str =
    "76210f1fef0a723d62dbcf92c1a7ce9ce1697b99f6a14530692fa8e3871f07df";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcosPolicy {
    Sse2Mxcsr1f80,
    LegacyX87,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcosError {
    UnsupportedPolicy,
    UnsupportedArchitecture,
    InputOutsideFiniteUnitInterval,
    InvalidConstantWindowLength,
    ConstantAddressOutsideWindow(u32),
    NonFiniteResult,
}
impl fmt::Display for AcosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded installed CRT acos: {self:?}")
    }
}
impl std::error::Error for AcosError {}

/// Original constant bytes remain external. The owner must verify the DLL
/// or window digest above; this dependency-free view checks shape and bounds.
#[derive(Debug, Clone, Copy)]
pub struct ConstantWindow<'a> {
    bytes: &'a [u8],
}
impl<'a> ConstantWindow<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, AcosError> {
        if bytes.len() != CONSTANT_WINDOW_LENGTH {
            return Err(AcosError::InvalidConstantWindowLength);
        }
        Ok(Self { bytes })
    }

    fn word(self, address: u32) -> Result<u64, AcosError> {
        let offset = address
            .checked_sub(CONSTANT_WINDOW_ADDRESS)
            .ok_or(AcosError::ConstantAddressOutsideWindow(address))? as usize;
        let bytes = self
            .bytes
            .get(offset..offset.saturating_add(8))
            .ok_or(AcosError::ConstantAddressOutsideWindow(address))?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn pair(self, address: u32) -> Result<V, AcosError> {
        Ok(V([
            self.word(address)?,
            self.word(
                address
                    .checked_add(8)
                    .ok_or(AcosError::ConstantAddressOutsideWindow(address))?,
            )?,
        ]))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseBranch {
    TinyConstant,
    SmallPolynomial,
    Table,
    SqrtTable,
    NearEndpoint,
    ExactOne,
}
impl SseBranch {
    pub const fn name(self) -> &'static str {
        match self {
            Self::TinyConstant => "tiny",
            Self::SmallPolynomial => "small",
            Self::Table => "table",
            Self::SqrtTable => "sqrt-table",
            Self::NearEndpoint => "near-one",
            Self::ExactOne => "one",
        }
    }
}

pub fn finite_branch(value: f64) -> Result<SseBranch, AcosError> {
    let bits = value.to_bits();
    let magnitude = bits & 0x7fff_ffff_ffff_ffff;
    if magnitude > 0x3ff0_0000_0000_0000 || bits >> 63 != 0 && magnitude != 0 {
        return Err(AcosError::InputOutsideFiniteUnitInterval);
    }
    let q = ((bits >> 44) & 0x7ffff) as u32;
    let mut a = q.wrapping_sub(0x3fb00);
    if a < 0x3bb {
        return Ok(SseBranch::Table);
    }
    a = a.wrapping_sub(0x3bb);
    if a < 0x41 {
        return Ok(SseBranch::SqrtTable);
    }
    a = a.wrapping_add(0x3bbb);
    if a < 0x3800 {
        return Ok(SseBranch::SmallPolynomial);
    }
    a = a.wrapping_sub(0x3bfc);
    if a < 4 {
        return Ok(SseBranch::NearEndpoint);
    }
    a = a.wrapping_add(0x3fefc);
    if a < 0x3ff00 {
        Ok(SseBranch::TinyConstant)
    } else {
        Ok(SseBranch::ExactOne)
    }
}

/// Reproduce only the recovered finite SSE2 policy, returning its committed
/// binary64 result. Signed zero is accepted; other negative inputs, NaNs,
/// infinities and values above one are rejected before arithmetic.
pub fn finite_nonnegative(
    value: f64,
    constants: ConstantWindow<'_>,
    policy: AcosPolicy,
) -> Result<f64, AcosError> {
    if policy != AcosPolicy::Sse2Mxcsr1f80 {
        return Err(AcosError::UnsupportedPolicy);
    }
    let branch = finite_branch(value)?;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if !std::is_x86_feature_detected!("sse2") {
            return Err(AcosError::UnsupportedArchitecture);
        }
        // SAFETY: the feature is checked and the guard remains on this thread
        // until all bounded scalar/packed operations have completed.
        let _state = unsafe { MxcsrGuard::install() };
        let result = evaluate(value.to_bits(), constants, branch)?;
        if !f64::from_bits(result).is_finite() {
            return Err(AcosError::NonFiniteResult);
        }
        Ok(f64::from_bits(result))
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = (constants, branch);
        Err(AcosError::UnsupportedArchitecture)
    }
}

// Only low scalar lanes are changed by SD operations. Packed operations are
// two independent binary64 operations with the same fixed environment. Bit
// operations model the source lanes, including PSHUFD's cross-lane copies.
#[derive(Debug, Clone, Copy)]
struct V([u64; 2]);
impl V {
    fn zero() -> Self {
        Self([0; 2])
    }
    fn low(bits: u64) -> Self {
        Self([bits, 0])
    }
    fn mov(self, rhs: Self) -> Self {
        Self([rhs.0[0], self.0[1]])
    }
    fn and(self, rhs: Self) -> Self {
        Self([self.0[0] & rhs.0[0], self.0[1] & rhs.0[1]])
    }
    fn and_not(self, rhs: Self) -> Self {
        Self([!self.0[0] & rhs.0[0], !self.0[1] & rhs.0[1]])
    }
    fn or(self, rhs: Self) -> Self {
        Self([self.0[0] | rhs.0[0], self.0[1] | rhs.0[1]])
    }
    fn xor(self, rhs: Self) -> Self {
        Self([self.0[0] ^ rhs.0[0], self.0[1] ^ rhs.0[1]])
    }
    fn shr(self, count: u32) -> Self {
        Self([self.0[0] >> count, self.0[1] >> count])
    }
    fn shl(self, count: u32) -> Self {
        Self([self.0[0] << count, self.0[1] << count])
    }
    fn shuffle(self, imm: u8) -> Self {
        let source = [
            self.0[0] as u32,
            (self.0[0] >> 32) as u32,
            self.0[1] as u32,
            (self.0[1] >> 32) as u32,
        ];
        let out: [u32; 4] = std::array::from_fn(|i| source[(imm as usize >> (i * 2)) & 3]);
        Self([
            u64::from(out[0]) | u64::from(out[1]) << 32,
            u64::from(out[2]) | u64::from(out[3]) << 32,
        ])
    }
    fn insert_word(mut self, value: u16, index: usize) -> Self {
        let shift = (index % 4) * 16;
        self.0[index / 4] = (self.0[index / 4] & !(0xffff << shift)) | (u64::from(value) << shift);
        self
    }
    fn word(self, index: usize) -> u16 {
        (self.0[index / 4] >> (16 * (index % 4))) as u16
    }
    fn byte_signs(self) -> u32 {
        (0..16).fold(0, |mask, i| {
            mask | (((self.0[i / 8] >> (8 * (i % 8) + 7)) & 1) as u32) << i
        })
    }
    fn unpack_low(self, rhs: Self) -> Self {
        Self([self.0[0], rhs.0[0]])
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
struct MxcsrGuard(u32);
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl MxcsrGuard {
    unsafe fn install() -> Self {
        let mut previous = 0u32;
        let requested = 0x1f80u32;
        // SAFETY: the caller has checked SSE2 and both pointers are local u32s.
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
        // SAFETY: the saved state came from this thread and remains valid.
        unsafe {
            std::arch::asm!("ldmxcsr [{old}]", old = in(reg) &self.0,
            options(nostack, preserves_flags));
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! scalar {
    ($name:ident, $opcode:literal) => {
        fn $name(left: u64, right: u64) -> u64 {
            let mut result = 0u64;
            // SAFETY: called only under the checked SSE2 policy/guard. All
            // pointers address live u64s; both scratch XMM registers are clobbers.
            unsafe { std::arch::asm!("movq xmm0, qword ptr [{left}]",
                "movq xmm1, qword ptr [{right}]", concat!($opcode, " xmm0, xmm1"),
                "movq qword ptr [{out}], xmm0", left = in(reg) &left,
                right = in(reg) &right, out = in(reg) &mut result,
                out("xmm0") _, out("xmm1") _, options(nostack, preserves_flags)); }
            result
        }
    };
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
scalar!(add, "addsd");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
scalar!(sub, "subsd");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
scalar!(mul, "mulsd");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
scalar!(div, "divsd");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
scalar!(sqrt, "sqrtsd");

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl V {
    fn add(self, rhs: Self) -> Self {
        Self([add(self.0[0], rhs.0[0]), self.0[1]])
    }
    fn sub(self, rhs: Self) -> Self {
        Self([sub(self.0[0], rhs.0[0]), self.0[1]])
    }
    fn mul(self, rhs: Self) -> Self {
        Self([mul(self.0[0], rhs.0[0]), self.0[1]])
    }
    fn div(self, rhs: Self) -> Self {
        Self([div(self.0[0], rhs.0[0]), self.0[1]])
    }
    fn sqrt(self) -> Self {
        Self([sqrt(0, self.0[0]), self.0[1]])
    }
    fn add_pair(self, rhs: Self) -> Self {
        Self([add(self.0[0], rhs.0[0]), add(self.0[1], rhs.0[1])])
    }
    fn sub_pair(self, rhs: Self) -> Self {
        Self([sub(self.0[0], rhs.0[0]), sub(self.0[1], rhs.0[1])])
    }
    fn mul_pair(self, rhs: Self) -> Self {
        Self([mul(self.0[0], rhs.0[0]), mul(self.0[1], rhs.0[1])])
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn evaluate(bits: u64, c: ConstantWindow<'_>, branch: SseBranch) -> Result<u64, AcosError> {
    // The intrinsic MOVQ initializes XMM0's upper lane to zero. Other source
    // upper lanes are not consumed before a full definition/cross-lane copy;
    // zero initializes those unobserved lanes in this representation.
    let mut x0 = V::low(bits);
    let mut x1 = V::low(bits);
    let mut x2 = V::low(c.word(0x7853_6038)?);
    let mut x3 = V::low(c.word(0x7853_6040)?);
    let mut x4 = V::low(c.word(0x7853_6030)?);
    let mut x5 = V::zero().insert_word(0x2000, 2);
    let mut x6;
    let mut x7 = V::low(bits);
    let mut edx = ((bits >> 44) & 0xffff_ffff) as u32;
    match branch {
        SseBranch::Table => {
            x1 = x1.mul(x1);
            edx &= 0xffff;
            x3 = x3.sub(x1).sqrt();
            x2 = x2.and(x7);
            edx = (edx & !3).wrapping_sub(0xfb00);
            x1 = x1.mov(V::low(
                c.word(0x7853_5770u32.wrapping_add(edx.wrapping_mul(2)))?,
            ));
            x2 = x2.or(x5);
            x4 = c.pair(0x7853_4870u32.wrapping_add(edx.wrapping_mul(4)))?;
            x6 = V::zero().mov(x7);
            x7 = x7.add(x2);
            x0 = x0.sub(x2);
            x7 = x7.mul(x0);
            x6 = x6.mul(x1);
            x3 = x3.mul(x2);
            x1 = x1.mov(x6);
            x6 = x6.add(x3);
            x7 = x7.div(x6);
            x0 = x0.mov(V::low(c.word(0x7853_5fe8)?));
            x5 = x5.mov(V::low(c.word(0x7853_5fd8)?));
            x1 = x1.sub(x3);
            x2 = x2.shr(63);
            x3 = x3.mov(x1);
            x2 = x2.shl(63);
            x1 = x1.mul(x1);
            x2 = x2.shuffle(0x44);
            x6 = x6.mov(V::low(c.word(0x7853_5fe0)?));
            x3 = x3.mul(x1);
            x0 = x0.mul(x1);
            x4 = x4.xor(x2);
            x5 = x5.mul(x3);
            x4 = x4.sub_pair(c.pair(0x7853_5f70)?);
            x3 = x3.mul(x1);
            x0 = x0.add(x6);
            x0 = x0.mul(x3);
            x5 = x5.sub(x4);
            x4 = x4.shuffle(0xee);
            x0 = x0.add(x5).sub(x7).sub(x4);
        }
        SseBranch::SqrtTable => {
            x7 = x7.shr(38).shl(38);
            let sign = x0.byte_signs() & 0x80;
            x4 = x4.and_not(x0);
            x1 = x1.sub(x7);
            x6 = V::zero().mov(x7);
            x7 = x7.mul(x7);
            x0 = x0.add(x6);
            x5 = x5.or(x4);
            x3 = x3.sub(x7);
            x0 = x0.mul(x1);
            x4 = x4.mov(x3);
            x3 = x3.sub(x0).sqrt();
            let mut eax = 0u32.wrapping_sub(sign >> 7);
            x7 = x7.mov(x3);
            x2 = x2.and(x3);
            x3 = x3.shl(2);
            edx = u32::from(x3.word(3));
            x2 = x2.or(x5);
            x3 = V::low(u64::from(eax)).shuffle(0);
            edx = edx.wrapping_sub(0xfec0).wrapping_mul(2);
            x7 = x7.mul(V::low(
                c.word(0x7853_5770u32.wrapping_add(edx.wrapping_mul(4)))?,
            ));
            x6 = x6.mul(x2);
            x1 = x1.mul(x2);
            x2 = x2.mul(x2);
            x6 = x6.sub(x7);
            x3 = x3.and(c.pair(0x7853_5f80)?);
            x6 = x6.add(x1);
            x4 = x4.sub(x2);
            x7 = x7.add(x7);
            x5 = x5.mov(V::low(c.word(0x7853_5fd8)?));
            x4 = x4.sub(x0);
            x7 = x7.add(x6);
            x0 = x0.mov(V::low(c.word(0x7853_5fe8)?));
            x4 = x4.div(x7);
            x2 = x2.mov(V::low(c.word(0x7853_5fe0)?));
            x3 = x3.add_pair(c.pair(0x7853_4870u32.wrapping_add(edx.wrapping_mul(8)))?);
            x1 = x1.mov(x6);
            x6 = x6.mul(x6);
            x0 = x0.mul(x6);
            x1 = x1.mul(x6);
            x5 = x5.mul(x1);
            x1 = x1.mul(x6);
            x0 = x0.add(x2);
            x6 = V::zero();
            x0 = x0.mul(x1);
            x5 = x5.add(x3);
            x0 = x0.add(x5);
            eax &= 0x8000;
            x6 = x6.insert_word(eax as u16, 3);
            x5 = x5.mov(x4);
            x3 = x3.shuffle(0xee);
            x4 = x4.add(x3);
            x3 = x3.sub(x4);
            x5 = x5.add(x3);
            x0 = x0.add(x5).add(x4).xor(x6);
        }
        SseBranch::SmallPolynomial => {
            x0 = x0.unpack_low(x0);
            x6 = c.pair(0x7853_5ff0)?;
            x1 = x1.unpack_low(x0);
            x2 = c.pair(0x7853_6000)?;
            x4 = c.pair(0x7853_6010)?;
            x0 = x0.mul_pair(x0);
            x5 = c.pair(0x7853_5f70)?;
            x1 = x1.mul_pair(x0);
            x6 = x6.mul_pair(x0);
            x0 = x0.mul_pair(x0);
            x3 = x3.mov(x1);
            x1 = x1.mul(x1);
            x6 = x6.add_pair(x2);
            x4 = x4.mul_pair(x0);
            x1 = x1.mul(x3);
            x6 = x6.add_pair(x4);
            x0 = x5.shuffle(0xee);
            x1 = x1.mul_pair(x6);
            x6 = x5.shuffle(0xee);
            x0 = x0.sub(x7);
            x2 = x1.shuffle(0xee);
            x5 = x5.sub(x1);
            x6 = x6.sub(x0);
            x5 = x5.sub(x2);
            x7 = x7.sub(x6);
            x5 = x5.sub(x7);
            x0 = x0.add(x5);
        }
        SseBranch::NearEndpoint => {
            x6 = V::zero();
            x7 = x7.and(c.pair(0x7853_6030)?);
            x4 = x4.mov(V::low(c.word(0x7853_6048)?));
            x1 = c.pair(0x7853_5ff0)?;
            x7 = x7.mul(x4);
            x2 = c.pair(0x7853_6000)?;
            x4 = x4.sub(x7);
            x3 = c.pair(0x7853_6010)?;
            x7 = x4.shuffle(0x44);
            x4 = x4.sqrt();
            x1 = x1.mul_pair(x7);
            x5 = x7.shuffle(0x44);
            let mut eax = u32::from(x0.word(3));
            x7 = x7.mul_pair(x7);
            x2 = x2.add_pair(x1);
            x1 = x1.mov(V::low(c.word(0x7853_5fb0)?));
            x3 = x3.mul_pair(x7);
            // Source CMPLTSD against zero. The admitted domain includes only
            // nonnegative inputs and signed zero, so its mask is exactly zero.
            x0 = x0.mov(V::zero());
            x7 = x7.mul(x5);
            x2 = x2.add_pair(x3);
            x0 = x0.shuffle(0x44);
            x2 = x2.mul(x7);
            x0 = x0.and(c.pair(0x7853_5f80)?);
            x2 = x2.mul_pair(x5);
            x1 = x1.and(x4);
            x3 = x4.shuffle(0x44);
            x4 = x4.sub(x1);
            x3 = x3.add(x3);
            x1 = x1.mul(x1);
            x3 = x3.sub(x4);
            x5 = x5.sub(x1);
            x4 = x4.mul(x3);
            x3 = x3.shuffle(0xee);
            x5 = x5.sub(x4).div(x3);
            x3 = x3.add_pair(x3);
            x2 = x2.mul_pair(x3);
            x4 = x2.shuffle(0xee);
            x2 = x2.add(x0);
            eax &= 0x8000;
            x6 = x6.insert_word(eax as u16, 3);
            x0 = x0.shuffle(0xee);
            x2 = x2.add(x4).add(x5).add(x3);
            x0 = x0.add(x2).xor(x6);
        }
        SseBranch::TinyConstant => {
            x2 = x2.mov(V::low(c.word(0x7853_5f70)?));
            x0 = x0.mov(V::low(c.word(0x7853_5f78)?));
            x0 = x0.add(x2);
        }
        SseBranch::ExactOne => {
            // The negative endpoint is outside this API's domain. The native
            // sign-selected masks are zero at +1, including both split terms.
            x7 = V::zero();
            x2 = x2.mov(V::low(c.word(0x7853_5f90)?)).and(x7);
            x0 = x0.mov(V::low(c.word(0x7853_5f98)?)).and(x7).add(x2);
        }
    }
    Ok(x0.0[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_domains_and_guard_edges() {
        for (value, branch) in [
            (0.0, SseBranch::TinyConstant),
            (-0.0, SseBranch::TinyConstant),
            (2f64.powi(-60), SseBranch::SmallPolynomial),
            (1.0 / 16.0, SseBranch::Table),
            (0.865234375, SseBranch::SqrtTable),
            (0.99951171875, SseBranch::NearEndpoint),
            (1.0, SseBranch::ExactOne),
        ] {
            assert_eq!(finite_branch(value), Ok(branch));
        }
        for value in [
            -1.0,
            -f64::MIN_POSITIVE,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(1.0f64.to_bits() + 1),
        ] {
            assert_eq!(
                finite_branch(value),
                Err(AcosError::InputOutsideFiniteUnitInterval)
            );
        }
        assert_eq!(
            ConstantWindow::new(&[]).unwrap_err(),
            AcosError::InvalidConstantWindowLength
        );
        let bytes = vec![0; CONSTANT_WINDOW_LENGTH];
        let c = ConstantWindow::new(&bytes).unwrap();
        assert_eq!(
            finite_nonnegative(0.5, c, AcosPolicy::LegacyX87),
            Err(AcosError::UnsupportedPolicy)
        );
        assert_eq!(
            c.word(CONSTANT_WINDOW_ADDRESS - 1),
            Err(AcosError::ConstantAddressOutsideWindow(
                CONSTANT_WINDOW_ADDRESS - 1
            ))
        );
        assert_eq!(
            c.word(CONSTANT_WINDOW_ADDRESS + CONSTANT_WINDOW_LENGTH as u32 - 7),
            Err(AcosError::ConstantAddressOutsideWindow(
                CONSTANT_WINDOW_ADDRESS + CONSTANT_WINDOW_LENGTH as u32 - 7
            ))
        );
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn scoped_policy_restores_mxcsr_on_success_and_error() {
        if !std::is_x86_feature_detected!("sse2") {
            return;
        }
        let mut saved = 0u32;
        // SAFETY: live local state, checked SSE2, and the outer guard restores
        // the test thread's exact incoming state even if an assertion panics.
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut saved,
            options(nostack, preserves_flags));
        }
        let _restore = MxcsrGuard(saved);
        let unusual = (saved & !0x6000) | 0x4000 | 0x1fa0;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]", p = in(reg) &unusual,
            options(nostack, preserves_flags));
        }
        let mut bytes = vec![0; CONSTANT_WINDOW_LENGTH];
        let constants = ConstantWindow::new(&bytes).unwrap();
        assert_eq!(
            finite_nonnegative(0.0, constants, AcosPolicy::Sse2Mxcsr1f80),
            Ok(0.0)
        );
        let mut after = 0u32;
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut after,
            options(nostack, preserves_flags));
        }
        assert_eq!(after, unusual);
        let offset = (0x7853_5f78 - CONSTANT_WINDOW_ADDRESS) as usize;
        bytes[offset..offset + 8].copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
        let constants = ConstantWindow::new(&bytes).unwrap();
        assert_eq!(
            finite_nonnegative(0.0, constants, AcosPolicy::Sse2Mxcsr1f80),
            Err(AcosError::NonFiniteResult)
        );
        unsafe {
            std::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut after,
            options(nostack, preserves_flags));
        }
        assert_eq!(after, unusual);
    }
}
