//! Bounded empty-program simple glyph caller, source 8C0582..8C086E.
//!
//! Admits only the observed font0 square 15 ppem profile (flags 0x8b,
//! mode 0, identity posttransform). The input has all real points followed
//! by four simple-glyph phantoms. Empty program takes the 8CC5C0 early
//! return: no VM frame or graphics/CVT/storage mutation is represented.
//! Does not model other flags, transforms, caching, allocation or lifetime.
use crate::spui_font_scaling::{self, Coordinate26Dot6, DesignCoordinate, PowerOfTwoScale};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error { NonEmptyProgram, PointCount, Scale }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultPoints {
    /// Coordinates immediately after the helper bypass.
    pub helper: Vec<[i32; 2]>,
    /// Coordinates after phantom-origin subtraction and horizontal gridlock.
    pub points: Vec<[i32; 2]>,
    pub advance16dot16: i32,
}

pub fn run(program: &[u8], design: &[DesignCoordinate], real_points: usize, units_per_em: u16) -> Result<ResultPoints, Error> {
    if !program.is_empty() { return Err(Error::NonEmptyProgram); }
    if !(4..=92).contains(&real_points) || design.len() != real_points + 4 { return Err(Error::PointCount); }
    // Restriction to the actual retained face, not an assertion about all faces.
    if units_per_em != 2048 { return Err(Error::Scale); }
    let scale = PowerOfTwoScale::for_ppem(15, units_per_em).map_err(|_| Error::Scale)?;
    let helper: Vec<_> = spui_font_scaling::scale_coordinates(design, scale, scale)
        .into_iter().map(|Coordinate26Dot6 { x, y }| [x, y]).collect();
    let mut points = helper.clone();
    let origin = points[real_points];
    for point in &mut points {
        point[0] = point[0].wrapping_sub(origin[0]);
        point[1] = point[1].wrapping_sub(origin[1]);
    }
    let left = points[real_points][0];
    let right = points[real_points + 1][0];
    let aligned_left = left.wrapping_add(32) & !63;
    let aligned_width = right.wrapping_sub(left).wrapping_add(32) & !63;
    points[real_points][0] = aligned_left;
    points[real_points + 1][0] = aligned_left.wrapping_add(aligned_width);
    let advance16dot16 = points[real_points + 1][0].wrapping_sub(points[real_points][0]).wrapping_shl(10);
    Ok(ResultPoints { helper, points, advance16dot16 })
}

/// Source 8C0F7D..0FA8; input is raster's intermediate left/top.
pub fn aligned_origin(left: i32, top: i32, first_phantom: [i32; 2]) -> [i32; 2] {
    [left.wrapping_sub(first_phantom[0]) & !63, top.wrapping_sub(first_phantom[1]) & !63]
}
