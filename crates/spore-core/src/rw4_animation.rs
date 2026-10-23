//! Typed RW4 skeletal animation objects measured from user-supplied game data.
//!
//! This module never evaluates opaque channel kinds as if they were transforms.
//! Scalar (0x100), rotation/translation (0x101), and scaled transforms (0x601)
//! retain their source key times. Sampling uses an explicit RustSpore policy:
//! linear scalar/vector interpolation and shortest-arc quaternion slerp. This
//! policy is not claimed to reproduce the original engine's interpolation.

use crate::rw4::{self, object_types, Rw4};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationError {
    Malformed { section: u32, reason: &'static str },
    UnsupportedTrack { kind: u32, stride: u32 },
    UnsupportedSkinning(&'static str),
    InvalidTime,
    InvalidQuaternion,
    EmptyTrack,
}

impl fmt::Display for AnimationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { section, reason } => {
                write!(f, "RW4 animation section {section}: {reason}")
            }
            Self::UnsupportedTrack { kind, stride } => {
                write!(f, "unsupported RW4 track {kind:#x}, stride {stride}")
            }
            Self::UnsupportedSkinning(reason) => write!(f, "unsupported RW4 skinning: {reason}"),
            Self::InvalidTime => f.write_str("animation sample time must be finite"),
            Self::InvalidQuaternion => {
                f.write_str("animation quaternion has zero or non-finite norm")
            }
            Self::EmptyTrack => f.write_str("animation track has no keys"),
        }
    }
}
impl std::error::Error for AnimationError {}

#[derive(Debug, Clone)]
pub struct Indexed<T> {
    pub section_index: u32,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    pub id: u32,
    pub flags: u32,
    pub parent: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    pub id: u32,
    pub flags: u32,
    pub bones: Vec<Bone>,
}

/// Three affine rows, multiplying a column vector [x,y,z,1].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineTransform {
    pub rows: [[f32; 4]; 3],
}

impl AffineTransform {
    pub const IDENTITY: Self = Self {
        rows: [[1., 0., 0., 0.], [0., 1., 0., 0.], [0., 0., 1., 0.]],
    };

    /// RW4 AnimationSkin stores basis columns and a translation column, each
    /// padded to four words. Padding words are not used as homogeneous values.
    pub fn from_skin_columns(v: [f32; 16]) -> Self {
        Self {
            rows: [
                [v[0], v[4], v[8], v[12]],
                [v[1], v[5], v[9], v[13]],
                [v[2], v[6], v[10], v[14]],
            ],
        }
    }

    pub fn transform_point(self, p: [f32; 3]) -> [f32; 3] {
        self.rows
            .map(|r| r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + r[3])
    }

    pub fn transform_vector(self, p: [f32; 3]) -> [f32; 3] {
        self.rows.map(|r| r[0] * p[0] + r[1] * p[1] + r[2] * p[2])
    }

    pub fn then_local(self, local: Self) -> Self {
        let mut result = Self::IDENTITY;
        for row in 0..3 {
            for col in 0..3 {
                result.rows[row][col] =
                    (0..3).map(|k| self.rows[row][k] * local.rows[k][col]).sum();
            }
            result.rows[row][3] = self.rows[row][3]
                + (0..3)
                    .map(|k| self.rows[row][k] * local.rows[k][3])
                    .sum::<f32>();
        }
        result
    }

    pub fn inverse(self) -> Option<Self> {
        let a = self.rows;
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if !det.is_finite() || det.abs() < 1e-12 {
            return None;
        }
        let r = 1.0 / det;
        let mut out = Self {
            rows: [
                [
                    (a[1][1] * a[2][2] - a[1][2] * a[2][1]) * r,
                    (a[0][2] * a[2][1] - a[0][1] * a[2][2]) * r,
                    (a[0][1] * a[1][2] - a[0][2] * a[1][1]) * r,
                    0.,
                ],
                [
                    (a[1][2] * a[2][0] - a[1][0] * a[2][2]) * r,
                    (a[0][0] * a[2][2] - a[0][2] * a[2][0]) * r,
                    (a[0][2] * a[1][0] - a[0][0] * a[1][2]) * r,
                    0.,
                ],
                [
                    (a[1][0] * a[2][1] - a[1][1] * a[2][0]) * r,
                    (a[0][1] * a[2][0] - a[0][0] * a[2][1]) * r,
                    (a[0][0] * a[1][1] - a[0][1] * a[1][0]) * r,
                    0.,
                ],
            ],
        };
        let translation = out.transform_vector([a[0][3], a[1][3], a[2][3]]);
        for row in 0..3 {
            out.rows[row][3] = -translation[row];
        }
        Some(out)
    }
}

#[derive(Debug, Clone)]
pub struct AnimationSkin {
    pub inverse_bind: Vec<AffineTransform>,
    pub raw_columns: Vec<[f32; 16]>,
}

#[derive(Debug, Clone)]
pub struct SkinMatrixBuffer {
    /// Serialized 12-word slots, interpreted as three affine rows. The entire
    /// measured corpus contains identity slots, so nonidentity row ordering
    /// remains unverified; use AnimationSkin for source inverse-bind matrices.
    pub matrices: Vec<AffineTransform>,
}

#[derive(Debug, Clone, Copy)]
pub struct SkinBinding {
    pub skin_matrix_buffer: u32,
    pub skeleton: u32,
    pub animation_skin: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    pub scale: [f32; 3],
}

impl Transform {
    pub const IDENTITY: Self = Self {
        rotation: [0., 0., 0., 1.],
        translation: [0.; 3],
        scale: [1.; 3],
    };
    pub fn affine(self) -> Result<AffineTransform, AnimationError> {
        let [x, y, z, w] = normalized(self.rotation)?;
        let [sx, sy, sz] = self.scale;
        Ok(AffineTransform {
            rows: [
                [
                    (1. - 2. * (y * y + z * z)) * sx,
                    2. * (x * y - z * w) * sy,
                    2. * (x * z + y * w) * sz,
                    self.translation[0],
                ],
                [
                    2. * (x * y + z * w) * sx,
                    (1. - 2. * (x * x + z * z)) * sy,
                    2. * (y * z - x * w) * sz,
                    self.translation[1],
                ],
                [
                    2. * (x * z - y * w) * sx,
                    2. * (y * z + x * w) * sy,
                    (1. - 2. * (x * x + y * y)) * sz,
                    self.translation[2],
                ],
            ],
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarKey {
    pub time: f32,
    pub value: f32,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformKey {
    pub time: f32,
    pub value: Transform,
}
#[derive(Debug, Clone)]
pub enum TrackValues {
    Scalar(Vec<ScalarKey>),
    Transform(Vec<TransformKey>),
    Unsupported(Vec<u8>),
}
#[derive(Debug, Clone)]
pub struct Track {
    pub bone_id: u32,
    pub kind: u32,
    pub stride: u32,
    pub values: TrackValues,
}
#[derive(Debug, Clone)]
pub struct KeyframeClip {
    pub id: u32,
    pub duration: f32,
    pub flags: u32,
    pub tracks: Vec<Track>,
    /// Validated zero-filled allocation after the serialized key values.
    /// Its original runtime purpose has not been established.
    pub reserved_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleValue {
    Scalar(f32),
    Transform(Transform),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackSample {
    pub bone_id: u32,
    pub value: SampleValue,
}
#[derive(Debug, Clone)]
pub struct AnimationMap {
    pub entries: Vec<(u32, u32)>,
}
#[derive(Debug, Clone, Default)]
pub struct AnimationSet {
    pub skeletons: Vec<Indexed<Skeleton>>,
    pub animation_skins: Vec<Indexed<AnimationSkin>>,
    pub skin_matrices: Vec<Indexed<SkinMatrixBuffer>>,
    pub bindings: Vec<Indexed<SkinBinding>>,
    pub clips: Vec<Indexed<KeyframeClip>>,
    pub animation_maps: Vec<Indexed<AnimationMap>>,
}

impl KeyframeClip {
    pub fn is_sampleable(&self) -> bool {
        self.tracks
            .iter()
            .all(|t| !matches!(t.values, TrackValues::Unsupported(_)))
    }
    pub fn sample(&self, time: f32, looping: bool) -> Result<Vec<TrackSample>, AnimationError> {
        if !time.is_finite() {
            return Err(AnimationError::InvalidTime);
        }
        let t = if looping && self.duration > 0. {
            time.rem_euclid(self.duration)
        } else {
            time.clamp(0., self.duration.max(0.))
        };
        self.tracks
            .iter()
            .map(|track| {
                let value = match &track.values {
                    TrackValues::Scalar(keys) => {
                        if keys.is_empty() {
                            return Err(AnimationError::EmptyTrack);
                        }
                        let (a, b, f) = interval(keys.len(), |i| keys[i].time, t);
                        SampleValue::Scalar(keys[a].value + (keys[b].value - keys[a].value) * f)
                    }
                    TrackValues::Transform(keys) => {
                        if keys.is_empty() {
                            return Err(AnimationError::EmptyTrack);
                        }
                        let (a, b, f) = interval(keys.len(), |i| keys[i].time, t);
                        SampleValue::Transform(Transform {
                            rotation: slerp(keys[a].value.rotation, keys[b].value.rotation, f)?,
                            translation: lerp3(
                                keys[a].value.translation,
                                keys[b].value.translation,
                                f,
                            ),
                            scale: lerp3(keys[a].value.scale, keys[b].value.scale, f),
                        })
                    }
                    TrackValues::Unsupported(_) => {
                        return Err(AnimationError::UnsupportedTrack {
                            kind: track.kind,
                            stride: track.stride,
                        })
                    }
                };
                Ok(TrackSample {
                    bone_id: track.bone_id,
                    value,
                })
            })
            .collect()
    }
}

fn interval(n: usize, time: impl Fn(usize) -> f32, t: f32) -> (usize, usize, f32) {
    if n <= 1 || t <= time(0) {
        return (0, 0, 0.);
    }
    let mut lo = 0;
    let mut hi = n;
    while lo < hi {
        let mid = (lo + hi) / 2;
        if time(mid) <= t {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo == n {
        return (n - 1, n - 1, 0.);
    }
    let a = lo - 1;
    let dt = time(lo) - time(a);
    (a, lo, if dt > 0. { (t - time(a)) / dt } else { 0. })
}
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}
fn normalized(q: [f32; 4]) -> Result<[f32; 4], AnimationError> {
    let n = q.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !n.is_finite() || n < 1e-12 {
        return Err(AnimationError::InvalidQuaternion);
    }
    Ok(q.map(|x| x / n))
}
fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> Result<[f32; 4], AnimationError> {
    let a = normalized(a)?;
    let mut b = normalized(b)?;
    let mut dot = (0..4).map(|i| a[i] * b[i]).sum::<f32>();
    if dot < 0. {
        b = b.map(|x| -x);
        dot = -dot;
    }
    if dot > 0.9995 {
        return normalized(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t));
    }
    let theta = dot.clamp(-1., 1.).acos();
    let denominator = theta.sin();
    let wa = ((1. - t) * theta).sin() / denominator;
    let wb = (t * theta).sin() / denominator;
    normalized(std::array::from_fn(|i| wa * a[i] + wb * b[i]))
}

fn bad(section: u32, reason: &'static str) -> AnimationError {
    AnimationError::Malformed { section, reason }
}
fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        b.get(o..o.checked_add(4)?)?.try_into().ok()?,
    ))
}
fn f32_at(b: &[u8], o: usize) -> Option<f32> {
    Some(f32::from_bits(u32_at(b, o)?))
}
fn finite_floats<const N: usize>(b: &[u8], o: usize) -> Option<[f32; N]> {
    let mut out = [0.; N];
    for (i, x) in out.iter_mut().enumerate() {
        *x = f32_at(b, o.checked_add(i.checked_mul(4)?)?)?;
        if !x.is_finite() {
            return None;
        }
    }
    Some(out)
}
fn array_offset(ptr: u32, start: usize, len: usize, bytes: usize) -> Option<usize> {
    let off = (ptr as usize).checked_sub(start)?;
    if off.checked_add(bytes)? <= len {
        Some(off)
    } else {
        None
    }
}

pub fn parse(rw: &Rw4, data: &[u8]) -> Result<AnimationSet, AnimationError> {
    let mut out = AnimationSet::default();
    for (i, section) in rw.sections.iter().enumerate() {
        let index = i as u32;
        let b = rw
            .object(data, index)
            .ok_or_else(|| bad(index, "object range"))?;
        match section.info.type_code {
            object_types::SKELETON => out.skeletons.push(Indexed {
                section_index: index,
                value: parse_skeleton(b, section.start, index)?,
            }),
            object_types::ANIMATION_SKIN => {
                let (count, off) = matrix_header(b, section.start, index, 64)?;
                let mut raw_columns = Vec::with_capacity(count);
                let mut inverse_bind = Vec::with_capacity(count);
                for j in 0..count {
                    let m = finite_floats::<16>(b, off + j * 64)
                        .ok_or_else(|| bad(index, "invalid skin matrix"))?;
                    inverse_bind.push(AffineTransform::from_skin_columns(m));
                    raw_columns.push(m);
                }
                out.animation_skins.push(Indexed {
                    section_index: index,
                    value: AnimationSkin {
                        inverse_bind,
                        raw_columns,
                    },
                });
            }
            object_types::SKIN_MATRIX_BUFFER => {
                let (count, off) = matrix_header(b, section.start, index, 48)?;
                let mut matrices = Vec::with_capacity(count);
                for j in 0..count {
                    let m = finite_floats::<12>(b, off + j * 48)
                        .ok_or_else(|| bad(index, "invalid runtime skin matrix"))?;
                    matrices.push(AffineTransform {
                        rows: [
                            m[0..4].try_into().unwrap(),
                            m[4..8].try_into().unwrap(),
                            m[8..12].try_into().unwrap(),
                        ],
                    });
                }
                out.skin_matrices.push(Indexed {
                    section_index: index,
                    value: SkinMatrixBuffer { matrices },
                });
            }
            object_types::SKINS_IN_K => {
                if b.len() != 20 {
                    return Err(bad(index, "skin binding length"));
                }
                let binding = SkinBinding {
                    skin_matrix_buffer: u32_at(b, 8).unwrap(),
                    skeleton: u32_at(b, 12).unwrap(),
                    animation_skin: u32_at(b, 16).unwrap(),
                };
                for (r, t) in [
                    (binding.skin_matrix_buffer, object_types::SKIN_MATRIX_BUFFER),
                    (binding.skeleton, object_types::SKELETON),
                    (binding.animation_skin, object_types::ANIMATION_SKIN),
                ] {
                    if rw.sections.get(r as usize).map(|s| s.info.type_code) != Some(t) {
                        return Err(bad(index, "skin binding reference type"));
                    }
                }
                out.bindings.push(Indexed {
                    section_index: index,
                    value: binding,
                });
            }
            object_types::KEYFRAME_ANIM => out.clips.push(Indexed {
                section_index: index,
                value: parse_clip(b, section.start, index)?,
            }),
            object_types::ANIMATIONS => {
                let count =
                    u32_at(b, 4).ok_or_else(|| bad(index, "animation map header"))? as usize;
                if count.checked_mul(8).and_then(|n| n.checked_add(8)) != Some(b.len()) {
                    return Err(bad(index, "animation map count"));
                }
                let mut entries = Vec::with_capacity(count);
                for j in 0..count {
                    let o = 8 + j * 8;
                    entries.push((u32_at(b, o).unwrap(), u32_at(b, o + 4).unwrap()));
                }
                out.animation_maps.push(Indexed {
                    section_index: index,
                    value: AnimationMap { entries },
                });
            }
            _ => {}
        }
    }
    for binding in &out.bindings {
        let n = out
            .skeletons
            .iter()
            .find(|x| x.section_index == binding.value.skeleton)
            .unwrap()
            .value
            .bones
            .len();
        let skin = out
            .animation_skins
            .iter()
            .find(|x| x.section_index == binding.value.animation_skin)
            .unwrap();
        let matrices = out
            .skin_matrices
            .iter()
            .find(|x| x.section_index == binding.value.skin_matrix_buffer)
            .unwrap();
        if skin.value.inverse_bind.len() != n || matrices.value.matrices.len() != n {
            return Err(bad(binding.section_index, "binding joint counts differ"));
        }
    }
    Ok(out)
}

fn parse_skeleton(b: &[u8], start: usize, index: u32) -> Result<Skeleton, AnimationError> {
    let count = u32_at(b, 12).ok_or_else(|| bad(index, "skeleton header"))? as usize;
    if count.checked_mul(12).and_then(|n| n.checked_add(24)) != Some(b.len()) {
        return Err(bad(index, "skeleton count"));
    }
    let bytes = count
        .checked_mul(4)
        .ok_or_else(|| bad(index, "skeleton count overflow"))?;
    let mut offsets = [0; 3];
    for (j, o) in offsets.iter_mut().enumerate() {
        *o = array_offset(u32_at(b, j * 4).unwrap(), start, b.len(), bytes)
            .ok_or_else(|| bad(index, "skeleton array pointer"))?;
    }
    let mut bones = Vec::with_capacity(count);
    for j in 0..count {
        let parent = u32_at(b, offsets[1] + j * 4).unwrap() as i32;
        if parent < -1 || parent >= count as i32 || parent == j as i32 {
            return Err(bad(index, "invalid bone parent"));
        }
        bones.push(Bone {
            id: u32_at(b, offsets[2] + j * 4).unwrap(),
            flags: u32_at(b, offsets[0] + j * 4).unwrap(),
            parent: if parent < 0 {
                None
            } else {
                Some(parent as usize)
            },
        });
    }
    for j in 0..count {
        let mut at = Some(j);
        let mut steps = 0;
        while let Some(k) = at {
            steps += 1;
            if steps > count {
                return Err(bad(index, "cyclic bone hierarchy"));
            }
            at = bones[k].parent;
        }
    }
    Ok(Skeleton {
        id: u32_at(b, 16).unwrap(),
        flags: u32_at(b, 20).unwrap(),
        bones,
    })
}

fn matrix_header(
    b: &[u8],
    start: usize,
    index: u32,
    stride: usize,
) -> Result<(usize, usize), AnimationError> {
    let count = u32_at(b, 4).ok_or_else(|| bad(index, "matrix header"))? as usize;
    let bytes = count
        .checked_mul(stride)
        .ok_or_else(|| bad(index, "matrix count overflow"))?;
    if bytes.checked_add(16) != Some(b.len()) {
        return Err(bad(index, "matrix count"));
    }
    let off = array_offset(u32_at(b, 0).unwrap(), start, b.len(), bytes)
        .ok_or_else(|| bad(index, "matrix array pointer"))?;
    Ok((count, off))
}

fn parse_clip(b: &[u8], start: usize, index: u32) -> Result<KeyframeClip, AnimationError> {
    if b.len() < 48 {
        return Err(bad(index, "keyframe header"));
    }
    let count = u32_at(b, 4).unwrap() as usize;
    if u32_at(b, 24).unwrap() as usize != count {
        return Err(bad(index, "keyframe channel counts differ"));
    }
    let ids = array_offset(
        u32_at(b, 0).unwrap(),
        start,
        b.len(),
        count
            .checked_mul(4)
            .ok_or_else(|| bad(index, "keyframe ID count overflow"))?,
    )
    .ok_or_else(|| bad(index, "keyframe ID pointer"))?;
    let descriptors = array_offset(
        u32_at(b, 44).unwrap(),
        start,
        b.len(),
        count
            .checked_mul(12)
            .ok_or_else(|| bad(index, "keyframe descriptor count overflow"))?,
    )
    .ok_or_else(|| bad(index, "keyframe descriptor pointer"))?;
    let payload_start = array_offset(u32_at(b, 16).unwrap(), start, b.len(), 0)
        .ok_or_else(|| bad(index, "keyframe payload pointer"))?;
    let allocated_end = array_offset(u32_at(b, 20).unwrap(), start, b.len(), 0)
        .ok_or_else(|| bad(index, "keyframe payload end"))?;
    let allocated_bytes = allocated_end
        .checked_sub(payload_start)
        .ok_or_else(|| bad(index, "reversed keyframe payload"))?;
    // Measured homogeneous channel allocations: scalar values occupy the
    // whole payload, RT values occupy half, and RTS values occupy one third.
    // The remainder is zero-filled. Do not reinterpret that allocation as
    // extra keys (zero quaternion/time records are not animation frames).
    let first_type = if count > 0 {
        Some((
            u32_at(b, descriptors + 8).unwrap(),
            u32_at(b, descriptors + 4).unwrap(),
        ))
    } else {
        None
    };
    let homogeneous = (0..count).all(|j| {
        Some((
            u32_at(b, descriptors + j * 12 + 8).unwrap(),
            u32_at(b, descriptors + j * 12 + 4).unwrap(),
        )) == first_type
    });
    let factor = if homogeneous {
        match first_type {
            Some((0x101, 32)) => 2,
            Some((0x601, 48)) => 3,
            _ => 1,
        }
    } else {
        1
    };
    if allocated_bytes % factor != 0 {
        return Err(bad(index, "keyframe allocation extent"));
    }
    let payload_end = payload_start + allocated_bytes / factor;
    if b[payload_end..allocated_end].iter().any(|v| *v != 0) {
        return Err(bad(index, "nonzero keyframe allocation reserve"));
    }
    let duration = f32_at(b, 32).unwrap();
    if !duration.is_finite() || duration < 0. {
        return Err(bad(index, "invalid keyframe duration"));
    }
    let mut tracks = Vec::with_capacity(count);
    for j in 0..count {
        let at = descriptors + j * 12;
        let off = u32_at(b, at).unwrap() as usize;
        let stride = u32_at(b, at + 4).unwrap();
        let kind = u32_at(b, at + 8).unwrap();
        let end = if j + 1 < count {
            u32_at(b, at + 12).unwrap() as usize
        } else {
            payload_end
        };
        if off < payload_start
            || end < off
            || end > payload_end
            || stride == 0
            || (end - off) % (stride as usize) != 0
        {
            return Err(bad(index, "keyframe channel extent"));
        }
        let n = (end - off) / stride as usize;
        if n == 0 {
            return Err(bad(index, "empty keyframe channel"));
        }
        let values = match (kind, stride) {
            (0x100, 8) => {
                let mut keys = Vec::with_capacity(n);
                for k in 0..n {
                    let v = finite_floats::<2>(b, off + k * 8)
                        .ok_or_else(|| bad(index, "invalid scalar key"))?;
                    keys.push(ScalarKey {
                        time: v[1],
                        value: v[0],
                    });
                }
                validate_times(keys.iter().map(|k| k.time), duration, index)?;
                TrackValues::Scalar(keys)
            }
            (0x101, 32) | (0x601, 48) => {
                let mut keys = Vec::with_capacity(n);
                for k in 0..n {
                    let p = off + k * stride as usize;
                    let rotation = finite_floats::<4>(b, p)
                        .ok_or_else(|| bad(index, "invalid transform rotation"))?;
                    let translation = finite_floats::<3>(b, p + 16)
                        .ok_or_else(|| bad(index, "invalid transform translation"))?;
                    let scale = if kind == 0x601 {
                        finite_floats::<3>(b, p + 28)
                            .ok_or_else(|| bad(index, "invalid transform scale"))?
                    } else {
                        [1.; 3]
                    };
                    let time = f32_at(b, p + stride as usize - 4).unwrap();
                    keys.push(TransformKey {
                        time,
                        value: Transform {
                            rotation,
                            translation,
                            scale,
                        },
                    });
                }
                validate_times(keys.iter().map(|k| k.time), duration, index)?;
                TrackValues::Transform(keys)
            }
            _ => TrackValues::Unsupported(b[off..end].to_vec()),
        };
        tracks.push(Track {
            bone_id: u32_at(b, ids + j * 4).unwrap(),
            kind,
            stride,
            values,
        });
    }
    Ok(KeyframeClip {
        id: u32_at(b, 8).unwrap(),
        duration,
        flags: u32_at(b, 40).unwrap(),
        tracks,
        reserved_bytes: allocated_end - payload_end,
    })
}
fn validate_times(
    times: impl Iterator<Item = f32>,
    duration: f32,
    index: u32,
) -> Result<(), AnimationError> {
    let mut previous = f32::NEG_INFINITY;
    for time in times {
        if !time.is_finite() || time < 0. || time < previous || time > duration + 1e-3 {
            return Err(bad(index, "invalid key times"));
        }
        previous = time;
    }
    Ok(())
}

/// Decode per-vertex RW4 influences for the same geometry selected by mesh::from_rw4.
/// Joint-register offsets use three shader registers per joint. Unused slots are
/// zeroed, active weights are retained (not silently normalized).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexSkinning {
    pub joints: [u16; 4],
    pub weights: [f32; 4],
}

pub fn vertex_skinning(
    rw: &Rw4,
    data: &[u8],
    mesh: &rw4::Mesh,
    joint_count: usize,
) -> Result<Vec<VertexSkinning>, AnimationError> {
    for &index in &mesh.vertex_buffers {
        let Some(vb) = rw.vertex_buffer(data, index) else {
            continue;
        };
        let Some(desc) = rw.vertex_description(data, vb.description) else {
            continue;
        };
        if !desc.elements.iter().any(|e| e.usage == 0) {
            continue;
        }
        let indices = desc
            .elements
            .iter()
            .find(|e| e.usage == 2 && e.stream == 0)
            .ok_or(AnimationError::UnsupportedSkinning(
                "vertex declaration has no joint indices",
            ))?;
        let weights = desc
            .elements
            .iter()
            .find(|e| e.usage == 1 && e.stream == 0)
            .ok_or(AnimationError::UnsupportedSkinning(
                "vertex declaration has no blend weights",
            ))?;
        let bytes = rw
            .object(data, vb.data)
            .ok_or(AnimationError::UnsupportedSkinning(
                "vertex buffer data reference",
            ))?;
        let stride = if vb.vertex_size > 0 {
            vb.vertex_size as usize
        } else {
            desc.vertex_size as usize
        };
        let count = usize::try_from(vb.vertex_count)
            .map_err(|_| AnimationError::UnsupportedSkinning("negative vertex count"))?;
        if stride == 0
            || count.checked_mul(stride).is_none_or(|n| n > bytes.len())
            || indices.offset < 0
            || weights.offset < 0
        {
            return Err(AnimationError::UnsupportedSkinning("vertex buffer extent"));
        }
        let mut out = Vec::with_capacity(count);
        for j in 0..count {
            let base = j * stride;
            let raw = match indices.element_type {
                6 => {
                    let at = base + indices.offset as usize;
                    let b = bytes
                        .get(at..at + 4)
                        .ok_or(AnimationError::UnsupportedSkinning(
                            "joint attribute extent",
                        ))?;
                    [
                        u16::from_le_bytes([b[0], b[1]]),
                        u16::from_le_bytes([b[2], b[3]]),
                        0,
                        0,
                    ]
                }
                5 => {
                    let b = bytes
                        .get(base + indices.offset as usize..base + indices.offset as usize + 4)
                        .ok_or(AnimationError::UnsupportedSkinning(
                            "joint attribute extent",
                        ))?;
                    [b[0] as u16, b[1] as u16, b[2] as u16, b[3] as u16]
                }
                7 | 12 => {
                    let at = base + indices.offset as usize;
                    let b = bytes
                        .get(at..at + 8)
                        .ok_or(AnimationError::UnsupportedSkinning(
                            "joint attribute extent",
                        ))?;
                    std::array::from_fn(|k| u16::from_le_bytes([b[k * 2], b[k * 2 + 1]]))
                }
                _ => {
                    return Err(AnimationError::UnsupportedSkinning(
                        "joint declaration storage type",
                    ))
                }
            };
            let at = base + weights.offset as usize;
            let w = match weights.element_type {
                0..=3 => {
                    let n = weights.element_type as usize + 1;
                    let mut w = [0.; 4];
                    for (k, v) in w.iter_mut().take(n).enumerate() {
                        *v = f32_at(bytes, at + k * 4).ok_or(
                            AnimationError::UnsupportedSkinning("weight attribute extent"),
                        )?;
                    }
                    w
                }
                8 => {
                    let b = bytes
                        .get(at..at + 4)
                        .ok_or(AnimationError::UnsupportedSkinning(
                            "weight attribute extent",
                        ))?;
                    [
                        b[0] as f32 / 255.,
                        b[1] as f32 / 255.,
                        b[2] as f32 / 255.,
                        b[3] as f32 / 255.,
                    ]
                }
                _ => {
                    return Err(AnimationError::UnsupportedSkinning(
                        "weight declaration storage type",
                    ))
                }
            };
            out.push(influences(raw, w, joint_count)?);
        }
        return Ok(out);
    }
    let bs = rw
        .blend_shape(data)
        .ok_or(AnimationError::UnsupportedSkinning(
            "no vertex buffer or blend shape",
        ))?;
    let count = usize::try_from(bs.vertex_count)
        .map_err(|_| AnimationError::UnsupportedSkinning("negative blend vertex count"))?;
    let n = usize::try_from(bs.bone_indices_count)
        .map_err(|_| AnimationError::UnsupportedSkinning("negative influence count"))?;
    if !(1..=4).contains(&n) || bs.offsets[9] < 0 || bs.offsets[10] < 0 {
        return Err(AnimationError::UnsupportedSkinning(
            "blend shape has no supported influences",
        ));
    }
    let ip = bs
        .data_at
        .checked_add(bs.offsets[9] as usize)
        .ok_or(AnimationError::UnsupportedSkinning("index offset overflow"))?;
    let wp = bs.data_at.checked_add(bs.offsets[10] as usize).ok_or(
        AnimationError::UnsupportedSkinning("weight offset overflow"),
    )?;
    let istride = (n * 2 + 3) & !3;
    let end = bs
        .data_at
        .checked_add(bs.data_len)
        .ok_or(AnimationError::UnsupportedSkinning(
            "blend data extent overflow",
        ))?;
    if count
        .checked_mul(istride)
        .and_then(|size| ip.checked_add(size))
        .is_none_or(|p| p > end)
        || count
            .checked_mul(n * 4)
            .and_then(|size| wp.checked_add(size))
            .is_none_or(|p| p > end)
    {
        return Err(AnimationError::UnsupportedSkinning(
            "blend influence extent",
        ));
    }
    let mut out = Vec::with_capacity(count);
    for j in 0..count {
        let mut raw = [0; 4];
        let mut weights = [0.; 4];
        for k in 0..n {
            let at = ip + j * istride + k * 2;
            raw[k] = u16::from_le_bytes([data[at], data[at + 1]]);
            weights[k] = f32_at(data, wp + (j * n + k) * 4).unwrap();
        }
        out.push(influences(raw, weights, joint_count)?);
    }
    Ok(out)
}
fn influences(
    raw: [u16; 4],
    weights: [f32; 4],
    count: usize,
) -> Result<VertexSkinning, AnimationError> {
    let mut joints = [0; 4];
    let sum = weights.iter().sum::<f32>();
    if !sum.is_finite()
        || (sum - 1.).abs() > 0.002
        || weights.iter().any(|w| !w.is_finite() || *w < -1e-6)
    {
        return Err(AnimationError::UnsupportedSkinning(
            "invalid or unnormalized weights",
        ));
    }
    for k in 0..4 {
        if weights[k] > 0. {
            if raw[k] % 3 != 0 || raw[k] as usize / 3 >= count {
                return Err(AnimationError::UnsupportedSkinning(
                    "joint register index outside binding",
                ));
            }
            joints[k] = raw[k] / 3;
        }
    }
    Ok(VertexSkinning { joints, weights })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn affine_inverse_handles_rotation_scale_and_translation() {
        let t = Transform {
            rotation: [0., 0., 0.70710677, 0.70710677],
            translation: [3., 4., 5.],
            scale: [2., 3., 4.],
        }
        .affine()
        .unwrap();
        let p = [1., 2., 3.];
        let q = t.inverse().unwrap().transform_point(t.transform_point(p));
        for i in 0..3 {
            assert!((p[i] - q[i]).abs() < 1e-5);
        }
    }
    #[test]
    fn skin_columns_place_translation_in_affine_column() {
        let t = AffineTransform::from_skin_columns([
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 4., 5., 6., 0.,
        ]);
        assert_eq!(t.transform_point([1., 2., 3.]), [5., 7., 9.]);
    }
    #[test]
    fn quaternion_interpolation_uses_shortest_arc() {
        let q = slerp([0., 0., 0., 1.], [0., 0., 0., -1.], 0.5).unwrap();
        assert!((q[3] - 1.).abs() < 1e-6);
        assert_eq!(normalized([0.; 4]), Err(AnimationError::InvalidQuaternion));
    }
    #[test]
    fn sampling_honors_endpoints_and_looping() {
        let clip = KeyframeClip {
            id: 1,
            duration: 2.,
            flags: 0,
            reserved_bytes: 0,
            tracks: vec![Track {
                bone_id: 9,
                kind: 0x100,
                stride: 8,
                values: TrackValues::Scalar(vec![
                    ScalarKey {
                        time: 0.,
                        value: 2.,
                    },
                    ScalarKey {
                        time: 2.,
                        value: 6.,
                    },
                ]),
            }],
        };
        assert_eq!(
            clip.sample(1., false).unwrap()[0].value,
            SampleValue::Scalar(4.)
        );
        assert_eq!(
            clip.sample(2., false).unwrap()[0].value,
            SampleValue::Scalar(6.)
        );
        assert_eq!(
            clip.sample(2., true).unwrap()[0].value,
            SampleValue::Scalar(2.)
        );
        assert!(clip.sample(f32::NAN, false).is_err());
    }
    #[test]
    fn public_empty_tracks_return_error() {
        let clip = KeyframeClip {
            id: 0,
            duration: 0.,
            flags: 0,
            reserved_bytes: 0,
            tracks: vec![Track {
                bone_id: 0,
                kind: 0x100,
                stride: 8,
                values: TrackValues::Scalar(vec![]),
            }],
        };
        assert_eq!(clip.sample(0., false), Err(AnimationError::EmptyTrack));
    }
    #[test]
    fn influence_indices_are_register_offsets_not_normalized_bytes() {
        let s = influences([0, 3, 6, 65535], [0.2, 0.3, 0.5, 0.], 3).unwrap();
        assert_eq!(s.joints, [0, 1, 2, 0]);
        assert!(influences([4, 0, 0, 0], [1., 0., 0., 0.], 3).is_err());
        assert!(influences([9, 0, 0, 0], [1., 0., 0., 0.], 3).is_err());
    }
    #[test]
    fn malformed_object_parsers_never_slice_outside_input() {
        for n in 0..200 {
            let b = vec![0xff; n];
            assert!(parse_skeleton(&b, 0, 0).is_err());
            assert!(parse_clip(&b, 0, 0).is_err());
            assert!(matrix_header(&b, 0, 0, 64).is_err());
        }
    }
    #[test]
    fn cyclic_skeletons_are_rejected() {
        let mut b = vec![0; 48];
        for (o, v) in [(0, 32u32), (4, 40), (8, 24), (12, 2)] {
            b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        b[40..44].copy_from_slice(&1u32.to_le_bytes());
        assert!(parse_skeleton(&b, 0, 0).is_err());
    }
    #[test]
    fn transform_track_parser_uses_source_times_and_detects_bad_extents() {
        let start = 512usize;
        let mut b = vec![0; 192];
        for (o, v) in [
            (0, (start + 48) as u32),
            (4, 1),
            (8, 0x1234),
            (16, (start + 64) as u32),
            (20, (start + 192) as u32),
            (24, 1),
            (44, (start + 52) as u32),
            (48, 0x5678),
            (52, 64),
            (56, 32),
            (60, 0x101),
        ] {
            b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        b[32..36].copy_from_slice(&2f32.to_le_bytes());
        for (o, v) in [
            (76, 1f32),
            (80, 2.),
            (92, 0.),
            (108, 1.),
            (112, 6.),
            (124, 2.),
        ] {
            b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        let clip = parse_clip(&b, start, 7).unwrap();
        assert_eq!(clip.reserved_bytes, 64);
        let SampleValue::Transform(t) = clip.sample(1., false).unwrap()[0].value else {
            panic!("transform channel")
        };
        assert_eq!(t.translation, [4., 0., 0.]);
        b[191] = 1;
        assert!(parse_clip(&b, start, 7).is_err());
        b[191] = 0;
        b[56..60].copy_from_slice(&31u32.to_le_bytes());
        assert!(parse_clip(&b, start, 7).is_err());
    }
    #[test]
    fn child_local_transform_composes_before_inverse_bind() {
        let z = std::f32::consts::FRAC_1_SQRT_2;
        let root = Transform {
            rotation: [0., 0., -z, z],
            translation: [0.; 3],
            scale: [1.; 3],
        }
        .affine()
        .unwrap();
        let child = Transform {
            rotation: [0., 0., z, z],
            translation: [2., 0., 0.],
            scale: [1.; 3],
        }
        .affine()
        .unwrap();
        let global = root.then_local(child);
        let bind = global.inverse().unwrap();
        let composed = global.then_local(bind);
        for (got, want) in composed
            .rows
            .iter()
            .flatten()
            .zip(AffineTransform::IDENTITY.rows.iter().flatten())
        {
            assert!((got - want).abs() < 1e-5);
        }
        assert!(child.then_local(bind).rows[0][0].abs() < 1e-5);
    }
}
