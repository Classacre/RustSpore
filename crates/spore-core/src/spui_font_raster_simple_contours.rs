//! Unexported original contour-start mutation and implied-start successor.
//! No coordinates or pixels are embedded. Caller geometry produces all events.
//! Original on/off-curve and half-grid start mutation, small-span integer mode and
//! gray3/quadratic2 are explicit admission policy, not general font identity.
pub use crate::spui_font_raster_glyph0::{Bitmap, CrossingRow, Parameters};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Control,
    Outline,
    ContourStart,
    PreviousEndpoint,
    Bounds,
    Depth,
    Fuel,
    Winding,
    Coverage,
}
type Point = [i32; 2];
fn event(rows: &mut [CrossingRow], scan: i32, value: i32) -> Result<(), Error> {
    let first = rows.first().ok_or(Error::Bounds)?.scan;
    let at = usize::try_from(scan - first).map_err(|_| Error::Bounds)?;
    let row = rows.get_mut(at).ok_or(Error::Bounds)?;
    if row.events.len() >= 512 {
        return Err(Error::Fuel);
    }
    row.events.push(value);
    Ok(())
}
struct Edges {
    horizontal: Vec<CrossingRow>,
    vertical: Vec<CrossingRow>,
    previous: [Option<i32>; 2],
    fuel: usize,
}
impl Edges {
    fn line(&mut self, a: Point, b: Point) -> Result<(), Error> {
        if self.fuel == 0 {
            return Err(Error::Fuel);
        }
        self.fuel -= 1;
        //8C7320 small-span vertical list: signed IDIV truncation, no bias.
        if a[0] != b[0] {
            let direction = b[0] > a[0];
            let (lo, hi) = if direction { (a, b) } else { (b, a) };
            let low = lo[0];
            let mut high = hi[0];
            let mut sample = ((low + 31) & !63) + 32;
            if a[0] & 63 == 32 {
                let previous = self.previous[0].ok_or(Error::PreviousEndpoint)?;
                if direction && previous < a[0] {
                    sample += 64
                }
                if !direction && previous > a[0] {
                    high = a[0] - 1
                }
            }
            while sample <= high {
                let value = lo[1] + (sample - low) * (hi[1] - lo[1]) / (hi[0] - lo[0]);
                event(
                    &mut self.vertical,
                    sample >> 6,
                    value * 2 | i32::from(direction),
                )?;
                sample += 64;
            }
        }
        //Horizontal list8C774C uses direction-dependent signed numerator bias.
        if a[1] != b[1] {
            let direction = b[1] > a[1];
            let (lo, hi) = if direction { (a, b) } else { (b, a) };
            let low = lo[1];
            let mut high = hi[1];
            let mut sample = ((low + 31) & !63) + 32;
            if a[1] & 63 == 32 {
                let previous = self.previous[1].ok_or(Error::PreviousEndpoint)?;
                if direction && previous < a[1] {
                    sample += 64
                }
                if !direction && previous > a[1] {
                    high = a[1] - 1
                }
            }
            let denominator = hi[1] - lo[1];
            let delta = hi[0] - lo[0];
            let bias = if (delta >= 0) == direction {
                denominator - 1
            } else {
                0
            };
            while sample <= high {
                let numerator = (sample - low) * delta;
                let quotient = if numerator < 0 {
                    -((bias - numerator) / denominator)
                } else {
                    (numerator + bias) / denominator
                };
                event(
                    &mut self.horizontal,
                    sample >> 6,
                    (lo[0] + quotient) * 2 | i32::from(direction),
                )?;
                sample += 64;
            }
        } else if a[1] & 63 == 32 && a[0] != b[0] {
            //8C7831..8C78D3 inserts both crossings for a half-grid horizontal edge.
            event(&mut self.horizontal, a[1] >> 6, a[0].min(b[0]) * 2 | 1)?;
            event(&mut self.horizontal, a[1] >> 6, a[0].max(b[0]) * 2)?;
        }
        //8C78D5 records the prior segment start only when that axis changed.
        for axis in 0..2 {
            if a[axis] != b[axis] {
                self.previous[axis] = Some(a[axis])
            }
        }
        Ok(())
    }
    fn quadratic(&mut self, a: Point, b: Point, c: Point) -> Result<(), Error> {
        //8C7900 selects one uniform quarter-error depth before subdivision.
        let mut error = ((a[0] - 2 * b[0] + c[0]) >> 2)
            .abs()
            .max(((a[1] - 2 * b[1] + c[1]) >> 2).abs());
        let mut depth = 0;
        while error > 4 {
            error >>= 2;
            depth += 1
        }
        if depth > 8 {
            return Err(Error::Depth);
        }
        let mut stack = vec![(a, b, c, depth)];
        while let Some((a, b, c, n)) = stack.pop() {
            if n == 0 {
                self.line(a, c)?;
                continue;
            }
            let mid = [
                (a[0] + 2 * b[0] + c[0] + 2) >> 2,
                (a[1] + 2 * b[1] + c[1] + 2) >> 2,
            ];
            let left = [(a[0] + b[0] + 1) >> 1, (a[1] + b[1] + 1) >> 1];
            let right = [(b[0] + c[0] + 1) >> 1, (b[1] + c[1] + 1) >> 1];
            stack.push((mid, right, c, n - 1));
            stack.push((a, left, mid, n - 1));
        }
        Ok(())
    }
}
fn intervals(events: &[i32]) -> Result<Vec<(i32, i32)>, Error> {
    let mut result = Vec::new();
    let mut at = 0;
    while at < events.len() {
        let mut end = at + 1;
        if end >= events.len() {
            return Err(Error::Winding);
        }
        let sign = |v: i32| if v & 1 != 0 { 1 } else { -1 };
        let mut winding = sign(events[at]) + sign(events[end]);
        while winding != 0 {
            end += 1;
            if end >= events.len() {
                return Err(Error::Winding);
            }
            winding += sign(events[end])
        }
        result.push((events[at] >> 1, events[end] >> 1));
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
fn blend(a: u8, b: u8) -> Result<u8, Error> {
    if a > 126 || b > 126 {
        return Err(Error::Coverage);
    }
    let (a, b) = (a as i32, b as i32);
    let wa = 3970 - (63 - a) * (63 - a);
    let wb = 3970 - (63 - b) * (63 - b);
    Ok(((a * wa + b * wb) / (wa + wb)) as u8)
}
/// Internal scan-converter metrics: left=xmin and top=ymax+64 before driver
/// phantom subtraction/alignment8C0F7D..0FA8. D/P/U reach aligned origins.
pub fn rasterize(
    points: &[Point],
    ends: &[u16],
    on_curve: &[bool],
    params: Parameters,
) -> Result<Bitmap, Error> {
    if params.gray_mode != 3
        || params.curve_kind != 2
        || params.dropout_x != params.dropout_y
        || params.scan_control_bits17_18 != 0
        || params.minimum_precision != 0
        || params.special_mode
        || params.bitmap_transform_or_lut
        || params.postprocess_flags != 0
    {
        return Err(Error::Control);
    }
    if !(4..=64).contains(&points.len())
        || on_curve.len() != points.len()
        || ends.is_empty()
        || ends.len() > 8
        || ends.last().copied() != Some((points.len() - 1) as u16)
        || ends.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(Error::Outline);
    }
    if points.iter().flatten().any(|v| !(-8192..=8192).contains(v)) {
        return Err(Error::Bounds);
    }
    let minx = points.iter().map(|p| p[0]).min().ok_or(Error::Outline)?;
    let maxx = points.iter().map(|p| p[0]).max().ok_or(Error::Outline)?;
    let miny = points.iter().map(|p| p[1]).min().ok_or(Error::Outline)?;
    let maxy = points.iter().map(|p| p[1]).max().ok_or(Error::Outline)?;
    let rows = |lo: i32, hi: i32| {
        (((lo - 64) >> 6)..=((hi + 64) >> 6))
            .map(|scan| CrossingRow {
                scan,
                events: Vec::new(),
            })
            .collect()
    };
    let mut edges = Edges {
        horizontal: rows(miny, maxy),
        vertical: rows(minx, maxx),
        previous: [None, None],
        fuel: 4096,
    };
    //8C8000 owns mutable coordinate arrays: first/last ties can be decremented.
    let mut points = points.to_vec();
    let mut start = 0;
    for &end in ends {
        let end = end as usize;
        let original_first = points[start];
        let first_on = on_curve[start];
        let last_on = on_curve[end];
        let mut current;
        let anchor;
        let mut i;
        let mut first_control = None;
        if first_on {
            for axis in 0..2 {
                let value = points[start][axis];
                if value & 63 == 32 {
                    edges.previous[axis] = Some(points[end][axis]);
                    if !last_on {
                        points[start][axis] -= 1;
                    } else if points[end][axis] == value {
                        if end != start && on_curve[end - 1] {
                            edges.previous[axis] = Some(points[end - 1][axis]);
                        }
                        if edges.previous[axis] == Some(value) {
                            points[start][axis] -= 1;
                        }
                    }
                }
            }
            current = points[start];
            anchor = Some(start);
            i = if start == end { start } else { start + 1 };
        } else if last_on {
            for axis in 0..2 {
                if points[end][axis] & 63 == 32 {
                    points[end][axis] -= 1;
                }
            }
            current = points[end];
            anchor = Some(end);
            i = start;
            first_control = Some(original_first);
        } else {
            current = [
                (points[start][0] + points[end][0] + 1) >> 1,
                (points[start][1] + points[end][1] + 1) >> 1,
            ];
            for axis in 0..2 {
                if current[axis] & 63 == 32 {
                    points[start][axis] -= 1;
                    points[end][axis] -= 1;
                    current[axis] -= 1;
                }
            }
            anchor = None;
            i = start;
            first_control = Some(original_first);
        }
        let mut traversed = 0;
        loop {
            traversed += 1;
            if traversed > 2 * (end - start + 2) {
                return Err(Error::Fuel);
            }
            let p = if i == start {
                first_control.take().unwrap_or(points[i])
            } else {
                points[i]
            };
            if on_curve[i] {
                edges.line(current, p)?;
                current = p;
                if anchor == Some(i) {
                    break;
                }
                i = if i == end { start } else { i + 1 };
            } else {
                let j = if i == end { start } else { i + 1 };
                let q = points[j];
                let target = if on_curve[j] {
                    q
                } else {
                    [(p[0] + q[0] + 1) >> 1, (p[1] + q[1] + 1) >> 1]
                };
                edges.quadratic(current, p, target)?;
                current = target;
                if anchor == Some(j) && on_curve[j] {
                    break;
                }
                if anchor.is_none() && j == start {
                    break;
                }
                i = if on_curve[j] {
                    if j == end { start } else { j + 1 }
                } else {
                    j
                };
            }
        }
        start = end + 1;
    }
    for r in edges.horizontal.iter_mut().chain(edges.vertical.iter_mut()) {
        r.events.sort_unstable()
    }
    //8C70A0 unions event extrema and populated scan half-grid coordinates.
    let h: Vec<_> = edges
        .horizontal
        .iter()
        .filter(|r| !r.events.is_empty())
        .collect();
    let v: Vec<_> = edges
        .vertical
        .iter()
        .filter(|r| !r.events.is_empty())
        .collect();
    if h.is_empty() || v.is_empty() {
        return Err(Error::Outline);
    }
    let xmin = h
        .iter()
        .map(|r| r.events[0] >> 1)
        .min()
        .unwrap()
        .min(v[0].scan * 64 + 32);
    let xmax = h
        .iter()
        .map(|r| r.events[r.events.len() - 1] >> 1)
        .max()
        .unwrap()
        .max(v[v.len() - 1].scan * 64 + 32);
    let ymin = v
        .iter()
        .map(|r| r.events[0] >> 1)
        .min()
        .unwrap()
        .min(h[0].scan * 64 + 32);
    let ymax = v
        .iter()
        .map(|r| r.events[r.events.len() - 1] >> 1)
        .max()
        .unwrap()
        .max(h[h.len() - 1].scan * 64 + 32);
    let width = ((xmax + 64) >> 6) - (xmin >> 6);
    let height = ((ymax + 64) >> 6) - (ymin >> 6);
    if !(1..=64).contains(&width) || !(1..=64).contains(&height) {
        return Err(Error::Bounds);
    }
    let mut result = Bitmap {
        width: width as usize,
        height: height as usize,
        pitch: width as usize,
        left26dot6: xmin,
        top26dot6: ymax + 64,
        horizontal: edges.horizontal,
        vertical: edges.vertical,
        alpha: vec![0; width as usize * height as usize],
    };
    //8C88CA horizontal same-pixel accumulation and multi-pixel endpoint writes.
    for row in 0..result.horizontal.len() {
        let scan = result.horizontal[row].scan;
        for (a, b) in intervals(&result.horizontal[row].events)? {
            let first = a >> 6;
            let last = b >> 6;
            let at = pixel(&result, first, scan)?;
            if first == last {
                result.alpha[at] = result.alpha[at].wrapping_add((2 * ((b & 63) - (a & 63))) as u8);
            } else {
                result.alpha[at] = result.alpha[at].wrapping_add((2 * (63 - (a & 63))) as u8);
                for x in first + 1..last {
                    let at = pixel(&result, x, scan)?;
                    result.alpha[at] = 126
                }
                let at = pixel(&result, last, scan)?;
                result.alpha[at] = (2 * (b & 63)) as u8;
            }
        }
    }
    //8C89F1 carries endpoint coverage across adjacent intervals in one pixel.
    for row in 0..result.vertical.len() {
        let scan = result.vertical[row].scan;
        let mut pending: Option<(usize, u8)> = None;
        for (a, b) in intervals(&result.vertical[row].events)? {
            let first = a >> 6;
            let last = b >> 6;
            let at = pixel(&result, scan, first)?;
            let mut carry = 0u8;
            if let Some((old, value)) = pending.take() {
                if old == at {
                    carry = value
                } else {
                    result.alpha[old] = blend(result.alpha[old], value)?
                }
            }
            if first == last {
                pending = Some((at, carry.wrapping_add((2 * ((b & 63) - (a & 63))) as u8)));
            } else {
                let value = carry.wrapping_add((2 * (63 - (a & 63))) as u8);
                result.alpha[at] = blend(result.alpha[at], value)?;
                pending = Some((pixel(&result, scan, last)?, (2 * (b & 63)) as u8));
            }
        }
        if let Some((at, value)) = pending {
            result.alpha[at] = blend(result.alpha[at], value)?
        }
    }
    Ok(result)
}
