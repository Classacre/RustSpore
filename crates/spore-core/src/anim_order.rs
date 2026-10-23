//! Native child-contiguous ordering for explicitly supplied runtime body blocks.
//!
//! This is the bounded ordering stage in `9C3150` / `9C2FE0`. Its inputs are
//! after native root selection and parent rerooting. Source CreatureData rows
//! are not these inputs: generated blocks, root choice, capability finalization,
//! rest transforms and palette association remain separate operations.

use std::fmt;
use std::ops::Range;

/// Inputs at the ordering boundary, in the runtime block's previous order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeBlock {
    /// Runtime parent ordinal at block+0x1FC, after rerooting; None is FFFFFFFF.
    pub parent: Option<usize>,
    /// Finalized capability mask at block+0x150. Ordering reads only bit 3.
    pub capability_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    Empty,
    RootOutOfBounds {
        root: usize,
        count: usize,
    },
    RootHasParent {
        root: usize,
        parent: usize,
    },
    UnexpectedRoot {
        block: usize,
        root: usize,
    },
    ParentOutOfBounds {
        block: usize,
        parent: usize,
        count: usize,
    },
    ParentCycle {
        block: usize,
    },
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("runtime ordering needs at least one block"),
            Self::RootOutOfBounds { root, count } => {
                write!(f, "root {root} is outside {count} runtime blocks")
            }
            Self::RootHasParent { root, parent } => {
                write!(f, "selected root {root} still has parent {parent}")
            }
            Self::UnexpectedRoot { block, root } => write!(
                f,
                "block {block} is parentless in addition to selected root {root}"
            ),
            Self::ParentOutOfBounds {
                block,
                parent,
                count,
            } => write!(
                f,
                "block {block}'s parent {parent} is outside {count} runtime blocks"
            ),
            Self::ParentCycle { block } => write!(f, "runtime parent cycle includes block {block}"),
        }
    }
}

impl std::error::Error for OrderError {}

/// Topology for one block in the resulting runtime order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedBlock {
    pub original_index: usize,
    /// Parent ordinal in the resulting order, corresponding to block+0x1FC.
    pub parent: Option<usize>,
    /// Contiguous direct children, corresponding to block+0x204 and block+0x208.
    pub children: Range<usize>,
    /// Contiguous descendants, corresponding to block+0x204 and block+0x20C.
    /// This range excludes the block itself and need not follow its ordinal.
    pub descendants: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBlockOrder {
    pub new_to_old: Vec<usize>,
    pub old_to_new: Vec<usize>,
    pub blocks: Vec<OrderedBlock>,
}

/// Order an already rerooted runtime tree through the native bounded operation.
///
/// Previous runtime ordinal determines insertion order. Spine-capable children
/// (mask bit 3) are swapped to the first position; the root retains its first
/// spine child and swaps subsequent ones into the second position. Then each
/// block appends all direct children before visiting their descendants. This
/// differs from ordinary preorder and breadth-first traversal.
///
/// The returned permutation does not change palette identities such as native
/// block+0x21C. Callers must keep those identities separate from runtime ordinals.
pub fn order_runtime_blocks(
    input: &[RuntimeBlock],
    root: usize,
) -> Result<RuntimeBlockOrder, OrderError> {
    let count = input.len();
    if count == 0 {
        return Err(OrderError::Empty);
    }
    if root >= count {
        return Err(OrderError::RootOutOfBounds { root, count });
    }
    if let Some(parent) = input[root].parent {
        return Err(OrderError::RootHasParent { root, parent });
    }
    for (block, value) in input.iter().enumerate() {
        match value.parent {
            Some(parent) if parent >= count => {
                return Err(OrderError::ParentOutOfBounds {
                    block,
                    parent,
                    count,
                });
            }
            None if block != root => return Err(OrderError::UnexpectedRoot { block, root }),
            _ => {}
        }
    }
    // Validate parent chains separately so malformed components never recurse.
    let mut state = vec![0u8; count];
    for start in 0..count {
        let mut chain = Vec::new();
        let mut at = Some(start);
        while let Some(block) = at {
            if state[block] == 1 {
                return Err(OrderError::ParentCycle { block });
            }
            if state[block] == 2 {
                break;
            }
            state[block] = 1;
            chain.push(block);
            at = input[block].parent;
        }
        for block in chain {
            state[block] = 2;
        }
    }

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (block, value) in input.iter().enumerate() {
        let Some(parent) = value.parent else { continue };
        let siblings = &mut children[parent];
        let position = siblings.len();
        siblings.push(block);
        if position != 0 && value.capability_mask & 8 != 0 {
            if parent == root && input[siblings[0]].capability_mask & 8 != 0 {
                if position >= 2 {
                    siblings.swap(1, position);
                }
            } else {
                siblings.swap(0, position);
            }
        }
    }

    let mut new_to_old = Vec::with_capacity(count);
    new_to_old.push(root);
    let mut child_ranges = vec![0..0; count];
    let mut descendant_counts = vec![0usize; count];
    // The explicit stack preserves native recursion order without host-stack
    // growth on a long caller-supplied runtime chain.
    let mut work = vec![(root, false)];
    while let Some((block, visited)) = work.pop() {
        if visited {
            descendant_counts[block] = children[block].len()
                + children[block]
                    .iter()
                    .map(|&child| descendant_counts[child])
                    .sum::<usize>();
        } else {
            let start = new_to_old.len();
            new_to_old.extend_from_slice(&children[block]);
            child_ranges[block] = start..new_to_old.len();
            work.push((block, true));
            for &child in children[block].iter().rev() {
                work.push((child, false));
            }
        }
    }
    let mut old_to_new = vec![0usize; count];
    for (new, &old) in new_to_old.iter().enumerate() {
        old_to_new[old] = new;
    }
    let blocks = new_to_old
        .iter()
        .map(|&old| OrderedBlock {
            original_index: old,
            parent: input[old].parent.map(|parent| old_to_new[parent]),
            children: child_ranges[old].clone(),
            descendants: child_ranges[old].start..child_ranges[old].start + descendant_counts[old],
        })
        .collect();
    Ok(RuntimeBlockOrder {
        new_to_old,
        old_to_new,
        blocks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(parent: Option<usize>, spine: bool) -> RuntimeBlock {
        RuntimeBlock {
            parent,
            capability_mask: if spine { 8 } else { 0 },
        }
    }

    #[test]
    fn children_are_contiguous_before_recursive_descendants() {
        let input = [
            block(Some(4), false),
            block(Some(4), false),
            block(Some(0), false),
            block(Some(0), false),
            block(None, false),
            block(Some(2), false),
            block(Some(1), false),
        ];
        let ordered = order_runtime_blocks(&input, 4).unwrap();
        assert_eq!(ordered.new_to_old, [4, 0, 1, 2, 3, 5, 6]);
        assert_eq!(ordered.old_to_new, [1, 2, 3, 4, 0, 5, 6]);
        assert_eq!(ordered.blocks[0].children, 1..3);
        assert_eq!(ordered.blocks[0].descendants, 1..7);
        assert_eq!(ordered.blocks[1].children, 3..5);
        assert_eq!(ordered.blocks[1].descendants, 3..6);
        assert_eq!(ordered.blocks[2].children, 6..7);
        assert_eq!(ordered.blocks[4].children, 6..6);
        assert_eq!(
            ordered.blocks.iter().map(|b| b.parent).collect::<Vec<_>>(),
            [None, Some(0), Some(0), Some(1), Some(1), Some(3), Some(2)]
        );
    }

    #[test]
    fn spine_swaps_retain_the_roots_first_spine_child() {
        let mut input = vec![block(None, false)];
        input.extend([false, true, true, false, true].map(|spine| block(Some(0), spine)));
        input.extend([false, true, true, false, true].map(|spine| block(Some(1), spine)));
        let ordered = order_runtime_blocks(&input, 0).unwrap();
        assert_eq!(ordered.new_to_old, [0, 2, 5, 1, 4, 3, 10, 6, 7, 9, 8]);
        assert_eq!(ordered.blocks[3].children, 6..11);
        assert_eq!(ordered.blocks[3].descendants, 6..11);
    }

    #[test]
    fn unrelated_capability_bits_leave_order_unchanged() {
        let input = [
            block(None, false),
            block(Some(0), false),
            block(Some(0), false),
        ];
        let expected = order_runtime_blocks(&input, 0).unwrap();
        let changed = input.map(|value| RuntimeBlock {
            capability_mask: !8,
            ..value
        });
        assert_eq!(order_runtime_blocks(&changed, 0).unwrap(), expected);
        let singleton = order_runtime_blocks(&[block(None, true)], 0).unwrap();
        assert_eq!(singleton.blocks[0].children, 1..1);
        assert_eq!(singleton.blocks[0].descendants, 1..1);
    }

    #[test]
    fn invalid_topology_is_typed() {
        assert_eq!(order_runtime_blocks(&[], 0).unwrap_err(), OrderError::Empty);
        assert!(matches!(
            order_runtime_blocks(&[block(None, false)], 1),
            Err(OrderError::RootOutOfBounds { root: 1, count: 1 })
        ));
        assert!(matches!(
            order_runtime_blocks(&[block(Some(0), false)], 0),
            Err(OrderError::RootHasParent { root: 0, parent: 0 })
        ));
        assert!(matches!(
            order_runtime_blocks(&[block(None, false), block(None, false)], 0),
            Err(OrderError::UnexpectedRoot { block: 1, root: 0 })
        ));
        assert!(matches!(
            order_runtime_blocks(&[block(None, false), block(Some(2), false)], 0),
            Err(OrderError::ParentOutOfBounds {
                block: 1,
                parent: 2,
                count: 2
            })
        ));
        let cycle = [
            block(None, false),
            block(Some(2), false),
            block(Some(1), false),
        ];
        assert!(matches!(
            order_runtime_blocks(&cycle, 0),
            Err(OrderError::ParentCycle { .. })
        ));
    }

    #[test]
    fn long_explicit_runtime_chain_uses_no_recursive_host_calls() {
        let input: Vec<_> = (0usize..4096)
            .map(|index| block(index.checked_sub(1), index & 1 != 0))
            .collect();
        let ordered = order_runtime_blocks(&input, 0).unwrap();
        assert_eq!(ordered.new_to_old, (0..4096).collect::<Vec<_>>());
        assert_eq!(ordered.blocks[0].descendants, 1..4096);
        assert_eq!(ordered.blocks[4095].descendants, 4096..4096);
    }
}
