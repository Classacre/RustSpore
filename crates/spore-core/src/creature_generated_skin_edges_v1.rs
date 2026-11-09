//! Controlled 4CB400 +54 edge-writing slice over frozen 0x8C row snapshots.
//!
//! The caller admits the prior row-map and +40 coordinate preparation succeeded,
//! edge storage was freshly zero-initialized, native allocations/vector operations
//! succeeded, and snapshots reflect any preceding 471D90 mutation. This models
//! neither that preparation nor runtime identity, geometry or palette. Checked
//! bounds and transactional errors are stricter than native writes.

pub const ROW_SIZE: usize = 0x8c;
pub const MAX_ROWS: usize = 256;
pub const MAX_ENDPOINTS: usize = 4096;
#[derive(Clone, Copy, Debug)]
pub struct FrozenPostMutationAdmission;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edges {
    pub endpoint_count: usize,
    pub pairs: Vec<[u32; 2]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    EmptyRows,
    TooManyRows,
    UnsupportedEndpointCount,
    InvalidParent { row: usize },
    InvalidEndpoint { edge: usize },
    AllocationFailed,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "skin edge construction: {self:?}")
    }
}
impl std::error::Error for Error {}
fn signed(row: &[u8; ROW_SIZE], offset: usize) -> i16 {
    i16::from_le_bytes([row[offset], row[offset + 1]])
}
fn flag(row: &[u8; ROW_SIZE]) -> bool {
    u16::from_le_bytes([row[8], row[9]]) & 2 != 0
}
fn checked_parent(row: &[u8; ROW_SIZE], index: usize, limit: usize) -> Result<usize, Error> {
    let p = signed(row, 4);
    if p < 0 || p as usize >= limit {
        Err(Error::InvalidParent { row: index })
    } else {
        Ok(p as usize)
    }
}
/// Execute the edge-writing substage. The initial zeroes matter if a parent
/// references an edge not yet written. Rows and their backing bytes are never
/// modified; no partially constructed result is returned on error.
pub fn build_edges(
    rows: &[[u8; ROW_SIZE]],
    _admit: FrozenPostMutationAdmission,
) -> Result<Edges, Error> {
    if rows.is_empty() {
        return Err(Error::EmptyRows);
    }
    if rows.len() > MAX_ROWS {
        return Err(Error::TooManyRows);
    }
    let endpoint_count = i32::from(signed(&rows[rows.len() - 1], 2)) + 1;
    if endpoint_count <= 0 || endpoint_count as usize > MAX_ENDPOINTS {
        return Err(Error::UnsupportedEndpointCount);
    }
    let endpoint_count = endpoint_count as usize;
    let spine = rows.iter().take_while(|r| r[11] == 5).count();
    let class3_end = spine + rows[spine..].iter().take_while(|r| r[11] == 3).count();
    let extras = if spine == 0 {
        0
    } else {
        rows[spine..class3_end].iter().filter(|r| flag(r)).count()
    };
    let mut pairs = Vec::new();
    pairs
        .try_reserve_exact(rows.len() + extras)
        .map_err(|_| Error::AllocationFailed)?;
    pairs.resize(rows.len() + extras, [0, 0]);
    for (i, pair) in pairs.iter_mut().enumerate().take(spine) {
        *pair = [i as u32, (i + 1) as u32];
    }
    for i in spine..class3_end {
        let r = &rows[i];
        let link = i32::from(signed(r, 2));
        let first = if flag(r) {
            link - 1
        } else {
            i32::from(signed(&rows[checked_parent(r, i, rows.len())?], 2))
        };
        if first < 0 || link < 0 {
            return Err(Error::InvalidEndpoint { edge: i });
        }
        pairs[i] = [first as u32, link as u32];
    }
    for i in class3_end..rows.len() {
        let r = &rows[i];
        let link = i32::from(signed(r, 2));
        let first = if spine == 0 {
            if flag(r) {
                link - 1
            } else {
                i32::from(signed(&rows[checked_parent(r, i, rows.len())?], 2))
            }
        } else {
            let parent = checked_parent(r, i, pairs.len())?;
            let side = usize::from(parent >= spine / 2);
            pairs[parent][side] as i32
        };
        if first < 0 || link < 0 {
            return Err(Error::InvalidEndpoint { edge: i });
        }
        pairs[i] = [first as u32, link as u32];
    }
    if spine != 0 {
        let mut out = rows.len();
        for (i, r) in rows.iter().enumerate().take(class3_end).skip(spine) {
            if flag(r) {
                let parent = checked_parent(r, i, rows.len())?;
                let second = i32::from(signed(r, 2)) - 1;
                if second < 0 {
                    return Err(Error::InvalidEndpoint { edge: out });
                }
                pairs[out] = [pairs[parent][1], second as u32];
                out += 1;
            }
        }
    }
    for (i, pair) in pairs.iter().enumerate() {
        if pair.iter().any(|&v| v as usize >= endpoint_count) {
            return Err(Error::InvalidEndpoint { edge: i });
        }
    }
    Ok(Edges {
        endpoint_count,
        pairs,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(kind: u8, link: i16, parent: i16, flags: u16) -> [u8; ROW_SIZE] {
        let mut r = [0; ROW_SIZE];
        r[2..4].copy_from_slice(&link.to_le_bytes());
        r[4..6].copy_from_slice(&parent.to_le_bytes());
        r[8..10].copy_from_slice(&flags.to_le_bytes());
        r[11] = kind;
        r
    }
    #[test]
    fn spine_flagged_class3_and_remaining_branch() {
        let r = [
            row(5, 0, 0, 0),
            row(5, 1, 0, 0),
            row(3, 3, 0, 2),
            row(4, 4, 1, 0),
        ];
        let got = build_edges(&r, FrozenPostMutationAdmission).unwrap();
        assert_eq!(got.endpoint_count, 5);
        assert_eq!(got.pairs, [[0, 1], [1, 2], [2, 3], [2, 4], [1, 2]]);
        assert_eq!(r[2][8], 2);
    }
    #[test]
    fn forward_parent_reads_zero_then_self_and_parallel_edges() {
        let r = [
            row(5, 0, 0, 0),
            row(5, 1, 0, 0),
            row(4, 2, 3, 0),
            row(4, 3, 0, 0),
        ];
        assert_eq!(
            build_edges(&r, FrozenPostMutationAdmission).unwrap().pairs,
            [[0, 1], [1, 2], [0, 2], [0, 3]]
        );
        let r = [row(4, 0, 0, 0), row(4, 0, 0, 0)];
        assert_eq!(
            build_edges(&r, FrozenPostMutationAdmission).unwrap().pairs,
            [[0, 0], [0, 0]]
        );
    }
    #[test]
    fn no_spine_class3_and_invalid_inputs_are_transactional() {
        let r = [row(3, 0, 0, 0), row(3, 1, 0, 2), row(4, 2, 1, 0)];
        assert_eq!(
            build_edges(&r, FrozenPostMutationAdmission).unwrap().pairs,
            [[0, 0], [0, 1], [1, 2]]
        );
        let r = [row(4, 0, 0, 0), row(4, 1, -1, 0)];
        // Parent 3 is within the four-edge allocation but outside three rows;
        // the extra path also copies its transform from rows[parent].
        let extra = [row(5, 0, 0, 0), row(3, 1, 3, 2), row(4, 2, 0, 0)];
        assert_eq!(
            build_edges(&extra, FrozenPostMutationAdmission),
            Err(Error::InvalidParent { row: 1 })
        );
        let original = r;
        assert!(matches!(
            build_edges(&r, FrozenPostMutationAdmission),
            Err(Error::InvalidParent { row: 1 })
        ));
        assert_eq!(r, original);
        assert_eq!(
            build_edges(&[], FrozenPostMutationAdmission),
            Err(Error::EmptyRows)
        );
        assert_eq!(
            build_edges(
                &vec![row(4, 0, 0, 0); MAX_ROWS + 1],
                FrozenPostMutationAdmission
            ),
            Err(Error::TooManyRows)
        );
        assert!(matches!(
            build_edges(&[row(3, 0, 0, 2)], FrozenPostMutationAdmission),
            Err(Error::InvalidEndpoint { .. })
        ));
    }
}
