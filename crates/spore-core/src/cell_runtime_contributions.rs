//! Original99D190 scalar contributions into explicit pose+A0 snapshots.
//!
//! This boundary starts with an already selected binding's low24 component
//! mask, prior POS/ROT descriptor masks and six sampled scalar/weight slots.
//! Active clip selection, advancing time, binding-to-pose assignment and reset
//! order remain caller inputs. The angular40 branch is explicitly unsupported
//! when reached; no sin/cos substitute or alternate gameplay rule is supplied.

use crate::{
    anim::AnimComponent,
    cell_runtime_data::{CellRuntimeDataAccumulator, CellRuntimeDataError, CellRuntimeDataWrite},
    native_sse::{self, Policy, SseError},
    native_x87::Context,
};
use std::fmt;

pub const SAMPLED_SCALAR_SLOTS: usize = 6;
pub const DESCRIPTOR_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeScalarSlot {
    pub value_bits: u32,
    pub weight_bits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeContributionContext {
    pub x87: Context,
    pub sse: Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellRuntimeContributionError {
    BindingMaskOutsideLow24(u32),
    DescriptorCapacity(usize),
    AngularDescriptor(usize),
    SampleSlot { descriptor: usize, slot: u32 },
    Data(CellRuntimeDataError),
    Sse(SseError),
}
impl From<CellRuntimeDataError> for CellRuntimeContributionError {
    fn from(error: CellRuntimeDataError) -> Self {
        Self::Data(error)
    }
}
impl From<SseError> for CellRuntimeContributionError {
    fn from(error: SseError) -> Self {
        Self::Sse(error)
    }
}
impl fmt::Display for CellRuntimeContributionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bounded native Cell scalar contribution: {self:?}"
        )
    }
}
impl std::error::Error for CellRuntimeContributionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeScalarContribution {
    pub descriptor: usize,
    pub tag: u32,
    pub raw_kind: u32,
    /// Result of sampleWeight-left MULSS callerBlend-right.
    pub scaled_weight_bits: u32,
    /// The same result passed through original FLD32/FSTP32 for the writer.
    pub weight_argument_bits: u32,
    /// Result of sampleValue-left MULSS scaledWeight-right.
    pub weighted_value_bits: u32,
    pub write: CellRuntimeDataWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRuntimeContributions {
    /// Original COMISS(+0,blend)/JAE: unordered blends take the active path.
    pub active: bool,
    /// The low24 binding mask after removing the earlier POS and ROT masks.
    /// The original tail does not clear each bit as it processes a descriptor.
    pub remaining_mask: u32,
    pub contributions: Vec<CellRuntimeScalarContribution>,
}

/// Apply the original ordered scalar branch to an explicit aggregate.
///
/// Native99D190 masks the packed binding word to24 bits before this boundary.
/// This bounded adapter requires that already extracted mask and at most32
/// descriptors. The source SHL index is therefore its literal descriptor bit;
/// no arbitrary mapping from tags to component indices is introduced.
/// Non-kind3 and masked descriptors do not consume their slot/value fields.
/// Reached angular descriptors or out-of-range slots are checked unsupported
/// inputs, with no fabricated result. Earlier accepted writes remain observable
/// if a later unsupported descriptor is reached.
///
/// All binary32 input/result patterns are retained, including nonfinite sample
/// values and blends. The separate finite reader can subsequently report a
/// reached unsupported division. There is no positive-weight shortcut before
/// the original99C170 call: zero/negative/NaN products still reach that writer.
pub fn accumulate_scalar_components(
    aggregate: &mut CellRuntimeDataAccumulator,
    descriptors: &[AnimComponent],
    slots: &[CellRuntimeScalarSlot; SAMPLED_SCALAR_SLOTS],
    binding_component_mask: u32,
    prior_position_mask: u32,
    prior_rotation_mask: u32,
    blend_bits: u32,
    context: CellRuntimeContributionContext,
) -> Result<CellRuntimeContributions, CellRuntimeContributionError> {
    native_sse::scoped(context.sse, || {
        let active = source_blend_is_active(blend_bits)?;
        if !active {
            return Ok(CellRuntimeContributions {
                active: false,
                remaining_mask: binding_component_mask,
                contributions: Vec::new(),
            });
        }
        if binding_component_mask & 0xFF00_0000 != 0 {
            return Err(CellRuntimeContributionError::BindingMaskOutsideLow24(
                binding_component_mask,
            ));
        }
        if descriptors.len() > DESCRIPTOR_CAPACITY {
            return Err(CellRuntimeContributionError::DescriptorCapacity(
                descriptors.len(),
            ));
        }
        let remaining_mask = binding_component_mask & !prior_position_mask & !prior_rotation_mask;
        let mut contributions = Vec::new();
        if remaining_mask != 0 {
            for (index, descriptor) in descriptors.iter().enumerate() {
                if remaining_mask & (1u32 << index) == 0 || descriptor.flags & 0xF != 3 {
                    continue;
                }
                if descriptor.flags & 0x40 != 0 {
                    return Err(CellRuntimeContributionError::AngularDescriptor(index));
                }
                if !(3..9).contains(&descriptor.slot) {
                    return Err(CellRuntimeContributionError::SampleSlot {
                        descriptor: index,
                        slot: descriptor.slot,
                    });
                }
                let slot = slots[(descriptor.slot - 3) as usize];
                let (scaled_weight_bits, weight_argument_bits, weighted_value_bits) =
                    source_products(slot, blend_bits, context.x87)?;
                let raw_kind = (descriptor.flags >> 7) & 1;
                let tag = u32::from_le_bytes(descriptor.tag);
                let write = aggregate.accumulate_scalar(
                    tag,
                    raw_kind,
                    weighted_value_bits,
                    weight_argument_bits,
                    context.sse,
                )?;
                contributions.push(CellRuntimeScalarContribution {
                    descriptor: index,
                    tag,
                    raw_kind,
                    scaled_weight_bits,
                    weight_argument_bits,
                    weighted_value_bits,
                    write,
                });
            }
        }
        Ok(CellRuntimeContributions {
            active: true,
            remaining_mask,
            contributions,
        })
    })
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn source_blend_is_active(blend: u32) -> Result<bool, SseError> {
    let active: u8;
    // SAFETY: register-only original COMISS operand order under the scoped
    // masked SSE policy. SETB retains CF=1 on unordered NaNs as well as +values.
    unsafe {
        std::arch::asm!(
            "xorps {zero}, {zero}",
            "movd {blend_value}, {blend:e}",
            "comiss {zero}, {blend_value}",
            "setb {active}",
            zero = out(xmm_reg) _,
            blend_value = out(xmm_reg) _,
            blend = in(reg) blend,
            active = lateout(reg_byte) active,
            options(nostack),
        );
    }
    Ok(active != 0)
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn source_blend_is_active(_: u32) -> Result<bool, SseError> {
    Err(SseError::UnsupportedArchitecture)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn source_products(
    slot: CellRuntimeScalarSlot,
    blend: u32,
    context: Context,
) -> Result<(u32, u32, u32), SseError> {
    #[repr(C, align(16))]
    struct Environment([u8; 32]);
    let mut environment = Environment([0; 32]);
    let control = context.control_word();
    let mut scaled_weight = 0u32;
    let mut weight_argument = 0u32;
    let weighted_value: u32;
    // SAFETY: all pointers are bounded locals. This preserves the original
    // weight/blend MULSS, stack commit, FLD/FSTP argument and value/weight MULSS
    // order, including raw NaN operands. All x87 registers are clobbered per ABI,
    // both stack operations balance, and FLDENV restores the caller environment.
    unsafe {
        std::arch::asm!(
            "movd {weight_value}, {weight:e}",
            "movd {blend_value}, {blend:e}",
            "mulss {weight_value}, {blend_value}",
            "movd dword ptr [{scaled_weight}], {weight_value}",
            "movd {sample_value}, {value:e}",
            "fnstenv [{environment}]",
            "fldcw [{control}]",
            "fld dword ptr [{scaled_weight}]",
            "fstp dword ptr [{weight_argument}]",
            "mulss {sample_value}, {weight_value}",
            "movd {weighted_value:e}, {sample_value}",
            "fldenv [{environment}]",
            weight_value = out(xmm_reg) _,
            blend_value = out(xmm_reg) _,
            sample_value = out(xmm_reg) _,
            weight = in(reg) slot.weight_bits,
            blend = in(reg) blend,
            value = in(reg) slot.value_bits,
            environment = in(reg) &mut environment,
            control = in(reg) &control,
            scaled_weight = in(reg) &mut scaled_weight,
            weight_argument = in(reg) &mut weight_argument,
            weighted_value = out(reg) weighted_value,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    Ok((scaled_weight, weight_argument, weighted_value))
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
fn source_products(
    _: CellRuntimeScalarSlot,
    _: u32,
    _: Context,
) -> Result<(u32, u32, u32), SseError> {
    Err(SseError::UnsupportedArchitecture)
}
