//! Controlled Cell progress selector and cursor-x arithmetic. No runtime state.
//!
//! Models the five constructor words `[2; 5]` at E283A0 and the SSE cursor
//! arithmetic at E29C5C. Caller supplies progress, x87 context, and the
//! selected control rectangle. Native FP status and original execution are
//! not modeled; only nearest-even, gradual-underflow SSE is admitted.
use crate::native_x87::{Context, Extended, X87Error};
use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection { pub segment: u32, pub bucket: u32, pub fraction: f32 }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error { Input, X87(X87Error), Sse(SseError) }
impl From<X87Error> for Error { fn from(e: X87Error) -> Self { Self::X87(e) } }
impl From<SseError> for Error { fn from(e: SseError) -> Self { Self::Sse(e) } }

/// The fixed words are source constructor data, not a selected live widget.
/// Negative, nonfinite, and above-one inputs are excluded to avoid the native
/// routine's unchecked indexed accesses. This is a model admission boundary.
pub fn select(progress: f32, x87: Context) -> Result<Selection, Error> {
    if !progress.is_finite() || !(0.0..=1.0).contains(&progress) { return Err(Error::Input); }
    let total = Extended::from_i32(10);
    let reciprocal = x87.div(Extended::from_i32(1), total)?;
    let scaled = x87.store_f32(x87.mul(total, Extended::from_f32(progress)?)?)?;
    if !scaled.is_finite() || !(0.0..=10.0).contains(&scaled) { return Err(Error::Input); }
    // For this bounded nonnegative range, CVTSS2SI under nearest-even,
    // CVTSI2SS, UCOMISS and the conditional decrement equal floor.
    let mut bucket = scaled.floor() as u32;
    if bucket == 10 { bucket = 9; }
    let mut segment = 0;
    let mut remaining = progress;
    while bucket >= 2 {
        bucket -= 2;
        let width = x87.mul(Extended::from_i32(2), reciprocal)?;
        remaining = x87.store_f32(x87.sub(Extended::from_f32(remaining)?, width)?)?;
        segment += 1;
    }
    let width = x87.mul(Extended::from_i32(2), reciprocal)?;
    let fraction = x87.store_f32(x87.div(Extended::from_f32(remaining)?, width)?)?;
    if segment >= 5 || !fraction.is_finite() { return Err(Error::Input); }
    Ok(Selection { segment, bucket, fraction })
}

/// Computes `left + (right-left)*fraction` in the source's SSE order.
/// The rectangle and fraction are supplied; no animation, clipping, or
/// visibility behavior is implied. A finite result is a model admission.
pub fn cursor_x(left: f32, right: f32, fraction: f32, policy: Policy) -> Result<f32, Error> {
    if !left.is_finite() || !right.is_finite() || !fraction.is_finite() { return Err(Error::Input); }
    native_sse::scoped(policy, || {
        let width = std::hint::black_box(right) - std::hint::black_box(left);
        let offset = std::hint::black_box(width) * std::hint::black_box(fraction);
        let result = std::hint::black_box(offset) + std::hint::black_box(left);
        if result.is_finite() { Ok(result) } else { Err(Error::Input) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context { Context::from_control_word(0x037f).unwrap() }
    #[test]
    fn source_widths_select_endpoints_and_interiors() {
        for (p, s, b, f) in [(0.0,0,0,0.0), (0.1,0,1,0.5), (0.3,1,1,0.5), (0.5,2,1,0.5), (0.9,4,1,0.5), (1.0,4,1,1.0)] {
            let result = select(p, context()).unwrap();
            assert_eq!((result.segment, result.bucket), (s,b), "{p}");
            assert!((result.fraction-f).abs() < 0.000001, "{p}: {result:?}");
        }
    }
    #[test]
    fn adjacent_inputs_and_invalid_domain_are_bounded() {
        for boundary in [0.2f32,0.4,0.6,0.8] {
            for bits in [boundary.to_bits()-1, boundary.to_bits(), boundary.to_bits()+1] {
                let r=select(f32::from_bits(bits),context()).unwrap();
                assert!(r.segment<5 && r.bucket<2 && r.fraction.is_finite());
            }
        }
        for p in [-0.1,1.1,f32::NAN,f32::INFINITY] { assert_eq!(select(p,context()),Err(Error::Input)); }
    }
    #[test]
    fn cursor_uses_explicit_rectangle_and_rejects_overflow() {
        assert_eq!(cursor_x(20.,100.,0.25,Policy::Mxcsr1f80),Ok(40.));
        assert_eq!(cursor_x(-f32::MAX,f32::MAX,0.5,Policy::Mxcsr1f80),Err(Error::Input));
    }
}
