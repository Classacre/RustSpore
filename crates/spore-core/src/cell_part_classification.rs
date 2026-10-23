//! Original Cell class order and ordinary PropertyList presence semantics.
//!
//! `E827A0` queries `6A1F20`/`612410` through the returned list's vtable. Lists
//! and already bound parent indices are explicit here; archive lookup, package
//! priority and source Parent-key loading remain separate operations.

use std::fmt;

pub const CELL_CLASS_PREDICATES: [(u32, u32); 11] = [
    (0x100f_0f5a, 12),
    (0x11b7_9a74, 2),
    (0x11b7_9a70, 3),
    (0x032f_92e6, 9),
    (0x11b7_9a72, 10),
    (0x11b7_9a71, 7),
    (0x11b7_9a78, 4),
    (0x11b7_9a75, 6),
    (0x11b7_9a76, 5),
    (0x11b7_9a77, 8),
    (0x11b7_9a73, 11),
];

#[derive(Clone, Copy, Debug)]
pub struct CellPartPropertyList<'a> {
    /// Original entry identifiers in stored order. Values/types are not read.
    pub identifiers: &'a [u32],
    /// Already resolved native parent pointer, expressed as a bounded index.
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellPartClassificationError {
    InvalidListIndex,
    CyclicParentChain,
    TooManyPropertyIds,
}
impl fmt::Display for CellPartClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell part classification: {self:?}")
    }
}
impl std::error::Error for CellPartClassificationError {}

/// Exact unsigned lower-bound/presence operation on ordinary native entries.
///
/// Identifiers are not sorted or treated as a set here. Ordinary source lists
/// are sorted, but even malformed ordering keeps the original search behavior.
/// The bound ensures the native signed byte-distance/24 count remains valid.
pub fn native_local_property_present(
    identifiers: &[u32],
    query: u32,
) -> Result<bool, CellPartClassificationError> {
    if identifiers.len() > i32::MAX as usize / 24 {
        return Err(CellPartClassificationError::TooManyPropertyIds);
    }
    let mut first = 0;
    let mut count = identifiers.len();
    while count > 0 {
        let step = count >> 1;
        let candidate = first + step;
        if identifiers[candidate] < query {
            first = candidate + 1;
            count -= step + 1;
        } else {
            count = step;
        }
    }
    Ok(first < identifiers.len() && query >= identifiers[first])
}

/// Query local identifiers first, then the exact bound parent chain.
///
/// A found identifier returns true regardless of its BOOL/numeric value. Invalid
/// parent indices and cycles are checked boundary errors only when reached;
/// original code would dereference or continue that path without these errors.
pub fn native_property_present(
    lists: &[CellPartPropertyList<'_>],
    root: usize,
    query: u32,
) -> Result<bool, CellPartClassificationError> {
    let mut current = Some(root);
    let mut remaining = lists.len();
    while let Some(index) = current {
        let list = lists
            .get(index)
            .ok_or(CellPartClassificationError::InvalidListIndex)?;
        if native_local_property_present(list.identifiers, query)? {
            return Ok(true);
        }
        if remaining == 0 {
            return Err(CellPartClassificationError::CyclicParentChain);
        }
        remaining -= 1;
        current = list.parent;
    }
    Ok(false)
}

/// Classify the already resolved ordinary part list with native first-true order.
///
/// `None` models the original missing-list result and returns class one without
/// inspecting any unused lists. This does not choose an archive or bind a part
/// key to a property list. No property payload is converted to a boolean.
pub fn classify_cell_part(
    lists: &[CellPartPropertyList<'_>],
    root: Option<usize>,
) -> Result<u32, CellPartClassificationError> {
    let Some(root) = root else { return Ok(1) };
    for (property, class) in CELL_CLASS_PREDICATES {
        if native_property_present(lists, root, property)? {
            return Ok(class);
        }
    }
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_class_order_precedes_child_locality_and_values_are_unneeded() {
        let lists = [
            CellPartPropertyList {
                identifiers: &[0x11b7_9a74],
                parent: Some(1),
            },
            CellPartPropertyList {
                identifiers: &[0x100f_0f5a],
                parent: None,
            },
        ];
        assert_eq!(classify_cell_part(&lists, Some(0)), Ok(12));
        let unsorted = [CellPartPropertyList {
            identifiers: &[0x11b7_9a74, 0x100f_0f5a],
            parent: None,
        }];
        assert_eq!(classify_cell_part(&unsorted, Some(0)), Ok(1));
    }

    #[test]
    fn checked_parent_errors_only_apply_to_consumed_lookup_paths() {
        let lists = [CellPartPropertyList {
            identifiers: &[0x100f_0f5a],
            parent: Some(99),
        }];
        assert_eq!(classify_cell_part(&lists, Some(0)), Ok(12));
        assert_eq!(classify_cell_part(&lists, None), Ok(1));
        assert_eq!(
            native_property_present(&lists, 0, 1),
            Err(CellPartClassificationError::InvalidListIndex)
        );
        let cycle = [CellPartPropertyList {
            identifiers: &[],
            parent: Some(0),
        }];
        assert_eq!(
            classify_cell_part(&cycle, Some(0)),
            Err(CellPartClassificationError::CyclicParentChain)
        );
    }
}
