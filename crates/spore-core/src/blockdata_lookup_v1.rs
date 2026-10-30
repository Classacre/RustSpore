//! Bounded result projection of source `414DC0`, with explicit supplied lookup state.
//!
//! The caller supplies presence and, separately, an optional validated view;
//! this does not resolve resource
//! keys, prove actor identity or implement native map ownership. The query
//! helper's defensive errors are returned as errors, not native false results.
//! Outputs model distinct, non-null output buffers when values are requested.
use crate::{blockdata::NativeSamples, blockdata_query_v1::{self, QueryError}};

pub enum Lookup<'a> {
    Missing,
    Found(Option<&'a NativeSamples<'a>>),
}

pub enum Request<'a> {
    PresenceOnly,
    Values(&'a [f32]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    UnvalidatedResource,
    Query(QueryError),
}

impl From<QueryError> for ProjectionError {
    fn from(error: QueryError) -> Self { Self::Query(error) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outcome {
    /// The source wrapper's boolean return, within the admitted query domain.
    pub accepted: bool,
    /// None means no output writes were requested.
    pub values: Option<([f32; 3], [f32; 3])>,
}

/// Return source fallback constants with their observed binary32 bit patterns.
fn fallback() -> ([f32; 3], [f32; 3]) {
    ([f32::from_bits(0xbdcc_cccd); 3], [f32::from_bits(0x3dcc_cccd); 3])
}

/// Compose an explicitly supplied lookup result with the bounded query.
///
/// Presence-only does not require validation, evaluate coordinates or read a resource.
/// A found entry without a validated view returns UnvalidatedResource only for
/// a values request; it is distinct from missing and source query failure.
/// For values, a missing entry returns false plus defaults; a found entry is
/// accepted only if its second triple's component zero is strictly greater
/// than the first triple's component zero. Other axes do not affect acceptance.
/// NaNs and unsupported floating-point control states are outside this projection's
/// admitted comparison domain; source COMISS also handles unordered inputs.
pub fn project(lookup: Lookup<'_>, request: Request<'_>) -> Result<Outcome, ProjectionError> {
    match (lookup, request) {
        (Lookup::Missing, Request::PresenceOnly) => Ok(Outcome { accepted: false, values: None }),
        (Lookup::Found(_), Request::PresenceOnly) => Ok(Outcome { accepted: true, values: None }),
        (Lookup::Missing, Request::Values(_)) => Ok(Outcome { accepted: false, values: Some(fallback()) }),
        (Lookup::Found(None), Request::Values(_)) => Err(ProjectionError::UnvalidatedResource),
        (Lookup::Found(Some(samples)), Request::Values(coords)) => {
            let (first, second) = blockdata_query_v1::query(samples, coords)?;
            if second[0] > first[0] {
                Ok(Outcome { accepted: true, values: Some((first, second)) })
            } else {
                Ok(Outcome { accepted: false, values: Some(fallback()) })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(values: [f32; 6]) -> [u8; 36] {
        let mut bytes = [0u8; 36];
        for (i, v) in values.into_iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn presence_does_not_consume_or_validate_the_resource() {
        let raw = header([f32::NAN, 0., 0., 0., 0., 0.]);
        let samples = NativeSamples::new(&raw).unwrap();
        assert_eq!(project(Lookup::Found(Some(&samples)), Request::PresenceOnly),
                   Ok(Outcome { accepted: true, values: None }));
        assert_eq!(project(Lookup::Missing, Request::PresenceOnly),
                   Ok(Outcome { accepted: false, values: None }));
    }

    #[test]
    fn missing_entry_with_values_returns_exact_defaults() {
        let result = project(Lookup::Missing, Request::Values(&[f32::NAN])).unwrap();
        let (a, b) = result.values.unwrap();
        assert!(!result.accepted);
        assert_eq!(a.map(f32::to_bits), [0xbdcc_cccd; 3]);
        assert_eq!(b.map(f32::to_bits), [0x3dcc_cccd; 3]);
    }

    #[test]
    fn only_first_axis_decides_acceptance() {
        let raw = header([1., 8., 9., 2., -8., -9.]);
        let samples = NativeSamples::new(&raw).unwrap();
        let result = project(Lookup::Found(Some(&samples)), Request::Values(&[])).unwrap();
        assert_eq!(result, Outcome { accepted: true, values: Some(([1., 8., 9.], [2., -8., -9.])) });
    }

    #[test]
    fn equal_and_reversed_first_axis_return_fallback() {
        for second in [1., -1.] {
            let raw = header([1., 0., 0., second, 3., 4.]);
            let samples = NativeSamples::new(&raw).unwrap();
            assert_eq!(project(Lookup::Found(Some(&samples)), Request::Values(&[])),
                       Ok(Outcome { accepted: false, values: Some(fallback()) }));
        }
    }

    #[test]
    fn defensive_query_error_is_not_reported_as_missing() {
        let raw = header([f32::NAN, 0., 0., 1., 0., 0.]);
        let samples = NativeSamples::new(&raw).unwrap();
        assert_eq!(project(Lookup::Found(Some(&samples)), Request::Values(&[])), Err(ProjectionError::Query(QueryError::NonFiniteInput)));
    }

    #[test]
    fn known_present_does_not_require_a_validated_view() {
        assert_eq!(project(Lookup::Found(None), Request::PresenceOnly),
                   Ok(Outcome { accepted: true, values: None }));
        assert_eq!(project(Lookup::Found(None), Request::Values(&[])),
                   Err(ProjectionError::UnvalidatedResource));
    }

    #[test]
    fn interpolated_grid_crosses_the_first_axis_acceptance_boundary() {
        let mut raw = vec![0u8; 42];
        for (i, value) in [0.0f32, 0., 0., 1., 1., 1., -1.].iter().enumerate() {
            raw[4 * i..4 * i + 4].copy_from_slice(&value.to_le_bytes());
        }
        raw[28] = 2; // One dimension, with two three-byte samples.
        raw[39] = 0x11; // Second sample's first-axis bounds are reversed.
        let samples = NativeSamples::new(&raw).unwrap();
        assert_eq!(project(Lookup::Found(Some(&samples)), Request::Values(&[0.25])),
                   Ok(Outcome { accepted: true, values: Some(([0.25, 0., 0.], [0.75, 1., 1.])) }));
        assert_eq!(project(Lookup::Found(Some(&samples)), Request::Values(&[0.5])),
                   Ok(Outcome { accepted: false, values: Some(fallback()) }));
        assert_eq!(project(Lookup::Found(Some(&samples)), Request::Values(&[0.75])),
                   Ok(Outcome { accepted: false, values: Some(fallback()) }));
    }

}
