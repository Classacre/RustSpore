//! Bounded native body topology, separate from transform and selector fields.
//!
//! `9CBFA0` initially permutes source rows around the first row's parent.
//! `9C3150` selects a root, reverses its parent chain, then applies the native
//! child-contiguous order. This module projects those topology fields only.
//! Optional foot construction, anchor changes, local quaternions, capability
//! relabeling after ordering and selector classes are separate operations.

use crate::anim_body_capabilities::{prepare_source_capabilities, BodyCapabilitiesError};
use crate::anim_body_source::{source_rest_blocks, BodySourceError};
use crate::anim_order::{order_runtime_blocks, OrderError, RuntimeBlock};
use crate::creature_data::{CreatureData, PART_ROW_SIZE};
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;
use crate::part_capabilities::PartCapabilities;
use std::{fmt, ops::Range};

pub const MAX_RUNTIME_BLOCKS: usize = 255;

/// Caller-supplied values of the original two root-selection globals.
///
/// The on-disk values are not a runtime policy guarantee: another original
/// caller temporarily changes both globals. Both choices stay explicit here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootPolicy {
    /// Global166CFE4: prefer the nearest presence-mask spine to the origin.
    pub nearest_origin: bool,
    /// Global1551A84: use the terminal-spine parent walk without foot votes.
    pub spine_midpoint_fallback: bool,
}

/// Inputs at the native finalizer boundary, including any generated blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyBlock {
    pub parent: Option<usize>,
    /// The word already written to runtime block+200, interpreted as an old
    /// runtime ordinal. It is distinct from a source row's symmetry ordinal.
    pub symmetric: Option<usize>,
    /// Native block+21C, independent of the current runtime ordinal.
    /// Generated feet may use FFFFFFFF; source rows retain their source index.
    pub source_palette_id: u32,
    /// Absolute rest center at block+10C..114, before runtime ordering.
    pub center: [f32; 3],
    /// Prepared native mask at block+150.
    pub presence_mask: u32,
    /// Prepared native mask at block+154.
    pub nonzero_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyError {
    Count(usize),
    InitialRootHasParent(usize),
    InitialRootMissingNonzeroSpine,
    ParentOutOfBounds { block: usize, parent: usize },
    AdditionalRoot(usize),
    ParentCycle(usize),
    SymmetryOutOfBounds { block: usize, symmetric: usize },
    NonFiniteCenter { block: usize, component: usize },
    FootWithoutSpineAncestor(usize),
    NoSelectedRoot,
    SelectedRootMissingNonzeroSpine(usize),
    SourceRowStorage { rows: usize, bytes: usize },
    SourceRootNomination(usize),
    OptionalFootConstructionRequired,
    Sse(SseError),
    Capabilities(BodyCapabilitiesError),
    Geometry(BodySourceError),
    Order(OrderError),
}
impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native body topology: {self:?}")
    }
}
impl std::error::Error for TopologyError {}
impl From<SseError> for TopologyError {
    fn from(error: SseError) -> Self {
        Self::Sse(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSelectionBranch {
    NearestOrigin,
    AuthoredRoot,
    HighestFootCandidate,
    TerminalSpineParentWalk,
    InitialRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSelection {
    pub root: usize,
    /// Definition+3F4: whether the accumulated maximum-foot list is empty.
    pub no_foot_candidates: bool,
    pub foot_votes: Vec<u32>,
    /// Original list order is significant for equal-height candidates.
    pub foot_candidates: Vec<usize>,
    pub branch: RootSelectionBranch,
}

/// Projection immediately before the native selected-root admission test.
/// A presence-only spine and the unset FFFFFFFF word remain observable here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootCandidate {
    pub root: Option<usize>,
    pub no_foot_candidates: bool,
    pub foot_votes: Vec<u32>,
    pub foot_candidates: Vec<usize>,
    pub branch: RootSelectionBranch,
}

fn validate(input: &[TopologyBlock]) -> Result<(), TopologyError> {
    if input.is_empty() || input.len() > MAX_RUNTIME_BLOCKS {
        return Err(TopologyError::Count(input.len()));
    }
    if let Some(parent) = input[0].parent {
        return Err(TopologyError::InitialRootHasParent(parent));
    }
    for (i, block) in input.iter().enumerate() {
        if let Some(parent) = block.parent {
            if parent >= input.len() {
                return Err(TopologyError::ParentOutOfBounds { block: i, parent });
            }
        } else if i != 0 {
            return Err(TopologyError::AdditionalRoot(i));
        }
        if let Some(symmetric) = block.symmetric {
            if symmetric >= input.len() {
                return Err(TopologyError::SymmetryOutOfBounds {
                    block: i,
                    symmetric,
                });
            }
        }
        for (component, value) in block.center.iter().enumerate() {
            if !value.is_finite() {
                return Err(TopologyError::NonFiniteCenter {
                    block: i,
                    component,
                });
            }
        }
    }
    // Check every edge before following any chain: a later row's invalid
    // parent must not become an unchecked subscript from an earlier row.
    for i in 0..input.len() {
        let mut at = Some(i);
        for step in 0..=input.len() {
            let Some(index) = at else { break };
            if step == input.len() {
                return Err(TopologyError::ParentCycle(i));
            }
            at = input[index].parent;
        }
    }
    Ok(())
}

// 9C38B7..9C3A32. The four-way loop's second lane and the tail add z^2
// before y^2. Other unrolled lanes add y^2 before z^2. No FMA is used.
fn native_squared_distance(center: [f32; 3], z_before_y: bool) -> f32 {
    let [x, y, z] = center;
    if z_before_y {
        (x * x + z * z) + y * y
    } else {
        (x * x + y * y) + z * z
    }
}

/// Select the root through the bounded native policy, with original SSE order.
///
/// The caller supplies a connected runtime tree with initial root ordinal0,
/// including optional generated feet if the loader required them. Centers must
/// be finite; their squared distances may overflow naturally. Binary32 work
/// installs MXCSR1F80 and restores the complete calling state on every exit.
/// This does not infer the retail process's later floating-point context.
///
/// The original foot-vote loop would index before its counter array if a foot
/// has no spine ancestor. This interface reports that unsafe input explicitly.
/// Root candidates exactly equal to -MAX also leave the native selected word
/// unset; that boundary is an explicit error rather than an invented tie rule.
pub fn select_root(
    input: &[TopologyBlock],
    policy: RootPolicy,
) -> Result<RootSelection, TopologyError> {
    let candidate = select_root_candidate(input, policy)?;
    let root = candidate.root.ok_or(TopologyError::NoSelectedRoot)?;
    if input[root].nonzero_mask & 8 == 0 {
        return Err(TopologyError::SelectedRootMissingNonzeroSpine(root));
    }
    Ok(RootSelection {
        root,
        no_foot_candidates: candidate.no_foot_candidates,
        foot_votes: candidate.foot_votes,
        foot_candidates: candidate.foot_candidates,
        branch: candidate.branch,
    })
}

/// Execute the native vote and root-candidate region before its final +154
/// admission check. See `select_root` for input and floating-point bounds.
pub fn select_root_candidate(
    input: &[TopologyBlock],
    policy: RootPolicy,
) -> Result<RootCandidate, TopologyError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        select_root_candidate_scoped(input, policy)
    })
}

fn select_root_candidate_scoped(
    input: &[TopologyBlock],
    policy: RootPolicy,
) -> Result<RootCandidate, TopologyError> {
    validate(input)?;
    if input[0].nonzero_mask & 8 == 0 {
        return Err(TopologyError::InitialRootMissingNonzeroSpine);
    }
    let mut votes = vec![0u32; input.len()];
    let mut candidates = Vec::new();
    let mut maximum_votes = 0;
    let mut authored = None;
    for (i, block) in input.iter().enumerate() {
        // This break occurs before counting the current block's foot bit.
        if block.presence_mask & 1 != 0 {
            authored = Some(i);
            break;
        }
        if block.nonzero_mask & 4 == 0 {
            continue;
        }
        let mut at = block.parent;
        while let Some(index) = at {
            if input[index].nonzero_mask & 8 != 0 {
                break;
            }
            at = input[index].parent;
        }
        let ancestor = at.ok_or(TopologyError::FootWithoutSpineAncestor(i))?;
        votes[ancestor] += 1;
        if votes[ancestor] > maximum_votes {
            candidates.clear();
            maximum_votes = votes[ancestor];
        }
        if votes[ancestor] == maximum_votes {
            candidates.push(ancestor);
        }
    }
    let no_foot_candidates = candidates.is_empty();
    let mut selected = None;
    let mut branch = RootSelectionBranch::InitialRoot;
    if policy.nearest_origin {
        let mut best = f32::MAX;
        let unrolled_end = input.len() / 4 * 4;
        for (i, block) in input.iter().enumerate() {
            // The source reads presence at+150 here, unlike the other spine
            // predicates. A zero-number nearest spine may fail admission below.
            if block.presence_mask & 8 == 0 {
                continue;
            }
            let distance = native_squared_distance(block.center, i >= unrolled_end || i % 4 == 1);
            if best > distance {
                best = distance;
                selected = Some(i);
            }
        }
        if selected.is_some() {
            branch = RootSelectionBranch::NearestOrigin;
        }
    }
    if selected.is_none() {
        selected = authored;
        if selected.is_some() {
            branch = RootSelectionBranch::AuthoredRoot;
        }
    }
    if selected.is_none() && !candidates.is_empty() {
        let mut best = -f32::MAX;
        for &index in &candidates {
            if input[index].center[1] > best {
                best = input[index].center[1];
                selected = Some(index);
            }
        }
        branch = RootSelectionBranch::HighestFootCandidate;
    }
    if selected.is_none() && candidates.is_empty() {
        selected = Some(0);
        if policy.spine_midpoint_fallback {
            let spines = input
                .iter()
                .enumerate()
                .filter_map(|(i, block)| (block.nonzero_mask & 8 != 0).then_some(i))
                .collect::<Vec<_>>();
            if spines.len() > 2 {
                let mut spine_children = vec![0usize; input.len()];
                for block in input {
                    if block.nonzero_mask & 8 != 0 {
                        if let Some(parent) = block.parent {
                            spine_children[parent] += 1;
                        }
                    }
                }
                if let Some(&terminal) = spines.iter().find(|&&i| spine_children[i] == 0) {
                    let mut at = Some(terminal);
                    for _ in 0..spines.len() / 2 {
                        at = at.and_then(|i| input[i].parent);
                        if at.is_none() {
                            break;
                        }
                    }
                    if let Some(index) = at {
                        selected = Some(index);
                        branch = RootSelectionBranch::TerminalSpineParentWalk;
                    }
                }
            }
        }
    }
    Ok(RootCandidate {
        root: selected,
        no_foot_candidates,
        foot_votes: votes,
        foot_candidates: candidates,
        branch,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedTopologyBlock {
    pub initial_ordinal: usize,
    pub source_palette_id: u32,
    pub parent: Option<usize>,
    pub symmetric: Option<usize>,
    pub children: Range<usize>,
    pub descendants: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedTopology {
    pub selection: RootSelection,
    pub new_to_initial: Vec<usize>,
    pub initial_to_new: Vec<usize>,
    pub blocks: Vec<OrderedTopologyBlock>,
}

/// Project native root selection, parent reversal and child-contiguous order.
///
/// Absolute centers, orientations and palette IDs survive this permutation.
/// This operation does not return transformed anchors, finalized capabilities
/// or selector classes; callers must not treat it as a complete runtime rig.
pub fn order_topology(
    input: &[TopologyBlock],
    policy: RootPolicy,
) -> Result<OrderedTopology, TopologyError> {
    let selection = select_root(input, policy)?;
    let mut runtime = input
        .iter()
        .map(|b| RuntimeBlock {
            parent: b.parent,
            capability_mask: b.presence_mask,
        })
        .collect::<Vec<_>>();
    let (mut previous, mut at) = (None, Some(selection.root));
    while let Some(index) = at {
        at = runtime[index].parent;
        runtime[index].parent = previous;
        previous = Some(index);
    }
    let order = order_runtime_blocks(&runtime, selection.root).map_err(TopologyError::Order)?;
    let blocks = order
        .blocks
        .iter()
        .map(|b| OrderedTopologyBlock {
            initial_ordinal: b.original_index,
            source_palette_id: input[b.original_index].source_palette_id,
            parent: b.parent,
            symmetric: input[b.original_index]
                .symmetric
                .map(|s| order.old_to_new[s]),
            children: b.children.clone(),
            descendants: b.descendants.clone(),
        })
        .collect();
    Ok(OrderedTopology {
        selection,
        new_to_initial: order.new_to_old,
        initial_to_new: order.old_to_new,
        blocks,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTopologyInputs {
    pub root_nomination: usize,
    pub source_to_initial: Vec<usize>,
    pub initial_to_source: Vec<usize>,
    pub blocks: Vec<TopologyBlock>,
    /// Exact9CBFA0 call gate for9CBE70, not a count of generated feet.
    pub optional_foot_construction_required: bool,
}

/// Recover the canonical loader's initial topology with an explicit dictionary
/// and source-geometry x87 context. This retains the raw symmetry word exactly:
/// unlike parents, the loader does not initially remap it around nomination.
///
/// Returned blocks precede optional foot construction and finalization. The
/// call gate is explicit and must be resolved before using this as an actual
/// finalizer input. In particular, optional construction is never bypassed.
pub fn source_topology_inputs(
    source: &CreatureData,
    dictionary: &PartCapabilities,
    context: Context,
) -> Result<SourceTopologyInputs, TopologyError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        source_topology_inputs_scoped(source, dictionary, context)
    })
}

fn source_topology_inputs_scoped(
    source: &CreatureData,
    dictionary: &PartCapabilities,
    context: Context,
) -> Result<SourceTopologyInputs, TopologyError> {
    let count = source.parts.len();
    if count == 0 || count > MAX_RUNTIME_BLOCKS {
        return Err(TopologyError::Count(count));
    }
    if source.data0.len() != count * PART_ROW_SIZE {
        return Err(TopologyError::SourceRowStorage {
            rows: count,
            bytes: source.data0.len(),
        });
    }
    let word = |row: usize, at: usize| {
        u16::from_le_bytes(
            source.data0[row * PART_ROW_SIZE + at..][..2]
                .try_into()
                .unwrap(),
        )
    };
    let first_parent = word(0, 4);
    let nomination = if first_parent == u16::MAX {
        0
    } else {
        first_parent as usize
    };
    if nomination >= count || word(nomination, 4) != u16::MAX {
        return Err(TopologyError::SourceRootNomination(nomination));
    }
    let source_to_initial = (0..count)
        .map(|i| {
            if i < nomination {
                i + 1
            } else if i == nomination {
                0
            } else {
                i
            }
        })
        .collect::<Vec<_>>();
    let mut initial_to_source = vec![0; count];
    for (source, &initial) in source_to_initial.iter().enumerate() {
        initial_to_source[initial] = source;
    }
    let rest = source_rest_blocks(source, context).map_err(TopologyError::Geometry)?;
    let mut blocks = Vec::with_capacity(count);
    for &i in &initial_to_source {
        let raw_parent = word(i, 4);
        let raw_symmetric = word(i, 6);
        let parent = if raw_parent == u16::MAX {
            None
        } else if (raw_parent as usize) < count {
            Some(source_to_initial[raw_parent as usize])
        } else {
            return Err(TopologyError::ParentOutOfBounds {
                block: source_to_initial[i],
                parent: raw_parent as usize,
            });
        };
        let symmetric = if raw_symmetric == u16::MAX {
            None
        } else {
            Some(raw_symmetric as usize)
        };
        let caps = prepare_source_capabilities(source, i, dictionary)
            .map_err(TopologyError::Capabilities)?;
        blocks.push(TopologyBlock {
            parent,
            symmetric,
            source_palette_id: i as u32,
            center: rest[i].center,
            presence_mask: caps.presence_mask(),
            nonzero_mask: caps.nonzero_mask(),
        });
    }
    validate(&blocks)?;
    let feet = blocks.iter().filter(|b| b.nonzero_mask & 4 != 0).count();
    Ok(SourceTopologyInputs {
        root_nomination: nomination,
        source_to_initial,
        initial_to_source,
        blocks,
        optional_foot_construction_required: feet == 0 && nomination == 0,
    })
}

impl SourceTopologyInputs {
    /// Order this source intake only when the native loader calls no optional
    /// foot constructor. Required generated blocks are an explicit open gate.
    pub fn order_without_optional_foot_construction(
        &self,
        policy: RootPolicy,
    ) -> Result<OrderedTopology, TopologyError> {
        if self.optional_foot_construction_required {
            return Err(TopologyError::OptionalFootConstructionRequired);
        }
        order_topology(&self.blocks, policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(
        parent: Option<usize>,
        presence: u32,
        nonzero: u32,
        center: [f32; 3],
    ) -> TopologyBlock {
        TopologyBlock {
            parent,
            symmetric: None,
            source_palette_id: 77,
            center,
            presence_mask: presence,
            nonzero_mask: nonzero,
        }
    }
    fn policy() -> RootPolicy {
        RootPolicy {
            nearest_origin: false,
            spine_midpoint_fallback: true,
        }
    }

    #[test]
    fn foot_votes_reset_candidates_and_preserve_equal_height_order() {
        let tree = [
            block(None, 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0, 1.0, 0.0]),
            block(Some(0), 8, 8, [0.0, 1.0, 0.0]),
            block(Some(1), 4, 4, [0.0; 3]),
            block(Some(2), 4, 4, [0.0; 3]),
            block(Some(1), 4, 4, [0.0; 3]),
            block(Some(2), 4, 4, [0.0; 3]),
        ];
        let root = select_root(&tree, policy()).unwrap();
        assert_eq!(root.foot_votes, [0, 2, 2, 0, 0, 0, 0]);
        assert_eq!(root.foot_candidates, [1, 2]);
        assert_eq!(root.root, 1);
        assert!(!root.no_foot_candidates);
    }

    #[test]
    fn authored_root_stops_later_votes_but_nearest_presence_can_override_it() {
        let mut tree = [
            block(None, 8, 8, [8.0, 0.0, 0.0]),
            block(Some(0), 9, 9, [4.0, 0.0, 0.0]),
            block(Some(1), 8, 0, [0.0; 3]),
            block(Some(1), 4, 4, [0.0; 3]),
        ];
        let root = select_root(&tree, policy()).unwrap();
        assert_eq!(root.root, 1);
        assert_eq!(root.foot_votes, [0; 4]);
        assert!(root.no_foot_candidates);
        let nearest = RootPolicy {
            nearest_origin: true,
            ..policy()
        };
        assert_eq!(
            select_root(&tree, nearest),
            Err(TopologyError::SelectedRootMissingNonzeroSpine(2))
        );
        tree[2].nonzero_mask = 8;
        assert_eq!(select_root(&tree, nearest).unwrap().root, 2);
    }

    #[test]
    fn fallback_walk_uses_all_parent_edges_and_keeps_initial_root_on_exhaustion() {
        let tree = [
            block(None, 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0; 3]),
            block(Some(1), 8, 8, [0.0; 3]),
            block(Some(2), 8, 8, [0.0; 3]),
            block(Some(3), 8, 8, [0.0; 3]),
        ];
        assert_eq!(select_root(&tree, policy()).unwrap().root, 2);
        let star = [
            tree[0],
            block(Some(0), 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0; 3]),
        ];
        assert_eq!(select_root(&star, policy()).unwrap().root, 0);
        let mixed = [
            tree[0],
            block(Some(0), 0, 0, [0.0; 3]),
            block(Some(1), 0, 0, [0.0; 3]),
            block(Some(2), 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0; 3]),
            block(Some(4), 8, 8, [0.0; 3]),
            block(Some(5), 8, 8, [0.0; 3]),
        ];
        assert_eq!(
            select_root_candidate(&mixed, policy()).unwrap().root,
            Some(1)
        );
        assert_eq!(
            select_root(&mixed, policy()),
            Err(TopologyError::SelectedRootMissingNonzeroSpine(1))
        );
    }

    #[test]
    fn reversal_and_permutation_keep_palette_and_raw_runtime_symmetry_distinct() {
        let mut tree = [
            block(None, 8, 8, [0.0; 3]),
            block(Some(0), 9, 9, [0.0; 3]),
            block(Some(1), 0, 0, [0.0; 3]),
        ];
        for (i, block) in tree.iter_mut().enumerate() {
            block.source_palette_id = [7, 2, u32::MAX][i];
        }
        tree[0].symmetric = Some(2);
        let ordered = order_topology(&tree, policy()).unwrap();
        assert_eq!(ordered.new_to_initial, [1, 0, 2]);
        assert_eq!(ordered.blocks[0].source_palette_id, 2);
        assert_eq!(ordered.blocks[1].source_palette_id, 7);
        assert_eq!(ordered.blocks[2].source_palette_id, u32::MAX);
        assert_eq!(ordered.blocks[1].symmetric, Some(2));
        assert_eq!(
            ordered.blocks.iter().map(|b| b.parent).collect::<Vec<_>>(),
            [None, Some(0), Some(0)]
        );
    }

    #[test]
    fn invalid_topologies_and_native_unsafe_foot_boundary_are_explicit() {
        let mut tree = [block(None, 8, 8, [0.0; 3]), block(Some(1), 0, 0, [0.0; 3])];
        assert_eq!(
            select_root(&tree, policy()),
            Err(TopologyError::ParentCycle(1))
        );
        tree[1].parent = None;
        assert_eq!(
            select_root(&tree, policy()),
            Err(TopologyError::AdditionalRoot(1))
        );
        tree[1].parent = Some(0);
        tree[0].nonzero_mask |= 4;
        assert_eq!(
            select_root(&tree, policy()),
            Err(TopologyError::FootWithoutSpineAncestor(0))
        );
    }

    #[test]
    fn actual_sse_lane_order_and_unset_height_boundary_remain_observable() {
        let center = [0x41B4_E6AE, 0x3FFA_4A0C, 0x41E4_4F42].map(f32::from_bits);
        let tree = [
            block(None, 8, 8, center),
            block(Some(0), 8, 8, center),
            block(Some(0), 0, 0, [0.0; 3]),
            block(Some(0), 0, 0, [0.0; 3]),
        ];
        let nearest = RootPolicy {
            nearest_origin: true,
            ..policy()
        };
        assert_eq!(select_root(&tree, nearest).unwrap().root, 1);
        assert_eq!(select_root(&tree[..3], nearest).unwrap().root, 0);
        let boundary = [
            block(None, 8, 8, [0.0; 3]),
            block(Some(0), 8, 8, [0.0, -f32::MAX, 0.0]),
            block(Some(1), 4, 4, [0.0; 3]),
        ];
        let candidate = select_root_candidate(&boundary, policy()).unwrap();
        assert_eq!(candidate.root, None);
        assert_eq!(candidate.foot_candidates, [1]);
        assert_eq!(
            select_root(&boundary, policy()),
            Err(TopologyError::NoSelectedRoot)
        );
    }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn hostile_caller_rounding_and_ftz_preserve_native_witness_and_restore_all_state() {
        fn current() -> u32 {
            let mut value = 0u32;
            // SAFETY: SSE2 target, bounded local, no memory alias escapes.
            unsafe {
                std::arch::asm!("stmxcsr [{value}]", value = in(reg) &mut value,
                options(nostack, preserves_flags));
            }
            value
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                // SAFETY: exact same-thread saved state, including on assertion failure.
                unsafe {
                    std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &self.0,
                    options(nostack, preserves_flags));
                }
            }
        }
        let _restore = Restore(current());
        let center = [0x41B4_E6AE, 0x3FFA_4A0C, 0x41E4_4F42].map(f32::from_bits);
        let tree = [
            block(None, 8, 8, center),
            block(Some(0), 8, 8, center),
            block(Some(0), 0, 0, [0.0; 3]),
            block(Some(0), 0, 0, [0.0; 3]),
        ];
        let hostile = 0xDF81u32; // Upward rounding, FTZ, masked exceptions, invalid sticky.
        let tree = std::hint::black_box(tree);
        // SAFETY: defined MXCSR bits; no optional DAZ bit is requested.
        unsafe {
            std::arch::asm!("ldmxcsr [{value}]", value = in(reg) &hostile,
            options(nostack, preserves_flags));
        }
        let nearest = RootPolicy {
            nearest_origin: true,
            ..policy()
        };
        assert_eq!(select_root(&tree, nearest).unwrap().root, 1);
        assert_eq!(current(), hostile);
        assert_eq!(
            select_root_candidate(&[], nearest),
            Err(TopologyError::Count(0))
        );
        assert_eq!(current(), hostile);
    }
}
