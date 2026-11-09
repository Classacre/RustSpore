//! Bounded `835F40` and `5797D0` string initialization projection.
//!
//! The caller supplies complete, stable, disjoint NUL-terminated strings and
//! admits successful scripted reserve/copy into separate buffers. This does
//! not model source allocation, destruction, reference counts or a live UI.

#[derive(Debug, PartialEq, Eq)]
pub struct Initialized {
    /// Non-terminating UTF-16 units copied by each of the three assignments.
    pub strings: [Vec<u16>; 3],
    /// Requested capacities, including one terminating unit each.
    pub capacities: [usize; 3],
    pub opaque_id: u32,
    pub scalar_bits: [u32; 2],
}

pub fn initialize(inputs: [&[u16]; 3], max_units: usize, opaque_id: u32,
                  scalar_bits: [u32; 2]) -> Option<Initialized> {
    let mut strings = [Vec::new(), Vec::new(), Vec::new()];
    let mut capacities = [0; 3];
    for (i, input) in inputs.into_iter().enumerate() {
        let count = input.iter().position(|&unit| unit == 0)?;
        let capacity = count.checked_add(1)?;
        if capacity > max_units { return None; }
        strings[i].extend_from_slice(&input[..count]);
        capacities[i] = capacity;
    }
    Some(Initialized { strings, capacities, opaque_id, scalar_bits })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_terminated_inputs() {
        let v = initialize([&[65, 0], &[0], &[66, 0]], 2, 7, [0, 1]).unwrap();
        assert_eq!(v.capacities, [2, 1, 2]);
        assert_eq!(v.strings[0], [65]);
        assert!(initialize([&[65], &[0], &[0]], 2, 0, [0; 2]).is_none());
        assert!(initialize([&[65, 0], &[0], &[0]], 1, 0, [0; 2]).is_none());
    }
}
