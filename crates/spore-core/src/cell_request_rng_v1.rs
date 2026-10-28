//! Bounded model of the nonzero-state RNG at 935D40 (base executable).
//!
//! The source loads the updated word as signed, multiplies by binary32 2^-32,
//! and adds binary32 0.5 in x87. With x87 precision set to 53 or 64 bits,
//! both operations are numerically exact for every state, under any rounding
//! mode. The representation omits the sign of zero: exact cancellation can
//! produce -0 under x87 round-down, but numerator zero converts to +0 here.
//! It is not bit-identical for zero or suitable for sign-sensitive downstream
//! operations. The result is a fraction with denominator 2^32, not a binary32 value: callers
//! must perform their own x87 scaling and eventual store/rounding.
//!
//! This model rejects zero initial state because the source instead takes an
//! RDTSC-dependent branch. A nonzero word never becomes zero: the multiplier
//! is odd. The source's >= 1 fallback cannot occur in this admitted domain;
//! 24-bit x87 precision and its potentially reachable fallback are excluded.
//! Floating-point status flags, concurrent access and actual live state are
//! outside the model. Static instruction evidence is not runtime validation.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroSeed;

/// Exact x87 return value: `numerator / 2^32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fraction {
    pub numerator: u32,
}

impl Fraction {
    /// Binary64 represents the numeric fraction exactly, with zero as +0.
    /// This is not the caller's
    /// binary32 store and must not be substituted for its x87 multiplication.
    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) * (1.0 / 4_294_967_296.0)
    }
}

impl Rng {
    pub fn new(state: u32) -> Result<Self, ZeroSeed> {
        if state == 0 { Err(ZeroSeed) } else { Ok(Self { state }) }
    }

    pub fn state(&self) -> u32 { self.state }

    pub fn next_fraction(&mut self) -> Fraction {
        self.state = self.state.wrapping_mul(0x278d_de6d);
        // Signed interpretation and adding 1/2 rotates the 32-bit range.
        Fraction { numerator: self.state ^ 0x8000_0000 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successive_words_and_signed_mapping() {
        let mut rng = Rng::new(1).unwrap();
        for (state, numerator) in [
            (0x278d_de6d, 0xa78d_de6d),
            (0xfb53_3a69, 0x7b53_3a69),
            (0x59eb_ecb5, 0xd9eb_ecb5),
        ] {
            assert_eq!(rng.next_fraction().numerator, numerator);
            assert_eq!(rng.state(), state);
        }
    }

    #[test]
    fn signed_minimum_produces_zero_without_zeroing_state() {
        let mut rng = Rng::new(0x8000_0000).unwrap();
        assert_eq!(rng.next_fraction().as_f64(), 0.0);
        assert_eq!(rng.state(), 0x8000_0000);
        assert_eq!(rng.next_fraction().numerator, 0);
    }

    #[test]
    fn endpoint_and_zero_seed_boundaries() {
        assert_eq!(Rng::new(0), Err(ZeroSeed));
        assert_eq!(Fraction { numerator: u32::MAX }.as_f64(), 1.0 - 2f64.powi(-32));
        assert!(Fraction { numerator: u32::MAX }.as_f64() < 1.0);
        // Binary32 would round this to 1, demonstrating why it is not stored here.
        assert_eq!(Fraction { numerator: u32::MAX }.as_f64() as f32, 1.0);
    }

    #[test]
    fn negative_signed_word_and_wrapping_multiplication() {
        let mut rng = Rng::new(u32::MAX).unwrap();
        assert_eq!(rng.next_fraction().numerator, 0x5872_2193);
        assert_eq!(rng.state(), 0xd872_2193);
    }
}
