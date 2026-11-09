//! Bounded `E4DD50` aggregation of caller-admitted post-`E4E730` bins.
//!
//! Input records are snapshots, not a native environment or player. This does
//! not construct bins, model callbacks/aliasing, or establish native execution.
//! It rejects domains the native routine does not explicitly reject and returns
//! a new result only after every bin succeeds. FP sticky flags are not preserved
//! as source outputs. PE: SporeApp.exe SHA-256
//! 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a.

use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::{Context, Extended, X87Error};

pub const BIN_COUNT: usize = 300;
pub const MAX_RECORDS: usize = 8;

/// The two fields consumed from one 32-byte scratch record, in source order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Record {
    pub vector: [f32; 3],
    pub scalar: f32,
}

/// Caller supplies a stable snapshot from the point after `E4E730` returns.
/// Exact slice length must match the nonnegative native count (0 through 8).
#[derive(Debug, Clone, Copy)]
pub struct Bin<'a> {
    pub count: i32,
    pub records: &'a [Record],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    pub vector: [f32; 3],
    pub magnitude: f32,
    pub mean_reciprocal: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidBinCount,
    InvalidRecordCount,
    NonFiniteInput,
    ZeroDenominator,
    NonFiniteIntermediate,
    Sse(SseError),
    X87(X87Error),
}
impl From<SseError> for Error { fn from(e: SseError) -> Self { Self::Sse(e) } }
impl From<X87Error> for Error { fn from(e: X87Error) -> Self { Self::X87(e) } }
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "bounded Cell flow aggregation: {self:?}") }
}
impl std::error::Error for Error {}

// Volatile scalar SSE preserves each source operation and its operand order;
// it is not marked pure/nomem because the scoped MXCSR is an implicit input.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut a: f32, b: f32) -> f32 {
    unsafe { std::arch::asm!("addss {a}, {b}", a = inout(xmm_reg) a, b = in(xmm_reg) b, options(nostack, preserves_flags)); }
    a
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut a: f32, b: f32) -> f32 {
    unsafe { std::arch::asm!("mulss {a}, {b}", a = inout(xmm_reg) a, b = in(xmm_reg) b, options(nostack, preserves_flags)); }
    a
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn div(mut a: f32, b: f32) -> f32 {
    unsafe { std::arch::asm!("divss {a}, {b}", a = inout(xmm_reg) a, b = in(xmm_reg) b, options(nostack, preserves_flags)); }
    a
}

/// Compute all 300 outputs under the supplied FP policies. Negative counts,
/// more than eight records, nonfinite reached values and zero divisors are
/// rejected by this checked model; native behavior is not asserted for them.
/// Inputs must represent valid, stable, nonaliasing post-callback snapshots.
pub fn aggregate(bins: &[Bin<'_>], policy: Policy, x87: Context) -> Result<[Output; BIN_COUNT], Error> {
    if bins.len() != BIN_COUNT { return Err(Error::InvalidBinCount); }
    for bin in bins {
        if !(0..=MAX_RECORDS as i32).contains(&bin.count) || bin.records.len() != bin.count as usize {
            return Err(Error::InvalidRecordCount);
        }
        for r in bin.records {
            if !r.scalar.is_finite() || !r.vector.iter().all(|v| v.is_finite()) { return Err(Error::NonFiniteInput); }
            if r.scalar == 0.0 { return Err(Error::ZeroDenominator); }
        }
    }
    native_sse::scoped(policy, || {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let mut result = [Output { vector: [0.0; 3], magnitude: 0.0, mean_reciprocal: 0.0 }; BIN_COUNT];
            for (output, bin) in result.iter_mut().zip(bins) {
                if bin.count == 0 { continue; }
                let mut sum = [0.0f32; 3];
                let mut reciprocal_sum = 0.0f32;
                for r in bin.records {
                    // E4DDE0..E4DF40 loads each record as ADDSS destination.
                    sum[0] = add(r.vector[0], sum[0]);
                    sum[1] = add(r.vector[1], sum[1]);
                    sum[2] = add(r.vector[2], sum[2]);
                    let reciprocal = div(1.0, r.scalar);
                    reciprocal_sum = add(reciprocal, reciprocal_sum);
                    if !reciprocal.is_finite() || !reciprocal_sum.is_finite() || !sum.iter().all(|v| v.is_finite()) {
                        return Err(Error::NonFiniteIntermediate);
                    }
                }
                // Counts 1..8 convert exactly to binary32 under every MXCSR mode.
                let inv_count = div(1.0, bin.count as f32);
                // E4DF51..73: X and Y use the sum as MULSS destination;
                // Z uses the count reciprocal as destination.
                let vector = [mul(sum[0], inv_count), mul(sum[1], inv_count), mul(inv_count, sum[2])];
                if !inv_count.is_finite() || !vector.iter().all(|v| v.is_finite()) { return Err(Error::NonFiniteIntermediate); }
                // E4DF78..E4DF92 computes (z*z + x*x) + y*y in x87.
                let x = Extended::from_f32(vector[0])?;
                let y = Extended::from_f32(vector[1])?;
                let z = Extended::from_f32(vector[2])?;
                let x2 = x87.mul(x, x)?;
                let z2 = x87.mul(z, z)?;
                let zx = x87.add(z2, x2)?;
                let y2 = x87.mul(y, y)?;
                let length2 = x87.add(zx, y2)?;
                let magnitude = x87.store_f32(x87.sqrt(length2)?)?;
                let mean_reciprocal = mul(inv_count, reciprocal_sum);
                if !magnitude.is_finite() || !mean_reciprocal.is_finite() { return Err(Error::NonFiniteIntermediate); }
                *output = Output { vector, magnitude, mean_reciprocal };
            }
            Ok(result)
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        { let _ = x87; Err(Error::Sse(SseError::UnsupportedArchitecture)) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn context() -> Context { Context { precision: Precision::Bits64, rounding: Rounding::NearestEven } }
    fn empty<'a>() -> Vec<Bin<'a>> { vec![Bin { count: 0, records: &[] }; BIN_COUNT] }
    #[test]
    fn empty_and_ordered_bins() {
        let records = [Record { vector: [16_777_216., 3., 4.], scalar: 1. },
            Record { vector: [-16_777_216., 0., 0.], scalar: 2. },
            Record { vector: [1., 0., 0.], scalar: 4. }];
        let mut bins = empty(); bins[2] = Bin { count: 3, records: &records };
        let out = aggregate(&bins, Policy::Mxcsr1f80, context()).unwrap();
        assert_eq!(out[0].vector.map(f32::to_bits), [0; 3]);
        assert_eq!(out[0].magnitude.to_bits(), 0);
        assert_eq!(out[2].vector[0].to_bits(), (1f32 / 3.).to_bits());
        assert_eq!(out[2].vector[1], 1.0);
        assert_eq!(out[2].mean_reciprocal.to_bits(), (1.75f32 * (1f32 / 3.)).to_bits());
        let reversed = [records[0], records[2], records[1]];
        bins[2] = Bin { count: 3, records: &reversed };
        let reordered = aggregate(&bins, Policy::Mxcsr1f80, context()).unwrap();
        assert_eq!(reordered[2].vector[0].to_bits(), 0);
    }
    #[test]
    fn one_record_has_x87_magnitude_and_signed_reciprocal() {
        let record = [Record { vector: [3., 4., 0.], scalar: -2. }];
        let mut bins = empty(); bins[0] = Bin { count: 1, records: &record };
        let out = aggregate(&bins, Policy::Mxcsr1f80, context()).unwrap();
        assert_eq!(out[0].vector, [3., 4., 0.]);
        assert_eq!(out[0].magnitude, 5.);
        assert_eq!(out[0].mean_reciprocal, -0.5);
    }
    #[test]
    fn reciprocal_then_multiply_differs_from_direct_division() {
        let records = [Record { vector: [f32::from_bits(0x4179eb76), 0., 0.], scalar: 1. },
            Record { vector: [0.; 3], scalar: 1. }, Record { vector: [0.; 3], scalar: 1. }];
        let mut bins = empty(); bins[0] = Bin { count: 3, records: &records };
        let out = aggregate(&bins, Policy::Mxcsr1f80, context()).unwrap();
        assert_eq!(out[0].vector[0].to_bits(), 0x40a69cfa);
        assert_ne!(out[0].vector[0].to_bits(), (records[0].vector[0] / 3.).to_bits());
    }
    #[test]
    fn invalid_inputs_return_no_partial_output() {
        let mut bins = empty();
        assert_eq!(aggregate(&bins[..299], Policy::Mxcsr1f80, context()), Err(Error::InvalidBinCount));
        bins[0] = Bin { count: -1, records: &[] };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::InvalidRecordCount));
        bins[0] = Bin { count: 9, records: &[] };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::InvalidRecordCount));
        bins[0] = Bin { count: 1, records: &[] };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::InvalidRecordCount));
        let bad = [Record { vector: [1., 2., 3.], scalar: -0.0 }];
        bins[0] = Bin { count: 1, records: &bad };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::ZeroDenominator));
        let bad = [Record { vector: [f32::NAN, 2., 3.], scalar: 1. }];
        bins[0] = Bin { count: 1, records: &bad };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::NonFiniteInput));
        let tiny = [Record { vector: [0.; 3], scalar: f32::from_bits(1) }];
        bins[0] = Bin { count: 1, records: &tiny };
        assert_eq!(aggregate(&bins, Policy::Mxcsr1f80, context()), Err(Error::NonFiniteIntermediate));
    }
}
