//! Bounded E50670 startup bounds through the dynamic XY writes.
use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, PartialEq)]
pub enum Error { Unadmitted, NonFinite, Sse(SseError) }
impl From<SseError> for Error { fn from(value: SseError) -> Self { Self::Sse(value) } }

/// Projects the source startup vector (100,100,0) with a supplied snapshot of
/// resolver result +68. Resolver callbacks and their storage must remain stable;
/// dynamic Z, downstream effects, and live startup state are outside this cut.
pub fn project(value_68: u32, policy: Policy, admitted: bool) -> Result<[u32; 10], Error> {
    if !admitted { return Err(Error::Unadmitted); }
    let value = f32::from_bits(value_68);
    if !value.is_finite() { return Err(Error::NonFinite); }
    native_sse::scoped(policy, || {
        use std::hint::black_box as b;
        let scale = b(b(value) * b(f32::from_bits(0x3d888889))) + b(1.0);
        let dx = b(b(scale * b(20.0)) * b(0.5));
        let dy = b(b(scale * b(15.0)) * b(0.5));
        let result = [90.0f32, 92.5, 0.0, 110.0, 107.5, 0.0,
            b(100.0f32 - dx), b(100.0f32 - dy), b(dx + 100.0), b(dy + 100.0)];
        if !result.iter().all(|x| x.is_finite()) { return Err(Error::NonFinite); }
        Ok(result.map(f32::to_bits))
    })
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn controlled_bounds_and_admission() {
        let out = project(15f32.to_bits(), Policy::Mxcsr1f80, true).unwrap();
        assert_eq!(&out[6..], &[80f32.to_bits(), 85f32.to_bits(), 120f32.to_bits(), 115f32.to_bits()]);
        assert_eq!(project(0, Policy::Mxcsr1f80, false), Err(Error::Unadmitted));
        assert_eq!(project(f32::NAN.to_bits(), Policy::Mxcsr1f80, true), Err(Error::NonFinite));
    }
}
