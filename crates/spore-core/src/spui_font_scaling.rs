//! Source-derived simple-glyph phantom coordinates and selector-0 scaling.
//!
//! Unexported pending review. This is unscaled/26.6 input preparation only:
//! no composite assembly, phantom flags, glyph hints or raster output.
//! Native addresses and call boundaries are documented in
//! re/spui-font-outline-native.md. Original font/bytecode payloads are not
//! included here. Unsupported scale selectors return explicit errors.

use crate::spui_font::{GlyphBounds, HorizontalMetrics};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignCoordinate {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate26Dot6 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerticalMetrics {
    pub advance: u16,
    pub top_side_bearing: i16,
}

/// Original 8CE8D2 fallback when the ordinary face has no vmtx table.
/// This uses the supplied hhea fields, wrapping to u16 before division.
pub fn vertical_fallback(ascender: i16, descender: i16, line_gap: i16) -> VerticalMetrics {
    let advance = line_gap.wrapping_sub(descender).wrapping_add(ascender) as u16;
    VerticalMetrics {
        advance,
        top_side_bearing: (advance / 10) as i16,
    }
}

/// The four coordinates appended by 8CD700 for a positive-contour packet.
/// This does not describe the distinct empty/zero-contour/composite branches.
pub fn simple_phantom_coordinates(
    bounds: GlyphBounds,
    horizontal: HorizontalMetrics,
    vertical: VerticalMetrics,
) -> [DesignCoordinate; 4] {
    let left = bounds.x_min.wrapping_sub(horizontal.left_side_bearing);
    let right = left.wrapping_add(horizontal.advance_width as i16);
    // Native CWDE/MOVSX followed by SAR occurs after each word was stored.
    let center = ((i32::from(left) + i32::from(right)) >> 1) as i16;
    let top = bounds.y_max.wrapping_add(vertical.top_side_bearing);
    let bottom = top.wrapping_sub(vertical.advance as i16);
    [
        DesignCoordinate { x: left, y: 0 },
        DesignCoordinate { x: right, y: 0 },
        DesignCoordinate { x: center, y: top },
        DesignCoordinate {
            x: center,
            y: bottom,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleError {
    ZeroPpem,
    ZeroUnitsPerEm,
    UnsupportedSelector(u16),
}
impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source UI coordinate scale: {self:?}")
    }
}
impl std::error::Error for ScaleError {}

/// Logical source fields, not a C-compatible packed structure. The native
/// structure's two trailing bytes are not assigned values or meanings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleParameters {
    pub multiplier: i16,
    pub shift: u16,
    pub denominator: i32,
    pub rounding_addend: i32,
    pub fixed_multiplier: i32,
    pub selector: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerOfTwoScale {
    parameters: ScaleParameters,
}
impl PowerOfTwoScale {
    /// Recover only 8CC8C0 selector 0 for positive 16-bit ppem/UPE inputs.
    /// Zero guards and unsupported-selector errors are this bounded API's
    /// contract, rather than equivalents of native error/recovery behavior.
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
        if multiplier > i32::from(i16::MAX) {
            return Err(ScaleError::UnsupportedSelector(2));
        }
        if !denominator.is_positive() || !(denominator as u32).is_power_of_two() {
            return Err(ScaleError::UnsupportedSelector(1));
        }
        // A positive u16 power-of-two denominator divides 65536 exactly.
        // The supported multiplier guard also keeps this result within i32,
        // so no native fixed-division rounding/overflow policy is substituted.
        let fixed_multiplier = ((i64::from(multiplier) << 16) / i64::from(denominator)) as i32;
        Ok(Self {
            parameters: ScaleParameters {
                multiplier: multiplier as i16,
                shift: denominator.trailing_zeros() as u16,
                denominator,
                rounding_addend: denominator >> 1,
                fixed_multiplier,
                selector: 0,
            },
        })
    }

    pub fn parameters(self) -> ScaleParameters {
        self.parameters
    }

    /// Native 8BF350 selector 0: signed design word, IMUL, ADD, SAR.
    /// Negative half ties round toward positive infinity in this branch.
    pub fn coordinate(self, value: i16) -> i32 {
        (i32::from(value) * i32::from(self.parameters.multiplier) + self.parameters.rounding_addend)
            >> self.parameters.shift
    }
}

pub fn scale_coordinates(
    input: &[DesignCoordinate],
    x: PowerOfTwoScale,
    y: PowerOfTwoScale,
) -> Vec<Coordinate26Dot6> {
    input
        .iter()
        .map(|p| Coordinate26Dot6 {
            x: x.coordinate(p.x),
            y: y.coordinate(p.y),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_wraps_before_the_unsigned_tenth() {
        assert_eq!(
            vertical_fallback(30000, -30000, 10000),
            VerticalMetrics {
                advance: 4464,
                top_side_bearing: 446
            }
        );
        assert_eq!(
            vertical_fallback(-1, 0, 0),
            VerticalMetrics {
                advance: 65535,
                top_side_bearing: 6553
            }
        );
    }

    #[test]
    fn phantom_center_uses_signed_narrowed_coordinates() {
        let p = simple_phantom_coordinates(
            GlyphBounds {
                x_min: -200,
                y_min: -20,
                x_max: 200,
                y_max: 300,
            },
            HorizontalMetrics {
                advance_width: 101,
                left_side_bearing: 100,
            },
            VerticalMetrics {
                advance: 1000,
                top_side_bearing: 50,
            },
        );
        assert_eq!(
            p,
            [
                DesignCoordinate { x: -300, y: 0 },
                DesignCoordinate { x: -199, y: 0 },
                DesignCoordinate { x: -250, y: 350 },
                DesignCoordinate { x: -250, y: -650 },
            ]
        );
        let p = simple_phantom_coordinates(
            GlyphBounds {
                x_min: 32767,
                y_min: 0,
                x_max: 32767,
                y_max: 32767,
            },
            HorizontalMetrics {
                advance_width: 65535,
                left_side_bearing: -1,
            },
            VerticalMetrics {
                advance: 65535,
                top_side_bearing: 1,
            },
        );
        assert_eq!(
            p,
            [
                DesignCoordinate { x: -32768, y: 0 },
                DesignCoordinate { x: 32767, y: 0 },
                DesignCoordinate { x: -1, y: -32768 },
                DesignCoordinate { x: -1, y: -32767 },
            ]
        );
    }

    #[test]
    fn source_fixture_parameters_and_negative_half_ties_are_explicit() {
        let scale = PowerOfTwoScale::for_ppem(15, 2048).unwrap();
        assert_eq!(
            scale.parameters(),
            ScaleParameters {
                multiplier: 15,
                shift: 5,
                denominator: 32,
                rounding_addend: 16,
                fixed_multiplier: 0x7800,
                selector: 0,
            }
        );
        assert_eq!((scale.coordinate(-16), scale.coordinate(16)), (-7, 8));
        assert_eq!(
            (scale.coordinate(i16::MIN), scale.coordinate(i16::MAX)),
            (-15360, 15360)
        );
        assert_eq!(scale.coordinate(0), 0);
    }

    #[test]
    fn other_selectors_and_zero_inputs_are_named_errors() {
        assert_eq!(
            PowerOfTwoScale::for_ppem(0, 2048),
            Err(ScaleError::ZeroPpem)
        );
        assert_eq!(
            PowerOfTwoScale::for_ppem(15, 0),
            Err(ScaleError::ZeroUnitsPerEm)
        );
        assert_eq!(
            PowerOfTwoScale::for_ppem(15, 1000),
            Err(ScaleError::UnsupportedSelector(1))
        );
        assert_eq!(
            PowerOfTwoScale::for_ppem(32767, 1),
            Err(ScaleError::UnsupportedSelector(2))
        );
    }

    #[test]
    fn each_axis_uses_its_explicit_parameters_and_bounds_remain_in_i32() {
        let x = PowerOfTwoScale::for_ppem(15, 2048).unwrap();
        let y = PowerOfTwoScale::for_ppem(16, 2048).unwrap();
        assert_eq!(
            scale_coordinates(&[DesignCoordinate { x: 16, y: -16 }], x, y),
            [Coordinate26Dot6 { x: 8, y: -8 }]
        );
        let largest = PowerOfTwoScale::for_ppem(32767, 64).unwrap();
        assert_eq!(largest.parameters().denominator, 1);
        assert_eq!(largest.parameters().fixed_multiplier, 2147418112);
        assert_eq!(largest.coordinate(i16::MIN), -1073709056);
        assert_eq!(largest.coordinate(i16::MAX), 1073676289);
    }
}
