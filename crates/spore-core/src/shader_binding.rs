//! Measured native uniform dispatch and material-parameter conversion.
//!
//! These rules recover constant data, not the complete draw's shader choice,
//! renderer features, lighting environment or texture bindings.
use crate::native_x87::{Context, Extended, X87Error};
use std::fmt;

/// Native callback779450 packs distance from current world translation to the
/// supplied camera position (override pointer, or current camera translation).
/// The three differences, squares, sum and square root remain in x87 until
/// the final binary32 store. Precision/rounding are explicit caller inputs.
pub fn camera_distance_constant(
    world_translation: [f32; 3],
    camera_position: [f32; 3],
    context: Context,
) -> Result<[f32; 4], X87Error> {
    let mut squares = [Extended::from_i32(0); 3];
    for axis in 0..3 {
        let difference = context.sub(
            Extended::from_f32(world_translation[axis])?,
            Extended::from_f32(camera_position[axis])?,
        )?;
        squares[axis] = context.mul(difference, difference)?;
    }
    let yz = context.add(squares[1], squares[2])?;
    let sum = context.add(squares[0], yz)?;
    Ok([context.store_f32(context.sqrt(sum)?)?, 0., 0., 1.])
}

/// The twelve bytes shared by source fragment and compiled uniform records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformDescriptor {
    pub raw: [u8; 12],
}
impl UniformDescriptor {
    fn short(self, at: usize) -> u16 {
        u16::from_be_bytes(self.raw[at..at + 2].try_into().unwrap())
    }
    /// Native callback table index, consumed at descriptor +0x10.
    pub fn callback_id(self) -> u16 {
        self.short(0)
    }
    /// Native per-shader-data dirty bit index, consumed at +0x12. It can
    /// differ from the callback ID, so the two must not be collapsed.
    pub fn dirty_data_id(self) -> u16 {
        self.short(2)
    }
    pub fn register_count(self) -> u16 {
        self.short(4)
    }
    /// Preserved fourth short; not used as the compiled program's register
    /// start. The compiled table stores those starts separately.
    pub fn fourth_word(self) -> u16 {
        self.short(6)
    }
    pub fn dependency_mask(self) -> u32 {
        u32::from_be_bytes(self.raw[8..12].try_into().unwrap())
    }
    /// Native 6FDDD0/6F9C50 upload gate. `program_changed` forces every
    /// nonzero dependency; pure shader-data dirtiness (bit 8) additionally
    /// checks this descriptor's own data bit. Mixed affected masks bypass
    /// that per-data test, exactly as in the native consumers.
    pub fn needs_upload(
        self,
        dirty_flags: u32,
        program_changed: bool,
        data_dirty: impl FnOnce(u16) -> bool,
    ) -> bool {
        let mask = self.dependency_mask();
        let affected = mask & (dirty_flags | if program_changed { mask } else { 0 });
        affected != 0 && (affected != 8 || program_changed || data_dirty(self.dirty_data_id()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParameterError {
    pub needed: usize,
    pub actual: usize,
}
impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "material parameters need {} bytes, got {}",
            self.needed, self.actual
        )
    }
}
impl std::error::Error for ParameterError {}

/// Shader-data 0x210 as consumed by native callback 778F50.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialParams {
    /// Byte zero is also consumed by fragment-selection conditions.
    pub mode: u8,
    pub has_fourth_component: bool,
    /// Exact float bits sent to one constant register, without normalization.
    pub constant_bits: [u32; 4],
}
impl MaterialParams {
    pub fn decode(data: &[u8]) -> Result<Self, ParameterError> {
        if data.len() < 16 {
            return Err(ParameterError {
                needed: 16,
                actual: data.len(),
            });
        }
        // The native reader explicitly compares each marker byte. This is
        // not unused alignment padding or an endian shader identifier.
        let has_fourth_component = data[1..4] == [0x5c, 0x44, 0x26];
        if has_fourth_component && data.len() < 20 {
            return Err(ParameterError {
                needed: 20,
                actual: data.len(),
            });
        }
        let word = |at| u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        Ok(Self {
            mode: data[0],
            has_fourth_component,
            constant_bits: [
                word(4),
                word(8),
                word(12),
                if has_fourth_component {
                    word(16)
                } else {
                    1.0f32.to_bits()
                },
            ],
        })
    }
    pub fn float4(self) -> [f32; 4] {
        self.constant_bits.map(f32::from_bits)
    }
}

/// One source 0x20D texture record. Native loading resolves the saved key into
/// a live texture pointer; the serialized pointer word is never dereferenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureBinding {
    pub sampler: u32,
    pub saved_pointer_word: u32,
    pub settings: [u8; 8],
    pub raster_instance: u64,
}
impl TextureBinding {
    /// 6FD370 changes address U/V/W together, then MAG/MIN, then MIP.
    /// A missing flag retains the previous sampler state, not a guessed default.
    pub fn address_mode(self) -> Option<i32> {
        (self.settings[0] & 1 != 0).then_some(i32::from(self.settings[1] as i8))
    }
    pub fn min_mag_filter(self) -> Option<i32> {
        (self.settings[0] & 2 != 0).then_some(i32::from(self.settings[2] as i8))
    }
    pub fn mip_filter(self) -> Option<i32> {
        (self.settings[0] & 4 != 0).then_some(i32::from(self.settings[3] as i8))
    }
    pub fn availability(self) -> Option<u8> {
        (self.settings[0] & 8 != 0).then_some(self.settings[4])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureBindingError {
    Truncated,
    TooManyTextures(u32),
    TrailingBytes,
}
impl fmt::Display for TextureBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source texture bindings: {self:?}")
    }
}
impl std::error::Error for TextureBindingError {}

pub fn texture_bindings(data: &[u8]) -> Result<Vec<TextureBinding>, TextureBindingError> {
    let count = u32::from_le_bytes(
        data.get(..4)
            .ok_or(TextureBindingError::Truncated)?
            .try_into()
            .unwrap(),
    );
    // Native 767790 rejects more than six entries. A live record has 16 bytes;
    // serialization additionally writes the eight-byte raster instance.
    if count > 6 {
        return Err(TextureBindingError::TooManyTextures(count));
    }
    let needed = 4 + count as usize * 24;
    if data.len() < needed {
        return Err(TextureBindingError::Truncated);
    }
    if data.len() != needed {
        return Err(TextureBindingError::TrailingBytes);
    }
    Ok(data[4..]
        .chunks_exact(24)
        .map(|row| TextureBinding {
            sampler: u32::from_le_bytes(row[..4].try_into().unwrap()),
            saved_pointer_word: u32::from_le_bytes(row[4..8].try_into().unwrap()),
            settings: row[8..16].try_into().unwrap(),
            raster_instance: u64::from_le_bytes(row[16..24].try_into().unwrap()),
        })
        .collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityLightingError {
    UnsupportedPrefix,
    Nonfinite,
}
impl fmt::Display for IdentityLightingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source identity lighting: {self:?}")
    }
}
impl std::error::Error for IdentityLightingError {}

/// Native 70A820's plain RGB coefficient route through 783780, an empty
/// 709640 accumulator, 7834E0 and final exposure multiplication. Supports the
/// first sixteen coefficients used by the measured static Cell programs.
/// Optional source combination properties and later frame rotation are outside
/// this explicit identity-frame operation. Callers must establish their absence.
pub fn identity_sh_prefix(
    colors: &[[f32; 3]],
    exposure: f32,
    count: usize,
) -> Result<Vec<[f32; 4]>, IdentityLightingError> {
    if count == 0 || count > 16 || colors.len() < count {
        return Err(IdentityLightingError::UnsupportedPrefix);
    }
    if !exposure.is_finite() || colors[..count].iter().flatten().any(|v| !v.is_finite()) {
        return Err(IdentityLightingError::Nonfinite);
    }
    // Measured startup routines 12E44A0..12E4690/12E47C0 compute the
    // positive real-SH basis factors from the stored f32 PI. Compute their
    // square roots in f64 before the native f32 store. This table is standard
    // basis mathematics, not an embedded game coefficient array.
    let rational = [
        (1., 4.),
        (3., 4.),
        (3., 4.),
        (3., 4.),
        (15., 4.),
        (15., 4.),
        (5., 16.),
        (15., 4.),
        (15., 16.),
        (35., 32.),
        (105., 4.),
        (21., 32.),
        (7., 16.),
        (21., 32.),
        (105., 16.),
        (35., 32.),
    ];
    let mut result = Vec::with_capacity(count);
    for (i, rgb) in colors.iter().take(count).enumerate() {
        let (numerator, denominator) = rational[i];
        let factor = (numerator / (denominator * f64::from(std::f32::consts::PI))).sqrt() as f32;
        // SDK disassembly confirms DIVSS then MULPS: keep the f32 reciprocal
        // rounding and each operation, including addition to the zero vector.
        let inverse = 1.0f32 / factor;
        let normalized = [rgb[0], rgb[1], rgb[2], 1.].map(|value| {
            let unpacked = value * inverse;
            let accumulated = unpacked + 0.0;
            let packed = accumulated * factor;
            packed * exposure
        });
        if normalized.iter().any(|v| !v.is_finite()) {
            return Err(IdentityLightingError::Nonfinite);
        }
        result.push(normalized);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_distance_uses_three_axes_and_exact_native_register_padding() {
        for control in [0x007f, 0x027f, 0x037f, 0x077f, 0x0b7f, 0x0f7f] {
            let context = Context::from_control_word(control).unwrap();
            assert_eq!(
                camera_distance_constant([4., 6., 9.], [1., 2., 9.], context).unwrap(),
                [5., 0., 0., 1.]
            );
            assert_eq!(
                camera_distance_constant([-1., -2., -3.], [-1., -2., -3.], context).unwrap(),
                [0., 0., 0., 1.]
            );
        }
        assert_eq!(
            camera_distance_constant(
                [f32::NAN, 0., 0.],
                [0.; 3],
                Context::from_control_word(0x027f).unwrap()
            ),
            Err(X87Error::NonFiniteInput)
        );
    }

    #[test]
    fn identity_sh_keeps_native_f32_rounding_and_rejects_unsupported_inputs() {
        // Independent Decimal-100 calculation rounds after each measured
        // native DIVSS/MULPS/ADDPS operation. Synthetic signed/HDR inputs
        // distinguish this sequence from raw RGB times exposure.
        let expected = [
            [0x3ce56042, 0xbef3b645, 0x3f611341, 0x3e8f5c29],
            [0x3ce56041, 0xbef3b646, 0x3f611340, 0x3e8f5c28],
            [0x3ce56041, 0xbef3b646, 0x3f611340, 0x3e8f5c28],
            [0x3ce56041, 0xbef3b646, 0x3f611340, 0x3e8f5c28],
            [0x3ce56042, 0xbef3b646, 0x3f611342, 0x3e8f5c29],
            [0x3ce56042, 0xbef3b646, 0x3f611342, 0x3e8f5c29],
            [0x3ce56041, 0xbef3b645, 0x3f611341, 0x3e8f5c28],
            [0x3ce56042, 0xbef3b646, 0x3f611342, 0x3e8f5c29],
            [0x3ce56042, 0xbef3b646, 0x3f611342, 0x3e8f5c29],
            [0x3ce56042, 0xbef3b646, 0x3f611341, 0x3e8f5c29],
            [0x3ce56041, 0xbef3b645, 0x3f611340, 0x3e8f5c29],
            [0x3ce56043, 0xbef3b647, 0x3f611341, 0x3e8f5c29],
            [0x3ce56041, 0xbef3b646, 0x3f611340, 0x3e8f5c28],
            [0x3ce56043, 0xbef3b647, 0x3f611341, 0x3e8f5c29],
            [0x3ce56041, 0xbef3b645, 0x3f611340, 0x3e8f5c29],
            [0x3ce56042, 0xbef3b646, 0x3f611341, 0x3e8f5c29],
        ];
        let colors = [[0.1, -1.7, 3.14]; 16];
        let output = identity_sh_prefix(&colors, 0.28, 16).unwrap();
        assert_eq!(
            output
                .iter()
                .map(|row| row.map(f32::to_bits))
                .collect::<Vec<_>>(),
            expected
        );
        let zeros = identity_sh_prefix(&[[-0., 0., -0.]], 0.28, 1).unwrap();
        assert!(zeros[0][..3].iter().all(|v| v.to_bits() == 0));
        assert!(identity_sh_prefix(&colors, 0.28, 17).is_err());
        assert!(identity_sh_prefix(&colors, 0.28, 0).is_err());
        assert!(identity_sh_prefix(&colors[..8], 0.28, 9).is_err());
        assert!(identity_sh_prefix(&[[f32::NAN, 0., 0.]], 1., 1).is_err());
        assert!(identity_sh_prefix(&colors, f32::INFINITY, 16).is_err());
        assert!(identity_sh_prefix(&[[f32::MAX; 3]], 1., 1).is_err());
    }

    #[test]
    fn material_marker_controls_fourth_component_and_required_length() {
        let mut data = vec![4, 0x5c, 0x44, 0x26];
        for v in [20.0f32, 0.95, 1.0, 0.28] {
            data.extend(v.to_le_bytes());
        }
        let p = MaterialParams::decode(&data).unwrap();
        assert_eq!(p.mode, 4);
        assert!(p.has_fourth_component);
        assert_eq!(p.float4(), [20.0, 0.95, 1.0, 0.28]);
        for length in 0..20 {
            assert!(MaterialParams::decode(&data[..length]).is_err());
        }
        // Changing each individual marker byte takes the legacy path. It
        // neither reads the missing fourth float nor uploads the saved 0.28.
        for at in 1..4 {
            let mut legacy = data.clone();
            legacy[at] ^= 1;
            for length in [16, 20] {
                let p = MaterialParams::decode(&legacy[..length]).unwrap();
                assert!(!p.has_fourth_component);
                assert_eq!(p.float4(), [20.0, 0.95, 1.0, 1.0]);
            }
        }
        data[0] = 255;
        data[4..8].copy_from_slice(&0x7fc01234u32.to_le_bytes());
        let p = MaterialParams::decode(&data).unwrap();
        assert_eq!(p.mode, 255);
        assert_eq!(p.constant_bits[0], 0x7fc01234);
    }

    #[test]
    fn uniform_dispatch_keeps_callback_and_dirty_data_distinct() {
        let mut d = UniformDescriptor {
            raw: [0x02, 0x1a, 0x02, 0x19, 0, 9, 0x12, 0x34, 0, 0, 0, 8],
        };
        assert_eq!(d.callback_id(), 0x21a);
        assert_eq!(d.dirty_data_id(), 0x219);
        assert_eq!(d.register_count(), 9);
        assert_eq!(d.fourth_word(), 0x1234);
        assert!(!d.needs_upload(0, false, |_| panic!("unaffected data consulted")));
        assert!(!d.needs_upload(8, false, |id| {
            assert_eq!(id, 0x219);
            false
        }));
        assert!(d.needs_upload(8, false, |id| id == 0x219));
        assert!(d.needs_upload(0, true, |_| panic!("forced data consulted")));
        d.raw[11] = 9;
        assert!(!d.needs_upload(8, false, |_| false));
        assert!(d.needs_upload(9, false, |_| panic!("mixed dirty mask consulted")));
        assert!(d.needs_upload(1, false, |_| panic!("non-data dirtiness consulted")));
        d.raw[11] = 0;
        assert!(!d.needs_upload(u32::MAX, true, |_| panic!("zero mask consulted")));
    }

    #[test]
    fn texture_flags_retain_unchanged_state_and_bound_saved_records() {
        let mut bytes = 1u32.to_le_bytes().to_vec();
        bytes.extend(2u32.to_le_bytes());
        bytes.extend(0xf0abcdefu32.to_le_bytes()); // Saved process word is opaque.
        bytes.extend([15, 3, 2, 2, 1, 0x52, 0x14, 0]);
        bytes.extend(0x1122334455667788u64.to_le_bytes());
        let b = texture_bindings(&bytes).unwrap()[0];
        assert_eq!(b.sampler, 2);
        assert_eq!(b.saved_pointer_word, 0xf0abcdef);
        assert_eq!(b.raster_instance, 0x1122334455667788);
        assert_eq!(
            (
                b.address_mode(),
                b.min_mag_filter(),
                b.mip_filter(),
                b.availability()
            ),
            (Some(3), Some(2), Some(2), Some(1))
        );
        let mut b = b;
        b.settings[0] = 0;
        assert_eq!(
            (
                b.address_mode(),
                b.min_mag_filter(),
                b.mip_filter(),
                b.availability()
            ),
            (None, None, None, None)
        );
        b.settings[0] = 1;
        b.settings[1] = 255;
        assert_eq!(b.address_mode(), Some(-1));
        for n in 0..bytes.len() {
            assert!(texture_bindings(&bytes[..n]).is_err());
        }
        bytes.push(0);
        assert_eq!(
            texture_bindings(&bytes),
            Err(TextureBindingError::TrailingBytes)
        );
        bytes[..4].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(
            texture_bindings(&bytes),
            Err(TextureBindingError::TooManyTextures(7))
        );
    }
}
