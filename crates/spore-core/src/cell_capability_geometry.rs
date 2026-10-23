//! Geometry/count/index prefix of original Cell cache builder `E51EC0`.
//!
//! This is the original skip-flag path: it stops before capability curves,
//! CellGlobals lookup, flags and speed masks are updated. Resolved class rows,
//! the three empty-center globals and the x87 arithmetic policy are explicit.

use crate::{
    cell_part_geometry::NativeCellPart,
    native_x87::{Context, Extended, X87Error},
};
use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy, Debug)]
pub struct CellCapabilityPart {
    /// Native byte zero; only literal one uses the doubled/extent branches.
    pub flag: u8,
    /// Raw native class used as the index of one of thirteen counters.
    pub class: u32,
    pub center: [f32; 3],
    pub radius: f32,
}

impl From<NativeCellPart> for CellCapabilityPart {
    fn from(value: NativeCellPart) -> Self {
        Self {
            flag: u8::from(value.word_06_is_ffff),
            class: value.class,
            center: value.center,
            radius: value.radius,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CellCapabilityGeometry {
    pub class_counts: [u32; 13],
    pub center: [f32; 3],
    pub radius: f32,
    pub stat_1284: u32,
    pub stat_1288: u32,
    pub lowest_class12_index: i32,
    selected_indices: Vec<u32>,
}

impl CellCapabilityGeometry {
    pub fn selected_indices(&self) -> &[u32] {
        &self.selected_indices
    }

    /// Write only fields changed by the skip-flag path, relative to native
    /// cache `1204..128F`. Later capability bytes and unused indices survive.
    pub fn write_native_tail(&self, tail: &mut [u8; 140]) {
        for (index, count) in self.class_counts.iter().enumerate() {
            tail[index * 4..index * 4 + 4].copy_from_slice(&count.to_le_bytes());
        }
        for (index, value) in self.center.iter().enumerate() {
            tail[52 + index * 4..56 + index * 4].copy_from_slice(&value.to_bits().to_le_bytes());
        }
        tail[64..68].copy_from_slice(&self.radius.to_bits().to_le_bytes());
        tail[112..116].copy_from_slice(&(self.selected_indices.len() as u32).to_le_bytes());
        for (index, value) in self.selected_indices.iter().enumerate() {
            tail[116 + index * 4..120 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        tail[128..132].copy_from_slice(&self.stat_1284.to_le_bytes());
        tail[132..136].copy_from_slice(&self.stat_1288.to_le_bytes());
        tail[136..140].copy_from_slice(&self.lowest_class12_index.to_le_bytes());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellCapabilityGeometryError {
    TooManyParts,
    InvalidClass,
    NonFiniteInput,
    NonFiniteIntermediate,
    X87(X87Error),
}
impl From<X87Error> for CellCapabilityGeometryError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}
impl fmt::Display for CellCapabilityGeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell capability geometry: {self:?}")
    }
}
impl std::error::Error for CellCapabilityGeometryError {}

fn result(value: f32) -> Result<f32, CellCapabilityGeometryError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CellCapabilityGeometryError::NonFiniteIntermediate)
    }
}

/// Run the complete finite-domain geometry/count/index reduction of `E51EC0`.
///
/// At most 64 rows and classes `0..=12` bound the original counter writes. Class
/// zero is memory-valid here although `E827A0` normally returns `1..=12`.
/// The raw flag is compared with literal one, rather than converted to boolean.
/// SSE operations require MXCSR control `1F80`; retained x87 radius arithmetic
/// uses the supplied policy, including its binary32 store-before-compare step.
/// The original unrolled extent scan is expressed in source row order.
pub fn derive_cell_capability_geometry(
    parts: &[CellCapabilityPart],
    empty_center: [f32; 3],
    context: Context,
) -> Result<CellCapabilityGeometry, CellCapabilityGeometryError> {
    use CellCapabilityGeometryError as Error;
    if parts.len() > 64 {
        return Err(Error::TooManyParts);
    }
    if !empty_center.iter().all(|value| value.is_finite()) {
        return Err(Error::NonFiniteInput);
    }
    for part in parts {
        if part.class > 12 {
            return Err(Error::InvalidClass);
        }
        if !part.radius.is_finite() || !part.center.iter().all(|value| value.is_finite()) {
            return Err(Error::NonFiniteInput);
        }
    }
    let mut minimum_center = empty_center;
    let mut maximum_center = empty_center;
    let mut minimum_y = empty_center[1];
    let mut maximum_y = empty_center[1];
    let mut lowest_class12_y = 10_000.;
    let mut lowest_class12_index = -1;
    for (index, part) in parts.iter().enumerate() {
        if part.flag == 1 {
            let top = result(part.center[1] + part.radius)?;
            if top > maximum_y {
                maximum_center = part.center;
                maximum_y = result(part.radius + part.center[1])?;
            } else {
                let bottom = result(part.center[1] - part.radius)?;
                if minimum_y > bottom {
                    minimum_center = part.center;
                    minimum_y = bottom;
                }
            }
        }
        if part.class == 12 && lowest_class12_y > part.center[1] {
            lowest_class12_y = part.center[1];
            lowest_class12_index = index as i32;
        }
    }
    let center = [
        result(result(minimum_center[0] + maximum_center[0])? * 0.5)?,
        result(result(minimum_y + maximum_y)? * 0.5)?,
        result(result(minimum_center[2] + maximum_center[2])? * 0.5)?,
    ];
    let mut class_counts = [0u32; 13];
    let mut radius = 0.;
    let mut last_negative_flagged_class = 0;
    let mut positive_flagged_class5 = None;
    let mut nonpositive_flagged_class5 = None;
    let mut positive_x_class5 = None;
    let mut nonpositive_x_class5 = None;
    for (index, part) in parts.iter().enumerate() {
        let index = index as u32;
        class_counts[part.class as usize] += if part.flag == 1 { 2 } else { 1 };
        if part.flag == 1 {
            if 0. > part.center[1] {
                last_negative_flagged_class = part.class;
            }
            if part.class == 5 {
                if part.center[1] > 0. {
                    positive_flagged_class5 = Some(index);
                } else {
                    nonpositive_flagged_class5 = Some(index);
                }
            }
        } else if 1_000. >= part.center[1] && part.class == 5 {
            if part.center[0] > 0. {
                positive_x_class5 = Some(index);
            } else {
                nonpositive_x_class5 = Some(index);
            }
        }
        let dx = context.sub(
            Extended::from_f32(center[0])?,
            Extended::from_f32(part.center[0])?,
        )?;
        let dy = context.sub(
            Extended::from_f32(center[1])?,
            Extended::from_f32(part.center[1])?,
        )?;
        let dz = context.sub(
            Extended::from_f32(center[2])?,
            Extended::from_f32(part.center[2])?,
        )?;
        let dx2 = context.mul(dx, dx)?;
        let dz2 = context.mul(dz, dz)?;
        let sum_xz = context.add(dx2, dz2)?;
        let dy2 = context.mul(dy, dy)?;
        let sum_xyz = context.add(sum_xz, dy2)?;
        let candidate = context.add(context.sqrt(sum_xyz)?, Extended::from_f32(part.radius)?)?;
        // FST commits a candidate float while leaving its extended value live.
        let committed = result(context.store_f32(candidate)?)?;
        if context.compare(candidate, Extended::from_f32(radius)?)? == Ordering::Greater {
            radius = committed;
        }
    }
    let mut selected_indices = Vec::with_capacity(3);
    if positive_x_class5.is_none() && nonpositive_x_class5.is_none() {
        selected_indices.extend(positive_flagged_class5.or(nonpositive_flagged_class5));
    } else {
        selected_indices.extend(positive_flagged_class5);
        selected_indices.extend(positive_x_class5);
        selected_indices.extend(nonpositive_x_class5);
    }
    let stat_1284 = (class_counts[10] / 2 * 2 + class_counts[9] / 4 + class_counts[8] / 2).min(10);
    let stat_1288 = if matches!(last_negative_flagged_class, 2 | 4 | 7) && stat_1284 != 0 {
        (class_counts[6] / 2 * 2 + class_counts[7] / 2 + class_counts[5] / 2 + 1).min(10)
    } else {
        0
    };
    Ok(CellCapabilityGeometry {
        class_counts,
        center,
        radius,
        stat_1284,
        stat_1288,
        lowest_class12_index,
        selected_indices,
    })
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
    fn part(flag: u8, class: u32, center: [f32; 3], radius: f32) -> CellCapabilityPart {
        CellCapabilityPart {
            flag,
            class,
            center,
            radius,
        }
    }

    #[test]
    fn extent_else_if_and_literal_one_flag_are_significant() {
        let rows = [part(1, 12, [0.; 3], 10.)];
        let value = derive_cell_capability_geometry(&rows, [0.; 3], context()).unwrap();
        assert_eq!(value.center[1], 5.);
        assert_eq!(value.class_counts[12], 2);
        assert_eq!(value.lowest_class12_index, 0);
        let rows = [part(2, 12, [0.; 3], 10.)];
        let value = derive_cell_capability_geometry(&rows, [0.; 3], context()).unwrap();
        assert_eq!(value.center[1].to_bits(), 0);
        assert_eq!(value.class_counts[12], 1);
    }

    #[test]
    fn ordered_index_selection_preserves_unwritten_fields_and_thresholds() {
        let rows = [
            part(1, 5, [0., 1., 0.], 0.),
            part(1, 5, [0., -1., 0.], 0.),
            part(0, 5, [1., 1_000., 0.], 0.),
            part(255, 5, [-1., 1_001., 0.], 0.),
        ];
        let value = derive_cell_capability_geometry(&rows, [0.; 3], context()).unwrap();
        assert_eq!(value.selected_indices(), &[0, 2]);
        let mut tail = [0xa5; 140];
        value.write_native_tail(&mut tail);
        assert_eq!(&tail[68..112], &[0xa5; 44]);
        assert_eq!(&tail[124..128], &[0xa5; 4]);
        let tied = [
            part(0, 12, [0., 10_000., 0.], 0.),
            part(0, 12, [0., 9_999., 0.], 0.),
            part(0, 12, [0., 9_999., 0.], 0.),
        ];
        assert_eq!(
            derive_cell_capability_geometry(&tied, [0.; 3], context())
                .unwrap()
                .lowest_class12_index,
            1
        );
    }
}
