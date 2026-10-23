//! Original Cell property-curve arithmetic (`E837F0` / `1041E40`).
//!
//! Points are the source Vector2 property values, supplied explicitly. The
//! property manager, capability-cache construction and actor update are separate
//! callers. This function preserves the native zero default, sequential search,
//! epsilon-adjusted x87 ratio and separate binary32 output operations.

use crate::native_x87::{Context, Extended, X87Error};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellCapabilityCurveError {
    TooManyPoints,
    NonFiniteInput,
    NonFiniteIntermediate,
    X87(X87Error),
}
impl From<X87Error> for CellCapabilityCurveError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}
impl fmt::Display for CellCapabilityCurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell capability curve: {self:?}")
    }
}
impl std::error::Error for CellCapabilityCurveError {}

fn input(value: f32) -> Result<f32, CellCapabilityCurveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CellCapabilityCurveError::NonFiniteInput)
    }
}
fn intermediate(value: f32) -> Result<f32, CellCapabilityCurveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CellCapabilityCurveError::NonFiniteIntermediate)
    }
}

/// Evaluate a supplied native Vector2 property under explicit x87 control.
///
/// Missing/empty points return literal positive zero without reading `query`.
/// Source manager lookup is not performed here. Scalar SSE operations require
/// nearest rounding with subnormals preserved (MXCSR control `1F80`). Only point
/// components reached by native search/evaluation are required to be finite.
///
/// The first-point test is strict, and the right-point test is inclusive. Exact
/// final x therefore still interpolates through the native denominator epsilon;
/// replacing this with equality-based endpoint copying changes original bits.
/// Duplicate or nonmonotonic x values retain source sequential-search behavior.
pub fn sample_capability_curve(
    points: Option<&[[f32; 2]]>,
    query: f32,
    context: Context,
) -> Result<f32, CellCapabilityCurveError> {
    let Some(points) = points.filter(|points| !points.is_empty()) else {
        return Ok(0.);
    };
    if points.len() > i32::MAX as usize {
        return Err(CellCapabilityCurveError::TooManyPoints);
    }
    let query = input(query)?;
    let first_x = input(points[0][0])?;
    if first_x > query {
        return input(points[0][1]);
    }
    for right in 1..points.len() {
        let right_x = input(points[right][0])?;
        if query <= right_x {
            let [left_x, left_y] = points[right - 1];
            let left_x = Extended::from_f32(input(left_x)?)?;
            let left_y = input(left_y)?;
            let right_y = input(points[right][1])?;
            let numerator = context.sub(Extended::from_f32(query)?, left_x)?;
            let denominator = context.add(
                context.sub(Extended::from_f32(right_x)?, left_x)?,
                Extended::from_f32(f32::from_bits(0x3780_0000))?,
            )?;
            let fraction = intermediate(context.store_f32(context.div(numerator, denominator)?)?)?;
            // E83923 / E83927 / E8392D are three scalar SSE operations.
            let difference = intermediate(right_y - left_y)?;
            let product = intermediate(difference * fraction)?;
            return intermediate(product + left_y);
        }
    }
    input(points.last().unwrap()[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn context() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }

    #[test]
    fn exact_right_endpoint_retains_native_epsilon_and_first_uses_interpolation() {
        let points = [[0., -0.], [1., 1.]];
        assert_eq!(
            sample_capability_curve(Some(&points), -1., context())
                .unwrap()
                .to_bits(),
            0x8000_0000
        );
        assert_eq!(
            sample_capability_curve(Some(&points), 0., context())
                .unwrap()
                .to_bits(),
            0
        );
        assert_eq!(
            sample_capability_curve(Some(&points), 1., context())
                .unwrap()
                .to_bits(),
            0x3f7f_ff00
        );
        assert_eq!(
            sample_capability_curve(Some(&points), 2., context()).unwrap(),
            1.
        );
    }

    #[test]
    fn missing_and_single_point_domains_do_not_read_unused_values() {
        assert_eq!(
            sample_capability_curve(None, f32::NAN, context())
                .unwrap()
                .to_bits(),
            0
        );
        assert_eq!(
            sample_capability_curve(Some(&[]), f32::NAN, context())
                .unwrap()
                .to_bits(),
            0
        );
        assert_eq!(
            sample_capability_curve(Some(&[[0., -0.]]), 0., context())
                .unwrap()
                .to_bits(),
            0x8000_0000
        );
        let points = [[0., 2.], [1., 4.], [f32::NAN, f32::NAN]];
        assert_eq!(
            sample_capability_curve(Some(&points), -1., context()).unwrap(),
            2.
        );
        assert!(sample_capability_curve(Some(&points), 0.5, context()).is_ok());
        assert_eq!(
            sample_capability_curve(Some(&points), 2., context()),
            Err(CellCapabilityCurveError::NonFiniteInput)
        );
        let duplicated = [[1., 2.], [1., 4.], [2., 6.]];
        assert_eq!(
            sample_capability_curve(Some(&duplicated), 1., context()).unwrap(),
            2.
        );
    }
}
