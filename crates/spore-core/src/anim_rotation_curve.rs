//! Authored native type2 ROT100 curves before selector/rig-space conversion.
//!
//! The caller supplies the original runtime blend policy and arithmetic state.
//! Native tangent flags select source neighbors or slerp extrapolations; weight
//! uses the same asymmetric scalar curve arithmetic as native position weight.
use crate::{
    anim::{Anim, AnimComponent},
    anim_curve::{self, CurveError, ScalarKey},
    anim_quaternion::{self as quaternion, ArithmeticContext, QuaternionError},
    native_crt_acos::ConstantWindow,
    native_x87::Extended,
};
use std::{cmp::Ordering, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationBlendPolicy {
    /// Original global015514F8 is zero: six slerps after tangent construction.
    Spherical,
    /// Original global015514F8 is nonzero: two slerps, four hemisphere sums
    /// and four normalizations. The source image's initial word is one; the
    /// runtime producer/history remains a separate integration input.
    NormalizedHemisphere,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotationCurveError {
    InvalidChannel,
    PreparedDescriptorMismatch,
    UnsupportedRotationDescriptor,
    InvalidRows,
    NonFiniteInput,
    NonFiniteIntermediate,
    Curve(CurveError),
    Quaternion(QuaternionError),
}
impl fmt::Display for RotationCurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native ANIM rotation curve: {self:?}")
    }
}
impl std::error::Error for RotationCurveError {}
impl From<CurveError> for RotationCurveError {
    fn from(value: CurveError) -> Self {
        Self::Curve(value)
    }
}
impl From<QuaternionError> for RotationCurveError {
    fn from(value: QuaternionError) -> Self {
        Self::Quaternion(value)
    }
}
impl From<crate::native_x87::X87Error> for RotationCurveError {
    fn from(value: crate::native_x87::X87Error) -> Self {
        Self::Quaternion(QuaternionError::X87(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeRotationSample {
    pub rotation_xyzw: [f32; 4],
    pub weight: f32,
    /// Previous, left, right, following source row indices.
    pub neighbors: [usize; 4],
    pub fraction: f32,
}

#[derive(Clone, Copy)]
struct RotationKey {
    quaternion: [f32; 4],
    previous_mode: u8,
    next_mode: u8,
    weight: ScalarKey,
}

fn finite(value: f32) -> Result<f32, RotationCurveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RotationCurveError::NonFiniteIntermediate)
    }
}

fn normalized_blend(
    context: ArithmeticContext,
    left: [f32; 4],
    right: [f32; 4],
    amount: f32,
) -> Result<[f32; 4], RotationCurveError> {
    let complement = finite(1. - amount)?;
    let mut first = [0.; 4];
    let mut second = [0.; 4];
    for i in 0..4 {
        first[i] = finite(left[i] * complement)?;
        second[i] = finite(right[i] * amount)?;
    }
    let sum = quaternion::hemisphere_add_source(context, first, second)?;
    Ok(quaternion::normalize(context, sum)?.quaternion)
}

fn orientation(
    context: ArithmeticContext,
    keys: [RotationKey; 4],
    amount: f32,
    policy: RotationBlendPolicy,
    constants: ConstantWindow<'_>,
) -> Result<[f32; 4], RotationCurveError> {
    let [previous, left, right, following] = keys;
    let first_control = match left.next_mode & 3 {
        1 => quaternion::slerp_extrapolated_source(
            context,
            left.quaternion,
            right.quaternion,
            -1.,
            constants,
        )?,
        2 => right.quaternion,
        _ => previous.quaternion,
    };
    let second_control = match right.previous_mode & 3 {
        1 => quaternion::slerp_extrapolated_source(
            context,
            right.quaternion,
            left.quaternion,
            -1.,
            constants,
        )?,
        2 => left.quaternion,
        _ => following.quaternion,
    };
    // Native normalized-policy t+1 uses x87 FADD/FST32 at99FD26..99FD46;
    // spherical-policy t+1 uses ADDSS at9A0173..9A0181. Preserve their
    // different rounding when x87 RC differs from the outer SSE scope.
    let plus_one = match policy {
        RotationBlendPolicy::Spherical => finite(amount + 1.)?,
        RotationBlendPolicy::NormalizedHemisphere => context.x87.store_f32(
            context
                .x87
                .add(Extended::from_f32(amount)?, Extended::from_i32(1))?,
        )?,
    };
    let first = quaternion::slerp_extrapolated_source(
        context,
        first_control,
        left.quaternion,
        plus_one,
        constants,
    )?;
    let middle = match policy {
        RotationBlendPolicy::Spherical => quaternion::slerp_source(
            context,
            left.quaternion,
            right.quaternion,
            amount,
            constants,
        )?,
        RotationBlendPolicy::NormalizedHemisphere => {
            normalized_blend(context, left.quaternion, right.quaternion, amount)?
        }
    };
    let last = quaternion::slerp_extrapolated_source(
        context,
        right.quaternion,
        second_control,
        finite(amount - 1.)?,
        constants,
    )?;
    let first_amount = finite(plus_one * 0.5)?;
    let second_amount = finite(amount * 0.5)?;
    match policy {
        RotationBlendPolicy::Spherical => {
            let first = quaternion::slerp_source(context, first, middle, first_amount, constants)?;
            let last = quaternion::slerp_source(context, middle, last, second_amount, constants)?;
            Ok(quaternion::slerp_source(
                context, first, last, amount, constants,
            )?)
        }
        RotationBlendPolicy::NormalizedHemisphere => {
            let first = normalized_blend(context, first, middle, first_amount)?;
            let last = normalized_blend(context, middle, last, second_amount)?;
            normalized_blend(context, first, last, amount)
        }
    }
}

/// Evaluate one prepared original ROT component, retaining its source controls.
///
/// Endpoint copies use slot2 even when the interpolation enable bit is clear.
/// Interpolation requires type2 and enable0x10. Unknown/unused ROT bytes are
/// retained by Anim, and are not treated as extra quaternion slope controls.
/// Defensive finite/shape errors bound this API without inventing source error
/// handling. In particular, large finite authored quaternions are never
/// normalized as input or discarded by a unit-quaternion tolerance filter.
pub fn sample_rotation(
    animation: &Anim,
    channel: usize,
    prepared: &[AnimComponent],
    frame: f32,
    context: ArithmeticContext,
    policy: RotationBlendPolicy,
    constants: ConstantWindow<'_>,
) -> Result<Option<NativeRotationSample>, RotationCurveError> {
    if !frame.is_finite() {
        return Err(RotationCurveError::NonFiniteInput);
    }
    let bone = animation
        .bones
        .get(channel)
        .ok_or(RotationCurveError::InvalidChannel)?;
    if prepared.len() != bone.components.len()
        || prepared.iter().zip(&bone.components).any(|(a, b)| {
            a.tag != b.tag
                || a.offset != b.offset
                || a.stride != b.stride
                || a.reserved != b.reserved
        })
    {
        return Err(RotationCurveError::PreparedDescriptorMismatch);
    }
    if usize::try_from(bone.keyframe_count).ok() != Some(bone.keyframes.len()) {
        return Err(RotationCurveError::InvalidRows);
    }
    if bone.keyframes.is_empty() {
        return Ok(None);
    }
    quaternion::scoped(context.sse, || {
        let frame_value = Extended::from_f32(frame)?;
        let mut upper = 0;
        for key in &bone.keyframes {
            if context.x87.compare(
                frame_value,
                anim_curve::unsigned_word(key.frame, context.x87)?,
            )? == Ordering::Less
            {
                break;
            }
            upper += 1;
        }
        let neighbors = anim_curve::key_neighbors(
            bone.keyframes.len(),
            upper,
            animation.header_flags_280 & 8 != 0,
        )?;
        let copied = neighbors[1] == neighbors[2];
        let mut enabled = prepared.iter().filter(|c| {
            c.flags & 0xF == 2
                && if copied {
                    c.slot == 2
                } else {
                    c.flags & 0x10 != 0
                }
        });
        let Some(component) = enabled.next() else {
            return Ok(None);
        };
        if component.slot != 2 || enabled.next().is_some() {
            return Err(RotationCurveError::UnsupportedRotationDescriptor);
        }
        let stride = component.stride as usize;
        let offset = component.offset as usize;
        if component.stride != bone.keyframe_stride
            || offset.checked_add(100).is_none_or(|end| end > stride)
            || bone.keyframes.len().checked_mul(stride) != Some(bone.raw_keyframes.len())
        {
            return Err(RotationCurveError::InvalidRows);
        }
        let key = |row: usize| {
            let bytes = &bone.raw_keyframes[row * stride + offset..row * stride + offset + 100];
            let float = |at| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
            RotationKey {
                quaternion: std::array::from_fn(|i| float(i * 4)),
                previous_mode: bytes[28],
                next_mode: bytes[29],
                weight: ScalarKey {
                    value: float(16),
                    previous_slope: float(84),
                    next_slope: float(88),
                    previous_mode: bytes[92],
                    next_mode: bytes[93],
                },
            }
        };
        if copied {
            let current = key(neighbors[1]);
            if !current.quaternion.iter().all(|v| v.is_finite())
                || !current.weight.value.is_finite()
            {
                return Err(RotationCurveError::NonFiniteInput);
            }
            return Ok(Some(NativeRotationSample {
                rotation_xyzw: current.quaternion,
                weight: current.weight.value,
                neighbors,
                fraction: 0.,
            }));
        }
        let frames = neighbors.map(|i| bone.keyframes[i].frame);
        let numerator = context.x87.sub(
            frame_value,
            anim_curve::unsigned_word(frames[1], context.x87)?,
        )?;
        let denominator =
            anim_curve::unsigned_word(frames[2].wrapping_sub(frames[1]), context.x87)?;
        let amount = context
            .x87
            .store_f32(context.x87.div(numerator, denominator)?)?;
        let t2 = finite(amount * amount)?;
        let t3 = finite(t2 * amount)?;
        let three_t2 = finite(t2 * 3.)?;
        let two_t3 = finite(t3 * 2.)?;
        let basis = [
            finite(finite(1. - three_t2)? + two_t3)?,
            finite(three_t2 - two_t3)?,
            finite(finite(amount - finite(t2 * 2.)?)? + t3)?,
            finite(t3 - t2)?,
        ];
        let duration = finite(animation.duration - 1.)?;
        let keys = neighbors.map(key);
        let rotation = orientation(context, keys, amount, policy, constants)?;
        let weight = anim_curve::scalar(
            keys.map(|key| key.weight),
            frames,
            duration,
            basis,
            true,
            context.x87,
        )?;
        Ok(Some(NativeRotationSample {
            rotation_xyzw: rotation,
            weight,
            neighbors,
            fraction: amount,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::{AnimBone, AnimKeyframe};
    use crate::anim_quaternion::SsePolicy;
    use crate::native_crt_acos::CONSTANT_WINDOW_LENGTH;
    use crate::native_x87::Context;

    fn animation(enabled: bool) -> Anim {
        let mut raw = vec![0u8; 240];
        let mut keyframes = vec![];
        for (i, frame) in [0u32, 4].into_iter().enumerate() {
            let row = &mut raw[i * 120..(i + 1) * 120];
            row[..4].copy_from_slice(&frame.to_le_bytes());
            row[32..36].copy_from_slice(&2f32.to_le_bytes());
            row[36..40].copy_from_slice(&(i as f32).to_le_bytes());
            row[48] = 1;
            row[49] = 1;
            row[112] = 1;
            row[113] = 1;
            keyframes.push(AnimKeyframe {
                frame,
                info: [0; 4],
                position: None,
                rotation: Some([0., 0., 0., 2.]),
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
                has_position: false,
                has_rotation: true,
                channel_flags: 0,
                selector_words: [0; 19],
                components: vec![
                    AnimComponent {
                        flags: 0,
                        tag: *b"INFO",
                        offset: 0,
                        stride: 120,
                        slot: 0,
                        reserved: [0; 3],
                    },
                    AnimComponent {
                        flags: if enabled { 0x12 } else { 2 },
                        tag: *b"ROT\0",
                        offset: 20,
                        stride: 120,
                        slot: 2,
                        reserved: [0; 3],
                    },
                ],
                keyframe_stride: 120,
                keyframes,
                raw_keyframes: raw,
            }],
        }
    }

    fn sample(
        animation: &Anim,
        frame: f32,
        policy: RotationBlendPolicy,
    ) -> Result<Option<NativeRotationSample>, RotationCurveError> {
        let constants = vec![0; CONSTANT_WINDOW_LENGTH];
        sample_rotation(
            animation,
            0,
            &animation.bones[0].components,
            frame,
            ArithmeticContext {
                x87: Context::from_control_word(0x27F).unwrap(),
                sse: SsePolicy::Mxcsr1f80,
            },
            policy,
            ConstantWindow::new(&constants).unwrap(),
        )
    }

    #[test]
    fn endpoint_copy_ignores_interpolation_enable_and_keeps_nonunit_bits() {
        let mut animation = animation(false);
        animation.bones[0].raw_keyframes[36..40].copy_from_slice(&2f32.to_le_bytes());
        let result = sample(&animation, -1., RotationBlendPolicy::Spherical)
            .unwrap()
            .unwrap();
        assert_eq!(result.rotation_xyzw, [0., 0., 0., 2.]);
        assert_eq!(result.weight, 2.);
        assert_eq!(result.neighbors[1..3], [0, 0]);
        assert_eq!(
            sample(&animation, 2., RotationBlendPolicy::Spherical).unwrap(),
            None
        );
    }

    #[test]
    fn runtime_blend_policy_controls_native_normalization() {
        let animation = animation(true);
        let spherical = sample(&animation, 2., RotationBlendPolicy::Spherical)
            .unwrap()
            .unwrap();
        let normalized = sample(&animation, 2., RotationBlendPolicy::NormalizedHemisphere)
            .unwrap()
            .unwrap();
        assert_eq!(spherical.rotation_xyzw, [0., 0., 0., 2.]);
        assert_eq!(normalized.rotation_xyzw, [0., 0., 0., 1.]);
        assert_eq!(spherical.weight, 0.5);
        assert_eq!(normalized.weight, 0.5);
    }

    #[test]
    fn unused_quaternion_control_floats_are_not_quality_filtered() {
        let mut animation = animation(true);
        for row in animation.bones[0].raw_keyframes.chunks_exact_mut(120) {
            row[40..44].copy_from_slice(&f32::NAN.to_le_bytes());
            row[44..48].copy_from_slice(&f32::INFINITY.to_le_bytes());
        }
        assert!(sample(&animation, 2., RotationBlendPolicy::Spherical)
            .unwrap()
            .is_some());
    }

    #[test]
    fn descriptor_extent_and_nonfinite_frame_are_explicit_errors() {
        let mut animation = animation(true);
        assert_eq!(
            sample(&animation, f32::NAN, RotationBlendPolicy::Spherical),
            Err(RotationCurveError::NonFiniteInput)
        );
        animation.bones[0].components[1].offset = 100;
        assert_eq!(
            sample(&animation, 2., RotationBlendPolicy::Spherical),
            Err(RotationCurveError::InvalidRows)
        );
    }
}
