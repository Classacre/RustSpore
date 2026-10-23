//! Source kind3 scalar/weight curves before runtime-data accumulation.
//!
//! This is the two-value branch99F8B6..99FAF7 of99F6A0, with the common
//! fraction/basis and99C6C0 neighbor selection. It does not choose active clips,
//! map root selectors to a pose or evaluate the later angular sin/cos path.

use crate::{
    anim::{Anim, AnimComponent},
    anim_curve::{key_neighbors, scalar, unsigned_word, CurveError, ScalarKey},
    native_sse::{self, Policy, SseError},
    native_x87::{Context, Extended},
};
use std::{cmp::Ordering, fmt};

pub const CUSTOM_ROW_SIZE: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRuntimeCurveContext {
    pub x87: Context,
    pub sse: Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellRuntimeCurveError {
    InvalidComponent,
    UnsupportedDescriptor,
    Curve(CurveError),
    Sse(SseError),
}
impl From<CurveError> for CellRuntimeCurveError {
    fn from(value: CurveError) -> Self {
        Self::Curve(value)
    }
}
impl From<SseError> for CellRuntimeCurveError {
    fn from(value: SseError) -> Self {
        Self::Sse(value)
    }
}
impl From<crate::native_x87::X87Error> for CellRuntimeCurveError {
    fn from(value: crate::native_x87::X87Error) -> Self {
        Self::Curve(CurveError::X87(value))
    }
}
impl fmt::Display for CellRuntimeCurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bounded native runtime-data curve: {self:?}")
    }
}
impl std::error::Error for CellRuntimeCurveError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellRuntimeCurveSample {
    pub value: f32,
    pub weight: f32,
    pub neighbors: [usize; 4],
    pub fraction: f32,
    /// Native99F630 copies the full40 bytes at an endpoint. Interpolation
    /// writes only the first two words; its other destination bytes are unused
    /// prior state, not newly synthesized curve data.
    pub endpoint_bytes: Option<[u8; CUSTOM_ROW_SIZE]>,
}

/// Evaluate one explicitly prepared source kind3 descriptor and source frame.
///
/// Both scalar values use the left-first endpoint product order at99FA9D/A1/A5.
/// This matches the recovered scalar helper's value path, including its SSE
/// left-auto tangent and retained-x87 right-auto tangent. The POS weight path
/// has a different endpoint order and is deliberately not selected here.
/// Source flag20 bypasses the clamp for both values. Endpoint copies bypass
/// interpolation/enable/clamp and preserve the complete40-byte source row.
///
/// Reached nonfinite inputs/intermediates are checked API errors. Some retail
/// nonfinite arithmetic can later be masked by its clamp; this finite adapter
/// reports the reached error instead of fabricating an intermediate. No
/// original entry-thread floating policy or active animation state is assumed.
pub fn sample_component(
    animation: &Anim,
    channel_index: usize,
    prepared: &[AnimComponent],
    component_index: usize,
    frame: f32,
    context: CellRuntimeCurveContext,
) -> Result<Option<CellRuntimeCurveSample>, CellRuntimeCurveError> {
    native_sse::scoped(context.sse, || {
        sample_impl(
            animation,
            channel_index,
            prepared,
            component_index,
            frame,
            context.x87,
        )
    })
}

fn finite_input(value: f32) -> Result<Extended, CellRuntimeCurveError> {
    if !value.is_finite() {
        return Err(CurveError::NonFiniteInput.into());
    }
    Ok(Extended::from_f32(value)?)
}

fn finite(value: f32) -> Result<f32, CellRuntimeCurveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CurveError::NonFiniteIntermediate.into())
    }
}

fn sample_impl(
    animation: &Anim,
    channel_index: usize,
    prepared: &[AnimComponent],
    component_index: usize,
    frame: f32,
    context: Context,
) -> Result<Option<CellRuntimeCurveSample>, CellRuntimeCurveError> {
    let frame_value = finite_input(frame)?;
    let bone = animation
        .bones
        .get(channel_index)
        .ok_or(CurveError::InvalidChannel)?;
    if prepared.len() != bone.components.len()
        || prepared.iter().zip(&bone.components).any(|(a, b)| {
            a.tag != b.tag
                || a.offset != b.offset
                || a.stride != b.stride
                || a.reserved != b.reserved
        })
    {
        return Err(CurveError::PreparedDescriptorMismatch.into());
    }
    let component = prepared
        .get(component_index)
        .ok_or(CellRuntimeCurveError::InvalidComponent)?;
    if component.flags & 0xF != 3 || bone.components[component_index].flags & 0xF != 3 {
        return Err(CellRuntimeCurveError::UnsupportedDescriptor);
    }
    if usize::try_from(bone.keyframe_count).ok() != Some(bone.keyframes.len()) {
        return Err(CurveError::InvalidRows.into());
    }
    if bone.keyframes.is_empty() {
        return Ok(None);
    }
    let mut upper = 0;
    for key in &bone.keyframes {
        if context.compare(frame_value, unsigned_word(key.frame, context)?)? == Ordering::Less {
            break;
        }
        upper += 1;
    }
    let neighbors = key_neighbors(
        bone.keyframes.len(),
        upper,
        animation.header_flags_280 & 8 != 0,
    )?;
    let copied = neighbors[1] == neighbors[2];
    if copied {
        if component.slot >= 9 {
            return Ok(None);
        }
    } else if component.flags & 0x10 == 0 {
        return Ok(None);
    }
    if !(3..9).contains(&component.slot) {
        return Err(CellRuntimeCurveError::UnsupportedDescriptor);
    }
    let stride = usize::try_from(component.stride).map_err(|_| CurveError::InvalidRows)?;
    let offset = usize::try_from(component.offset).map_err(|_| CurveError::InvalidRows)?;
    if component.stride != bone.keyframe_stride
        || offset
            .checked_add(CUSTOM_ROW_SIZE)
            .is_none_or(|end| end > stride)
        || bone.keyframes.len().checked_mul(stride) != Some(bone.raw_keyframes.len())
    {
        return Err(CurveError::InvalidRows.into());
    }
    let row = |index: usize| -> &[u8] {
        &bone.raw_keyframes[index * stride + offset..index * stride + offset + CUSTOM_ROW_SIZE]
    };
    let key = |index: usize, scalar_index: usize| {
        let raw = row(index);
        let float = |at| f32::from_le_bytes(raw[at..at + 4].try_into().unwrap());
        let control = 8 + scalar_index * 16;
        ScalarKey {
            value: float(scalar_index * 4),
            previous_slope: float(control),
            next_slope: float(control + 4),
            previous_mode: raw[control + 8],
            next_mode: raw[control + 9],
        }
    };
    if copied {
        let raw: [u8; CUSTOM_ROW_SIZE] = row(neighbors[1]).try_into().unwrap();
        let value = f32::from_le_bytes(raw[..4].try_into().unwrap());
        let weight = f32::from_le_bytes(raw[4..8].try_into().unwrap());
        finite_input(value)?;
        finite_input(weight)?;
        return Ok(Some(CellRuntimeCurveSample {
            value,
            weight,
            neighbors,
            fraction: 0.,
            endpoint_bytes: Some(raw),
        }));
    }
    let frames = neighbors.map(|index| bone.keyframes[index].frame);
    let numerator = context.sub(frame_value, unsigned_word(frames[1], context)?)?;
    let denominator = unsigned_word(frames[2].wrapping_sub(frames[1]), context)?;
    let fraction = context.store_f32(context.div(numerator, denominator)?)?;
    let squared = finite(fraction * fraction)?;
    let cubed = finite(squared * fraction)?;
    let three_squared = finite(squared * 3.)?;
    let two_cubed = finite(cubed * 2.)?;
    let h00 = finite(finite(1. - three_squared)? + two_cubed)?;
    let h01 = finite(three_squared - two_cubed)?;
    let h10 = finite(finite(fraction - finite(squared * 2.)?)? + cubed)?;
    let h11 = finite(cubed - squared)?;
    let duration_minus_one = finite(animation.duration - 1.)?;
    let mut sampled = [0.; 2];
    for (scalar_index, value) in sampled.iter_mut().enumerate() {
        *value = scalar(
            neighbors.map(|index| key(index, scalar_index)),
            frames,
            duration_minus_one,
            [h00, h01, h10, h11],
            false,
            context,
        )?;
        if component.flags & 0x20 == 0 {
            // Native strict COMISS/JA replaces both signs of zero with+0.
            *value = if *value > 0. { (*value).min(1.) } else { 0. };
        }
    }
    Ok(Some(CellRuntimeCurveSample {
        value: sampled[0],
        weight: sampled[1],
        neighbors,
        fraction,
        endpoint_bytes: None,
    }))
}
