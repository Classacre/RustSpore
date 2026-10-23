//! Original Cell sampled environment-vector arithmetic (`E4DB80/E4DFD0/E4E220`).
//!
//! Runtime frame origin/scale and all source vectors are supplied explicitly.
//! This is the native centered 20-by-15 sampling rule, including duplicated edge
//! samples and sequential binary32 additions, before actor flow multipliers.

use std::fmt;

pub const CELL_FLOW_WIDTH: usize = 20;
pub const CELL_FLOW_HEIGHT: usize = 15;
pub const CELL_FLOW_SAMPLES: usize = CELL_FLOW_WIDTH * CELL_FLOW_HEIGHT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellFlowFrame {
    /// Runtime words `16B4C68/6C`.
    pub origin: [f32; 2],
    /// Runtime words `15A8B9C/BA0`.
    pub scale: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellFlowError {
    NonFiniteInput,
    NonFiniteIntermediate,
}

impl fmt::Display for CellFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFiniteInput => "nonfinite Cell field-sampling input",
            Self::NonFiniteIntermediate => "Cell field sampling left its finite arithmetic domain",
        })
    }
}
impl std::error::Error for CellFlowError {}

#[derive(Clone, Copy)]
struct AxisWeights {
    base: usize,
    neighbor: usize,
    first: f32,
    second: f32,
}

fn weights(coordinate: f32, extent: usize) -> AxisWeights {
    // E4DFD0's nearest-integer conversion plus correction produces floor in
    // the clamped finite 0..20 domain. No out-of-range integer conversion occurs.
    let base = coordinate.floor() as usize;
    let fraction = coordinate - base as f32;
    let (first, neighbor) = if fraction >= 0.5 {
        (1.5 - fraction, (base + 1).min(extent - 1))
    } else {
        (fraction + 0.5, base.saturating_sub(1))
    };
    AxisWeights {
        base,
        neighbor,
        first,
        second: 1.0 - first,
    }
}

/// Sample the complete finite-domain native environment vector rule.
///
/// `field` is the row-major source environment array beginning at `1C210`.
/// Nearest rounding and preserved subnormals are the declared binary32 policy.
/// Only the four sampled vectors are consumed; unused field entries may retain
/// arbitrary bits. Source runtime frame creation and field population remain
/// separate from this helper. Actor/global multipliers are not applied here.
pub fn sample_flow_field(
    point: [f32; 2],
    frame: CellFlowFrame,
    field: &[[f32; 3]; CELL_FLOW_SAMPLES],
) -> Result<[f32; 3], CellFlowError> {
    if !point
        .into_iter()
        .chain(frame.origin)
        .chain(frame.scale)
        .all(f32::is_finite)
    {
        return Err(CellFlowError::NonFiniteInput);
    }
    let coordinates: [f32; 2] = std::array::from_fn(|axis| {
        let shifted = point[axis] - frame.origin[axis];
        shifted * frame.scale[axis]
    });
    if !coordinates.into_iter().all(f32::is_finite) {
        return Err(CellFlowError::NonFiniteIntermediate);
    }
    // Exact source upper-bound words from E4DB80. The MAXSS zero operand
    // commits positive zero when the source coordinate is either signed zero.
    let clamp = |value: f32, upper: f32| {
        let positive = if value > 0.0 { value } else { 0.0 };
        if positive < upper {
            positive
        } else {
            upper
        }
    };
    let x = weights(
        clamp(coordinates[0], f32::from_bits(0x419f_fff8)),
        CELL_FLOW_WIDTH,
    );
    let y = weights(
        clamp(coordinates[1], f32::from_bits(0x416f_fff0)),
        CELL_FLOW_HEIGHT,
    );
    let samples = [
        field[x.base + CELL_FLOW_WIDTH * y.base],
        field[x.neighbor + CELL_FLOW_WIDTH * y.base],
        field[x.base + CELL_FLOW_WIDTH * y.neighbor],
        field[x.neighbor + CELL_FLOW_WIDTH * y.neighbor],
    ];
    if !samples.into_iter().flatten().all(f32::is_finite) {
        return Err(CellFlowError::NonFiniteInput);
    }
    let amounts = [
        y.first * x.first,
        y.first * x.second,
        y.second * x.first,
        y.second * x.second,
    ];
    let mut result = [0.; 3];
    for axis in 0..3 {
        let first = samples[0][axis] * amounts[0];
        let second = samples[1][axis] * amounts[1] + first;
        let third = samples[2][axis] * amounts[2] + second;
        let fourth = if axis == 0 {
            amounts[3] * samples[3][axis]
        } else {
            samples[3][axis] * amounts[3]
        };
        result[axis] = fourth + third;
        if ![first, second, third, fourth, result[axis]]
            .into_iter()
            .all(f32::is_finite)
        {
            return Err(CellFlowError::NonFiniteIntermediate);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_sampling_crosses_half_cells_and_preserves_edge_duplicates() {
        let field = std::array::from_fn(|index| {
            [
                (index % CELL_FLOW_WIDTH) as f32,
                (index / CELL_FLOW_WIDTH) as f32,
                3.,
            ]
        });
        let frame = CellFlowFrame {
            origin: [0.; 2],
            scale: [1.; 2],
        };
        assert_eq!(
            sample_flow_field([1., 1.], frame, &field).unwrap(),
            [0.5, 0.5, 3.]
        );
        assert_eq!(
            sample_flow_field([1.5, 1.5], frame, &field).unwrap(),
            [1., 1., 3.]
        );
        assert_eq!(
            sample_flow_field([-10., -10.], frame, &field).unwrap(),
            [0., 0., 3.]
        );
        assert_eq!(
            sample_flow_field([100., 100.], frame, &field).unwrap(),
            [19., 14., 3.]
        );
    }

    #[test]
    fn only_sampled_vectors_are_consumed_and_nonfinite_domains_are_explicit() {
        let mut field = [[f32::NAN; 3]; CELL_FLOW_SAMPLES];
        field[0] = [1., -2., 3.];
        let frame = CellFlowFrame {
            origin: [0.; 2],
            scale: [1.; 2],
        };
        assert_eq!(
            sample_flow_field([0.; 2], frame, &field).unwrap(),
            [1., -2., 3.]
        );
        assert_eq!(
            sample_flow_field([1.5, 0.], frame, &field),
            Err(CellFlowError::NonFiniteInput)
        );
        assert_eq!(
            sample_flow_field([f32::NAN, 0.], frame, &field),
            Err(CellFlowError::NonFiniteInput)
        );
        assert_eq!(
            sample_flow_field(
                [f32::MAX, 0.],
                CellFlowFrame {
                    scale: [2., 1.],
                    ..frame
                },
                &field
            ),
            Err(CellFlowError::NonFiniteIntermediate)
        );
    }
}
