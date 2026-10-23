//! Bounded original Cell movement helpers, before the full actor integrator.
//!
//! E67CB0 selects a movement style; the SSE block E6F707..E6F82A adjusts
//! an axial seek target. Basis vectors are explicit inputs because static image
//! values alone do not establish every runtime frame. The arithmetic contract
//! is binary32 nearest rounding with subnormals preserved. This does not replace
//! E6EF30's jet/cilia/flagella integration, state transitions or input handling.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellMovementAxes {
    pub primary: [f32; 3],
    pub secondary: [f32; 3],
    pub tertiary: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellMovementError {
    NonFiniteInput,
    NonFiniteIntermediate,
}

impl fmt::Display for CellMovementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput => f.write_str("nonfinite Cell movement input"),
            Self::NonFiniteIntermediate => {
                f.write_str("Cell movement arithmetic left its finite domain")
            }
        }
    }
}
impl std::error::Error for CellMovementError {}

/// E67CB0: zero actor +FC word selects flagella (1), otherwise an explicit AI
/// style wins, followed by the creation capability cache's +1254 style word.
/// This function does not guess a creation ID or build that capability cache.
pub fn resolve_movement_style(actor_word_fc: u32, ai_style: u32, cached_style: u32) -> u32 {
    if actor_word_fc == 0 {
        1
    } else if ai_style != 0 {
        ai_style
    } else {
        cached_style
    }
}

/// Exact scalar operation/branch order of the source SSE axial-target block.
///
/// The comparison deliberately uses secondary > tertiary, then projects through
/// the primary axis in either branch. Do not substitute a geometric dominant-
/// axis rule. All nonzero axial bytes enter the native adjustment branch.
pub fn axial_seek_target(
    position: [f32; 3],
    target: [f32; 3],
    axes: CellMovementAxes,
    axial_byte: u8,
) -> Result<[f32; 3], CellMovementError> {
    if !target.into_iter().all(f32::is_finite) {
        return Err(CellMovementError::NonFiniteInput);
    }
    if axial_byte == 0 {
        return Ok(target);
    }
    if !position
        .into_iter()
        .chain(axes.primary)
        .chain(axes.secondary)
        .chain(axes.tertiary)
        .all(f32::is_finite)
    {
        return Err(CellMovementError::NonFiniteInput);
    }
    let delta = std::array::from_fn(|i| target[i] - position[i]);
    let secondary = native_dot(delta, axes.secondary);
    let tertiary = native_dot(delta, axes.tertiary);
    let amount = if secondary > tertiary {
        secondary
    } else {
        native_dot(delta, axes.primary)
    };
    let result = std::array::from_fn(|i| position[i] + axes.primary[i] * amount);
    if !delta
        .into_iter()
        .chain([secondary, tertiary, amount])
        .chain(result)
        .all(f32::is_finite)
    {
        return Err(CellMovementError::NonFiniteIntermediate);
    }
    Ok(result)
}

fn native_dot(delta: [f32; 3], axis: [f32; 3]) -> f32 {
    let z = delta[2] * axis[2];
    let x = axis[0] * delta[0];
    let zx = z + x;
    let y = axis[1] * delta[1];
    zx + y
}

#[cfg(test)]
mod tests {
    use super::*;

    const AXES: CellMovementAxes = CellMovementAxes {
        primary: [1.0, 0.0, 0.0],
        secondary: [0.0, 1.0, 0.0],
        tertiary: [0.0, 0.0, 1.0],
    };

    #[test]
    fn source_axial_branch_is_not_a_dominant_axis_substitute() {
        assert_eq!(
            axial_seek_target([1.0, 2.0, 3.0], [11.0, 22.0, 33.0], AXES, 1).unwrap(),
            [11.0, 2.0, 3.0]
        );
        assert_eq!(
            axial_seek_target([1.0, 2.0, 3.0], [11.0, 42.0, 33.0], AXES, 2).unwrap(),
            [41.0, 2.0, 3.0]
        );
        assert_eq!(
            axial_seek_target([1.0, 2.0, 3.0], [11.0, 32.0, 33.0], AXES, 1).unwrap(),
            [11.0, 2.0, 3.0]
        );
    }

    #[test]
    fn disabled_axial_keeps_target_bits_and_does_not_read_unused_axes() {
        let target = [-0.0, 2.0, 3.0];
        let axes = CellMovementAxes {
            primary: [f32::NAN; 3],
            ..AXES
        };
        let actual = axial_seek_target([f32::NAN; 3], target, axes, 0).unwrap();
        assert_eq!(actual.map(f32::to_bits), target.map(f32::to_bits));
        assert_eq!(
            axial_seek_target([0.0; 3], target, axes, 1),
            Err(CellMovementError::NonFiniteInput)
        );
        assert_eq!(resolve_movement_style(0, 8, 7), 1);
        assert_eq!(resolve_movement_style(1, 8, 7), 8);
        assert_eq!(resolve_movement_style(1, 0, 7), 7);
    }
}
