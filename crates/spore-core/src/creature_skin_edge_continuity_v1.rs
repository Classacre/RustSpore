//! Checked 50B210 directed-edge continuity predicate.
//!
//! The four indices address a supplied per-corner +6C stream. This does not
//! infer a live UV seam, execute the face-group traversal or establish object identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { InvalidLane, FaceOutOfRange }

pub fn matches(indices: &[u32], face_a: u32, lane_a: u32, face_b: u32, lane_b: u32) -> Result<bool, Error> {
    if lane_a >= 3 || lane_b >= 3 { return Err(Error::InvalidLane); }
    let edge = |face: u32, lane: u32| -> Result<(u32, u32), Error> {
        let start = (face as usize).checked_mul(3).ok_or(Error::FaceOutOfRange)?;
        let end = start.checked_add(3).ok_or(Error::FaceOutOfRange)?;
        let triangle = indices.get(start..end).ok_or(Error::FaceOutOfRange)?;
        Ok((triangle[lane as usize], triangle[((lane + 1) % 3) as usize]))
    };
    let (a, b) = edge(face_a, lane_a)?;
    let (c, d) = edge(face_b, lane_b)?;
    Ok(a == d && b == c)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compares_reversed_endpoints_and_rejects_invalid_indices() {
        let mut values = [2, 5, 7, 5, 2, 8];
        assert_eq!(matches(&values, 0, 0, 1, 0), Ok(true));
        values[4] = 3; assert_eq!(matches(&values, 0, 0, 1, 0), Ok(false));
        values[4] = 2; values[3] = 9; assert_eq!(matches(&values, 0, 0, 1, 0), Ok(false));
        assert_eq!(matches(&values, 0, 3, 1, 0), Err(Error::InvalidLane));
        assert_eq!(matches(&values, 0, 0, u32::MAX, 0), Err(Error::FaceOutOfRange));
        assert_eq!(matches(&values[..5], 0, 0, 1, 0), Err(Error::FaceOutOfRange));
    }
}
