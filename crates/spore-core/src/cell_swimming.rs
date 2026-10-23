//! Bounded original Cell steering, with explicit retained x87 arithmetic.
//!
//! This does not select the retail floating-point policy, populate an
//! environment field, calculate actor capabilities or replace the actor update.

use crate::native_x87::{Context, Extended, X87Error};
use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy, Debug)]
pub struct CellTurnInPlace {
    pub position: [f32; 3],
    /// Source current vector, supplied from actor `90..98` by E6EF30.
    pub current_vector: [f32; 3],
    pub target: [f32; 3],
    /// Fourth native argument. Its source tuning/scaling is a caller concern.
    pub step: f32,
    /// E4E220's sampled environment vector at the actor position.
    pub sampled_environment: [f32; 3],
    /// Source global vector `16B4C08..10`, used only for either signed zero step.
    pub zero_step_vector: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellSwimmingError {
    NonFiniteInput,
    NonFiniteIntermediate,
    X87(X87Error),
}
impl From<X87Error> for CellSwimmingError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}
impl fmt::Display for CellSwimmingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded Cell steering: {self:?}")
    }
}
impl std::error::Error for CellSwimmingError {}

/// Complete finite-domain scalar arithmetic and branch of original `E55E40`.
///
/// Source x87 operations retain their full context-selected values across the
/// binary32 store and subsequent comparison. SSE remains binary32 nearest with
/// preserved subnormals; the explicitly supplied x87 precision/rounding may
/// differ. No retail control word is assumed. The unused fifth native argument
/// has no arithmetic effect and is not modeled as a new gameplay parameter.
pub fn turn_in_place_delta(
    input: CellTurnInPlace,
    x87: Context,
) -> Result<[f32; 3], CellSwimmingError> {
    use CellSwimmingError as Error;
    if !input.step.is_finite() {
        return Err(Error::NonFiniteInput);
    }
    if input.step == 0.0 {
        return if input.zero_step_vector.into_iter().all(f32::is_finite) {
            Ok(input.zero_step_vector)
        } else {
            Err(Error::NonFiniteInput)
        };
    }
    if !input
        .position
        .into_iter()
        .chain(input.current_vector)
        .chain(input.target)
        .chain(input.sampled_environment)
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteInput);
    }
    let delta: [f32; 3] = std::array::from_fn(|axis| input.target[axis] - input.position[axis]);
    let y_squared = delta[1] * delta[1];
    let z_squared = delta[2] * delta[2];
    let x_squared = delta[0] * delta[0];
    let sum_yz = y_squared + z_squared;
    let squared = sum_yz + x_squared;
    if !delta
        .into_iter()
        .chain([y_squared, z_squared, x_squared, sum_yz, squared])
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteIntermediate);
    }
    let retained_squared = Extended::from_f32(squared)?;
    // E55EC1 loads this binary32 source operand before FADD ST0,ST1.
    let epsilon = Extended::from_f32(f32::from_bits(0x322b_cc77))?;
    let adjusted = x87.add(epsilon, retained_squared)?;
    let inverse = x87.store_f32(x87.div(Extended::from_f32(1.)?, x87.sqrt(adjusted)?)?)?;
    let normalized = [inverse * delta[0], delta[1] * inverse, inverse * delta[2]];
    let retained_distance = x87.sqrt(retained_squared)?;
    let committed_distance = x87.store_f32(retained_distance)?;
    let amount =
        if x87.compare(Extended::from_f32(input.step)?, retained_distance)? == Ordering::Greater {
            committed_distance
        } else {
            input.step
        };
    let scaled = normalized.map(|component| amount * component);
    let combined: [f32; 3] =
        std::array::from_fn(|axis| input.sampled_environment[axis] + scaled[axis]);
    let result = std::array::from_fn(|axis| combined[axis] - input.current_vector[axis]);
    if !normalized
        .into_iter()
        .chain([committed_distance])
        .chain(scaled)
        .chain(combined)
        .chain(result)
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteIntermediate);
    }
    Ok(result)
}

/// Explicit native arguments of E55B20/E557A0. Numbered fields retain their
/// parameter positions until the actor caller's tuning/units are established.
#[derive(Clone, Copy, Debug)]
pub struct CellPropulsion {
    pub direction: [f32; 3],
    pub position: [f32; 3],
    pub current_vector: [f32; 3],
    pub target: [f32; 3],
    pub speed: f32,
    pub factor_6: f32,
    pub heading_test_7: f32,
    /// Only the low source byte is consumed by these native functions.
    pub hint_8: u8,
    pub factor_9: f32,
    pub sampled_environment: [f32; 3],
    pub zero_speed_vector: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellJetResult {
    pub delta: [f32; 3],
    pub active: bool,
}

fn ext(value: f32) -> Result<Extended, CellSwimmingError> {
    Ok(Extended::from_f32(value)?)
}

// MAXSS with a +0 destination takes its second operand on equality, so a
// supplied -0 remains -0. This differs from E4DB80's operand order.
fn clamp_unit(value: f32) -> f32 {
    let lower = if value >= 0. { value } else { 0. };
    if lower >= 1. {
        1.
    } else {
        lower
    }
}

/// E51230/1041E40: inverse interpolation commits to f32 before its SSE clamp;
/// the output interpolation stays retained in x87 until its caller commits it.
fn retained_ramp(
    x87: Context,
    start: f32,
    end: f32,
    value: f32,
    output_start: f32,
    output_end: f32,
) -> Result<Extended, CellSwimmingError> {
    let numerator = x87.sub(ext(value)?, ext(start)?)?;
    let denominator = x87.add(
        x87.sub(ext(end)?, ext(start)?)?,
        ext(f32::from_bits(0x3780_0000))?,
    )?;
    let factor = clamp_unit(x87.store_f32(x87.div(numerator, denominator)?)?);
    Ok(x87.add(
        x87.mul(x87.sub(ext(output_end)?, ext(output_start)?)?, ext(factor)?)?,
        ext(output_start)?,
    )?)
}

fn normalized_heading(x87: Context, vector: [f32; 3]) -> Result<[f32; 3], CellSwimmingError> {
    let squared = vector.map(|component| {
        let value = ext(component)?;
        Ok::<_, CellSwimmingError>(x87.mul(value, value)?)
    });
    let [x, y, z] = squared;
    let sum = x87.add(x87.add(x?, y?)?, z?)?;
    let adjusted = x87.add(sum, ext(f32::from_bits(0x322b_cc77))?)?;
    let inverse = x87.store_f32(x87.div(ext(1.)?, x87.sqrt(adjusted)?)?)?;
    let result = vector.map(|component| component * inverse);
    if !result.into_iter().all(f32::is_finite) {
        return Err(CellSwimmingError::NonFiniteIntermediate);
    }
    Ok(result)
}

fn heading_factor(
    x87: Context,
    direction: [f32; 3],
    target_delta: [f32; 3],
    hint: u8,
) -> Result<Extended, CellSwimmingError> {
    let direction = normalized_heading(x87, direction)?;
    let target = normalized_heading(x87, target_delta)?;
    let z = target[2] * direction[2];
    let y = target[1] * direction[1];
    let x = target[0] * direction[0];
    let zy = z + y;
    let dot = zy + x;
    if ![z, y, x, zy, dot].into_iter().all(f32::is_finite) {
        return Err(CellSwimmingError::NonFiniteIntermediate);
    }
    if hint != 0 {
        return ext(if dot > f32::from_bits(0x3f66_6666) {
            1.
        } else {
            0.
        });
    }
    let value = x87.store_f32(retained_ramp(x87, -1., 1., dot, -3., 1.)?)?;
    ext(clamp_unit(value))
}

fn propulsion_delta(
    input: CellPropulsion,
    x87: Context,
    jet: bool,
) -> Result<CellJetResult, CellSwimmingError> {
    use CellSwimmingError as Error;
    if !input.speed.is_finite() {
        return Err(Error::NonFiniteInput);
    }
    if input.speed == 0. {
        return if input.zero_speed_vector.into_iter().all(f32::is_finite) {
            Ok(CellJetResult {
                delta: input.zero_speed_vector,
                active: false,
            })
        } else {
            Err(Error::NonFiniteInput)
        };
    }
    if !input
        .direction
        .into_iter()
        .chain(input.position)
        .chain(input.current_vector)
        .chain(input.target)
        .chain(input.sampled_environment)
        .chain([input.factor_6, input.heading_test_7, input.factor_9])
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteInput);
    }
    let delta: [f32; 3] = std::array::from_fn(|axis| input.target[axis] - input.position[axis]);
    let squared_components = delta.map(|component| component * component);
    let squared = if jet {
        (squared_components[1] + squared_components[0]) + squared_components[2]
    } else {
        (squared_components[2] + squared_components[1]) + squared_components[0]
    };
    if !delta
        .into_iter()
        .chain(squared_components)
        .chain([squared])
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteIntermediate);
    }
    let retained_squared = ext(squared)?;
    let retained_distance = x87.sqrt(retained_squared)?;
    let distance = x87.store_f32(retained_distance)?;
    let use_original_direction = if jet {
        x87.compare(ext(1.)?, retained_distance)? != Ordering::Greater
    } else {
        1. <= distance
    };
    let mut direction = if use_original_direction {
        input.direction
    } else {
        let adjusted = x87.add(retained_squared, ext(f32::from_bits(0x322b_cc77))?)?;
        let inverse = x87.store_f32(x87.div(ext(1.)?, x87.sqrt(adjusted)?)?)?;
        let normalized = [inverse * delta[0], delta[1] * inverse, delta[2] * inverse];
        std::array::from_fn(|axis| {
            (input.direction[axis] - normalized[axis]) * distance + normalized[axis]
        })
    };
    if !direction.into_iter().all(f32::is_finite) {
        return Err(Error::NonFiniteIntermediate);
    }
    let mut active = true;
    if input.heading_test_7 > f32::from_bits(0x3780_0000) {
        let factor = heading_factor(x87, direction, delta, input.hint_8)?;
        for component in &mut direction {
            *component = x87.store_f32(x87.mul(ext(*component)?, factor)?)?;
        }
        if jet {
            active = x87.compare(ext(f32::from_bits(0x3780_0000))?, factor)? != Ordering::Greater;
        }
    }
    let mut relative = [ext(0.)?; 3];
    let mut committed_relative = [0.; 3];
    for axis in 0..3 {
        relative[axis] = x87.sub(
            ext(input.current_vector[axis])?,
            ext(input.sampled_environment[axis])?,
        )?;
        committed_relative[axis] = x87.store_f32(relative[axis])?;
    }
    let x_squared = x87.mul(relative[0], relative[0])?;
    let z_squared = x87.mul(relative[2], relative[2])?;
    let y_squared = x87.mul(relative[1], relative[1])?;
    let relative_distance = x87.sqrt(x87.add(x87.add(x_squared, z_squared)?, y_squared)?)?;
    let reverse = x87.compare(relative_distance, ext(distance)?)? == Ordering::Greater
        && (!jet || input.hint_8 != 0);
    let desired = if reverse {
        committed_relative.map(|component| component * -1.)
    } else {
        let ramp = x87.store_f32(retained_ramp(
            x87,
            0.5,
            3.,
            distance,
            if jet { 0. } else { 0.5 },
            1.,
        )?)?;
        let scaled = direction.map(|component| {
            if jet {
                ((component * input.speed) * ramp) * input.factor_6
            } else {
                ((component * ramp) * input.speed) * input.factor_6
            }
        });
        if !scaled.into_iter().all(f32::is_finite) {
            return Err(Error::NonFiniteIntermediate);
        }
        std::array::from_fn(|axis| {
            if jet && axis == 0 {
                scaled[axis] + input.sampled_environment[axis]
            } else {
                input.sampled_environment[axis] + scaled[axis]
            }
        })
    };
    let response = if jet && !active && input.hint_8 != 0 {
        10.
    } else {
        1.
    };
    let result = std::array::from_fn(|axis| {
        let amount = (desired[axis] - input.current_vector[axis]) * input.factor_9;
        if jet {
            amount * response
        } else {
            amount
        }
    });
    if !desired
        .into_iter()
        .chain(committed_relative)
        .chain(result)
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteIntermediate);
    }
    Ok(CellJetResult {
        delta: result,
        active,
    })
}

/// Complete bounded E55B20 math, used for flagella/cilia by the source caller.
pub fn flagella_cilia_delta(
    input: CellPropulsion,
    x87: Context,
) -> Result<[f32; 3], CellSwimmingError> {
    Ok(propulsion_delta(input, x87, false)?.delta)
}

/// Complete bounded E557A0 math and its native boolean return.
pub fn jet_delta(input: CellPropulsion, x87: Context) -> Result<CellJetResult, CellSwimmingError> {
    propulsion_delta(input, x87, true)
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
    fn zero_step_preserves_supplied_source_vector_and_ignores_unused_inputs() {
        for step in [0., -0.] {
            let result = turn_in_place_delta(
                CellTurnInPlace {
                    position: [f32::NAN; 3],
                    current_vector: [f32::NAN; 3],
                    target: [f32::NAN; 3],
                    step,
                    sampled_environment: [f32::NAN; 3],
                    zero_step_vector: [-0., 1., -2.],
                },
                context(),
            )
            .unwrap();
            assert_eq!(
                result.map(f32::to_bits),
                [-0_f32, 1., -2.].map(f32::to_bits)
            );
        }
    }

    #[test]
    fn sampled_environment_and_current_vector_remain_in_the_source_expression() {
        let input = CellTurnInPlace {
            position: [0.; 3],
            current_vector: [1., 2., 3.],
            target: [0.; 3],
            step: 2.,
            sampled_environment: [3., 1., -3.],
            zero_step_vector: [f32::NAN; 3],
        };
        assert_eq!(
            turn_in_place_delta(input, context()).unwrap(),
            [2., -1., -6.]
        );
        assert_eq!(
            turn_in_place_delta(
                CellTurnInPlace {
                    target: [f32::MAX; 3],
                    ..input
                },
                context()
            ),
            Err(CellSwimmingError::NonFiniteIntermediate)
        );
    }

    #[test]
    fn propulsion_zero_speed_keeps_early_return_and_jet_boolean() {
        let input = CellPropulsion {
            direction: [f32::NAN; 3],
            position: [f32::NAN; 3],
            current_vector: [f32::NAN; 3],
            target: [f32::NAN; 3],
            speed: -0.,
            factor_6: f32::NAN,
            heading_test_7: f32::NAN,
            hint_8: 255,
            factor_9: f32::NAN,
            sampled_environment: [f32::NAN; 3],
            zero_speed_vector: [-0., 1., -2.],
        };
        assert_eq!(
            flagella_cilia_delta(input, context())
                .unwrap()
                .map(f32::to_bits),
            input.zero_speed_vector.map(f32::to_bits)
        );
        let jet = jet_delta(input, context()).unwrap();
        assert!(!jet.active);
        assert_eq!(
            jet.delta.map(f32::to_bits),
            input.zero_speed_vector.map(f32::to_bits)
        );
    }

    #[test]
    fn jet_hint_changes_the_original_reverse_branch() {
        let input = CellPropulsion {
            direction: [1., 0., 0.],
            position: [0.; 3],
            current_vector: [2., 0., 0.],
            target: [0.; 3],
            speed: 1.,
            factor_6: 1.,
            heading_test_7: 0.,
            hint_8: 1,
            factor_9: 0.25,
            sampled_environment: [0.; 3],
            zero_speed_vector: [0.; 3],
        };
        assert_eq!(flagella_cilia_delta(input, context()).unwrap()[0], -1.);
        assert_eq!(jet_delta(input, context()).unwrap().delta[0], -1.);
        assert_eq!(
            jet_delta(CellPropulsion { hint_8: 0, ..input }, context())
                .unwrap()
                .delta[0],
            -0.5
        );
        let stopped = jet_delta(
            CellPropulsion {
                heading_test_7: 1.,
                ..input
            },
            context(),
        )
        .unwrap();
        assert!(!stopped.active);
        assert_eq!(stopped.delta[0], -10.);
    }
}
