//! Projected native finalizer fields after canonical optional construction.
//!
//! Root selection and child-contiguous ordering use the separately conformed
//! topology operations. This module adds9C3150's anchor reversal/root zeroing,
//! ordered local quaternions and sequential9B5DA0 capability relabels. Absolute
//! rest inputs and source palette identities survive. Later selector classes,
//! constraints, channel selection and evaluated poses remain separate gates.

use crate::anim_body_capabilities::CapabilitySlots;
use crate::anim_body_optional_feet::{
    source_optional_feet, FootConstructionBlock, OptionalFeetError, SourceOptionalFeet,
};
use crate::anim_body_topology::{select_root, RootPolicy, RootSelection, TopologyError};
use crate::anim_order::{order_runtime_blocks, OrderError, RuntimeBlock, RuntimeBlockOrder};
use crate::creature_data::CreatureData;
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::{Context, Extended, X87Error};
use crate::part_capabilities::PartCapabilities;
use std::{fmt, ops::Range};

const ROOT: u32 = 0x746F_6F72;
const LIMB: u32 = 0x626D_696C;
const SPINE: u32 = 0x6E69_7073;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalizerFieldsError {
    Count(usize),
    Ordinal { block: usize, ordinal: u32 },
    ParentOutOfBounds { block: usize, parent: usize },
    InitialRootHasParent,
    AdditionalRoot(usize),
    ParentCycle(usize),
    SymmetryOutOfBounds { block: usize, symmetric: usize },
    ChildCount { block: usize, declared: u32, actual: u32 },
    CapabilityMasks(usize),
    RootOutOfBounds(usize),
    InitialRootMissingNonzeroSpine,
    SelectedRootMissingNonzeroSpine(usize),
    ConsumedFieldOutsideBound { block: usize, offset: usize },
    ChildRanges(usize),
    ChildRangeParent { block: usize, child: usize },
    X87 { block: usize, error: X87Error },
    Sse(SseError),
    Order(OrderError),
    Topology(TopologyError),
    Optional(OptionalFeetError),
}
impl From<SseError> for FinalizerFieldsError {
    fn from(error: SseError) -> Self { Self::Sse(error) }
}
impl fmt::Display for FinalizerFieldsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native body finalizer fields: {self:?}")
    }
}
impl std::error::Error for FinalizerFieldsError {}

fn mask(id: u32) -> u32 {
    match id {
        ROOT => 1, LIMB => 2, 0x746F_6F66 => 4, SPINE => 8,
        0x7466_7370 => 16, 0x7475_6F6D => 32, 0x7073_7267 => 64,
        0x7274_736E => 128, _ => 0,
    }
}
fn masks(slots: &CapabilitySlots) -> (u32, u32) {
    let (mut presence, mut nonzero) = (0, 0);
    for (&id, &number) in slots.ids.iter().zip(&slots.numbers) {
        if id == 0 { break; }
        let bit = mask(id);
        presence |= bit;
        if number != 0 { nonzero |= bit; }
    }
    (presence, nonzero)
}
fn ensure(block: &mut FootConstructionBlock, id: u32, number: u8) {
    // Native searches stop at the first target or hole. A full table without
    // either silently keeps its slots. Later duplicate/stale bytes survive.
    if let Some(slot) = block.capabilities.ids.iter().position(|value| *value == id || *value == 0) {
        block.capabilities.ids[slot] = id;
        block.capabilities.numbers[slot] = number;
    }
    (block.presence_mask, block.nonzero_mask) = masks(&block.capabilities);
}

fn validate_tree(input: &[FootConstructionBlock]) -> Result<(), FinalizerFieldsError> {
    let count = input.len();
    if count == 0 || count > 255 { return Err(FinalizerFieldsError::Count(count)); }
    if input[0].parent.is_some() { return Err(FinalizerFieldsError::InitialRootHasParent); }
    let mut child_counts = vec![0u32; count];
    for (i, value) in input.iter().enumerate() {
        if value.ordinal != i as u32 {
            return Err(FinalizerFieldsError::Ordinal { block: i, ordinal: value.ordinal });
        }
        if let Some(parent) = value.parent {
            if parent >= count { return Err(FinalizerFieldsError::ParentOutOfBounds { block: i, parent }); }
            child_counts[parent] += 1;
        } else if i != 0 { return Err(FinalizerFieldsError::AdditionalRoot(i)); }
        if let Some(symmetric) = value.symmetric {
            if symmetric >= count { return Err(FinalizerFieldsError::SymmetryOutOfBounds { block: i, symmetric }); }
        }
        if masks(&value.capabilities) != (value.presence_mask, value.nonzero_mask) {
            return Err(FinalizerFieldsError::CapabilityMasks(i));
        }
    }
    for (i, value) in input.iter().enumerate() {
        if value.child_count != child_counts[i] {
            return Err(FinalizerFieldsError::ChildCount { block: i, declared: value.child_count, actual: child_counts[i] });
        }
        let mut at = Some(i);
        for step in 0..=count {
            let Some(index) = at else { break };
            if step == count { return Err(FinalizerFieldsError::ParentCycle(i)); }
            at = input[index].parent;
        }
    }
    Ok(())
}

fn bounded(values: &[f32], block: usize, offset: usize, limit: u32) -> Result<(), FinalizerFieldsError> {
    for (i, value) in values.iter().enumerate() {
        if value.to_bits() & 0x7FFF_FFFF > limit {
            return Err(FinalizerFieldsError::ConsumedFieldOutsideBound { block, offset: offset+i*4 });
        }
    }
    Ok(())
}

fn length(delta: [f32; 3], context: Context) -> Result<f32, X87Error> {
    let [x, y, z] = delta.map(Extended::from_f32);
    let [x, y, z] = [x?, y?, z?];
    let xx = context.mul(x, x)?;
    let yy = context.mul(y, y)?;
    let zz = context.mul(z, z)?;
    let sum = context.add(zz, context.add(yy, xx)?)?;
    context.store_f32(context.sqrt(sum)?)
}

/// Add the selected root capability, reverse its anchors and zero root fields.
///
/// Inputs are the canonical optional result in its initial runtime order,
/// with connected parent tree rooted at ordinal0. Both initial and selected
/// root must have a nonzero-spin mask, matching9C3150's earlier admission.
/// Only anchors on the reversed chain and spin local origins are consumed;
/// those finite values are bounded by2^100. Unconsumed opaque words survive.
/// Full capability arrays keep native first-target/hole/full-table behavior.
pub fn reroot_anchor_fields(
    input: &[FootConstructionBlock], selected_root: usize, context: Context,
) -> Result<Vec<FootConstructionBlock>, FinalizerFieldsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || reroot_scoped(input, selected_root, context))
}

fn reroot_scoped(
    input: &[FootConstructionBlock], root: usize, context: Context,
) -> Result<Vec<FootConstructionBlock>, FinalizerFieldsError> {
    validate_tree(input)?;
    if root >= input.len() { return Err(FinalizerFieldsError::RootOutOfBounds(root)); }
    if input[0].nonzero_mask & 8 == 0 { return Err(FinalizerFieldsError::InitialRootMissingNonzeroSpine); }
    if input[root].nonzero_mask & 8 == 0 {
        return Err(FinalizerFieldsError::SelectedRootMissingNonzeroSpine(root));
    }
    if root != 0 {
        let mut at = Some(root);
        while let Some(index) = at {
            bounded(&input[index].child_anchor, index, 0x344, 0x7180_0000)?;
            bounded(&input[index].parent_anchor, index, 0x350, 0x7180_0000)?;
            if input[index].presence_mask & 8 != 0 {
                bounded(&input[index].local_origin, index, 0x144, 0x7180_0000)?;
            }
            at = input[index].parent;
        }
    }
    let mut blocks = input.to_vec();
    ensure(&mut blocks[root], ROOT, 1);
    if root != 0 {
        let (mut incoming_parent, mut incoming_child) = (blocks[root].parent_anchor, blocks[root].child_anchor);
        let (mut previous, mut at) = (None, Some(root));
        while let Some(index) = at {
            let block = &mut blocks[index];
            let (old_parent, old_child_anchor, old_parent_anchor) = (block.parent, block.child_anchor, block.parent_anchor);
            block.parent = previous;
            block.child_anchor = incoming_parent;
            block.parent_anchor = incoming_child;
            block.parent_delta = if block.presence_mask & 8 != 0 {
                std::array::from_fn(|i| block.child_anchor[i]-block.local_origin[i])
            } else { block.child_anchor };
            block.parent_delta_length = length(block.parent_delta, context)
                .map_err(|error| FinalizerFieldsError::X87 { block: index, error })?;
            incoming_parent = old_parent_anchor;
            incoming_child = old_child_anchor;
            previous = Some(index);
            at = old_parent;
        }
        blocks[0].child_count -= 1;
        blocks[root].child_count += 1;
    }
    let root = &mut blocks[root];
    root.child_anchor = [0.0; 3];
    root.parent_anchor = [0.0; 3];
    root.parent_delta = [0.0; 3];
    root.parent_delta_length = 0.0;
    Ok(blocks)
}

/// Recompute local quaternions after the caller's native child ordering.
///
/// A root (ordinal0, no parent) retains its existing local quaternion, including
/// signed zeros and opaque values. Other blocks use absolute child and parent
/// quaternions with finite components bounded by2. No normalization, inversion
/// by a squared norm, quality threshold or hemisphere change is performed.
pub fn ordered_local_orientations(
    input: &[FootConstructionBlock],
) -> Result<Vec<FootConstructionBlock>, FinalizerFieldsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        validate_tree(input)?;
        local_scoped(input)
    })
}
fn local_scoped(input: &[FootConstructionBlock]) -> Result<Vec<FootConstructionBlock>, FinalizerFieldsError> {
    for (i, block) in input.iter().enumerate() {
        if let Some(parent) = block.parent {
            bounded(&block.absolute_orientation, i, 0x118, 0x4000_0000)?;
            bounded(&input[parent].absolute_orientation, parent, 0x118, 0x4000_0000)?;
        }
    }
    let mut output = input.to_vec();
    for (i, block) in input.iter().enumerate() {
        let Some(parent) = block.parent else { continue };
        let [cx, cy, cz, cw] = block.absolute_orientation;
        let [px, py, pz, pw] = input[parent].absolute_orientation;
        output[i].local_orientation = [
            ((cx*pw-cw*px)+cy*pz)-cz*py,
            ((cy*pw-cw*py)-cx*pz)+cz*px,
            ((cx*py-cw*pz)-cy*px)+cz*pw,
            ((cy*py+cx*px)+cw*pw)+cz*pz,
        ];
    }
    Ok(output)
}

/// Apply the two sequential native9B5DA0 parent walks to an ordered tree.
///
/// `starts` are the remapped original root and original last nonzero-spin
/// ordinals (definition+3AC/+3B0). No walk runs when3F4 is true. A walk stops
/// before ordinal0 or a block with more than one nonzero-limb direct child.
/// It ensures limb number1, rebuilds masks, then spin number0 and rebuilds
/// again. Presence-spin can therefore survive while nonzero-spin is cleared.
pub fn relabel_ordered_capabilities(
    input: &[FootConstructionBlock], children: &[Range<usize>], starts: [usize; 2],
    no_foot_candidates: bool,
) -> Result<Vec<FootConstructionBlock>, FinalizerFieldsError> {
    validate_tree(input)?;
    validate_ranges(input, children)?;
    for root in starts {
        if root >= input.len() { return Err(FinalizerFieldsError::RootOutOfBounds(root)); }
    }
    let mut output = input.to_vec();
    if !no_foot_candidates {
        for start in starts {
            let mut at = start;
            while at != 0 {
                if children[at].clone().filter(|&child| output[child].nonzero_mask & 2 != 0).count() > 1 { break; }
                ensure(&mut output[at], LIMB, 1);
                ensure(&mut output[at], SPINE, 0);
                at = output[at].parent.expect("validated unique root ordinal0");
            }
        }
    }
    Ok(output)
}
fn validate_ranges(input: &[FootConstructionBlock], children: &[Range<usize>]) -> Result<(), FinalizerFieldsError> {
    if children.len() != input.len() { return Err(FinalizerFieldsError::ChildRanges(children.len())); }
    let mut seen = vec![false; input.len()];
    for (i, range) in children.iter().enumerate() {
        if range.start > range.end || range.end > input.len() || range.len() != input[i].child_count as usize {
            return Err(FinalizerFieldsError::ChildRanges(i));
        }
        for child in range.clone() {
            if seen[child] || input[child].parent != Some(i) {
                return Err(FinalizerFieldsError::ChildRangeParent { block: i, child });
            }
            seen[child] = true;
        }
    }
    if seen[0] || seen.iter().skip(1).any(|seen| !seen) { return Err(FinalizerFieldsError::ChildRanges(0)); }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalizedBodyFields {
    pub blocks: Vec<FootConstructionBlock>,
    pub order: RuntimeBlockOrder,
    /// Definition+3AC, remapped initial ordinal0, distinct from selected root0.
    pub initial_root_in_final_order: usize,
    /// Definition+3B0: remapped last initial nonzero-spin ordinal.
    pub last_spine_in_final_order: usize,
    /// Definition+37C removes only trailing FFFFFFFF palette IDs.
    pub source_palette_prefix_count: usize,
}

/// Finish projected fields using an explicit accepted root-selection result.
///
/// This connects optional construction to the measured native order. It does
/// not select a root by an invented geometric rule. Palette IDs and absolute
/// rest geometry remain unchanged across the returned permutation. Runtime
/// parent/child pointer addresses are represented by validated ordinals/ranges.
pub fn finalize_body_fields(
    input: &[FootConstructionBlock], selected_root: usize, no_foot_candidates: bool, context: Context,
) -> Result<FinalizedBodyFields, FinalizerFieldsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        let rerooted = reroot_scoped(input, selected_root, context)?;
        let last_spine = input.iter().rposition(|value| value.nonzero_mask & 8 != 0).unwrap();
        let source_palette_prefix_count = input.iter().rposition(|value| value.source_palette_id != u32::MAX)
            .map_or(0, |index| index+1);
        let runtime = rerooted.iter().map(|block| RuntimeBlock {
            parent: block.parent, capability_mask: block.presence_mask,
        }).collect::<Vec<_>>();
        let order = order_runtime_blocks(&runtime, selected_root).map_err(FinalizerFieldsError::Order)?;
        let mut ordered = Vec::with_capacity(input.len());
        for (new, value) in order.blocks.iter().enumerate() {
            let mut block = rerooted[value.original_index].clone();
            block.ordinal = new as u32;
            block.parent = value.parent;
            block.symmetric = block.symmetric.map(|index| order.old_to_new[index]);
            block.child_count = value.children.len() as u32;
            ordered.push(block);
        }
        let ordered = local_scoped(&ordered)?;
        let starts = [order.old_to_new[0], order.old_to_new[last_spine]];
        let children = order.blocks.iter().map(|block| block.children.clone()).collect::<Vec<_>>();
        let blocks = relabel_ordered_capabilities(&ordered, &children, starts, no_foot_candidates)?;
        Ok(FinalizedBodyFields { blocks, order, initial_root_in_final_order: starts[0],
            last_spine_in_final_order: starts[1], source_palette_prefix_count })
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFinalizerFields {
    pub optional: SourceOptionalFeet,
    pub selection: RootSelection,
    pub finalized: FinalizedBodyFields,
}
/// Read the original source path through capabilities/geometry/anchors,
/// optional construction, explicit native root policy and these final fields.
/// Records failing actual initial/selected nonzero-spin admission remain
/// typed exclusions. No source row is silently removed or given a fake rig.
pub fn source_finalizer_fields(
    source: &CreatureData, dictionary: &PartCapabilities, context: Context,
    threshold: f32, initial_field3f5: bool, root_policy: RootPolicy,
) -> Result<SourceFinalizerFields, FinalizerFieldsError> {
    let optional = source_optional_feet(source, dictionary, context, threshold, initial_field3f5)
        .map_err(FinalizerFieldsError::Optional)?;
    let topology = optional.result.blocks.iter().map(FootConstructionBlock::topology).collect::<Vec<_>>();
    let selection = select_root(&topology, root_policy).map_err(FinalizerFieldsError::Topology)?;
    let finalized = finalize_body_fields(&optional.result.blocks, selection.root, selection.no_foot_candidates, context)?;
    Ok(SourceFinalizerFields { optional, selection, finalized })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx() -> Context { Context::from_control_word(0x027F).unwrap() }
    fn block(i: usize, parent: Option<usize>, caps: &[(u32,u8)]) -> FootConstructionBlock {
        let mut capabilities = CapabilitySlots::default();
        for (slot, &(id, number)) in caps.iter().enumerate() {
            capabilities.ids[slot] = id;
            capabilities.numbers[slot] = number;
        }
        let (presence_mask,nonzero_mask) = masks(&capabilities);
        FootConstructionBlock { capabilities,presence_mask,nonzero_mask,part_instance:123,part_scope:456,
            ordinal:i as u32,parent,symmetric:None,source_palette_id:i as u32,child_count:0,
            center:[0.;3],absolute_orientation:[0.,0.,0.,1.],local_orientation:[0.,0.,0.,1.],
            half_extents:[0.5;3],local_origin:[0.;3],child_anchor:[0.;3],parent_anchor:[0.;3],
            parent_delta:[0.;3],parent_delta_length:0.,field36c:f32::MAX,source_scale:1.,field37c:0.5 }
    }
    fn counts(blocks: &mut [FootConstructionBlock]) {
        let parents = blocks.iter().map(|value| value.parent).collect::<Vec<_>>();
        for block in blocks.iter_mut() { block.child_count=0; }
        for parent in parents.into_iter().flatten() { blocks[parent].child_count += 1; }
    }
    #[test]
    fn unchanged_root_zeroes_all_ten_anchor_words_without_consuming_opaque_values() {
        let mut input = [block(0,None,&[(SPINE,1)])];
        input[0].child_anchor = [f32::NAN;3]; input[0].parent_anchor = [f32::INFINITY;3];
        input[0].parent_delta = [-0.;3]; input[0].parent_delta_length = f32::NEG_INFINITY;
        let result = reroot_anchor_fields(&input,0,ctx()).unwrap();
        for value in result[0].child_anchor.into_iter().chain(result[0].parent_anchor)
            .chain(result[0].parent_delta).chain([result[0].parent_delta_length]) { assert_eq!(value.to_bits(),0); }
        assert_eq!((result[0].capabilities.ids[1], result[0].capabilities.numbers[1]),(ROOT,1));
    }
    #[test]
    fn reversed_chain_uses_childs_old_anchor_and_presence_only_spine_origin() {
        let mut input = [block(0,None,&[(SPINE,1)]),block(1,Some(0),&[(SPINE,0)]),block(2,Some(1),&[(SPINE,1)])];
        counts(&mut input);
        input[1].child_anchor=[1.,2.,3.];input[1].parent_anchor=[4.,5.,6.];
        input[2].child_anchor=[7.,8.,9.];input[2].parent_anchor=[10.,11.,12.];
        input[1].local_origin=[1.,1.,1.];
        let result=reroot_anchor_fields(&input,2,ctx()).unwrap();
        assert_eq!(result.iter().map(|b|b.parent).collect::<Vec<_>>(),[Some(1),Some(2),None]);
        assert_eq!(result[1].child_anchor,[10.,11.,12.]);assert_eq!(result[1].parent_anchor,[7.,8.,9.]);
        assert_eq!(result[1].parent_delta,[9.,10.,11.]);
        assert_eq!(result[0].child_anchor,[4.,5.,6.]);assert_eq!(result[0].parent_anchor,[1.,2.,3.]);
        assert_eq!(result.iter().map(|b|b.child_count).collect::<Vec<_>>(),[0,1,1]);
    }
    #[test]
    fn local_quaternion_retains_root_and_nonunit_scale() {
        let mut input=[block(0,None,&[(SPINE,1)]),block(1,Some(0),&[])];counts(&mut input);
        input[0].local_orientation=[-0.,1.,2.,3.];
        input[0].absolute_orientation=[0.,0.,0.,2.];input[1].absolute_orientation=[0.25,-0.5,1.,2.];
        let result=ordered_local_orientations(&input).unwrap();
        assert_eq!(result[0].local_orientation.map(f32::to_bits),input[0].local_orientation.map(f32::to_bits));
        assert_eq!(result[1].local_orientation,[0.5,-1.,2.,4.]);
    }
    #[test]
    fn full_capability_table_without_root_is_preserved() {
        let mut input=[block(0,None,&[(SPINE,1)])];
        for slot in 1..32 { input[0].capabilities.ids[slot]=0x1000+slot as u32;input[0].capabilities.numbers[slot]=0xA5; }
        let result=reroot_anchor_fields(&input,0,ctx()).unwrap();
        assert_eq!(result[0].capabilities,input[0].capabilities);assert_eq!(result[0].presence_mask,8);
    }
    #[test]
    fn relabel_keeps_zero_number_spine_presence_and_stops_at_branch() {
        let mut input=[block(0,None,&[(SPINE,1)]),block(1,Some(0),&[(SPINE,1)]),
            block(2,Some(1),&[(LIMB,1)]),block(3,Some(1),&[(LIMB,1)])];counts(&mut input);
        let children=vec![1..2,2..4,4..4,4..4];
        let stopped=relabel_ordered_capabilities(&input,&children,[1,2],false).unwrap();
        assert_eq!(stopped[1].capabilities,input[1].capabilities);
        assert_eq!(stopped[2].presence_mask,10);assert_eq!(stopped[2].nonzero_mask,2);
        assert_eq!(stopped[0].presence_mask,8);
        let skipped=relabel_ordered_capabilities(&input,&children,[1,2],true).unwrap();assert_eq!(skipped,input);
    }
    #[test]
    fn connected_pipeline_preserves_palette_and_generated_sentinel() {
        let mut input=[block(0,None,&[(SPINE,1)]),block(1,Some(0),&[(SPINE,1)]),block(2,Some(1),&[(0x746F6F66,1)])];
        counts(&mut input);input[2].source_palette_id=u32::MAX;input[0].symmetric=Some(1);input[1].symmetric=Some(0);
        let result=finalize_body_fields(&input,1,false,ctx()).unwrap();
        assert_eq!(result.source_palette_prefix_count,2);
        assert_eq!(result.blocks.iter().map(|b|b.source_palette_id).collect::<Vec<_>>(),
            result.order.new_to_old.iter().map(|&old|input[old].source_palette_id).collect::<Vec<_>>());
        assert_eq!(result.blocks[0].source_palette_id,1);assert_eq!(result.blocks[0].parent,None);
        assert_eq!(result.blocks[0].symmetric,Some(result.order.old_to_new[0]));
    }
    #[test]
    #[cfg(any(target_arch="x86",target_arch="x86_64"))]
    fn hostile_ftz_rounding_caller_keeps_subnormal_fields_and_is_restored() {
        fn current()->u32 {
            let mut word=0u32;
            // SAFETY: same-thread four-byte storage on an SSE2 target.
            unsafe { std::arch::asm!("stmxcsr [{v}]",v=in(reg)&mut word,options(nostack,preserves_flags)); }
            word
        }
        struct Restore(u32);
        impl Drop for Restore { fn drop(&mut self) {
            // SAFETY: restore the exact saved same-thread environment.
            unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&self.0,options(nostack,preserves_flags)); }
        }}
        let _restore=Restore(current());
        let mut input=[block(0,None,&[(SPINE,1)]),block(1,Some(0),&[(SPINE,0)]),block(2,Some(1),&[(SPINE,1)])];
        counts(&mut input);input[2].parent_anchor[0]=f32::from_bits(16);
        input[1].local_origin[0]=f32::from_bits(8);input[1].absolute_orientation[0]=f32::from_bits(1);
        let input=std::hint::black_box(input);let hostile=0xDF81u32;
        // SAFETY: defined masked control/status bits; no DAZ request.
        unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&hostile,options(nostack,preserves_flags)); }
        let rerooted=reroot_anchor_fields(&input,2,ctx()).unwrap();
        assert_eq!(rerooted[1].parent_delta[0].to_bits(),8);assert_eq!(current(),hostile);
        let locals=ordered_local_orientations(&input).unwrap();
        assert_eq!(locals[1].local_orientation[0].to_bits(),1);assert_eq!(current(),hostile);
        assert!(reroot_anchor_fields(&input,999,ctx()).is_err());assert_eq!(current(),hostile);
    }
}
