//! Parser for the SPORE anim format (type `0xEE17C6AD`).
//!
//! ANIM stores interleaved keyframe rows, addressed by 32-byte component
//! descriptors. Position and quaternion values are exposed; unrecognised
//! components and authored curve payloads remain available as raw row bytes.
//! Sampling uses an explicit generic engine policy, not a recovered SPORE curve.
//!
//! Layout:
//!
//! ```text
//! Header (12 bytes):
//!   offset  0: "ANIM" (4 bytes magic)
//!   offset  4: LE u32 = file size
//!   offset  8: LE u32 = 25 or 21 (package-dependent constant)
//!
//! String field (260 bytes at offset 12):
//!   Null-terminated ASCII Windows file path, padded to 260 bytes
//!
//! Animation data (starts at offset 272):
//!   +0:  LE u32 (hash/ID)
//!   +4:  LE u32 = 0
//!   +8:  LE u32 = 0 or small value
//!   +12: LE float = frame duration (0.0333 = 30 FPS)
//!   +16: LE float = duration in frames
//!   +20: LE u32 = 0
//!   +24: LE u32 = unestablished field (not universally the last frame)
//!   +28: LE u32 (varies)
//!   +32: LE u32 (unestablished field, not the channel count)
//!   +36: LE u32 = 0
//!   +52: LE u32 = channel count
//!   +56: LE u32 = channel offset table pointer (512 in the measured corpus)
//!
//! Offset table (at file offset 512):
//!   N u32 values (file offsets to each bone's CHAN section)
//!   "CHAN" magic (4 bytes)
//!
//! Per-bone CHAN section:
//!   +0:   "CHAN" (4 bytes)
//!   +4:   LE u32 = 0
//!   +8:   bone name (null-terminated ASCII)
//!   +136..212: selector/flag words, retained without invented bone semantics
//!   +212: row count; +216: absolute row pointer
//!   +220: component count; +224: absolute component-descriptor pointer
//!   Descriptor: flags, tag, offset-within-row, row-stride, slot, 3 reserved words.
//!   INFO contains an integer frame followed by four unestablished words;
//!   POS begins with float3, ROT begins with quaternion xyzw. Remaining component
//!   bytes are authored curve data whose evaluation semantics remain unknown.
//!
//! Trailer:
//!   LE u32 = 0x60, repeated LE u32 = 2, hash, animation name (ASCII)
//! ```

use std::fmt;

/// Errors that can occur when parsing an anim resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimError {
    /// The resource does not start with "ANIM".
    BadMagic,
    /// The resource is too short.
    TooShort,
    /// The file size field does not match the actual size.
    SizeMismatch,
    /// The string field has no null terminator.
    NoStringTerminator,
    /// The CHAN magic was not found.
    NoChanMagic,
    /// A bone offset does not leave room for its eight-byte channel header.
    InvalidBoneOffset { offset: usize },
    /// The channel table does not fit the resource or point at CHAN sections.
    InvalidChannelTable,
    /// A descriptor or interleaved keyframe range is invalid.
    InvalidChannelData { offset: usize },
    /// Frame indices are not in nondecreasing order.
    UnorderedFrames { offset: usize },
}

impl fmt::Display for AnimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnimError::BadMagic => write!(f, "bad ANIM magic"),
            AnimError::TooShort => write!(f, "anim resource too short"),
            AnimError::SizeMismatch => write!(f, "file size mismatch"),
            AnimError::NoStringTerminator => write!(f, "no string terminator"),
            AnimError::NoChanMagic => write!(f, "CHAN magic not found"),
            AnimError::InvalidBoneOffset { offset } => {
                write!(f, "bone offset {offset} is outside the channel data")
            }
            AnimError::InvalidChannelTable => write!(f, "invalid animation channel table"),
            AnimError::InvalidChannelData { offset } => {
                write!(f, "invalid channel data at {offset}")
            }
            AnimError::UnorderedFrames { offset } => {
                write!(f, "unordered frame indices at {offset}")
            }
        }
    }
}

impl std::error::Error for AnimError {}

/// A parsed anim resource.
#[derive(Debug, Clone)]
pub struct Anim {
    /// File path from the string field.
    pub path: String,
    /// Hash/ID from the animation data header.
    pub hash: u32,
    /// Raw file word at 280. Native key-neighbor selection reads bit 8;
    /// this is separate from caller-selected playback looping.
    pub header_flags_280: u32,
    /// Frame duration in seconds (e.g. 0.0333 for 30 FPS).
    pub frame_duration: f32,
    /// Duration in frames.
    pub duration: f32,
    /// Legacy name for the raw word at offset 296. It is not universally the
    /// last frame; use the channel's `frame_range` for actual key bounds.
    pub last_frame: u32,
    /// Channel count, from offset 324 (not the unrelated word at 304).
    pub bone_count: u32,
    /// Unestablished header word formerly misidentified as `bone_count`.
    pub header_field_304: u32,
    /// Animation name from the trailer.
    pub anim_name: String,
    /// Bone channels.
    pub bones: Vec<AnimBone>,
}

/// One bone channel from an anim resource.
#[derive(Debug, Clone)]
pub struct AnimBone {
    /// Bone name (e.g. "root", "head", "MouthMorphs").
    pub name: String,
    /// Keyframe count.
    pub keyframe_count: u32,
    /// Whether this bone has position keyframes.
    pub has_position: bool,
    /// Whether this bone has rotation keyframes.
    pub has_rotation: bool,
    /// Raw channel flags at +136; selector interpretation remains open.
    pub channel_flags: u32,
    /// Raw selector/flag words at +136..212, including `channel_flags`.
    pub selector_words: [u32; 19],
    pub components: Vec<AnimComponent>,
    pub keyframe_stride: u32,
    pub keyframes: Vec<AnimKeyframe>,
    /// Complete interleaved rows, retaining unknown and curve payloads.
    pub raw_keyframes: Vec<u8>,
}

/// One component descriptor; tag may be ASCII or an unestablished hashed ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimComponent {
    pub flags: u32,
    pub tag: [u8; 4],
    pub offset: u32,
    pub stride: u32,
    pub slot: u32,
    pub reserved: [u32; 3],
}

impl AnimComponent {
    pub fn is_supported(&self) -> bool {
        matches!(&self.tag, b"INFO" | b"POS\0" | b"ROT\0")
    }
}

#[derive(Debug, Clone)]
pub struct AnimKeyframe {
    pub frame: u32,
    /// INFO's remaining four words. Their meanings are not established.
    pub info: [u32; 4],
    /// Raw channel coordinates; bind-pose/selector mapping is a separate step.
    pub position: Option<[f32; 3]>,
    /// Raw quaternion xyzw. Sampling normalises valid nonzero values.
    pub rotation: Option<[f32; 4]>,
}

/// Generic interpolation chosen by the caller. Neither mode evaluates the
/// proprietary curve payloads retained in `raw_keyframes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationPolicy {
    Step,
    LinearSlerp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimSample {
    pub frame: f32,
    pub position: Option<[f32; 3]>,
    pub rotation: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimSampleError {
    NonFiniteFrame,
    InvalidPosition,
    InvalidRotation,
}

impl fmt::Display for AnimSampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteFrame => write!(f, "sample frame is not finite"),
            Self::InvalidPosition => write!(f, "position key is not finite"),
            Self::InvalidRotation => write!(f, "rotation key cannot be normalised"),
        }
    }
}

impl std::error::Error for AnimSampleError {}

impl AnimBone {
    pub fn frame_range(&self) -> Option<(u32, u32)> {
        Some((self.keyframes.first()?.frame, self.keyframes.last()?.frame))
    }

    /// Clamp to the actual key range and sample the recognised components.
    /// Missing components stay `None`; malformed numeric values return an error.
    pub fn sample(
        &self,
        frame: f32,
        policy: InterpolationPolicy,
    ) -> Result<Option<AnimSample>, AnimSampleError> {
        if !frame.is_finite() {
            return Err(AnimSampleError::NonFiniteFrame);
        }
        let Some((first, last)) = self.frame_range() else {
            return Ok(None);
        };
        let frame = frame.clamp(first as f32, last as f32);
        let left = self
            .keyframes
            .partition_point(|key| key.frame as f32 <= frame)
            .saturating_sub(1);
        let right = (left + 1).min(self.keyframes.len() - 1);
        let a = &self.keyframes[left];
        let b = &self.keyframes[right];
        let t = if policy == InterpolationPolicy::Step || a.frame == b.frame {
            0.0
        } else {
            (frame - a.frame as f32) / (b.frame - a.frame) as f32
        };
        let b = if t == 0.0 { a } else { b };
        let position = match (a.position, b.position) {
            (Some(a), Some(b)) => {
                if !a.iter().chain(b.iter()).all(|x| x.is_finite()) {
                    return Err(AnimSampleError::InvalidPosition);
                }
                let value = std::array::from_fn(|i| {
                    ((1.0 - t as f64) * a[i] as f64 + t as f64 * b[i] as f64) as f32
                });
                if !value.iter().all(|x| x.is_finite()) {
                    return Err(AnimSampleError::InvalidPosition);
                }
                Some(value)
            }
            _ => None,
        };
        let rotation = match (a.rotation, b.rotation) {
            (Some(a), Some(b)) => Some(slerp(a, b, t)?),
            _ => None,
        };
        Ok(Some(AnimSample {
            frame,
            position,
            rotation,
        }))
    }
}

fn normalise(q: [f32; 4]) -> Result<[f32; 4], AnimSampleError> {
    let norm = q
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm < 1e-12 {
        return Err(AnimSampleError::InvalidRotation);
    }
    Ok(q.map(|x| (x as f64 / norm) as f32))
}

fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> Result<[f32; 4], AnimSampleError> {
    let a = normalise(a)?;
    let mut b = normalise(b)?;
    let mut dot = a.iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
    if dot < 0.0 {
        b = b.map(|x| -x);
        dot = -dot;
    }
    let dot = dot.clamp(0.0, 1.0);
    let result = if dot > 0.9995 {
        std::array::from_fn(|i| a[i] * (1.0 - t) + b[i] * t)
    } else {
        let theta = dot.acos();
        let wa = ((1.0 - t) * theta).sin() / theta.sin();
        let wb = (t * theta).sin() / theta.sin();
        std::array::from_fn(|i| a[i] * wa + b[i] * wb)
    };
    normalise(result)
}

fn word(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn channel(data: &[u8], offset: usize) -> Result<AnimBone, AnimError> {
    let bone = data
        .get(offset..)
        .filter(|b| b.len() >= 8)
        .ok_or(AnimError::InvalidBoneOffset { offset })?;
    if &bone[..4] != b"CHAN" {
        return Err(AnimError::InvalidChannelData { offset });
    }
    let name_bytes = &bone[8..bone.len().min(136)];
    let name_end = name_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(name_bytes.len());
    let name = String::from_utf8_lossy(&name_bytes[..name_end]).to_string();
    let mut result = AnimBone {
        name,
        keyframe_count: 0,
        has_position: false,
        has_rotation: false,
        channel_flags: 0,
        selector_words: [0; 19],
        components: Vec::new(),
        keyframe_stride: 0,
        keyframes: Vec::new(),
        raw_keyframes: Vec::new(),
    };
    // Keep the established metadata-only behavior for a truncated optional header.
    if bone.len() < 228 {
        return Ok(result);
    }
    result.selector_words = std::array::from_fn(|i| word(bone, 136 + i * 4));
    result.channel_flags = result.selector_words[0];
    result.keyframe_count = word(bone, 212);
    let keys = word(bone, 216) as usize;
    let count = word(bone, 220) as usize;
    let descriptors = word(bone, 224) as usize;
    let invalid = || AnimError::InvalidChannelData { offset };
    let descriptor_bytes = count.checked_mul(32).ok_or_else(invalid)?;
    let desc = data
        .get(
            descriptors
                ..descriptors
                    .checked_add(descriptor_bytes)
                    .ok_or_else(invalid)?,
        )
        .ok_or_else(invalid)?;
    for d in desc.chunks_exact(32) {
        result.components.push(AnimComponent {
            flags: word(d, 0),
            tag: d[4..8].try_into().unwrap(),
            offset: word(d, 8),
            stride: word(d, 12),
            slot: word(d, 16),
            reserved: [word(d, 20), word(d, 24), word(d, 28)],
        });
    }
    result.has_position = result.components.iter().any(|c| c.tag == *b"POS\0");
    result.has_rotation = result.components.iter().any(|c| c.tag == *b"ROT\0");
    let Some(first) = result.components.first() else {
        if result.keyframe_count != 0 {
            return Err(invalid());
        }
        return Ok(result);
    };
    let stride = first.stride as usize;
    result.keyframe_stride = first.stride;
    if stride == 0
        || result
            .components
            .iter()
            .any(|c| c.stride as usize != stride || c.offset as usize >= stride)
    {
        return Err(invalid());
    }
    let bytes = (result.keyframe_count as usize)
        .checked_mul(stride)
        .ok_or_else(invalid)?;
    let raw = data
        .get(keys..keys.checked_add(bytes).ok_or_else(invalid)?)
        .ok_or_else(invalid)?;
    result.raw_keyframes = raw.to_vec();
    let info = result
        .components
        .iter()
        .find(|c| c.tag == *b"INFO")
        .ok_or_else(invalid)?;
    let pos = result.components.iter().find(|c| c.tag == *b"POS\0");
    let rot = result.components.iter().find(|c| c.tag == *b"ROT\0");
    if info.offset as usize + 20 > stride
        || pos.is_some_and(|c| c.offset as usize + 12 > stride)
        || rot.is_some_and(|c| c.offset as usize + 16 > stride)
    {
        return Err(invalid());
    }
    for row in raw.chunks_exact(stride) {
        let at = info.offset as usize;
        let frame = word(row, at);
        if result.keyframes.last().is_some_and(|key| key.frame > frame) {
            return Err(AnimError::UnorderedFrames { offset });
        }
        result.keyframes.push(AnimKeyframe {
            frame,
            info: std::array::from_fn(|i| word(row, at + 4 + i * 4)),
            position: pos.map(|c| {
                std::array::from_fn(|i| f32::from_bits(word(row, c.offset as usize + i * 4)))
            }),
            rotation: rot.map(|c| {
                std::array::from_fn(|i| f32::from_bits(word(row, c.offset as usize + i * 4)))
            }),
        });
    }
    Ok(result)
}

/// Parse an anim resource.
pub fn parse(data: &[u8]) -> Result<Anim, AnimError> {
    if data.len() < 272 + 64 {
        return Err(AnimError::TooShort);
    }
    if &data[0..4] != b"ANIM" {
        return Err(AnimError::BadMagic);
    }

    let file_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if file_size != data.len() {
        return Err(AnimError::SizeMismatch);
    }

    // String field
    let str_end = data[12..272].iter().position(|&b| b == 0).map(|p| p + 12);
    let str_end = str_end.ok_or(AnimError::NoStringTerminator)?;
    let path = String::from_utf8_lossy(&data[12..str_end]).to_string();

    // Animation data header
    let hash = u32::from_le_bytes([data[272], data[273], data[274], data[275]]);
    let frame_duration = f32::from_le_bytes([data[284], data[285], data[286], data[287]]);
    let duration = f32::from_le_bytes([data[288], data[289], data[290], data[291]]);
    let last_frame = u32::from_le_bytes([data[296], data[297], data[298], data[299]]);
    let header_field_304 = word(data, 304);
    let bone_count = word(data, 324);

    data.get(512..).ok_or(AnimError::TooShort)?;
    let table = word(data, 328) as usize;
    let num_offsets = bone_count as usize;
    let table_len = num_offsets
        .checked_mul(4)
        .ok_or(AnimError::InvalidChannelTable)?;
    let table_data = data
        .get(
            table
                ..table
                    .checked_add(table_len)
                    .ok_or(AnimError::InvalidChannelTable)?,
        )
        .filter(|_| table >= 512)
        .ok_or(AnimError::InvalidChannelTable)?;
    let mut offsets = Vec::with_capacity(num_offsets);
    for i in 0..num_offsets {
        offsets.push(word(table_data, i * 4) as usize);
    }

    // Parse each bone
    let mut bones = Vec::with_capacity(num_offsets);
    for bone_offset in offsets {
        bones.push(channel(data, bone_offset)?);
    }

    // Animation name from trailer (ASCII, null-terminated, at end of file)
    let mut name_end = data.len();
    while name_end > 0 && data[name_end - 1] == 0 {
        name_end -= 1;
    }
    let mut name_start = name_end;
    while name_start > 0 && data[name_start - 1].is_ascii_graphic() {
        name_start -= 1;
    }
    let anim_name = if name_start < name_end {
        String::from_utf8_lossy(&data[name_start..name_end]).to_string()
    } else {
        String::new()
    };

    Ok(Anim {
        path,
        hash,
        header_flags_280: word(data, 280),
        frame_duration,
        duration,
        last_frame,
        bone_count,
        header_field_304,
        anim_name,
        bones,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal anim resource for testing.
    fn build_test_anim() -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(b"ANIM");
        let size_pos = data.len();
        data.extend_from_slice(&0u32.to_le_bytes()); // placeholder for size
        data.extend_from_slice(&25u32.to_le_bytes());

        // String field (260 bytes)
        let path = b"C:\\test\\test.animation\0";
        data.extend_from_slice(path);
        data.resize(12 + 260, 0);

        // Anim data header (40 bytes at offset 272)
        let hash = 0x12345678u32;
        data.extend_from_slice(&hash.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0.033333f32.to_le_bytes()); // frame_dur
        data.extend_from_slice(&30.0f32.to_le_bytes()); // duration
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&29u32.to_le_bytes()); // last_frame
        data.extend_from_slice(&100u32.to_le_bytes()); // field_28
        data.extend_from_slice(&1u32.to_le_bytes()); // bone_count
        data.extend_from_slice(&0u32.to_le_bytes());

        // Pad to offset 512
        data.resize(512, 0);
        data[324..328].copy_from_slice(&1u32.to_le_bytes());
        data[328..332].copy_from_slice(&512u32.to_le_bytes());

        // Offset table: 1 bone at offset 552
        data.extend_from_slice(&552u32.to_le_bytes());
        data.extend_from_slice(b"CHAN");

        // Pad to bone offset 552
        data.resize(552, 0);

        // Bone section at 552
        let bone_start = data.len();
        assert_eq!(bone_start, 552);
        data.extend_from_slice(b"CHAN");
        data.extend_from_slice(&0u32.to_le_bytes());
        let name = b"root\0";
        data.extend_from_slice(name);
        data.resize(bone_start + 228, 0);
        data[bone_start + 212..bone_start + 216].copy_from_slice(&2u32.to_le_bytes());
        data[bone_start + 216..bone_start + 220]
            .copy_from_slice(&((bone_start + 324) as u32).to_le_bytes());
        data[bone_start + 220..bone_start + 224].copy_from_slice(&3u32.to_le_bytes());
        data[bone_start + 224..bone_start + 228]
            .copy_from_slice(&((bone_start + 228) as u32).to_le_bytes());
        for (tag, offset, slot, flags) in [
            (*b"INFO", 0u32, 0u32, 0u32),
            (*b"POS\0", 20, 1, 0x71),
            (*b"ROT\0", 100, 2, 0x32),
        ] {
            data.extend_from_slice(&flags.to_le_bytes());
            data.extend_from_slice(&tag);
            for word in [offset, 200, slot, 0, 0, 0] {
                data.extend_from_slice(&word.to_le_bytes());
            }
        }
        for frame in [0u32, 29] {
            let start = data.len();
            data.resize(start + 200, 0);
            data[start..start + 4].copy_from_slice(&frame.to_le_bytes());
            data[start + 20..start + 24].copy_from_slice(&(frame as f32).to_le_bytes());
            data[start + 112..start + 116].copy_from_slice(&1.0f32.to_le_bytes());
        }

        // Trailer
        data.extend_from_slice(&0x60u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        // Animation name
        data.extend_from_slice(b"test_anim\0");

        // Fix file size
        let size = data.len() as u32;
        data[size_pos..size_pos + 4].copy_from_slice(&size.to_le_bytes());

        data
    }

    #[test]
    fn parse_header() {
        let data = build_test_anim();
        let anim = parse(&data).unwrap();
        assert_eq!(anim.path, "C:\\test\\test.animation");
        assert_eq!(anim.hash, 0x12345678);
        assert!((anim.frame_duration - 0.033333).abs() < 1e-5);
        assert!((anim.duration - 30.0).abs() < 1e-5);
        assert_eq!(anim.last_frame, 29);
        assert_eq!(anim.anim_name, "test_anim");
    }

    #[test]
    fn parse_bones() {
        let data = build_test_anim();
        let anim = parse(&data).unwrap();
        assert_eq!(anim.bones.len(), 1);
        assert_eq!(anim.bones[0].name, "root");
        assert_eq!(anim.bones[0].keyframe_count, 2);
        assert!(anim.bones[0].has_position);
        assert!(anim.bones[0].has_rotation);
    }

    #[test]
    fn bad_magic() {
        let data = vec![0u8; 400];
        assert!(matches!(parse(&data), Err(AnimError::BadMagic)));
    }

    #[test]
    fn header_truncated_before_channel_table_is_too_short() {
        let full = build_test_anim();
        for len in 336..512 {
            let mut data = full[..len].to_vec();
            data[4..8].copy_from_slice(&(len as u32).to_le_bytes());
            assert!(
                matches!(parse(&data), Err(AnimError::TooShort)),
                "length {len}"
            );
        }
    }

    #[test]
    fn bone_offsets_outside_resource_are_rejected() {
        let full = build_test_anim();
        for offset in [
            full.len() - 7,
            full.len(),
            full.len() + 1,
            u32::MAX as usize,
        ] {
            let mut data = full.clone();
            data[512..516].copy_from_slice(&(offset as u32).to_le_bytes());
            assert!(
                matches!(parse(&data), Err(AnimError::InvalidBoneOffset { offset: bad }) if bad == offset)
            );
        }
    }

    #[test]
    fn truncated_optional_channel_metadata_remains_absent() {
        let mut data = build_test_anim();
        data.truncate(552 + 8);
        let len = data.len() as u32;
        data[4..8].copy_from_slice(&len.to_le_bytes());
        let anim = parse(&data).unwrap();
        assert_eq!(anim.bones.len(), 1);
        assert_eq!(anim.bones[0].keyframe_count, 0);
        assert!(!anim.bones[0].has_position);
        assert!(!anim.bones[0].has_rotation);
    }

    #[test]
    fn path_terminator_must_be_inside_fixed_string_field() {
        let mut data = build_test_anim();
        data[12..272].fill(b'A');
        assert!(matches!(parse(&data), Err(AnimError::NoStringTerminator)));
    }

    #[test]
    fn every_truncated_prefix_is_safe_to_parse() {
        let full = build_test_anim();
        for len in 0..full.len() {
            let mut data = full[..len].to_vec();
            if len >= 8 {
                // Avoid stopping at SizeMismatch so each remaining parser boundary
                // is exercised with a self-consistent declared resource size.
                data[4..8].copy_from_slice(&(len as u32).to_le_bytes());
            }
            let _ = parse(&data);
        }
    }

    #[test]
    fn interleaved_rows_expose_actual_keys_and_component_descriptors() {
        let anim = parse(&build_test_anim()).unwrap();
        assert_eq!(anim.bone_count, 1);
        let bone = &anim.bones[0];
        assert_eq!(bone.keyframe_count, 2);
        assert_eq!(bone.keyframe_stride, 200);
        assert_eq!(bone.frame_range(), Some((0, 29)));
        assert_eq!(bone.components[1].offset, 20);
        assert_eq!(bone.components[2].offset, 100);
        assert_eq!(bone.raw_keyframes.len(), 400);
        assert_eq!(bone.keyframes[1].position, Some([29.0, 0.0, 0.0]));
        assert_eq!(bone.keyframes[0].rotation, Some([0.0, 0.0, 0.0, 1.0]));
    }

    #[test]
    fn actual_frame_range_does_not_use_the_legacy_header_word() {
        let mut data = build_test_anim();
        data[296..300].copy_from_slice(&72u32.to_le_bytes());
        let anim = parse(&data).unwrap();
        assert_eq!(anim.last_frame, 72);
        assert_eq!(anim.bones[0].frame_range(), Some((0, 29)));
        assert_eq!(
            anim.bones[0]
                .sample(100.0, InterpolationPolicy::Step)
                .unwrap()
                .unwrap()
                .frame,
            29.0
        );
    }

    #[test]
    fn unsupported_component_tags_and_source_rows_are_retained() {
        let mut data = build_test_anim();
        data[552 + 264..552 + 268].copy_from_slice(&[1, 2, 3, 4]);
        let anim = parse(&data).unwrap();
        let bone = &anim.bones[0];
        assert!(!bone.has_position);
        assert!(!bone.components[1].is_supported());
        assert_eq!(bone.components[1].tag, [1, 2, 3, 4]);
        assert_eq!(&bone.raw_keyframes[220..224], &29.0f32.to_le_bytes());
        assert_eq!(bone.keyframes[1].position, None);
    }

    #[test]
    fn impossible_descriptor_and_key_ranges_are_rejected() {
        let full = build_test_anim();
        for at in [552 + 212, 552 + 216, 552 + 220, 552 + 224] {
            let mut data = full.clone();
            data[at..at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(matches!(
                parse(&data),
                Err(AnimError::InvalidChannelData { .. })
            ));
        }
        let mut data = full.clone();
        data[552 + 228 + 12..552 + 228 + 16].copy_from_slice(&0u32.to_le_bytes());
        assert!(matches!(
            parse(&data),
            Err(AnimError::InvalidChannelData { .. })
        ));
        let mut data = full;
        data[324..328].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(parse(&data), Err(AnimError::InvalidChannelTable)));
    }

    #[test]
    fn unordered_frames_are_rejected() {
        let mut data = build_test_anim();
        data[552 + 324..552 + 328].copy_from_slice(&30u32.to_le_bytes());
        assert!(matches!(
            parse(&data),
            Err(AnimError::UnorderedFrames { .. })
        ));
    }

    #[test]
    fn generic_interpolation_clamps_steps_and_lerps_positions() {
        let anim = parse(&build_test_anim()).unwrap();
        let bone = &anim.bones[0];
        assert_eq!(
            bone.sample(-4.0, InterpolationPolicy::Step)
                .unwrap()
                .unwrap()
                .position,
            Some([0.0, 0.0, 0.0])
        );
        assert_eq!(
            bone.sample(40.0, InterpolationPolicy::Step)
                .unwrap()
                .unwrap()
                .position,
            Some([29.0, 0.0, 0.0])
        );
        assert_eq!(
            bone.sample(14.5, InterpolationPolicy::Step)
                .unwrap()
                .unwrap()
                .position,
            Some([0.0, 0.0, 0.0])
        );
        assert_eq!(
            bone.sample(14.5, InterpolationPolicy::LinearSlerp)
                .unwrap()
                .unwrap()
                .position,
            Some([14.5, 0.0, 0.0])
        );
    }

    #[test]
    fn rotation_sampling_normalises_and_uses_the_shortest_arc() {
        let mut anim = parse(&build_test_anim()).unwrap();
        let bone = &mut anim.bones[0];
        bone.keyframes[1].rotation = Some([0.0, 0.0, 0.0, -2.0]);
        assert_eq!(
            bone.sample(14.5, InterpolationPolicy::LinearSlerp)
                .unwrap()
                .unwrap()
                .rotation,
            Some([0.0, 0.0, 0.0, 1.0])
        );
        bone.keyframes[1].rotation = Some([
            0.0,
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ]);
        let q = bone
            .sample(14.5, InterpolationPolicy::LinearSlerp)
            .unwrap()
            .unwrap()
            .rotation
            .unwrap();
        assert!((q[2] - (std::f32::consts::FRAC_PI_8).sin()).abs() < 1e-5);
        assert!((q[3] - (std::f32::consts::FRAC_PI_8).cos()).abs() < 1e-5);
    }

    #[test]
    fn invalid_numeric_values_are_explicit_sampling_errors() {
        let mut anim = parse(&build_test_anim()).unwrap();
        let bone = &mut anim.bones[0];
        assert_eq!(
            bone.sample(f32::NAN, InterpolationPolicy::Step),
            Err(AnimSampleError::NonFiniteFrame)
        );
        bone.keyframes[1].position = Some([f32::NAN, 0.0, 0.0]);
        assert!(bone.sample(0.0, InterpolationPolicy::Step).is_ok());
        assert_eq!(
            bone.sample(20.0, InterpolationPolicy::LinearSlerp),
            Err(AnimSampleError::InvalidPosition)
        );
        bone.keyframes[1].position = Some([1.0, 0.0, 0.0]);
        bone.keyframes[1].rotation = Some([0.0; 4]);
        assert_eq!(
            bone.sample(20.0, InterpolationPolicy::LinearSlerp),
            Err(AnimSampleError::InvalidRotation)
        );
    }
}
