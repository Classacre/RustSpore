//! Bounded conditional E74B98..E74C0D null-orientation value slice.
//! Caller admits definition byte +AC != 1, null orientation pointer,
//! successful prior allocation and supplies post-size RNG state. Neither this
//! admission nor explicit globals establish an actor, retail RNG/FP state,
//! original CPU, package choice, side effects or source-byte parity. The
//! result is host-processor dependent. No matrix or allocation is modeled.
//!
//! Checked, pure interface: errors return no advanced state even where native
//! has already advanced RNG; excludes native exceptional and status behavior.
use crate::{cell_request_rng_v1::Rng,
    native_sse::{self, Policy, SseError},
    native_x87::{Context, Extended, Precision, X87Error}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { UnsupportedInput, UnsupportedControl, UnsupportedRng, X87(X87Error), Sse(SseError) }
impl From<X87Error> for Error { fn from(e: X87Error) -> Self { Self::X87(e) } }
impl From<SseError> for Error { fn from(e: SseError) -> Self { Self::Sse(e) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultV1 {
    pub rng_state_after: u32,
    pub angle_extended_le: [u8; 10],
    pub sine_bits: u32,
    pub cosine_bits: u32,
    pub quaternion_bits: [u32; 4],
}

/// Inputs are explicit binary32 bit patterns. Positive normal angle global;
/// finite normal or signed-zero axes. The C2-checked helpers reject |angle|>4.
/// SSE uses only the explicitly admitted MXCSR1F80 policy. Actual native flag
/// history, exception paths, intervening callbacks and resource lifetimes are
/// outside this model. No original game control word is inferred.
pub fn null_orientation(post_size_rng: u32, angle_global_bits: u32,
    axis_bits: [u32; 3], control_word: u16, sse_policy: Policy) -> Result<ResultV1, Error> {
    let global = f32::from_bits(angle_global_bits);
    if !global.is_normal() || global <= 0.0 || axis_bits.iter().any(|&bits| {
        let v = f32::from_bits(bits);
        !v.is_finite() || (v != 0.0 && !v.is_normal())
    }) { return Err(Error::UnsupportedInput); }
    if control_word & 0x3f != 0x3f { return Err(Error::UnsupportedControl); }
    let x87 = Context::from_control_word(control_word).map_err(|_| Error::UnsupportedControl)?;
    if x87.precision == Precision::Bits24 { return Err(Error::UnsupportedControl); }
    let mut rng = Rng::new(post_size_rng).map_err(|_| Error::UnsupportedRng)?;
    let fraction = rng.next_fraction();
    // The RNG helper omits the sign of exact zero, which can affect the output.
    if fraction.numerator == 0 { return Err(Error::UnsupportedRng); }
    let angle = x87.mul(Extended::from_f64(fraction.as_f64())?, Extended::from_f32(global)?)?;
    let angle = x87.mul(angle, Extended::from_f32(0.5)?)?;
    let sin = x87.sin_magnitude_at_most_four(angle)?;
    let sine_bits = x87.store_f32(sin)?.to_bits();
    // Source FCOS consumes the retained angle, not a binary32 approximation.
    let cos = x87.cos_magnitude_at_most_four(angle)?;
    // Source commits cosine after MULSS; preserve that operation order.
    let xyz = native_sse::scoped(sse_policy, || -> Result<[u32; 3], Error> {
        Ok([multiply(sine_bits, axis_bits[0])?, multiply(sine_bits, axis_bits[1])?,
            multiply(sine_bits, axis_bits[2])?])
    })?;
    let cosine_bits = x87.store_f32(cos)?.to_bits();
    if xyz.iter().any(|bits| !f32::from_bits(*bits).is_finite()) { return Err(Error::UnsupportedInput); }
    Ok(ResultV1 { rng_state_after: rng.state(), angle_extended_le: angle.to_le_bytes(), sine_bits,
        cosine_bits, quaternion_bits: [xyz[0], xyz[1], xyz[2], cosine_bits] })
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn multiply(a: u32, b: u32) -> Result<u32, Error> {
    let result: u32;
    // SAFETY: called only inside scoped SSE2-checked, masked MXCSR policy.
    unsafe { std::arch::asm!("movd {x}, {a:e}", "movd {y}, {b:e}",
        "mulss {x}, {y}", "movd {result:e}, {x}",
        x = out(xmm_reg) _, y = out(xmm_reg) _, a = in(reg) a, b = in(reg) b,
        result = lateout(reg) result, options(nostack, preserves_flags)); }
    Ok(result)
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn multiply(_: u32, _: u32) -> Result<u32, Error> { Err(Error::Sse(SseError::UnsupportedArchitecture)) }

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;
    const P: Policy = Policy::Mxcsr1f80;
    #[test]
    fn retains_angle_and_consumes_exactly_one_draw() {
        let v = null_orientation(1, 1f32.to_bits(), [1f32.to_bits(), (-1f32).to_bits(), 0], 0x027f, P).unwrap();
        assert_eq!(v.rng_state_after, 0x278d_de6d);
        assert_eq!(v.quaternion_bits[0], v.sine_bits);
        assert_eq!(v.quaternion_bits[1], v.sine_bits ^ 0x8000_0000);
        assert_eq!(v.quaternion_bits[2], 0);
        assert_eq!(v.quaternion_bits[3], v.cosine_bits);
        assert!(f32::from_bits(v.sine_bits) > 0.0);
        assert!(f32::from_bits(v.cosine_bits) > 0.0);
        let ctx = Context::from_control_word(0x027f).unwrap();
        let fraction = Rng::new(1).unwrap().next_fraction();
        let expected = ctx.mul(Extended::from_f64(fraction.as_f64()).unwrap(), Extended::from_f32(0.5).unwrap()).unwrap();
        assert_eq!(v.angle_extended_le, expected.to_le_bytes());
        assert_eq!(v.cosine_bits, ctx.store_f32(ctx.cos_magnitude_at_most_four(expected).unwrap()).unwrap().to_bits());
    }
    #[test]
    fn signed_zero_axis_is_preserved_by_mulss() {
        let v = null_orientation(1, 1f32.to_bits(), [0, 0x8000_0000, 1f32.to_bits()], 0x027f, P).unwrap();
        assert_eq!(v.quaternion_bits[0], 0);
        assert_eq!(v.quaternion_bits[1], 0x8000_0000);
        assert_eq!(v.quaternion_bits[2], v.sine_bits);
    }
    #[test]
    fn rejects_unknown_rng_control_and_nonfinite_or_subnormal_inputs() {
        let good = [0, 0, 1f32.to_bits()];
        assert_eq!(null_orientation(0, 1f32.to_bits(), good, 0x027f, P), Err(Error::UnsupportedRng));
        assert_eq!(null_orientation(0x8000_0000, 1f32.to_bits(), good, 0x027f, P), Err(Error::UnsupportedRng));
        for cw in [0x007f, 0x017f, 0x027e] {
            assert_eq!(null_orientation(1, 1f32.to_bits(), good, cw, P), Err(Error::UnsupportedControl));
        }
        assert_eq!(null_orientation(1, f32::NAN.to_bits(), good, 0x027f, P), Err(Error::UnsupportedInput));
        assert_eq!(null_orientation(1, 1f32.to_bits(), [0, 1, 0], 0x027f, P), Err(Error::UnsupportedInput));
    }
    #[test]
    fn oversized_angle_is_rejected_before_trigonometry() {
        let out = null_orientation(1, 100f32.to_bits(), [0,0,0], 0x027f, P);
        assert_eq!(out, Err(Error::X87(X87Error::SinArgumentOutsideBound)));
    }
}
