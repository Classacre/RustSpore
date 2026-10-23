//! Canonical source parent/child anchors before optional construction.
//!
//! This is the source-row region9CC9D0..9CCE21 and its9B5200 getter.
//! Source palette ordinals identify all rows, independently of the initial
//! runtime permutation. Finalizer reroot/local-orientation changes are later.

use crate::anim_body_capabilities::{prepare_source_capabilities, BodyCapabilitiesError};
use crate::anim_body_source_fields::{
    source_body_fields_for_creature, SourceBodyFields, SourceBodyFieldsError,
};
use crate::creature_data::{CreatureData, PART_ROW_SIZE};
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::{Context, Extended, X87Error};
use crate::part_capabilities::PartCapabilities;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceBodyAnchors {
    pub source_palette_id: u32,
    /// Source row+4, before the initial runtime nomination/permutation.
    pub source_parent: Option<u32>,
    /// The exact direct-child count at block+208 before optional construction.
    pub child_count: u32,
    /// Child-local anchor at block+344/348/34C.
    pub child_anchor: [f32; 3],
    /// Parent-local anchor at block+350/354/358.
    pub parent_anchor: [f32; 3],
    /// Block+35C/360/364, produced by9B5200 with literal scale1.
    pub parent_delta: [f32; 3],
    /// Block+368, the retained x87 norm of the committed parent_delta.
    pub parent_delta_length: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceAnchorsError {
    Count(usize),
    InputLengths { rows: usize, bytes: usize, masks: usize },
    PaletteIdentity { row: usize, palette_id: u32 },
    ParentOutOfBounds { row: usize, parent: usize },
    NonFiniteField { row: usize, block_offset: usize },
    NonFiniteOutput { row: usize, block_offset: usize },
    Fields(SourceBodyFieldsError),
    Capabilities(BodyCapabilitiesError),
    X87 { row: usize, error: X87Error },
    Sse(SseError),
}
impl From<SseError> for SourceAnchorsError {
    fn from(error: SseError) -> Self { Self::Sse(error) }
}
impl From<SourceBodyFieldsError> for SourceAnchorsError {
    fn from(error: SourceBodyFieldsError) -> Self { Self::Fields(error) }
}
impl From<BodyCapabilitiesError> for SourceAnchorsError {
    fn from(error: BodyCapabilitiesError) -> Self { Self::Capabilities(error) }
}
impl fmt::Display for SourceAnchorsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "canonical source body anchors: {self:?}")
    }
}
impl std::error::Error for SourceAnchorsError {}

// Parent-local expansion9CCA8C..9CCC1A. Retain the off-axis-first sums.
fn parent_local(q: [f32; 4], [dx, dy, dz]: [f32; 3]) -> [f32; 3] {
    let [x, y, z, w] = q;
    let nx = -(x*x);
    let ny = -(y*y);
    let nz = -(z*z);
    let nwx = -(x*w);
    let nwy = -(y*w);
    let nwz = -(z*w);
    let xy = y*x;
    let xz = z*x;
    let yz = z*y;
    [
        (((xy-nwz)*dy + (xz+nwy)*dz) + (nz+ny)*dx)*2.0 + dx,
        (((nz+nx)*dy + (yz-nwx)*dz) + (xy+nwz)*dx)*2.0 + dy,
        (((xz-nwy)*dx + (ny+nx)*dz) + (yz+nwx)*dy)*2.0 + dz,
    ]
}

// Child-local expansion9CCC22..9CCDB8. Its Z sum differs from the parent.
fn child_local(q: [f32; 4], [dx, dy, dz]: [f32; 3]) -> [f32; 3] {
    let [x, y, z, w] = q;
    let nx = -(x*x);
    let ny = -(y*y);
    let nz = -(z*z);
    let nwx = -(x*w);
    let nwy = -(y*w);
    let nwz = -(z*w);
    let xy = y*x;
    let xz = z*x;
    let yz = z*y;
    [
        (((xy-nwz)*dy + (xz+nwy)*dz) + (nz+ny)*dx)*2.0 + dx,
        (((nz+nx)*dy + (yz-nwx)*dz) + (xy+nwz)*dx)*2.0 + dy,
        (((ny+nx)*dz + (yz+nwx)*dy) + (xz-nwy)*dx)*2.0 + dz,
    ]
}

fn retained_length(value: [f32; 3], context: Context) -> Result<f32, X87Error> {
    let [x, y, z] = value.map(Extended::from_f32);
    let [x, y, z] = [x?, y?, z?];
    let xx = context.mul(x, x)?;
    let yy = context.mul(y, y)?;
    let zz = context.mul(z, z)?;
    // FLD z,y,x;FMUL ST0;FLD ST1;FMULP ST2;FADDP;
    // FLD ST1;FMULP ST2;FADDP;FSQRT;FSTP32 at9CCDF9..9CCE1B.
    let sum = context.add(zz, context.add(yy, xx)?)?;
    context.store_f32(context.sqrt(sum)?)
}

/// Compute the canonical source anchor fields from measured source-row inputs.
///
/// `fields` and `presence_masks` are in source palette order, not reordered
/// runtime order. Each complete140-byte row is required. The literal source
/// class5 midpoint and native getter's presence-spin bit8 are retained.
/// Parentless rows keep the constructor's positive-zero anchors and length.
/// All SSE operations use1F80 with complete caller-state restoration; the x87
/// context is explicit. These finite/bounds errors are interface restrictions,
/// not claims about native error diagnostics. No local quaternion, optional
/// block, reroot, finalized class or evaluated animation binding is inferred.
pub fn source_anchors_from_fields(
    raw_rows: &[u8],
    fields: &[SourceBodyFields],
    presence_masks: &[u32],
    context: Context,
) -> Result<Vec<SourceBodyAnchors>, SourceAnchorsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        let count = fields.len();
        if count == 0 || count > 255 { return Err(SourceAnchorsError::Count(count)); }
        if raw_rows.len() != count*PART_ROW_SIZE || presence_masks.len() != count {
            return Err(SourceAnchorsError::InputLengths {
                rows: count, bytes: raw_rows.len(), masks: presence_masks.len(),
            });
        }
        let rows: Vec<_> = raw_rows.chunks_exact(PART_ROW_SIZE).collect();
        let mut parents = Vec::with_capacity(count);
        let mut child_counts = vec![0u32; count];
        for (row, value) in fields.iter().enumerate() {
            if value.rest.palette_id != row as u32 {
                return Err(SourceAnchorsError::PaletteIdentity { row, palette_id: value.rest.palette_id });
            }
            for (values, offset) in [(&value.rest.center[..], 0x10C),
                (&value.rest.orientation_xyzw[..], 0x118), (&value.local_origin[..], 0x144)] {
                for (i, value) in values.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(SourceAnchorsError::NonFiniteField { row, block_offset: offset+i*4 });
                    }
                }
            }
            let parent = u16::from_le_bytes(rows[row][4..6].try_into().unwrap());
            let parent = if parent == u16::MAX { None } else { Some(usize::from(parent)) };
            if let Some(parent) = parent {
                if parent >= count { return Err(SourceAnchorsError::ParentOutOfBounds { row, parent }); }
                child_counts[parent] += 1;
            }
            parents.push(parent);
        }
        let translation = |row: usize| -> [f32; 3] {
            std::array::from_fn(|i| f32::from_le_bytes(rows[row][0x54+i*4..0x58+i*4].try_into().unwrap()))
        };
        let mut output = Vec::with_capacity(count);
        for (row, value) in fields.iter().enumerate() {
            let mut result = SourceBodyAnchors {
                source_palette_id: row as u32, source_parent: parents[row].map(|p| p as u32),
                child_count: child_counts[row], child_anchor: [0.; 3], parent_anchor: [0.; 3],
                parent_delta: [0.; 3], parent_delta_length: 0.,
            };
            if let Some(parent) = parents[row] {
                let mut point = translation(row);
                if rows[row][0xB] == 5 {
                    let other = translation(parent);
                    // X uses own+parent; Y/Z use parent+own before *0.5.
                    point = [(point[0]+other[0])*0.5, (other[1]+point[1])*0.5, (other[2]+point[2])*0.5];
                }
                result.parent_anchor = parent_local(fields[parent].rest.orientation_xyzw,
                    std::array::from_fn(|i| point[i]-fields[parent].rest.center[i]));
                result.child_anchor = child_local(value.rest.orientation_xyzw,
                    std::array::from_fn(|i| point[i]-value.rest.center[i]));
                //9B5200 keeps separate binary32 products, even for literal1.
                result.parent_delta = if presence_masks[row] & 8 != 0 {
                    std::array::from_fn(|i| result.child_anchor[i]*1.0-value.local_origin[i]*1.0)
                } else { result.child_anchor.map(|v| v*1.0) };
                for (values, offset) in [(&result.child_anchor, 0x344), (&result.parent_anchor, 0x350),
                    (&result.parent_delta, 0x35C)] {
                    for (i, value) in values.iter().enumerate() {
                        if !value.is_finite() {
                            return Err(SourceAnchorsError::NonFiniteOutput { row, block_offset: offset+i*4 });
                        }
                    }
                }
                result.parent_delta_length = retained_length(result.parent_delta, context)
                    .map_err(|error| SourceAnchorsError::X87 { row, error })?;
            }
            output.push(result);
        }
        Ok(output)
    })
}

/// Read the raw canonical rows, conformed geometry and selected dictionary's
/// exact capability preparation, then return anchors in source palette order.
pub fn source_body_anchors_for_creature(
    source: &CreatureData,
    dictionary: &PartCapabilities,
    context: Context,
) -> Result<Vec<SourceBodyAnchors>, SourceAnchorsError> {
    native_sse::scoped(Policy::Mxcsr1f80, || {
        let fields = source_body_fields_for_creature(source, context)?;
        let masks: Vec<_> = (0..source.parts.len()).map(|i|
            prepare_source_capabilities(source, i, dictionary).map(|v| v.presence_mask())
        ).collect::<Result<_, _>>()?;
        source_anchors_from_fields(&source.data0, &fields, &masks, context)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim_body_pose::RestBlock;
    fn context() -> Context { Context::from_control_word(0x27F).unwrap() }
    fn fields(palette_id: u32, center: [f32; 3], local_origin: [f32; 3]) -> SourceBodyFields {
        SourceBodyFields { rest: RestBlock { palette_id, center, orientation_xyzw: [0.,0.,0.,1.] },
            half_extents: [1.;3], local_origin, field324: [0.;3], effects_origin: [0.;3] }
    }
    fn rows(parent: u16, class: u8, a: [f32; 3], b: [f32; 3]) -> Vec<u8> {
        let mut rows = vec![0; 2*140];
        rows[4..6].copy_from_slice(&u16::MAX.to_le_bytes());
        rows[144..146].copy_from_slice(&parent.to_le_bytes());
        rows[151] = class;
        for (row, point) in [a,b].into_iter().enumerate() {
            for (i, v) in point.into_iter().enumerate() {
                rows[row*140+0x54+i*4..row*140+0x58+i*4].copy_from_slice(&v.to_le_bytes());
            }
        }
        rows
    }
    #[test]
    fn parentless_defaults_and_direct_children_use_source_ordinals() {
        let raw = rows(0, 3, [10.,20.,30.], [7.,8.,9.]);
        let input = [fields(0,[1.,2.,3.],[0.;3]), fields(1,[4.,5.,6.],[1.,2.,3.])];
        let out = source_anchors_from_fields(&raw, &input, &[8,0], context()).unwrap();
        assert_eq!(out[0].child_count,1);
        assert_eq!(out[0].parent_anchor.map(f32::to_bits),[0;3]);
        assert_eq!(out[0].parent_delta_length.to_bits(),0);
        assert_eq!(out[1].parent_anchor,[6.;3]);
        assert_eq!(out[1].child_anchor,[3.;3]);
        assert_eq!(out[1].parent_delta,[3.;3]);
        assert_eq!(out[1].parent_delta_length,27.0f32.sqrt());
    }
    #[test]
    fn class_five_midpoint_and_presence_only_spin_change_the_native_fields() {
        let raw = rows(0,5,[0.,2.,4.],[8.,10.,12.]);
        let input = [fields(0,[0.;3],[0.;3]),fields(1,[1.,2.,3.],[0.5,1.,1.5])];
        let out = source_anchors_from_fields(&raw,&input,&[8,8],context()).unwrap();
        assert_eq!(out[1].parent_anchor,[4.,6.,8.]);
        assert_eq!(out[1].child_anchor,[3.,4.,5.]);
        assert_eq!(out[1].parent_delta,[2.5,3.,3.5]);
        let ordinary = rows(0,4,[0.,2.,4.],[8.,10.,12.]);
        let out = source_anchors_from_fields(&ordinary,&input,&[8,0],context()).unwrap();
        assert_eq!(out[1].parent_anchor,[8.,10.,12.]);
        assert_eq!(out[1].parent_delta,[7.,8.,9.]);
    }
    #[test]
    fn malformed_source_parent_and_palette_identity_are_explicit_errors() {
        let raw = rows(2,3,[0.;3],[0.;3]);
        let mut input = [fields(0,[0.;3],[0.;3]),fields(1,[0.;3],[0.;3])];
        assert_eq!(source_anchors_from_fields(&raw,&input,&[8,0],context()),
            Err(SourceAnchorsError::ParentOutOfBounds { row:1,parent:2 }));
        input[0].rest.palette_id=7;
        assert_eq!(source_anchors_from_fields(&raw,&input,&[8,0],context()),
            Err(SourceAnchorsError::PaletteIdentity { row:0,palette_id:7 }));
    }
    #[test]
    #[cfg(any(target_arch="x86",target_arch="x86_64"))]
    fn hostile_ftz_caller_preserves_subnormal_anchor_and_restores_state() {
        fn current() -> u32 {
            let mut v=0u32;
            // SAFETY: bounded same-thread storage on an SSE2 target.
            unsafe { std::arch::asm!("stmxcsr [{v}]",v=in(reg)&mut v,options(nostack,preserves_flags)); }
            v
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                // SAFETY: exact same-thread original control/status word.
                unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&self.0,options(nostack,preserves_flags)); }
            }
        }
        let _restore=Restore(current());
        let raw=std::hint::black_box(rows(0,3,[0.;3],[f32::from_bits(8);3]));
        let input=std::hint::black_box([fields(0,[0.;3],[0.;3]),fields(1,[0.;3],[0.;3])]);
        let hostile=0xDF81u32;
        // SAFETY: defined status/control bits, masked exceptions, no DAZ.
        unsafe { std::arch::asm!("ldmxcsr [{v}]",v=in(reg)&hostile,options(nostack,preserves_flags)); }
        let out=source_anchors_from_fields(&raw,&input,&[8,0],context()).unwrap();
        assert_eq!(out[1].parent_anchor.map(f32::to_bits),[8;3]);
        assert_eq!(out[1].child_anchor.map(f32::to_bits),[8;3]);
        assert_eq!(current(),hostile);
        assert!(source_anchors_from_fields(&raw[..139],&input,&[8,0],context()).is_err());
        assert_eq!(current(),hostile);
    }
}
