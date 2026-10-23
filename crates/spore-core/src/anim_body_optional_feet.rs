//! Canonical optional-foot construction before body finalization.
//!
//! Source9CBFA0 calls9CBE70 only without nonzero feet and root nomination.
//! This projects the actual driver/minima/point/append fields. Admitted leaf
//! limbs have no mouth/grasp bits, giving a proved progress/capacity bound.

use crate::anim_body_capabilities::{prepare_source_capabilities, BodyCapabilitiesError, CapabilitySlots};
use crate::anim_body_source_anchors::{source_body_anchors_for_creature, SourceAnchorsError};
use crate::anim_body_source_fields::{source_body_fields_for_creature, SourceBodyFieldsError};
use crate::anim_body_topology::TopologyBlock;
use crate::creature_data::{CreatureData, PART_ROW_SIZE};
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;
use crate::part_capabilities::PartCapabilities;
use std::fmt;

/// Explicit pre-finalizer fields read/written by the optional-foot path.
/// Debug names, unrelated constructor bytes and allocator state are excluded.
#[derive(Debug, Clone, PartialEq)]
pub struct FootConstructionBlock {
    pub capabilities: CapabilitySlots,
    pub presence_mask: u32,
    pub nonzero_mask: u32,
    pub part_instance: u32,
    pub part_scope: u32,
    pub ordinal: u32,
    pub parent: Option<usize>,
    pub symmetric: Option<usize>,
    pub source_palette_id: u32,
    pub child_count: u32,
    pub center: [f32; 3],
    pub absolute_orientation: [f32; 4],
    pub local_orientation: [f32; 4],
    pub half_extents: [f32; 3],
    pub local_origin: [f32; 3],
    pub child_anchor: [f32; 3],
    pub parent_anchor: [f32; 3],
    pub parent_delta: [f32; 3],
    pub parent_delta_length: f32,
    pub field36c: f32,
    pub source_scale: f32,
    pub field37c: f32,
}
impl FootConstructionBlock {
    pub fn topology(&self) -> TopologyBlock {
        TopologyBlock { parent: self.parent, symmetric: self.symmetric,
            source_palette_id: self.source_palette_id, center: self.center,
            presence_mask: self.presence_mask, nonzero_mask: self.nonzero_mask }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalFeet {
    pub blocks: Vec<FootConstructionBlock>,
    pub driver_called: bool,
    pub append_batches: usize,
    pub appended_blocks: usize,
    /// Definition3F5 is cleared by append, and preserved on every empty path.
    pub field3f5: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionalFeetError {
    Count(usize),
    NonFiniteThreshold,
    Ordinal { block: usize, ordinal: u32 },
    PaletteIdentity { block: usize, palette_id: u32 },
    ParentOutOfBounds { block: usize, parent: usize },
    SymmetryOutOfBounds { block: usize, symmetric: usize },
    InitialRootHasParent,
    AdditionalRoot(usize),
    ParentCycle(usize),
    ChildCount { block: usize, declared: u32, actual: u32 },
    CapabilityMasks(usize),
    ExistingNonzeroFoot(usize),
    NonFiniteConsumedField { block: usize, offset: usize },
    ConsumedFieldOutsideBound { block: usize, offset: usize },
    NonPositiveExtent { block: usize, axis: usize },
    MouthOrGraspCandidate(usize),
    PotentialCapacityExceeded { blocks: usize, candidates: usize },
    ProgressInvariant,
    SourceRows { rows: usize, bytes: usize },
    SourceRootNomination(usize),
    Fields(SourceBodyFieldsError),
    Anchors(SourceAnchorsError),
    Capabilities(BodyCapabilitiesError),
    Sse(SseError),
}
impl From<SseError> for OptionalFeetError { fn from(v: SseError)->Self { Self::Sse(v) } }
impl From<SourceBodyFieldsError> for OptionalFeetError { fn from(v: SourceBodyFieldsError)->Self { Self::Fields(v) } }
impl From<SourceAnchorsError> for OptionalFeetError { fn from(v: SourceAnchorsError)->Self { Self::Anchors(v) } }
impl From<BodyCapabilitiesError> for OptionalFeetError { fn from(v: BodyCapabilitiesError)->Self { Self::Capabilities(v) } }
impl fmt::Display for OptionalFeetError {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"native optional feet: {self:?}") }
}
impl std::error::Error for OptionalFeetError {}

fn eligible(b: &FootConstructionBlock) -> bool {
    b.child_count==0 && b.presence_mask&2!=0 && b.presence_mask&4==0
}

fn validate(blocks: &[FootConstructionBlock], threshold: f32) -> Result<usize,OptionalFeetError> {
    let count=blocks.len();
    if count==0 || count>255 { return Err(OptionalFeetError::Count(count)); }
    if !threshold.is_finite() { return Err(OptionalFeetError::NonFiniteThreshold); }
    let mut children=vec![0u32;count];
    for (i,b) in blocks.iter().enumerate() {
        if b.ordinal!=i as u32 { return Err(OptionalFeetError::Ordinal { block:i,ordinal:b.ordinal }); }
        if b.source_palette_id!=i as u32 { return Err(OptionalFeetError::PaletteIdentity { block:i,palette_id:b.source_palette_id }); }
        if let Some(parent)=b.parent {
            if parent>=count { return Err(OptionalFeetError::ParentOutOfBounds { block:i,parent }); }
            children[parent]+=1;
        } else if i!=0 { return Err(OptionalFeetError::AdditionalRoot(i)); }
        if let Some(symmetric)=b.symmetric {
            if symmetric>=count { return Err(OptionalFeetError::SymmetryOutOfBounds { block:i,symmetric }); }
        }
        if b.presence_mask>255 || b.nonzero_mask & !b.presence_mask!=0 {
            return Err(OptionalFeetError::CapabilityMasks(i));
        }
        if b.nonzero_mask&4!=0 { return Err(OptionalFeetError::ExistingNonzeroFoot(i)); }
        for (values,offset,bound) in [(&b.center[..],0x10C,0x7180_0000u32),
            (&b.absolute_orientation[..],0x118,0x4000_0000),(&b.local_orientation[..],0x128,0x4000_0000),
            (&b.half_extents[..],0x138,0x7180_0000),(&b.local_origin[..],0x144,0x7180_0000)] {
            for (j,value) in values.iter().enumerate() {
                if !value.is_finite() { return Err(OptionalFeetError::NonFiniteConsumedField { block:i,offset:offset+j*4 }); }
                if value.to_bits()&0x7FFF_FFFF>bound { return Err(OptionalFeetError::ConsumedFieldOutsideBound { block:i,offset:offset+j*4 }); }
            }
        }
        for (axis,v) in b.half_extents.iter().enumerate() {
            if *v<=0. { return Err(OptionalFeetError::NonPositiveExtent { block:i,axis }); }
        }
        if eligible(b) && b.presence_mask&0x60!=0 { return Err(OptionalFeetError::MouthOrGraspCandidate(i)); }
    }
    if blocks[0].parent.is_some() { return Err(OptionalFeetError::InitialRootHasParent); }
    for (i,b) in blocks.iter().enumerate() {
        if b.child_count!=children[i] { return Err(OptionalFeetError::ChildCount { block:i,declared:b.child_count,actual:children[i] }); }
        let mut at=Some(i);
        for step in 0..=count {
            let Some(index)=at else { break };
            if step==count { return Err(OptionalFeetError::ParentCycle(i)); }
            at=blocks[index].parent;
        }
    }
    let candidates=blocks.iter().filter(|b| eligible(b)).count();
    if count+2*candidates>255 { return Err(OptionalFeetError::PotentialCapacityExceeded { blocks:count,candidates }); }
    Ok(candidates)
}

// Literal9C84B0/9C8560 offsets. Third coordinate114 is used, not110.
fn height(b: &FootConstructionBlock) -> f32 {
    let a=b.half_extents[1]*f32::from_bits(b.local_orientation[3].to_bits()&0x7FFF_FFFF);
    let c=b.half_extents[2]*f32::from_bits(b.half_extents[0].to_bits()&0x7FFF_FFFF);
    let d=b.half_extents[0]*f32::from_bits(b.local_orientation[2].to_bits()&0x7FFF_FFFF);
    b.center[2]-((a+c)+d)
}
fn limb_minimum(blocks: &[FootConstructionBlock])->Option<(usize,f32)> {
    let mut best=f32::MAX;
    let mut selected=None;
    for (i,b) in blocks.iter().enumerate() {
        if eligible(b) {
            let h=height(b);
            if h<best { best=h;selected=Some(i); }
        }
    }
    selected.map(|i|(i,best))
}

// Original99BE70 source order. The caller excludes mouth/grasp candidates,
// so9C8660 selects self and subtracts this rotation from the absolute center.
fn rotate_point([x,y,z,w]:[f32;4],[vx,vy,vz]:[f32;3])->[f32;3] {
    let nx=-(x*x);let ny=-(y*y);let nz=-(z*z);
    let xy=y*x;let xz=z*x;let yz=z*y;let wx=x*w;let wy=y*w;let wz=z*w;
    [(((nz+ny)*vx+(xz+wy)*vz)+(xy-wz)*vy)*2.0+vx,
     (((xy+wz)*vx+(yz-wx)*vz)+(nz+nx)*vy)*2.0+vy,
     (((xz-wy)*vx+(ny+nx)*vz)+(yz+wx)*vy)*2.0+vz]
}

// Original append9CBC79..9CBE23. Z order differs from canonical parent350.
fn append_anchor([x,y,z,w]:[f32;4],[dx,dy,dz]:[f32;3])->[f32;3] {
    let nx=-(x*x);let ny=-(y*y);let nz=-(z*z);
    let nwx=-(x*w);let nwy=-(y*w);let nwz=-(z*w);
    let xy=y*x;let xz=z*x;let yz=z*y;
    [(((xy-nwz)*dy+(xz+nwy)*dz)+(nz+ny)*dx)*2.0+dx,
     (((nz+nx)*dy+(yz-nwx)*dz)+(xy+nwz)*dx)*2.0+dy,
     (((ny+nx)*dz+(yz+nwx)*dy)+(xz-nwy)*dx)*2.0+dz]
}

fn append(blocks:&mut Vec<FootConstructionBlock>,selected:usize,point:[f32;3]) {
    let mirrored=blocks[selected].symmetric;
    let first=blocks.len();
    for (n,parent) in std::iter::once(selected).chain(mirrored).enumerate() {
        let mut center=point;
        if n==1 { center[0]=f32::from_bits(center[0].to_bits()^0x8000_0000); }
        let difference=std::array::from_fn(|i|center[i]-blocks[parent].center[i]);
        let parent_anchor=append_anchor(blocks[parent].absolute_orientation,difference);
        let mut capabilities=CapabilitySlots::default();
        capabilities.ids[0]=0x746F_6F66;capabilities.numbers[0]=1;
        let ordinal=blocks.len() as u32;
        blocks.push(FootConstructionBlock { capabilities,presence_mask:4,nonzero_mask:4,
            part_instance:0x6B2C_0771,part_scope:0x4060_0000,ordinal,parent:Some(parent),
            symmetric:mirrored.map(|_|if n==0 { first+1 } else { first }),
            source_palette_id:u32::MAX,child_count:0,center,
            absolute_orientation:[0.,0.,0.,1.],local_orientation:[0.,0.,0.,1.],
            half_extents:[f32::from_bits(0x3C23_D70A);3],local_origin:[0.;3],
            child_anchor:[0.;3],parent_anchor,parent_delta:[0.;3],parent_delta_length:0.,
            field36c:f32::MAX,source_scale:1.,field37c:0.5 });
        blocks[parent].child_count+=1;
    }
}

/// Run the original optional driver on a bounded explicit source prefix.
///
/// This is the no-mouth/grasp candidate domain actually found in every source
/// call-gate resource. Every append makes its selected leaf ineligible; new
/// foot-only blocks cannot become candidates. The upfront capacity bound is
/// conservative and explicit. Unconsumed anchor/scale/field words pass through
/// bit-for-bit; only finite bounded fields actually read by native math are
/// restricted. Original overflow/unordered minimum/threshold branches remain.
/// All SSE uses1F80 with full caller-state restoration. No full block/debug
/// string, allocator, finalizer or evaluated-animation claim is made.
pub fn run_optional_foot_driver(
    input:&[FootConstructionBlock],threshold:f32,initial_field3f5:bool,
)->Result<OptionalFeet,OptionalFeetError> {
    native_sse::scoped(Policy::Mxcsr1f80,|| {
        let initial_candidates=validate(input,threshold)?;
        let mut blocks=input.to_vec();
        let mut spine=f32::MAX;
        for b in &blocks {
            if b.presence_mask&8!=0 {
                let h=height(b);if h<spine { spine=h; }
            }
        }
        let mut batches=0usize;
        if let Some((mut selected,mut minimum))=limb_minimum(&blocks) {
            let initial_minimum=minimum;
            loop {
                // COMISS/JBE also returns on unordered intermediates.
                if !(spine>minimum) { break; }
                let allowed=(spine-initial_minimum)*threshold;
                let moved=minimum-initial_minimum;
                if !(allowed>moved) { break; }
                if batches>=initial_candidates { return Err(OptionalFeetError::ProgressInvariant); }
                let rotation=rotate_point(blocks[selected].absolute_orientation,blocks[selected].local_origin);
                let point=std::array::from_fn(|i|blocks[selected].center[i]-rotation[i]);
                append(&mut blocks,selected,point);batches+=1;
                let Some(next)=limb_minimum(&blocks) else { break };
                selected=next.0;minimum=next.1;
            }
        }
        let appended_blocks=blocks.len()-input.len();
        Ok(OptionalFeet { blocks,driver_called:true,append_batches:batches,appended_blocks,
            field3f5:if batches>0 { false } else { initial_field3f5 } })
    })
}

#[derive(Debug,Clone,PartialEq)]
pub struct SourceOptionalFeet {
    pub root_nomination:usize,
    pub source_to_initial:Vec<usize>,
    pub initial_to_source:Vec<usize>,
    pub result:OptionalFeet,
}

/// Read the canonical original source prefix, then resolve its exact optional
/// call gate. Local128..134 remains the original constructor identity before
/// this driver. Finalizer admission of a nonzero-spin root remains separate;
/// this intake therefore retains creation records whose later finalizer fails.
pub fn source_optional_feet(
    source:&CreatureData,dictionary:&PartCapabilities,context:Context,
    threshold:f32,initial_field3f5:bool,
)->Result<SourceOptionalFeet,OptionalFeetError> {
    native_sse::scoped(Policy::Mxcsr1f80,|| {
        let count=source.parts.len();
        if count==0 || count>255 { return Err(OptionalFeetError::Count(count)); }
        if source.data0.len()!=count*PART_ROW_SIZE { return Err(OptionalFeetError::SourceRows { rows:count,bytes:source.data0.len() }); }
        let rows=source.data0.chunks_exact(PART_ROW_SIZE).collect::<Vec<_>>();
        let parent=|i:usize|u16::from_le_bytes(rows[i][4..6].try_into().unwrap());
        let nomination=if parent(0)==u16::MAX { 0 } else { usize::from(parent(0)) };
        if nomination>=count || parent(nomination)!=u16::MAX { return Err(OptionalFeetError::SourceRootNomination(nomination)); }
        let source_to_initial=(0..count).map(|i|if i<nomination { i+1 } else if i==nomination { 0 } else { i }).collect::<Vec<_>>();
        let mut initial_to_source=vec![0usize;count];
        for (i,&initial) in source_to_initial.iter().enumerate() { initial_to_source[initial]=i; }
        let fields=source_body_fields_for_creature(source,context)?;
        let anchors=source_body_anchors_for_creature(source,dictionary,context)?;
        let mut blocks=Vec::with_capacity(count);
        for (ordinal,&i) in initial_to_source.iter().enumerate() {
            let value=&fields[i];let anchor=&anchors[i];
            let caps=prepare_source_capabilities(source,i,dictionary)?;
            let raw_parent=parent(i);
            let parent=if raw_parent==u16::MAX { None } else if usize::from(raw_parent)<count {
                Some(source_to_initial[usize::from(raw_parent)])
            } else { return Err(OptionalFeetError::ParentOutOfBounds { block:ordinal,parent:usize::from(raw_parent) }); };
            let raw_sym=u16::from_le_bytes(rows[i][6..8].try_into().unwrap());
            let symmetric=if raw_sym==u16::MAX { None } else { Some(usize::from(raw_sym)) };
            if symmetric.is_some_and(|v|v>=count) { return Err(OptionalFeetError::SymmetryOutOfBounds { block:ordinal,symmetric:usize::from(raw_sym) }); }
            let word=|at:usize|u32::from_le_bytes(rows[i][at..at+4].try_into().unwrap());
            blocks.push(FootConstructionBlock { capabilities:*caps.slots(),presence_mask:caps.presence_mask(),nonzero_mask:caps.nonzero_mask(),
                part_instance:word(0x88),part_scope:word(0x84),ordinal:ordinal as u32,parent,symmetric,
                source_palette_id:i as u32,child_count:anchor.child_count,center:value.rest.center,
                absolute_orientation:value.rest.orientation_xyzw,local_orientation:[0.,0.,0.,1.],
                half_extents:value.half_extents,local_origin:value.local_origin,
                child_anchor:anchor.child_anchor,parent_anchor:anchor.parent_anchor,parent_delta:anchor.parent_delta,
                parent_delta_length:anchor.parent_delta_length,field36c:f32::MAX,
                source_scale:f32::from_bits(word(0x2C)),field37c:f32::from_bits(word(0x6C)) });
        }
        let called=nomination==0 && !blocks.iter().any(|b|b.nonzero_mask&4!=0);
        let result=if called { run_optional_foot_driver(&blocks,threshold,initial_field3f5)? }
            else { OptionalFeet { blocks,driver_called:false,append_batches:0,appended_blocks:0,field3f5:initial_field3f5 } };
        Ok(SourceOptionalFeet { root_nomination:nomination,source_to_initial,initial_to_source,result })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn block(i:usize,parent:Option<usize>,mask:u32,center:[f32;3])->FootConstructionBlock {
        FootConstructionBlock { capabilities:CapabilitySlots::default(),presence_mask:mask,nonzero_mask:mask,
            part_instance:0,part_scope:0,ordinal:i as u32,parent,symmetric:None,source_palette_id:i as u32,
            child_count:0,center,absolute_orientation:[0.,0.,0.,1.],local_orientation:[0.,0.,0.,1.],
            half_extents:[0.5;3],local_origin:[0.;3],child_anchor:[0.;3],parent_anchor:[0.;3],
            parent_delta:[0.;3],parent_delta_length:0.,field36c:f32::MAX,source_scale:1.,field37c:0.5 }
    }
    #[test]
    fn empty_candidate_path_keeps_definition_flag_and_all_source_words() {
        let input=[block(0,None,8,[0.;3])];
        let out=run_optional_foot_driver(&input,0.1,true).unwrap();
        assert_eq!(out.blocks,input);assert_eq!(out.appended_blocks,0);assert!(out.field3f5);
    }
    #[test]
    fn mirrored_append_uses_sentinel_palette_pair_links_and_x_sign_flip() {
        let mut input=[block(0,None,8,[0.,0.,10.]),block(1,Some(0),2,[2.,3.,0.]),block(2,Some(0),2,[-2.,3.,0.])];
        input[0].child_count=2;input[1].symmetric=Some(2);input[2].symmetric=Some(1);
        let out=run_optional_foot_driver(&input,0.1,true).unwrap();
        assert_eq!((out.append_batches,out.appended_blocks,out.field3f5),(1,2,false));
        assert_eq!(out.blocks[3].center,[2.,3.,0.]);assert_eq!(out.blocks[4].center,[-2.,3.,0.]);
        assert_eq!((out.blocks[3].parent,out.blocks[4].parent),(Some(1),Some(2)));
        assert_eq!((out.blocks[3].symmetric,out.blocks[4].symmetric),(Some(4),Some(3)));
        for foot in &out.blocks[3..] { assert_eq!(foot.source_palette_id,u32::MAX);assert_eq!(foot.presence_mask,4);
            assert_eq!(foot.capabilities.ids[0],0x746F6F66);assert_eq!(foot.capabilities.numbers[0],1); }
        assert_eq!((out.blocks[1].child_count,out.blocks[2].child_count),(1,1));
    }
    #[test]
    fn exact_threshold_endpoint_returns_before_append_and_progress_domain_is_explicit() {
        let mut input=[block(0,None,8,[0.,0.,10.]),block(1,Some(0),2,[0.;3])];input[0].child_count=1;
        let out=run_optional_foot_driver(&input,0.,true).unwrap();assert_eq!(out.append_batches,0);
        input[1].presence_mask|=0x20;input[1].nonzero_mask|=0x20;
        assert_eq!(run_optional_foot_driver(&input,0.1,true),Err(OptionalFeetError::MouthOrGraspCandidate(1)));
    }
    #[test]
    fn native_unordered_threshold_returns_without_inventing_an_insertion() {
        let mut input=[block(0,None,8,[0.,0.,1.]),block(1,Some(0),2,[0.;3])];input[0].child_count=1;
        input[1].half_extents=[f32::from_bits(0x71800000);3];
        let out=run_optional_foot_driver(&input,0.1,true).unwrap();
        assert_eq!(out.appended_blocks,0);assert!(out.field3f5);
    }
    #[test]
    #[cfg(any(target_arch="x86",target_arch="x86_64"))]
    fn hostile_ftz_caller_retains_subnormal_point_and_restores_state() {
        fn current()->u32 {
            let mut value=0u32;
            // SAFETY: bounded same-thread storage on an SSE2 target.
            unsafe { std::arch::asm!("stmxcsr [{v}]",v=in(reg)&mut value,options(nostack,preserves_flags)); }
            value
        }
        struct Restore(u32);
        impl Drop for Restore { fn drop(&mut self) {
            // SAFETY: restore the exact saved same-thread environment.
            unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&self.0,options(nostack,preserves_flags)); }
        }}
        let _restore=Restore(current());
        let mut input=[block(0,None,8,[0.,0.,10.]),block(1,Some(0),2,[f32::from_bits(16),0.,0.])];
        input[0].child_count=1;input[1].local_origin[0]=f32::from_bits(8);
        let input=std::hint::black_box(input);let hostile=0xDF81u32;
        // SAFETY: defined masked control/status bits, with no DAZ request.
        unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&hostile,options(nostack,preserves_flags)); }
        let result=run_optional_foot_driver(&input,0.1,true).unwrap();
        assert_eq!(result.blocks[2].center[0].to_bits(),8);
        assert_eq!(result.blocks[2].parent_anchor[0].to_bits(),0x80000008);
        assert_eq!(current(),hostile);
        assert!(run_optional_foot_driver(&input,f32::NAN,true).is_err());
        assert_eq!(current(),hostile);
    }
}
