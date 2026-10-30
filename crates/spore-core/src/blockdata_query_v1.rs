//! Bounded blockdata query, derived from 414FC0/4151C0/4152A0/413C50.
//! Source-guided evaluation, not yet integrated into a gameplay system.
//! Supports finite inputs and finite intermediate results; no native exceptional
//! float, FP control-state, runtime selection, or gameplay equivalence is claimed.
use crate::blockdata::{NativeSamples, NativeSamplesError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryError {
    Sample(NativeSamplesError),
    NonFiniteInput,
    NonFiniteResult,
    IndexOverflow,
}

impl From<NativeSamplesError> for QueryError {
    fn from(value: NativeSamplesError) -> Self { Self::Sample(value) }
}

/// Evaluate the raw buffer query (414FC0), excluding the lookup wrapper's
/// first-component validity test and default replacement. Missing coordinates
/// are zero; extra coordinates are ignored, including any nonfinite extras.
/// Even with zero dimensions the header triples must be finite. Nonfinite
/// decoded leaves, including those caused by a nonfinite scale, are reported
/// as NonFiniteResult. A calculated out-of-range index is an error here; the
/// native recursive fallback returns default triples instead.
pub fn query(samples: &NativeSamples<'_>, coordinates: &[f32]) -> Result<([f32; 3], [f32; 3]), QueryError> {
    let (first, second) = samples.header_triples();
    if first.iter().chain(second.iter()).any(|x| !x.is_finite()) {
        return Err(QueryError::NonFiniteInput);
    }
    let dims = samples.dimensions();
    if dims.is_empty() { return Ok((first, second)); }
    // NativeSamples validates the resource shape but does not expose its scale.
    // Checking every decoded leaf below also rejects any nonfinite scale result.
    let mut lower = [0usize; 8];
    let mut upper = [0usize; 8];
    let mut weights = [0f32; 8];
    let mut stride = 1usize;
    for i in (0..dims.len()).rev() {
        let coordinate = coordinates.get(i).copied().unwrap_or(0.0);
        if !coordinate.is_finite() { return Err(QueryError::NonFiniteInput); }
        let x = coordinate.max(0.0).min(1.0);
        let n = usize::from(dims[i]) - 1;
        if n > 0 {
            let scaled = (n as f32) * x;
            let index = scaled as usize; // finite, nonnegative, at most 254
            let weight = scaled - (index as f32);
            lower[i] = index.checked_mul(stride).ok_or(QueryError::IndexOverflow)?;
            upper[i] = lower[i];
            weights[i] = weight;
            if weight > 0.0 && index < n {
                upper[i] = lower[i].checked_add(stride).ok_or(QueryError::IndexOverflow)?;
            }
        }
        stride = stride.checked_mul(usize::from(dims[i])).ok_or(QueryError::IndexOverflow)?;
    }
    evaluate(samples, &lower, &upper, &weights, 0, 0)
}

fn evaluate(samples: &NativeSamples<'_>, lower: &[usize; 8], upper: &[usize; 8],
            weights: &[f32; 8], dimension: usize, index: usize)
    -> Result<([f32; 3], [f32; 3]), QueryError> {
    if index >= samples.sample_count() { return Err(QueryError::IndexOverflow); }
    if dimension == samples.dimensions().len() {
        let value = samples.sample(index)?;
        if value.0.iter().chain(value.1.iter()).any(|x| !x.is_finite()) {
            return Err(QueryError::NonFiniteResult);
        }
        return Ok(value);
    }
    let left_index = index.checked_add(lower[dimension]).ok_or(QueryError::IndexOverflow)?;
    let left = evaluate(samples, lower, upper, weights, dimension + 1, left_index)?;
    if lower[dimension] == upper[dimension] { return Ok(left); }
    let right_index = index.checked_add(upper[dimension]).ok_or(QueryError::IndexOverflow)?;
    let right = evaluate(samples, lower, upper, weights, dimension + 1, right_index)?;
    let mut result = ([0.0; 3], [0.0; 3]);
    for axis in 0..3 {
        // Preserve subtraction, multiplication, and addition order. Exact
        // equivalence for all compiler targets and FP environments is unproven.
        let d0 = right.0[axis] - left.0[axis];
        let m0 = d0 * weights[dimension];
        result.0[axis] = left.0[axis] + m0;
        let d1 = right.1[axis] - left.1[axis];
        let m1 = d1 * weights[dimension];
        result.1[axis] = left.1[axis] + m1;
        if !d0.is_finite() || !m0.is_finite() || !result.0[axis].is_finite()
            || !d1.is_finite() || !m1.is_finite() || !result.1[axis].is_finite() {
            return Err(QueryError::NonFiniteResult);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(dims: &[u8]) -> Vec<u8> {
        let count = if dims.is_empty() { 0 } else { dims.iter().map(|&d| usize::from(d)).product() };
        let mut data = vec![0u8; 36 + 3 * count];
        for (i, value) in [0.0f32, 0., 0., 20., 20., 20., 1.].iter().enumerate() {
            data[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        data[28..28 + dims.len()].copy_from_slice(dims);
        for i in 0..count {
            let packed = ((i as u8) << 4) | (i as u8);
            data[36 + i * 3..39 + i * 3].fill(packed);
        }
        data
    }

    #[test]
    fn endpoints_and_ordered_multidimensional_blend() {
        let data = grid(&[2, 3]);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[0., 0.]).unwrap(), ([0.; 3], [20.; 3]));
        assert_eq!(query(&s, &[1., 1.]).unwrap(), ([-5.; 3], [25.; 3]));
        // Last dimension varies fastest: inner halfway gives 0.5 and 3.5;
        // outer halfway produces sample value 2.0.
        assert_eq!(query(&s, &[0.5, 0.25]).unwrap(), ([-2.; 3], [22.; 3]));
    }

    #[test]
    fn binary32_rounding_and_recursive_order_are_observable() {
        let mut data = grid(&[2, 2]);
        let base = f32::from_bits(0xc408a5a6);
        let scale = f32::from_bits(0x4259d573);
        for axis in 0..3 { data[axis * 4..axis * 4 + 4].copy_from_slice(&base.to_le_bytes()); }
        data[24..28].copy_from_slice(&scale.to_le_bytes());
        for (i, high) in [2u8, 4, 7, 6].iter().enumerate() {
            data[36 + 3 * i..39 + 3 * i].fill(*high << 4);
        }
        let s = NativeSamples::new(&data).unwrap();
        let result = query(&s, &[f32::from_bits(0x3e015ce3), f32::from_bits(0x3f346ee2)]).unwrap();
        // tools/blockdata_query_rounding_reference_v1.py independently rounds
        // each binary32 operation; swapping dimension order gives c43c07e9.
        assert_eq!(result.0.map(f32::to_bits), [0xc43c07e8; 3]);
    }

    #[test]
    fn zero_dimension_requires_finite_header() {
        let mut data = grid(&[]);
        data[0..4].copy_from_slice(&f32::NAN.to_le_bytes());
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[]), Err(QueryError::NonFiniteInput));
    }

    #[test]
    fn three_dimensions_use_last_axis_as_fastest_stride() {
        let data = grid(&[2, 2, 2]);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[1., 0.5, 0.5]), Ok(([-5.5; 3], [25.5; 3])));
        assert_eq!(query(&s, &[0., 0., 1.]), Ok(([-1.; 3], [21.; 3])));
    }

    #[test]
    fn clamp_missing_extra_and_singleton_dimension() {
        let data = grid(&[2, 3]);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[-2., 5.]), query(&s, &[0., 1.]));
        assert_eq!(query(&s, &[1.]), query(&s, &[1., 0.]));
        assert_eq!(query(&s, &[1., 0., f32::NAN]), query(&s, &[1., 0.]));
        assert_eq!(query(&s, &[f32::NAN]), Err(QueryError::NonFiniteInput));
        let singleton = grid(&[1, 3]);
        let one = NativeSamples::new(&singleton).unwrap();
        assert_eq!(query(&one, &[0., 0.5]), query(&one, &[1., 0.5]));
    }

    #[test]
    fn no_dimensions_return_header_without_accessing_samples() {
        let data = grid(&[]);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[0.7]), Ok(([0.; 3], [20.; 3])));
        assert_eq!(s.sample_count(), 0);
    }

    #[test]
    fn nonfinite_leaf_is_rejected() {
        let mut data = grid(&[2]);
        data[24..28].copy_from_slice(&f32::INFINITY.to_le_bytes());
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[1.]), Err(QueryError::NonFiniteResult));
    }

    #[test]
    fn finite_inputs_with_overflowing_decoded_branch_are_rejected() {
        let mut data = grid(&[2]);
        data[24..28].copy_from_slice(&(f32::MAX / 2.0).to_le_bytes());
        // The upper branch multiplies a finite scale by 15, overflowing f32.
        data[39..42].fill(0xf0);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[0.5]), Err(QueryError::NonFiniteResult));
        // The exact lower endpoint does not visit the overflowing branch.
        assert_eq!(query(&s, &[0.0]), Ok(([0.; 3], [20.; 3])));
    }

    #[test]
    fn eight_singleton_dimensions_do_not_access_an_upper_branch() {
        let data = grid(&[1; 8]);
        let s = NativeSamples::new(&data).unwrap();
        assert_eq!(query(&s, &[1.0; 8]), Ok(([0.; 3], [20.; 3])));
        let mut coordinates = [0.0; 8];
        coordinates[7] = f32::NAN;
        assert_eq!(query(&s, &coordinates), Err(QueryError::NonFiniteInput));
    }
}
