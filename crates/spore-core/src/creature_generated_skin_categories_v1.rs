//! Controlled 46E3C0 per-row category words and 467F70 retained prefix.
//!
//! This models only the ordinary branch which truncates trailing class-1 rows
//! without external-count additions. Stable post-mutation rows, successful
//! vector operations, a successful export gate and no intervening mutation of
//! the constructed vector are explicit admissions. Categories are not palette
//! IDs; no model-world binding, geometry regeneration or runtime is implied.
use crate::creature_generated_skin_edges_v1::ROW_SIZE;
pub const MAX_ROWS: usize = 256;
#[derive(Clone, Copy, Debug)]
pub struct FrozenOrdinaryExportAdmission;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Categories {
    pub all_rows: Vec<u32>,
    pub retained_prefix: Vec<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooManyRows,
    AllocationFailed,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "skin category prefix: {self:?}")
    }
}
impl std::error::Error for Error {}
/// Classify all rows in ordinal order, then preserve the prefix ending before
/// the trailing run of class 1. Full and prefix vectors are owned snapshots.
pub fn classify(
    rows: &[[u8; ROW_SIZE]],
    _admit: FrozenOrdinaryExportAdmission,
) -> Result<Categories, Error> {
    if rows.len() > MAX_ROWS {
        return Err(Error::TooManyRows);
    }
    let mut all_rows = Vec::new();
    all_rows
        .try_reserve_exact(rows.len())
        .map_err(|_| Error::AllocationFailed)?;
    for row in rows {
        let flags = u16::from_le_bytes([row[8], row[9]]);
        let value = match row[11] {
            5 => 1,
            3 if flags & 1 != 0 => 2,
            3 | 4 => 3,
            _ if flags & 8 == 0 => 4,
            _ => 5,
        };
        all_rows.push(value);
    }
    let prefix = rows
        .iter()
        .rposition(|row| row[11] != 1)
        .map_or(0, |i| i + 1);
    let mut retained_prefix = Vec::new();
    retained_prefix
        .try_reserve_exact(prefix)
        .map_err(|_| Error::AllocationFailed)?;
    retained_prefix.extend_from_slice(&all_rows[..prefix]);
    Ok(Categories {
        all_rows,
        retained_prefix,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(class: u8, flags: u16) -> [u8; ROW_SIZE] {
        let mut r = [0; ROW_SIZE];
        r[11] = class;
        r[8..10].copy_from_slice(&flags.to_le_bytes());
        r
    }
    #[test]
    fn mixed_classes_flags_and_trailing_only() {
        let rows = [
            row(5, 9),
            row(3, 9),
            row(3, 8),
            row(4, 8),
            row(7, 0),
            row(7, 8),
            row(1, 0),
            row(4, 0),
            row(1, 8),
            row(1, 0),
        ];
        let x = classify(&rows, FrozenOrdinaryExportAdmission).unwrap();
        assert_eq!(x.all_rows, [1, 2, 3, 3, 4, 5, 4, 3, 5, 4]);
        assert_eq!(x.retained_prefix, [1, 2, 3, 3, 4, 5, 4, 3]);
    }
    #[test]
    fn empty_all_class_one_and_capacity() {
        assert!(classify(&[], FrozenOrdinaryExportAdmission)
            .unwrap()
            .retained_prefix
            .is_empty());
        let one = [row(1, 0), row(1, 8)];
        assert_eq!(
            classify(&one, FrozenOrdinaryExportAdmission)
                .unwrap()
                .all_rows,
            [4, 5]
        );
        assert!(classify(&one, FrozenOrdinaryExportAdmission)
            .unwrap()
            .retained_prefix
            .is_empty());
        assert_eq!(
            classify(
                &vec![row(0, 0); MAX_ROWS + 1],
                FrozenOrdinaryExportAdmission
            ),
            Err(Error::TooManyRows)
        );
    }
}
