//! Bounded construction of +40 endpoint membership from +54 edge snapshots.
//!
//! Native 4CBF96..4CC004 appends each edge ordinal to its first and then
//! second endpoint. This controlled bridge assumes stable, valid snapshots and
//! successful native vector appends; it does not construct edges, associate a
//! CreatureData payload or establish palette, generated mesh or live identity.
//! Bounds, allocation errors and transactional output are stricter than native.

/// Model limits, not native limits.
pub const MAX_ENDPOINTS: usize = 4096;
pub const MAX_EDGES: usize = 4096;

/// Caller admits stable, nonaliasing source arrays and successful native vector
/// operations, including the grow path. Existing native member vectors are
/// assumed empty as prepared by the caller; only newly formed lists are modeled.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotAdmission;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyEndpoints,
    TooManyEdges,
    MissingEndpoint { edge: usize, endpoint: u32 },
    AllocationFailed,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "skin edge membership: {self:?}")
    }
}
impl std::error::Error for Error {}

/// Returns one ordered list per endpoint. Each edge contributes exactly twice,
/// even when both indices are equal. Inputs are untouched on any failure.
pub fn build_membership(
    endpoint_count: usize,
    edges: &[[u32; 2]],
    _admission: SnapshotAdmission,
) -> Result<Vec<Vec<u32>>, Error> {
    if endpoint_count > MAX_ENDPOINTS {
        return Err(Error::TooManyEndpoints);
    }
    if edges.len() > MAX_EDGES {
        return Err(Error::TooManyEdges);
    }
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(endpoint_count)
        .map_err(|_| Error::AllocationFailed)?;
    counts.resize(endpoint_count, 0usize);
    for (i, edge) in edges.iter().enumerate() {
        for &endpoint in edge {
            let count = counts
                .get_mut(endpoint as usize)
                .ok_or(Error::MissingEndpoint { edge: i, endpoint })?;
            *count += 1; // At most twice MAX_EDGES, well below usize capacity.
        }
    }
    let mut lists = Vec::new();
    lists
        .try_reserve_exact(endpoint_count)
        .map_err(|_| Error::AllocationFailed)?;
    for count in counts {
        let mut list = Vec::new();
        list.try_reserve_exact(count)
            .map_err(|_| Error::AllocationFailed)?;
        lists.push(list);
    }
    for (i, edge) in edges.iter().enumerate() {
        lists[edge[0] as usize].push(i as u32);
        lists[edge[1] as usize].push(i as u32);
    }
    Ok(lists)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_order_self_edges_and_parallel_edges() {
        let edges = [[2, 0], [1, 1], [2, 0], [0, 1]];
        assert_eq!(
            build_membership(4, &edges, SnapshotAdmission).unwrap(),
            vec![vec![0, 2, 3], vec![1, 1, 3], vec![0, 2], vec![]]
        );
        assert_eq!(edges, [[2, 0], [1, 1], [2, 0], [0, 1]]);
    }
    #[test]
    fn empty_and_boundaries() {
        assert_eq!(
            build_membership(0, &[], SnapshotAdmission).unwrap(),
            Vec::<Vec<u32>>::new()
        );
        assert_eq!(
            build_membership(2, &[], SnapshotAdmission).unwrap(),
            vec![vec![], vec![]]
        );
        let edges = vec![[0, 0]; MAX_EDGES];
        let result = build_membership(1, &edges, SnapshotAdmission).unwrap();
        assert_eq!(result[0].len(), 2 * MAX_EDGES);
        assert_eq!(&result[0][..4], &[0, 0, 1, 1]);
        assert_eq!(
            &result[0][result[0].len() - 2..],
            &[(MAX_EDGES - 1) as u32; 2]
        );
    }
    #[test]
    fn rejects_invalid_endpoints_and_capacities() {
        let edges = [[0, 0], [0, u32::MAX]];
        assert_eq!(
            build_membership(1, &edges, SnapshotAdmission),
            Err(Error::MissingEndpoint {
                edge: 1,
                endpoint: u32::MAX
            })
        );
        assert_eq!(edges[0], [0, 0]);
        assert_eq!(
            build_membership(MAX_ENDPOINTS + 1, &[], SnapshotAdmission),
            Err(Error::TooManyEndpoints)
        );
        assert_eq!(
            build_membership(1, &vec![[0, 0]; MAX_EDGES + 1], SnapshotAdmission),
            Err(Error::TooManyEdges)
        );
    }
}
