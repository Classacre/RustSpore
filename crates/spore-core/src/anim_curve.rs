//! Recovered ANIM position curves, before selector/rig-space conversion.
//!
//! Native 9A14C0 selects an upper key; 99C6C0 selects four neighbors;
//! 99F6A0 evaluates the enabled position component. Its asymmetric SSE/x87
//! tangent arithmetic and stores are preserved under an explicit x87 context.
//! This module does not evaluate quaternion/deformation components or infer
//! the original process control word or playback-loop state.

use crate::{
    anim::{Anim, AnimComponent},
    native_x87::{Context, Extended, X87Error},
};
use std::{cmp::Ordering, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
    InvalidChannel,
    PreparedDescriptorMismatch,
    UnsupportedPositionDescriptor,
    InvalidRows,
    NonFiniteInput,
    NonFiniteIntermediate,
    X87(X87Error),
}
impl From<X87Error> for CurveError {
    fn from(value: X87Error) -> Self {
        Self::X87(value)
    }
}
impl fmt::Display for CurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native ANIM position curve: {self:?}")
    }
}
impl std::error::Error for CurveError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativePositionSample {
    pub position: [f32; 3],
    pub weight: f32,
    /// Previous, left, right, following, in source row indices.
    pub neighbors: [usize; 4],
    pub fraction: f32,
}

/// 99C6C0's endpoint/wrapped neighbors. Bit8 of ANIM's word280 controls
/// neighbor wrapping; it is separate from the caller's playback-loop mode.
pub fn key_neighbors(count: usize, upper: usize, wrap: bool) -> Result<[usize; 4], CurveError> {
    if count == 0 || upper > count || count > i32::MAX as usize {
        return Err(CurveError::InvalidRows);
    }
    let right = upper.min(count - 1);
    let left = if upper == count {
        right
    } else {
        upper.saturating_sub(1)
    };
    let previous = if left > 0 {
        left - 1
    } else if wrap {
        (count - (2 % count)) % count
    } else {
        left
    };
    let following = if right < count - 1 {
        right + 1
    } else if wrap {
        (right + 2) % count
    } else {
        right
    };
    Ok([previous, left, right, following])
}

/// Native FILD signed-word conversion followed by conditional FADD 2^32.
/// Keeping the FILD value avoids prematurely rounding a frame to binary32.
pub(crate) fn unsigned_word(value: u32, ctx: Context) -> Result<Extended, CurveError> {
    let signed = Extended::from_i32(value as i32);
    if value & 0x8000_0000 == 0 {
        Ok(signed)
    } else {
        Ok(ctx.add(signed, Extended::from_f32(4294967296.)?)?)
    }
}
fn finite(value: f32) -> Result<f32, CurveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(CurveError::NonFiniteIntermediate)
    }
}
fn load(value: f32) -> Result<Extended, CurveError> {
    if !value.is_finite() {
        return Err(CurveError::NonFiniteInput);
    }
    Ok(Extended::from_f32(value)?)
}

#[derive(Clone, Copy)]
pub(crate) struct ScalarKey {
    pub(crate) value: f32,
    pub(crate) previous_slope: f32,
    pub(crate) next_slope: f32,
    pub(crate) previous_mode: u8,
    pub(crate) next_mode: u8,
}

pub(crate) fn scalar(
    keys: [ScalarKey; 4],
    frames: [u32; 4],
    duration_minus_one: f32,
    basis: [f32; 4],
    weight: bool,
    ctx: Context,
) -> Result<f32, CurveError> {
    let [previous, left, right, following] = keys;
    let [pf, lf, rf, ff] = frames;
    load(left.value)?;
    load(right.value)?;
    let duration = load(duration_minus_one)?;
    let previous_span = if lf < pf {
        ctx.sub(duration, unsigned_word(pf, ctx)?)?
    } else {
        unsigned_word(lf.wrapping_sub(pf), ctx)?
    };
    let previous_span = ctx.store_f32(previous_span)?;
    let left_span = unsigned_word(if lf <= rf { rf.wrapping_sub(lf) } else { rf }, ctx)?;
    let left_span = ctx.store_f32(left_span)?;
    // The left auto tangent is scalar SSE, including separate rounded add/div/mul.
    let left_tangent = match left.next_mode & 3 {
        0 => {
            load(previous.value)?;
            let difference = finite(right.value - previous.value)?;
            let denominator = finite(left_span + previous_span)?;
            finite(finite(difference / denominator)? * left_span)?
        }
        1 => finite(right.value - left.value)?,
        2 => 0.,
        _ => {
            load(left.next_slope)?;
            finite(left.next_slope * left_span)?
        }
    };
    let right_span = if rf < lf {
        ctx.sub(duration, unsigned_word(lf, ctx)?)?
    } else {
        unsigned_word(rf.wrapping_sub(lf), ctx)?
    };
    let committed_right_span = ctx.store_f32(right_span)?;
    // The right auto tangent retains FILD spans and x87 subtraction/division
    // until its final binary32 store. Other modes consume the committed span.
    let right_tangent = match right.previous_mode & 3 {
        0 => {
            let following_span =
                unsigned_word(if rf <= ff { ff.wrapping_sub(rf) } else { ff }, ctx)?;
            let difference = ctx.sub(load(following.value)?, load(left.value)?)?;
            let denominator = ctx.add(following_span, right_span)?;
            ctx.store_f32(ctx.mul(ctx.div(difference, denominator)?, right_span)?)?
        }
        1 => finite(right.value - left.value)?,
        2 => 0.,
        _ => {
            load(right.previous_slope)?;
            finite(right.previous_slope * committed_right_span)?
        }
    };
    let [h00, h01, h10, h11] = basis;
    // Weight's two endpoint products have the opposite source operand order.
    let first = if weight {
        finite(finite(right.value * h01)? + finite(left.value * h00)?)?
    } else {
        finite(finite(left.value * h00)? + finite(right.value * h01)?)?
    };
    let second = finite(first + finite(right_tangent * h11)?)?;
    let result = finite(second + finite(left_tangent * h10)?)?;
    Ok(if weight {
        if result > 0. {
            result.min(1.)
        } else {
            0.
        }
    } else {
        result
    })
}

/// Evaluate only the native position component of a prepared source channel.
///
/// `prepared` is the corresponding descriptor slice returned by anim_deform.
/// Its geometry/tag/stride identity is checked against the parsed source.
/// Time is in source frames. No generic clamp/loop policy is added: endpoint
/// copies and duplicate-time behavior follow the native key-index helper.
/// Finite checked errors bound source memory/arithmetic domains without
/// pretending they are the original executable's error handling.
pub fn sample_position(
    animation: &Anim,
    channel: usize,
    prepared: &[AnimComponent],
    frame: f32,
    ctx: Context,
) -> Result<Option<NativePositionSample>, CurveError> {
    if !frame.is_finite() {
        return Err(CurveError::NonFiniteInput);
    }
    let bone = animation
        .bones
        .get(channel)
        .ok_or(CurveError::InvalidChannel)?;
    if prepared.len() != bone.components.len()
        || prepared.iter().zip(&bone.components).any(|(a, b)| {
            a.tag != b.tag
                || a.offset != b.offset
                || a.stride != b.stride
                || a.reserved != b.reserved
        })
    {
        return Err(CurveError::PreparedDescriptorMismatch);
    }
    if usize::try_from(bone.keyframe_count).ok() != Some(bone.keyframes.len()) {
        return Err(CurveError::InvalidRows);
    }
    if bone.keyframes.is_empty() {
        return Ok(None);
    }
    let frame_value = load(frame)?;
    let mut upper = 0;
    for key in &bone.keyframes {
        if ctx.compare(frame_value, unsigned_word(key.frame, ctx)?)? == Ordering::Less {
            break;
        }
        upper += 1;
    }
    let neighbors = key_neighbors(
        bone.keyframes.len(),
        upper,
        animation.header_flags_280 & 8 != 0,
    )?;
    // The direct-copy helper tests slots, while interpolation tests enable.
    // A matched source POS with enable clear can still enter endpoint copying.
    let copied = neighbors[1] == neighbors[2];
    let mut enabled = prepared.iter().filter(|c| {
        c.flags & 0xf == 1
            && if copied {
                c.slot == 1
            } else {
                c.flags & 0x10 != 0
            }
    });
    let Some(component) = enabled.next() else {
        return Ok(None);
    };
    if component.slot != 1 || enabled.next().is_some() {
        return Err(CurveError::UnsupportedPositionDescriptor);
    }
    let stride = usize::try_from(component.stride).map_err(|_| CurveError::InvalidRows)?;
    let offset = usize::try_from(component.offset).map_err(|_| CurveError::InvalidRows)?;
    if component.stride != bone.keyframe_stride
        || offset.checked_add(80).is_none_or(|end| end > stride)
        || bone.keyframes.len().checked_mul(stride) != Some(bone.raw_keyframes.len())
    {
        return Err(CurveError::InvalidRows);
    }
    let key = |row: usize, axis: usize| {
        let data = &bone.raw_keyframes[row * stride + offset..row * stride + offset + 80];
        let float = |at| f32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        let control = 16 + axis * 16;
        ScalarKey {
            value: float(axis * 4),
            previous_slope: float(control),
            next_slope: float(control + 4),
            previous_mode: data[control + 8],
            next_mode: data[control + 9],
        }
    };
    if neighbors[1] == neighbors[2] {
        let mut values = [0.; 4];
        for (axis, value) in values.iter_mut().enumerate() {
            *value = key(neighbors[1], axis).value;
            load(*value)?;
        }
        return Ok(Some(NativePositionSample {
            position: values[..3].try_into().unwrap(),
            weight: values[3],
            neighbors,
            fraction: 0.,
        }));
    }
    let frames = neighbors.map(|i| bone.keyframes[i].frame);
    let numerator = ctx.sub(frame_value, unsigned_word(frames[1], ctx)?)?;
    let denominator = unsigned_word(frames[2].wrapping_sub(frames[1]), ctx)?;
    let t = ctx.store_f32(ctx.div(numerator, denominator)?)?;
    let t2 = finite(t * t)?;
    let t3 = finite(t2 * t)?;
    let three_t2 = finite(t2 * 3.)?;
    let two_t3 = finite(t3 * 2.)?;
    let h00 = finite(finite(1. - three_t2)? + two_t3)?;
    let h01 = finite(three_t2 - two_t3)?;
    let h10 = finite(finite(t - finite(t2 * 2.)?)? + t3)?;
    let h11 = finite(t3 - t2)?;
    let duration_minus_one = finite(animation.duration - 1.)?;
    let mut values = [0.; 4];
    for (axis, value) in values.iter_mut().enumerate() {
        *value = scalar(
            neighbors.map(|i| key(i, axis)),
            frames,
            duration_minus_one,
            [h00, h01, h10, h11],
            axis == 3,
            ctx,
        )?;
    }
    Ok(Some(NativePositionSample {
        position: values[..3].try_into().unwrap(),
        weight: values[3],
        neighbors,
        fraction: t,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{AnimBone, AnimKeyframe};
    use crate::native_x87::{Precision, Rounding};
    fn context() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }
    fn key(value: f32, mode: u8) -> ScalarKey {
        ScalarKey {
            value,
            previous_slope: 0.,
            next_slope: 0.,
            previous_mode: mode,
            next_mode: mode,
        }
    }
    fn two_key_animation() -> Anim {
        let components = vec![
            AnimComponent {
                flags: 0,
                tag: *b"INFO",
                offset: 0,
                stride: 100,
                slot: 0,
                reserved: [0; 3],
            },
            AnimComponent {
                flags: 0x11,
                tag: *b"POS\0",
                offset: 20,
                stride: 100,
                slot: 1,
                reserved: [0; 3],
            },
        ];
        let mut raw = vec![0; 200];
        let mut keys = vec![];
        for (index, frame) in [0_u32, 4].into_iter().enumerate() {
            let row = &mut raw[index * 100..(index + 1) * 100];
            row[..4].copy_from_slice(&frame.to_le_bytes());
            for (axis, value) in [index as f32 * 8., 0., 0., 2.].into_iter().enumerate() {
                row[20 + axis * 4..24 + axis * 4].copy_from_slice(&value.to_le_bytes());
                row[20 + 16 + axis * 16 + 8] = 1;
                row[20 + 16 + axis * 16 + 9] = 1;
            }
            keys.push(AnimKeyframe {
                frame,
                info: [0; 4],
                position: Some([index as f32 * 8., 0., 0.]),
                rotation: None,
            });
        }
        Anim {
            path: String::new(),
            hash: 0,
            header_flags_280: 0,
            frame_duration: 1.,
            duration: 5.,
            last_frame: 4,
            bone_count: 1,
            header_field_304: 0,
            anim_name: String::new(),
            bones: vec![AnimBone {
                name: String::new(),
                keyframe_count: 2,
                has_position: true,
                has_rotation: false,
                channel_flags: 0,
                selector_words: [0; 19],
                components,
                keyframe_stride: 100,
                keyframes: keys,
                raw_keyframes: raw,
            }],
        }
    }
    #[test]
    fn interleaved_rows_preserve_endpoint_weight_and_enable_semantics() {
        let animation = two_key_animation();
        let prepared = &animation.bones[0].components;
        let sampled = sample_position(&animation, 0, prepared, 1., context())
            .unwrap()
            .unwrap();
        assert_eq!(sampled.position, [2., 0., 0.]);
        assert_eq!(sampled.weight, 1.);
        assert_eq!(sampled.neighbors, [0, 0, 1, 1]);
        assert_eq!(sampled.fraction, 0.25);
        let endpoint = sample_position(&animation, 0, prepared, 4., context())
            .unwrap()
            .unwrap();
        assert_eq!((endpoint.position, endpoint.weight), ([8., 0., 0.], 2.));
        let mut disabled = prepared.clone();
        disabled[1].flags &= !0x10;
        assert_eq!(
            sample_position(&animation, 0, &disabled, 1., context()).unwrap(),
            None
        );
        assert_eq!(
            sample_position(&animation, 0, &disabled, 4., context())
                .unwrap()
                .unwrap()
                .weight,
            2.
        );
    }
    #[test]
    fn inconsistent_rows_and_prepared_offsets_are_rejected() {
        let mut animation = two_key_animation();
        let prepared = animation.bones[0].components.clone();
        let mut changed = prepared.clone();
        changed[1].offset += 4;
        assert_eq!(
            sample_position(&animation, 0, &changed, 1., context()),
            Err(CurveError::PreparedDescriptorMismatch)
        );
        animation.bones[0].keyframe_count = 1;
        assert_eq!(
            sample_position(&animation, 0, &prepared, 1., context()),
            Err(CurveError::InvalidRows)
        );
        animation.bones[0].keyframe_count = 2;
        animation.bones[0].keyframe_stride = 99;
        assert_eq!(
            sample_position(&animation, 0, &prepared, 1., context()),
            Err(CurveError::InvalidRows)
        );
        animation.bones[0].keyframe_stride = 100;
        animation.bones[0].raw_keyframes.pop();
        assert_eq!(
            sample_position(&animation, 0, &prepared, 1., context()),
            Err(CurveError::InvalidRows)
        );
    }
    #[test]
    fn native_neighbors_skip_duplicate_loop_endpoints() {
        assert_eq!(key_neighbors(4, 0, false).unwrap(), [0, 0, 0, 1]);
        assert_eq!(key_neighbors(4, 1, false).unwrap(), [0, 0, 1, 2]);
        assert_eq!(key_neighbors(4, 1, true).unwrap(), [2, 0, 1, 2]);
        assert_eq!(key_neighbors(4, 3, true).unwrap(), [1, 2, 3, 1]);
        assert_eq!(key_neighbors(4, 4, true).unwrap(), [2, 3, 3, 1]);
        assert_eq!(key_neighbors(1, 1, true).unwrap(), [0, 0, 0, 0]);
        assert_eq!(key_neighbors(0, 0, true), Err(CurveError::InvalidRows));
    }
    #[test]
    fn authored_tangents_change_the_cubic_and_weight_has_native_clamp() {
        let basis = [0.84375, 0.15625, 0.140625, -0.046875];
        let frames = [0, 0, 4, 4];
        let secant = [key(0., 1), key(0., 1), key(8., 1), key(8., 1)];
        let flat = secant.map(|mut k| {
            k.previous_mode = 2;
            k.next_mode = 2;
            k
        });
        assert_eq!(
            scalar(secant, frames, 4., basis, false, context()).unwrap(),
            2.
        );
        assert_eq!(
            scalar(flat, frames, 4., basis, false, context()).unwrap(),
            1.25
        );
        let mut explicit = flat;
        explicit[1].next_mode = 3;
        explicit[1].next_slope = 4.;
        assert_eq!(
            scalar(explicit, frames, 4., basis, false, context()).unwrap(),
            3.5
        );
        assert_eq!(
            scalar(explicit, frames, 4., basis, true, context()).unwrap(),
            1.
        );
        explicit[1].next_mode = 0x82;
        explicit[1].next_slope = f32::NAN;
        assert_eq!(
            scalar(explicit, frames, 4., basis, false, context()).unwrap(),
            1.25
        );
    }
}
