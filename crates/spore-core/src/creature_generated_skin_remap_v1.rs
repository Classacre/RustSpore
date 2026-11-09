//! Checked 4CC060 row/endpoint/edge remap over controlled frozen snapshots.
//!
//! The source performs unchecked signed indexing and can loop on class-1
//! cycles. This model rejects invalid indices, unmapped sentinels, cycles and
//! capacity failures transactionally. It assumes stable post-mutation rows,
//! valid native vector allocations and successful prior preparation; it does
//! not authenticate source payload, live object, palette or generated mesh.
use crate::{
    creature_generated_skin_edges_v1 as edges, creature_generated_skin_membership_v1 as membership,
};

pub const MAX_ROWS: usize = 256;
pub const MAX_ENDPOINTS: usize = 4096;
pub const MAX_EDGES: usize = 4096;
#[derive(Clone, Copy, Debug)]
pub struct FrozenPreparedAdmission;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remap {
    pub endpoint_to_row: Vec<u32>,
    pub edge_to_row: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composed {
    pub endpoint_count: usize,
    pub edges: Vec<[u32; 2]>,
    pub members: Vec<Vec<u32>>,
    pub remap: Remap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    EmptyRows,
    TooManyRows,
    TooManyEdges,
    TooFewEdges,
    EndpointCount,
    InvalidLink { row: usize },
    InvalidParent { row: usize },
    ParentCycle { row: usize },
    InvalidExtraEndpoint { edge: usize },
    UnmappedExtraEndpoint { edge: usize },
    AllocationFailed,
    Edges(edges::Error),
    Membership(membership::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "skin row remap: {self:?}")
    }
}
impl std::error::Error for Error {}
fn word(row: &[u8; edges::ROW_SIZE], at: usize) -> i16 {
    i16::from_le_bytes([row[at], row[at + 1]])
}
fn climb(rows: &[[u8; edges::ROW_SIZE]], start: usize) -> Result<u32, Error> {
    let mut i = start;
    for _ in 0..=rows.len() {
        let row = rows.get(i).ok_or(Error::InvalidParent { row: i })?;
        if row[11] != 1 {
            return Ok(i as u32);
        }
        let p = word(row, 4);
        if p < 0 || p as usize >= rows.len() {
            return Err(Error::InvalidParent { row: i });
        }
        i = p as usize;
    }
    Err(Error::ParentCycle { row: start })
}
/// Construct both vectors in native order from stable row and edge snapshots.
/// `u32::MAX` in endpoint_to_row is the source's unassigned sentinel, not a
/// valid row. It is allowed only when no extra edge consumes it.
pub fn build_remap(
    rows: &[[u8; edges::ROW_SIZE]],
    edge_pairs: &[[u32; 2]],
    _admit: FrozenPreparedAdmission,
) -> Result<Remap, Error> {
    if rows.is_empty() {
        return Err(Error::EmptyRows);
    }
    if rows.len() > MAX_ROWS {
        return Err(Error::TooManyRows);
    }
    if edge_pairs.len() > MAX_EDGES {
        return Err(Error::TooManyEdges);
    }
    if edge_pairs.len() < rows.len() {
        return Err(Error::TooFewEdges);
    }
    let n = i32::from(word(&rows[rows.len() - 1], 2)) + 1;
    if n < 0 || n as usize > MAX_ENDPOINTS {
        return Err(Error::EndpointCount);
    }
    let n = n as usize;
    let mut endpoint_to_row = Vec::new();
    endpoint_to_row
        .try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailed)?;
    endpoint_to_row.resize(n, u32::MAX);
    for (i, row) in rows.iter().enumerate() {
        let link = word(row, 2);
        if link == -1 {
            continue;
        }
        if link < 0 || link as usize >= n {
            return Err(Error::InvalidLink { row: i });
        }
        if u16::from_le_bytes([row[8], row[9]]) & 2 != 0 {
            if link == 0 {
                return Err(Error::InvalidLink { row: i });
            }
            endpoint_to_row[link as usize - 1] = i as u32;
        }
        endpoint_to_row[link as usize] = i as u32;
    }
    let mut edge_to_row = Vec::new();
    edge_to_row
        .try_reserve_exact(edge_pairs.len())
        .map_err(|_| Error::AllocationFailed)?;
    for i in 0..rows.len() {
        edge_to_row.push(climb(rows, i)?);
    }
    for (i, pair) in edge_pairs.iter().enumerate().skip(rows.len()) {
        let e = pair[1] as usize;
        let mapped = *endpoint_to_row
            .get(e)
            .ok_or(Error::InvalidExtraEndpoint { edge: i })?;
        if mapped == u32::MAX {
            return Err(Error::UnmappedExtraEndpoint { edge: i });
        }
        let r = mapped as usize;
        let p = word(&rows[r], 4);
        if p < 0 || p as usize >= rows.len() {
            return Err(Error::InvalidParent { row: r });
        }
        edge_to_row.push(climb(rows, p as usize)?);
    }
    Ok(Remap {
        endpoint_to_row,
        edge_to_row,
    })
}
/// Compose the previously bounded edge and membership snapshots with the
/// remap. Native also performs coordinate/query/geometry and other work outside
/// these slices; this function does not run or validate those operations.
pub fn compose(
    rows: &[[u8; edges::ROW_SIZE]],
    _admit: FrozenPreparedAdmission,
) -> Result<Composed, Error> {
    let built =
        edges::build_edges(rows, edges::FrozenPostMutationAdmission).map_err(Error::Edges)?;
    let members = membership::build_membership(
        built.endpoint_count,
        &built.pairs,
        membership::SnapshotAdmission,
    )
    .map_err(Error::Membership)?;
    let remap = build_remap(rows, &built.pairs, FrozenPreparedAdmission)?;
    Ok(Composed {
        endpoint_count: built.endpoint_count,
        edges: built.pairs,
        members,
        remap,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(class: u8, link: i16, parent: i16, flag: u16) -> [u8; edges::ROW_SIZE] {
        let mut r = [0; edges::ROW_SIZE];
        r[2..4].copy_from_slice(&link.to_le_bytes());
        r[4..6].copy_from_slice(&parent.to_le_bytes());
        r[8..10].copy_from_slice(&flag.to_le_bytes());
        r[11] = class;
        r
    }
    #[test]
    fn overwrite_sentinel_and_flagged_extra_order() {
        let rows = [row(4, 0, 0, 0), row(1, 1, 0, 2), row(3, 3, 1, 0)];
        let pairs = [[0, 0], [0, 1], [1, 3], [0, 0]];
        let out = build_remap(&rows, &pairs, FrozenPreparedAdmission).unwrap();
        assert_eq!(out.endpoint_to_row, [1, 1, u32::MAX, 2]);
        assert_eq!(out.edge_to_row, [0, 0, 2, 0]);
        let mut bad = pairs;
        bad[3][1] = 2;
        assert_eq!(
            build_remap(&rows, &bad, FrozenPreparedAdmission),
            Err(Error::UnmappedExtraEndpoint { edge: 3 })
        );
    }
    #[test]
    fn skips_minus_one_but_rejects_other_invalid_and_cycles() {
        let rows = [row(4, -1, 0, 2), row(4, 1, 0, 0)];
        let pairs = [[0, 0], [0, 1]];
        assert_eq!(
            build_remap(&rows, &pairs, FrozenPreparedAdmission)
                .unwrap()
                .endpoint_to_row,
            [u32::MAX, 1]
        );
        let bad = [row(4, -2, 0, 0), row(4, 1, 0, 0)];
        assert_eq!(
            build_remap(&bad, &pairs, FrozenPreparedAdmission),
            Err(Error::InvalidLink { row: 0 })
        );
        let bad = [row(4, 0, 0, 2)];
        assert_eq!(
            build_remap(&bad, &[[0, 0]], FrozenPreparedAdmission),
            Err(Error::InvalidLink { row: 0 })
        );
        let cycle = [row(1, 0, 1, 0), row(1, 1, 0, 0)];
        assert!(matches!(
            build_remap(&cycle, &pairs, FrozenPreparedAdmission),
            Err(Error::ParentCycle { .. })
        ));
        let invalid = [row(1, 0, -1, 0), row(4, 1, 0, 0)];
        assert_eq!(
            build_remap(&invalid, &pairs, FrozenPreparedAdmission),
            Err(Error::InvalidParent { row: 0 })
        );
    }
    #[test]
    fn composed_spine_and_extra() {
        let rows = [
            row(5, 0, 0, 0),
            row(5, 1, 0, 0),
            row(3, 3, 0, 2),
            row(4, 4, 1, 0),
        ];
        let out = compose(&rows, FrozenPreparedAdmission).unwrap();
        assert_eq!(out.edges, [[0, 1], [1, 2], [2, 3], [2, 4], [1, 2]]);
        assert_eq!(out.members[2], [1, 2, 3, 4]);
        assert_eq!(out.remap.endpoint_to_row, [0, 1, 2, 2, 3]);
        assert_eq!(out.remap.edge_to_row, [0, 1, 2, 3, 0]);
    }
}
