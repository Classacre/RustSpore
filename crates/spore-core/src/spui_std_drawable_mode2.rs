//! Bounded StdDrawable mode-2 rectangle emission from source 95D120.
//! By-value inputs exclude pointer aliasing and assume the native draw callback
//! does not mutate its input rectangles between calls. Finite checks are model admission,
//! not native behavior. Requires ordinary round-to-nearest binary32; native
//! MXCSR rounding, FTZ/DAZ, clipping, effects and receiver state are unverified.
use crate::spui_layout::LayoutError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageRectCall {
    pub destination: [f32; 4],
    pub relative_uv: [f32; 4],
}

/// Produce the calls made to Graphics2D slot +58, before atlas UV mapping.
/// Inputs correspond to destination, image width/height, relative UV rectangle,
/// split rectangle and x/y scale. Neither ordered rectangles nor positive image
/// dimensions are required by the observed arithmetic; nonfinite results fail.
pub fn mode2_calls(
    destination: [f32; 4], image_size: [i32; 2], uv: [f32; 4],
    split: [f32; 4], scale: [f32; 2],
) -> Result<Vec<ImageRectCall>, LayoutError> {
    if destination.iter().chain(&uv).chain(&split).chain(&scale).any(|x| !x.is_finite()) {
        return Err(LayoutError::NonfiniteRectangle);
    }
    let u_span = uv[2] - uv[0];
    let v_span = uv[3] - uv[1];
    let width = (image_size[0] as f32 * u_span) * scale[0];
    let height = (image_size[1] as f32 * v_span) * scale[1];
    let mut x1 = (split[0] * width) + destination[0];
    let mut y1 = (split[1] * height) + destination[1];
    let mut x2 = destination[2] - ((1.0 - split[2]) * width);
    let mut y2 = destination[3] - ((1.0 - split[3]) * height);
    if x1 > x2 { let midpoint = (x2 + x1) * 0.5; x1 = midpoint; x2 = midpoint; }
    if y1 > y2 { let midpoint = (y2 + y1) * 0.5; y1 = midpoint; y2 = midpoint; }
    let u1 = (u_span * split[0]) + uv[0];
    let v1 = (v_span * split[1]) + uv[1];
    let u2 = (u_span * split[2]) + uv[0];
    let v2 = (v_span * split[3]) + uv[1];
    if [u_span, v_span, width, height, x1, y1, x2, y2, u1, v1, u2, v2].iter().any(|x| !x.is_finite()) {
        return Err(LayoutError::NonfiniteRectangle);
    }
    // Native computes these subtractions before its strip branches; the
    // diagnostic admits only finite intermediates, even when edges are finite.
    let gap_x = x2 - x1;
    let gap_y = y2 - y1;
    if !gap_x.is_finite() || !gap_y.is_finite() {
        return Err(LayoutError::NonfiniteRectangle);
    }
    let x = [destination[0], x1, x2, destination[2]];
    let y = [destination[1], y1, y2, destination[3]];
    let u = [uv[0], u1, u2, uv[2]];
    let v = [uv[1], v1, v2, uv[3]];
    let mut calls = Vec::with_capacity(9);
    let mut emit = |col: usize, row: usize| calls.push(ImageRectCall {
        destination: [x[col], y[row], x[col + 1], y[row + 1]],
        relative_uv: [u[col], v[row], u[col + 1], v[row + 1]],
    });
    emit(0, 0); emit(2, 0); emit(0, 2); emit(2, 2);
    if gap_x > 0.0 { emit(1, 0); emit(1, 2); }
    if gap_y > 0.0 { emit(0, 1); emit(2, 1); }
    if x1 != x2 && y1 != y2 { emit(1, 1); }
    Ok(calls)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nine_calls_in_source_order_and_collapsed_center() {
        let calls = mode2_calls([0., 0., 90., 60.], [30, 30], [0., 0., 1., 1.], [0.25, 0.25, 0.75, 0.75], [1., 1.]).unwrap();
        assert_eq!(calls.len(), 9);
        assert_eq!(calls[0].destination, [0., 0., 7.5, 7.5]);
        assert_eq!(calls[8].destination, [7.5, 7.5, 82.5, 52.5]);
        assert_eq!(mode2_calls([0., 0., 2., 2.], [30, 30], [0., 0., 1., 1.], [0.25, 0.25, 0.75, 0.75], [1., 1.]).unwrap().len(), 4);
    }
    #[test]
    fn rejects_nonfinite_and_overflow() {
        assert!(mode2_calls([0.; 4], [1, 1], [0., 0., f32::NAN, 1.], [0.; 4], [1.; 2]).is_err());
        assert!(mode2_calls([0.; 4], [i32::MAX, 1], [0., 0., f32::MAX, 1.], [0.; 4], [1.; 2]).is_err());
        assert!(mode2_calls([-f32::MAX, -1., f32::MAX, 1.], [0, 0], [0., 0., 1., 1.], [0., 0., 1., 1.], [1.; 2]).is_err());
    }
}
