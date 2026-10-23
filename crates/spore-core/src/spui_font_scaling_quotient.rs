//! Source 8BF350 selector 1: sign-aware integer quotient scaling.
//!
//! Unexported pending the separate source-instruction reference gate. This
//! extends the explicit positive-u16 callee-input domain without changing
//! selector-0's frozen implementation. Selectors 0 and 2 are named unsupported
//! here. The caller chooses the admitted scaler; no font hints or rasterizer
//! are selected by this module.

use crate::spui_font_scaling::{ScaleError, ScaleParameters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerQuotientScale {
    parameters: ScaleParameters,
}

impl IntegerQuotientScale {
    pub fn for_ppem(ppem: u16, units_per_em: u16) -> Result<Self, ScaleError> {
        if ppem == 0 {
            return Err(ScaleError::ZeroPpem);
        }
        if units_per_em == 0 {
            return Err(ScaleError::ZeroUnitsPerEm);
        }
        let mut multiplier = i32::from(ppem) << 6;
        let mut denominator = i32::from(units_per_em);
        while (multiplier | denominator) & 1 == 0 {
            multiplier >>= 1;
            denominator >>= 1;
        }
        // Native 8CC90D preserves default selector 2 before the denominator
        // is classified, including oversized/non-power-of-two combinations.
        if multiplier > i32::from(i16::MAX) {
            return Err(ScaleError::UnsupportedSelector(2));
        }
        if (denominator as u32).is_power_of_two() {
            return Err(ScaleError::UnsupportedSelector(0));
        }
        // For this positive domain, 8D0DB0's first quotient/remainder split
        // and second unsigned DIV equal the floor of this exact ratio.
        // The admitted multiplier keeps the entire numerator within i32.
        let fixed_multiplier = ((i64::from(multiplier) << 16) / i64::from(denominator)) as i32;
        Ok(Self {
            parameters: ScaleParameters {
                multiplier: multiplier as i16,
                shift: denominator.trailing_zeros() as u16,
                denominator,
                rounding_addend: denominator >> 1,
                fixed_multiplier,
                selector: 1,
            },
        })
    }

    pub fn parameters(self) -> ScaleParameters {
        self.parameters
    }

    /// 8BF3C4..8BF3E0: IMUL, sign test, positive magnitude IDIV, optional NEG.
    /// At exact half ties this selector rounds the magnitude away from zero.
    /// The native selector-1 branch does not use its stored shift/fixed field.
    pub fn coordinate(self, value: i16) -> i32 {
        let product = i32::from(value) * i32::from(self.parameters.multiplier);
        if product >= 0 {
            (product + self.parameters.rounding_addend) / self.parameters.denominator
        } else {
            -((self.parameters.rounding_addend - product) / self.parameters.denominator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_1000_upe_faces_keep_integer_coefficients_and_fixed_floor() {
        let scale = IntegerQuotientScale::for_ppem(15, 1000).unwrap();
        assert_eq!(
            scale.parameters(),
            ScaleParameters {
                multiplier: 120,
                shift: 0,
                denominator: 125,
                rounding_addend: 62,
                fixed_multiplier: 62914,
                selector: 1,
            }
        );
        assert_eq!((scale.coordinate(-16), scale.coordinate(16)), (-15, 15));
        assert_eq!(
            (scale.coordinate(i16::MIN), scale.coordinate(i16::MAX)),
            (-31457, 31456)
        );
    }

    #[test]
    fn negative_half_ties_use_the_native_magnitude_branch() {
        let scale = IntegerQuotientScale::for_ppem(1, 384).unwrap();
        assert_eq!(scale.parameters().multiplier, 1);
        assert_eq!(scale.parameters().denominator, 6);
        assert_eq!(scale.parameters().shift, 1);
        assert_eq!((scale.coordinate(-3), scale.coordinate(3)), (-1, 1));
        assert_eq!((scale.coordinate(-2), scale.coordinate(2)), (0, 0));
        assert_eq!((scale.coordinate(-9), scale.coordinate(9)), (-2, 2));
    }

    #[test]
    fn largest_word_multiplier_and_negative_input_fit_before_idiv() {
        let scale = IntegerQuotientScale::for_ppem(32767, 192).unwrap();
        assert_eq!(scale.parameters().multiplier, 32767);
        assert_eq!(scale.parameters().denominator, 3);
        assert_eq!(scale.parameters().fixed_multiplier, 715806037);
        assert_eq!(scale.coordinate(i16::MIN), -357903019);
        assert_eq!(scale.coordinate(i16::MAX), 357892096);
    }

    #[test]
    fn coefficient_selector_order_is_preserved_for_large_positive_ppem() {
        assert_eq!(
            IntegerQuotientScale::for_ppem(32768, 1000),
            Err(ScaleError::UnsupportedSelector(2))
        );
        assert_eq!(
            IntegerQuotientScale::for_ppem(32768, 2048),
            Err(ScaleError::UnsupportedSelector(0))
        );
        assert_eq!(
            IntegerQuotientScale::for_ppem(0, 1000),
            Err(ScaleError::ZeroPpem)
        );
        assert_eq!(
            IntegerQuotientScale::for_ppem(15, 0),
            Err(ScaleError::ZeroUnitsPerEm)
        );
    }

    #[test]
    fn largest_denominator_keeps_unsigned_input_and_signed_output() {
        let scale = IntegerQuotientScale::for_ppem(1, 65535).unwrap();
        assert_eq!(scale.parameters().denominator, 65535);
        assert_eq!(scale.parameters().rounding_addend, 32767);
        assert_eq!(scale.parameters().fixed_multiplier, 64);
        assert_eq!(scale.coordinate(-1), 0);
        assert_eq!(scale.coordinate(i16::MIN), -32);
        assert_eq!(scale.coordinate(i16::MAX), 32);
    }
}
