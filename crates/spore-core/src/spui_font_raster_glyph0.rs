//! Source-derived grid-aligned glyph0 contour scan conversion.
//!
//! This module admits the measured on-curve nested-rectangle path
//! through8C8310/8C8000/8C7320/8C70A0/8C86E0. Input geometry produces events
//! and pixels; no font bytes, output bitmap or source code bytes are embedded.
//! Shape/size/control checks are owned policy, not source malformed recovery.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parameters {
    pub gray_mode: u8,
    pub curve_kind: u8,
    pub dropout_x: bool,
    pub dropout_y: bool,
    pub scan_control_bits17_18: u8,
    pub minimum_precision: i32,
    pub special_mode: bool,
    pub bitmap_transform_or_lut: bool,
    pub postprocess_flags: u32,
}
impl Parameters {
    pub fn source_glyph0(dropout: bool) -> Self {
        Self {
            gray_mode: 3,
            curve_kind: 2,
            dropout_x: dropout,
            dropout_y: dropout,
            scan_control_bits17_18: 0,
            minimum_precision: 0,
            special_mode: false,
            bitmap_transform_or_lut: false,
            postprocess_flags: 0,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnsupportedGrayMode(u8),
    UnsupportedCurveKind(u8),
    UnsupportedControl,
    UnsupportedOutline,
    UnsupportedFractionalPoint,
    Bounds,
    UnbalancedWinding,
    CoverageState,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossingRow {
    pub scan: i32,
    pub events: Vec<i32>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub left26dot6: i32,
    pub top26dot6: i32,
    pub horizontal: Vec<CrossingRow>,
    pub vertical: Vec<CrossingRow>,
    pub alpha: Vec<u8>,
}
fn intervals(events: &[i32]) -> Result<Vec<(i32, i32)>, Error> {
    let mut result = Vec::new();
    let mut at = 0;
    while at < events.len() {
        let mut end = at + 1;
        if end >= events.len() {
            return Err(Error::UnbalancedWinding);
        }
        let direction = |v: i32| if v & 1 == 0 { -1 } else { 1 };
        let mut winding = direction(events[at]) + direction(events[end]);
        while winding != 0 {
            end += 1;
            if end >= events.len() {
                return Err(Error::UnbalancedWinding);
            }
            winding += direction(events[end]);
        }
        result.push((events[at], events[end]));
        at = end + 1;
    }
    Ok(result)
}
fn pixel(bitmap: &Bitmap, x: i32, y: i32) -> Result<usize, Error> {
    let column = x - (bitmap.left26dot6 >> 6);
    let row = (bitmap.top26dot6 >> 6) - y - 1;
    if column < 0 || row < 0 || column >= bitmap.width as i32 || row >= bitmap.height as i32 {
        return Err(Error::Bounds);
    }
    Ok(row as usize * bitmap.pitch + column as usize)
}
/// Original gray second-axis weighted blend8C86E0. Admitted grid inputs yield
/// byte values0..126; no external alpha/gamma convention is substituted.
fn blend(a: u8, b: u8) -> Result<u8, Error> {
    if a > 126 || b > 126 {
        return Err(Error::CoverageState);
    }
    let a = a as i32;
    let b = b as i32;
    let wa = 3970 - (63 - a) * (63 - a);
    let wb = 3970 - (63 - b) * (63 - b);
    Ok(((a * wa + b * wb) / (wa + wb)) as u8)
}
fn add_event(rows: &mut [CrossingRow], scan: i32, value: i32) -> Result<(), Error> {
    let first = rows.first().ok_or(Error::Bounds)?.scan;
    let index = scan - first;
    if index < 0 || index >= rows.len() as i32 {
        return Err(Error::Bounds);
    }
    rows[index as usize].events.push(value);
    Ok(())
}
/// Real points only, original inclusive contour endpoints and on-curve bits.
/// The positive scope is8 grid-aligned points, two strictly nested axis-aligned
/// rectangles in the original contour order. This is an explicit geometry
/// admission gate, not a font identity check or a general rasterizer claim.
pub fn rasterize(
    points: &[[i32; 2]],
    ends: &[u16],
    on_curve: &[bool],
    params: Parameters,
) -> Result<Bitmap, Error> {
    if params.gray_mode != 3 {
        return Err(Error::UnsupportedGrayMode(params.gray_mode));
    }
    if params.curve_kind != 2 {
        return Err(Error::UnsupportedCurveKind(params.curve_kind));
    }
    if params.dropout_x != params.dropout_y
        || params.scan_control_bits17_18 != 0
        || params.minimum_precision != 0
        || params.special_mode
        || params.bitmap_transform_or_lut
        || params.postprocess_flags != 0
    {
        return Err(Error::UnsupportedControl);
    }
    if points.len() != 8 || ends != [3, 7] || on_curve.len() != 8 || on_curve.iter().any(|v| !*v) {
        return Err(Error::UnsupportedOutline);
    }
    if points.iter().flatten().any(|v| v % 64 != 0) {
        return Err(Error::UnsupportedFractionalPoint);
    }
    if points.iter().flatten().any(|v| !(-8192..=8192).contains(v)) {
        return Err(Error::Bounds);
    }
    let [x0, y0] = points[0];
    let [x1, y1] = points[2];
    let [a, b] = points[4];
    let [c, d] = points[6];
    if points[1] != [x0, y1]
        || points[3] != [x1, y0]
        || points[5] != [c, b]
        || points[7] != [a, d]
        || !(x0 < a && a < c && c < x1 && y0 < b && b < d && d < y1)
    {
        return Err(Error::UnsupportedOutline);
    }
    let width = ((x1 + 64) >> 6) - (x0 >> 6);
    let height = ((y1 + 64) >> 6) - (y0 >> 6);
    if !(1..=30).contains(&width) || !(1..=30).contains(&height) {
        return Err(Error::Bounds);
    }
    //8C8310 uses one-pixel padded, inclusive event-list domains; both lists
    //exist under gray mode3. Rust vectors replace owned allocation/provider state.
    let mut horizontal: Vec<_> = (((y0 - 64) >> 6)..=((y1 + 64) >> 6))
        .map(|scan| CrossingRow {
            scan,
            events: Vec::new(),
        })
        .collect();
    let mut vertical: Vec<_> = (((x0 - 64) >> 6)..=((x1 + 64) >> 6))
        .map(|scan| CrossingRow {
            scan,
            events: Vec::new(),
        })
        .collect();
    for start in [0, 4] {
        for i in start..start + 4 {
            let p = points[i];
            let q = points[if i == start + 3 { start } else { i + 1 }];
            if p[1] != q[1] {
                if p[0] != q[0] {
                    return Err(Error::UnsupportedOutline);
                }
                let lo = p[1].min(q[1]);
                let hi = p[1].max(q[1]);
                let encoded = p[0] * 2 + i32::from(q[1] > p[1]);
                //8C7320 samples((minimum+31)&!63)+32, stepping64. Grid endpoints
                //cannot reach its half-grid endpoint tie/large-span interpolation branches.
                let mut sample = ((lo + 31) & !63) + 32;
                while sample <= hi {
                    add_event(&mut horizontal, sample >> 6, encoded)?;
                    sample += 64;
                }
            } else if p[0] != q[0] {
                let lo = p[0].min(q[0]);
                let hi = p[0].max(q[0]);
                let encoded = p[1] * 2 + i32::from(q[0] > p[0]);
                let mut sample = ((lo + 31) & !63) + 32;
                while sample <= hi {
                    add_event(&mut vertical, sample >> 6, encoded)?;
                    sample += 64;
                }
            }
        }
    }
    for row in horizontal.iter_mut().chain(vertical.iter_mut()) {
        row.events.sort_unstable();
    }
    let mut result = Bitmap {
        width: width as usize,
        height: height as usize,
        pitch: width as usize,
        left26dot6: x0,
        top26dot6: y1 + 64,
        horizontal,
        vertical,
        alpha: vec![0; width as usize * height as usize],
    };
    //8C86E0 zeroes the complete pitched allocation before horizontal fill.
    for row_index in 0..result.horizontal.len() {
        let scan = result.horizontal[row_index].scan;
        let runs = intervals(&result.horizontal[row_index].events)?;
        for (start, end) in runs {
            let first = start >> 7;
            let last = end >> 7;
            if first >= last {
                return Err(Error::CoverageState);
            }
            let first_index = pixel(&result, first, scan)?;
            let last_index = pixel(&result, last, scan)?;
            result.alpha[first_index] =
                result.alpha[first_index].wrapping_add(((63 - ((start >> 1) & 63)) * 2) as u8);
            for x in first + 1..last {
                let at = pixel(&result, x, scan)?;
                result.alpha[at] = 126;
            }
            result.alpha[last_index] = (((end >> 1) & 63) * 2) as u8;
        }
    }
    //The original vertical pass blends only interval endpoints and carries a
    //pending last endpoint. Same-pixel reuse is outside the strict grid shape.
    for row_index in 0..result.vertical.len() {
        let scan = result.vertical[row_index].scan;
        let runs = intervals(&result.vertical[row_index].events)?;
        let mut pending: Option<(usize, u8)> = None;
        for (start, end) in runs {
            let first = start >> 7;
            let last = end >> 7;
            if first >= last {
                return Err(Error::CoverageState);
            }
            let first_index = pixel(&result, scan, first)?;
            let last_index = pixel(&result, scan, last)?;
            if let Some((at, value)) = pending.take() {
                if at == first_index {
                    return Err(Error::CoverageState);
                }
                result.alpha[at] = blend(result.alpha[at], value)?;
            }
            let value = ((63 - ((start >> 1) & 63)) * 2) as u8;
            result.alpha[first_index] = blend(result.alpha[first_index], value)?;
            pending = Some((last_index, (((end >> 1) & 63) * 2) as u8));
        }
        if let Some((at, value)) = pending {
            result.alpha[at] = blend(result.alpha[at], value)?;
        }
    }
    Ok(result)
}
