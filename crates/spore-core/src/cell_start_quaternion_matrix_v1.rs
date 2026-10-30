//! Bounded 59C000 four-word to nine-word SSE projection.
//!
//! Static source re/cell-start-quaternion-matrix-static-v1.md. Explicit
//! MXCSR1F80 only; no actor, quaternion normalization, row/column convention,
//! aliasing, native runtime state, exception flags or source-byte execution.
//! Nonfinite inputs/intermediates are rejected as safe policy. Finite
//! subnormals, including intermediate results, remain enabled by MXCSR1F80.
use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { NonFiniteInput, NonFiniteIntermediate, Sse(SseError) }
impl From<SseError> for Error { fn from(e: SseError) -> Self { Self::Sse(e) } }

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
macro_rules! operation {
    ($name:ident, $instruction:literal) => {
        fn $name(a: u32, b: u32) -> Result<u32, Error> {
            let result: u32;
            // SAFETY: called inside SSE2-checked scoped Mxcsr1f80; only XMM
            // operands and integer result are touched, no memory or stack.
            unsafe { std::arch::asm!("movd {x}, {a:e}", "movd {y}, {b:e}",
                $instruction, "movd {result:e}, {x}",
                x=out(xmm_reg) _, y=out(xmm_reg) _, a=in(reg) a, b=in(reg) b,
                result=lateout(reg) result, options(nostack, preserves_flags)); }
            if !f32::from_bits(result).is_finite() { return Err(Error::NonFiniteIntermediate); }
            Ok(result)
        }
    };
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
operation!(mul, "mulss {x}, {y}");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
operation!(add, "addss {x}, {y}");
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
operation!(sub, "subss {x}, {y}");
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn mul(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn add(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn sub(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }

/// Four binary32 input words to nine binary32 words in physical source order.
/// Finite inputs only; arithmetic stops on the first nonfinite intermediate.
/// Policy/FP flags and nonaliasing are safe-fixture choices, not source checks.
pub fn matrix_words(q: [u32; 4], policy: Policy) -> Result<[u32; 9], Error> {
    if q.iter().any(|&v| !f32::from_bits(v).is_finite()) { return Err(Error::NonFiniteInput); }
    native_sse::scoped(policy, || {
        let [x,y,z,w] = q;
        let (one,two)=(1f32.to_bits(),2f32.to_bits());
        // Keep even independent source products in their original sequence.
        let zz=mul(z,z)?; let zx=mul(z,x)?; let wx=mul(w,x)?;
        let yx=mul(y,x)?; let xx=mul(x,x)?; let zy=mul(z,y)?;
        let yy=mul(y,y)?; let wz=mul(w,z)?; let wy=mul(w,y)?;
        let d0=mul(add(zz,yy)?,two)?; let m0=sub(one,d0)?;
        let m1=mul(add(wz,yx)?,two)?;
        // Source computes (zx-wy), then (wy+zx), before later products.
        let d2=sub(zx,wy)?; let d6=add(wy,zx)?; let m2=mul(d2,two)?;
        let d3=sub(yx,wz)?; let m3=mul(d3,two)?;
        let d4=mul(add(zz,xx)?,two)?; let m4=sub(one,d4)?;
        let d5=add(wx,zy)?; let d8=add(yy,xx)?;
        let d7=sub(zy,wx)?; let d8=mul(d8,two)?;
        let m5=mul(d5,two)?; let m6=mul(d6,two)?; let m7=mul(d7,two)?;
        let m8=sub(one,d8)?;
        Ok([m0,m1,m2,m3,m4,m5,m6,m7,m8])
    })
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    fn b(v: f32) -> u32 { v.to_bits() }
    #[test]
    fn zero_vector_and_half_turn_basis() {
        assert_eq!(matrix_words([b(0.),b(0.),b(0.),b(1.)],Policy::Mxcsr1f80).unwrap(),
                   [b(1.),b(0.),b(0.),b(0.),b(1.),b(0.),b(0.),b(0.),b(1.)]);
        assert_eq!(matrix_words([b(0.),b(0.),b(1.),b(0.)],Policy::Mxcsr1f80).unwrap(),
                   [b(-1.),b(0.),b(0.),b(0.),b(-1.),b(0.),b(0.),b(0.),b(1.)]);
    }
    #[test]
    fn signed_zero_and_exact_cancellation_follow_source_operand_order() {
        // With x=z=-0, y=+0, w=1, wz and yx are -0, as are wx
        // and zy. Under nearest-even, both sums remain -0; their products
        // by +2 stay -0. Each subtraction of equal signed-zero operands
        // in the remaining off-diagonals produces +0.
        let signed = matrix_words([0x8000_0000,0,0x8000_0000,b(1.)], Policy::Mxcsr1f80).unwrap();
        assert_eq!(signed,[b(1.),0x8000_0000,0,0,b(1.),0x8000_0000,0,0,b(1.)]);
        // All four components exactly 1/2: products are exactly 1/4;
        // diagonals subtract 2*(1/2) from 1, and paired differences are
        // 1/4 - 1/4. All those exact cancellations give +0 in this mode.
        let half = matrix_words([b(0.5);4], Policy::Mxcsr1f80).unwrap();
        assert_eq!(half,[0,b(1.),0,0,0,b(1.),b(1.),0,0]);
    }
    #[test]
    fn nonnormalized_input_is_not_normalized() {
        assert_eq!(matrix_words([b(1.),b(0.),b(0.),b(1.)],Policy::Mxcsr1f80).unwrap(),
                   [b(1.),b(0.),b(0.),b(0.),b(-1.),b(2.),b(0.),b(-2.),b(-1.)]);
    }
    #[test]
    fn binary32_intermediate_rounding_is_observable() {
        let actual=matrix_words([0x3f00_0001,0x3eaa_aaab,0x3f35_04f3,0x3f1e_3779],Policy::Mxcsr1f80).unwrap();
        assert_eq!(actual,[0xbe63_8e38,0x3f9a_8af2,0x3e97_1546,0xbf0a_6b39,
                           0xbf00_0000,0x3f8b_72b9,0x3f8f_3fa2,0xbe16_260a,0x3e8e_38e0]);
        // A double-precision reassociation of slot 0 rounds to BE638E37.
        assert_ne!(actual[0],0xbe63_8e37);
    }
    #[test]
    fn rejects_nonfinite_input_and_overflow_without_output() {
        assert_eq!(matrix_words([f32::NAN.to_bits(),0,0,0],Policy::Mxcsr1f80),Err(Error::NonFiniteInput));
        assert_eq!(matrix_words([f32::MAX.to_bits(),0,0,0],Policy::Mxcsr1f80),Err(Error::NonFiniteIntermediate));
    }
    #[test]
    fn finite_subnormal_inputs_are_not_silently_flushed() {
        let q=[1,0,0,b(1.)];
        let out=matrix_words(q,Policy::Mxcsr1f80).unwrap();
        assert_eq!(out[5],2); // 2 * (1 * smallest positive subnormal)
        assert_eq!(out[7],0x8000_0002);
    }
}
