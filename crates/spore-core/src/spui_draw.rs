//! Measured native UTFWin image flags and explicit D3D9 state updates.
//!
//! Native 952800 maps Image+24 flags into batch flags; 82E990 changes the
//! filter/blend factors. Native 9630A0 supplies the common state. This module
//! describes only those explicit updates, not the complete inherited device
//! state, texture upload conversion, or a retail screen's active shader path.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiGeometryError {
    Nonfinite,
    InvalidViewport,
    ObjectIndex(u8),
    InvalidProgress,
}
impl std::fmt::Display for UiGeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native UI geometry: {self:?}")
    }
}
impl std::error::Error for UiGeometryError {}

/// Native 95F9E0 foreground inheritance. Each packed ARGB channel uses the
/// floor of child * parent / 255; there is no gamma conversion or rounding up.
pub fn multiply_argb(child: u32, parent: u32) -> u32 {
    [0, 8, 16, 24].into_iter().fold(0, |color, shift| {
        let first = (child >> shift) & 255;
        let second = (parent >> shift) & 255;
        color | ((first * second / 255) << shift)
    })
}

/// Native FadeEffect callback 96EFB0 replaces the foreground with white and
/// truncated alpha from (1-progress)*255. The surrounding effect must provide
/// a finite unit progress; out-of-domain conversion is deliberately rejected.
pub fn fade_white_argb(progress: f32) -> Result<u32, UiGeometryError> {
    if !progress.is_finite() {
        return Err(UiGeometryError::Nonfinite);
    }
    if !(0.0..=1.0).contains(&progress) {
        return Err(UiGeometryError::InvalidProgress);
    }
    let alpha = ((1.0f32 - progress) * 255.0f32) as u32;
    Ok((alpha << 24) | 0x00ff_ffff)
}

/// Native 9630A0 declaration, populated by 82E990 from 20-byte draw commands.
/// Colors are packed ARGB words consumed by D3DDECLTYPE_D3DCOLOR.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub argb: u32,
    pub control: u32,
    pub uv: [f32; 2],
}
impl UiVertex {
    pub fn new(
        position: [f32; 2],
        argb: u32,
        uv: [f32; 2],
        object_index: u8,
        textured: bool,
    ) -> Result<Self, UiGeometryError> {
        if object_index >= 16 {
            return Err(UiGeometryError::ObjectIndex(object_index));
        }
        if position.iter().chain(&uv).any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        // The G byte becomes the six-register object offset; the B byte
        // selects textured versus solid drawing in the original programs.
        let control = u32::from(object_index) * 0x600 | if textured { 0xff } else { 0 };
        Ok(Self {
            position,
            argb,
            control,
            uv,
        })
    }
}

/// Native row-major matrix plus its explicit native classification word.
/// Kind zero is the native identity shortcut; rows are not inspected to infer it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiMatrix {
    pub rows: [[f32; 4]; 4],
    pub kind: i32,
}
impl UiMatrix {
    /// Native 95F980 composes current * argument. Its identity shortcuts copy
    /// or retain all 17 words. 45F430 accumulates four products in this order.
    pub fn then(self, argument: Self) -> Result<Self, UiGeometryError> {
        if self
            .rows
            .iter()
            .flatten()
            .chain(argument.rows.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(UiGeometryError::Nonfinite);
        }
        if self.kind == 0 {
            return Ok(argument);
        }
        if argument.kind == 0 {
            return Ok(self);
        }
        let rows = std::array::from_fn(|r| {
            std::array::from_fn(|c| {
                let first = self.rows[r][0] * argument.rows[0][c];
                let second = self.rows[r][1] * argument.rows[1][c];
                let third = self.rows[r][2] * argument.rows[2][c];
                let fourth = self.rows[r][3] * argument.rows[3][c];
                ((first + second) + third) + fourth
            })
        });
        if rows.iter().flatten().any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        Ok(Self {
            rows,
            kind: self.kind.max(argument.kind),
        })
    }
}

/// Native 95BE80 viewport transform. The backend's half-pixel capability must
/// be supplied explicitly; this module does not infer the active retail backend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiViewport {
    pub matrix: UiMatrix,
    scale: [f32; 2],
    translation: [f32; 2],
}
impl UiViewport {
    pub fn new(area: [f32; 4], half_pixel: bool) -> Result<Self, UiGeometryError> {
        if area.iter().any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        let width = area[2] - area[0];
        let height = area[3] - area[1];
        if width <= 0. || height <= 0. || !width.is_finite() || !height.is_finite() {
            return Err(UiGeometryError::InvalidViewport);
        }
        let sx = 2.0f32 / width;
        let y_scale = 2.0f32 / height;
        let sy = -y_scale;
        let (tx, ty) = if half_pixel {
            (-1.0 - (area[0] + 0.5) * sx, (area[1] + 0.5) * y_scale + 1.0)
        } else {
            (-1.0 - area[0] * sx, area[1] * y_scale + 1.0)
        };
        let rows = [
            [sx, 0., 0., 0.],
            [0., sy, 0., 0.],
            [0., 0., 0., 0.],
            [tx, ty, 0.5, 1.],
        ];
        if rows.iter().flatten().any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        Ok(Self {
            matrix: UiMatrix { rows, kind: 1 },
            scale: [sx, sy],
            translation: [tx, ty],
        })
    }

    /// Native 82EDC0 shader clip order is left,bottom,right,top after projection.
    /// Hardware-scissor draws supply a generous shader clip instead.
    pub fn shader_clip(
        self,
        clip: [f32; 4],
        path: UiClipPath,
    ) -> Result<[f32; 4], UiGeometryError> {
        if clip.iter().any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        if path == UiClipPath::Scissor {
            return Ok([-10000., -10000., 10000., 10000.]);
        }
        let result = [
            clip[0] * self.scale[0] + self.translation[0],
            clip[3] * self.scale[1] + self.translation[1],
            clip[2] * self.scale[0] + self.translation[0],
            clip[1] * self.scale[1] + self.translation[1],
        ];
        if result.iter().any(|v| !v.is_finite()) {
            return Err(UiGeometryError::Nonfinite);
        }
        Ok(result)
    }

    /// One object's exact six-register utfWin upload: RGBA tint, four matrix
    /// rows, then projected clip. Source color conversion multiplies by the
    /// stored float32 reciprocal of 255 instead of dividing each component.
    pub fn object_registers(
        self,
        argb: u32,
        local: UiMatrix,
        clip: [f32; 4],
        path: UiClipPath,
    ) -> Result<[[f32; 4]; 6], UiGeometryError> {
        let reciprocal = f32::from_bits(0x3b80_8081);
        let tint = [
            ((argb >> 16) & 255) as f32 * reciprocal,
            ((argb >> 8) & 255) as f32 * reciprocal,
            (argb & 255) as f32 * reciprocal,
            (argb >> 24) as f32 * reciprocal,
        ];
        let matrix = local.then(self.matrix)?;
        let clip = self.shader_clip(clip, path)?;
        Ok([
            tint,
            matrix.rows[0],
            matrix.rows[1],
            matrix.rows[2],
            matrix.rows[3],
            clip,
        ])
    }
}

/// Raw Image+24 word. Only mask bits 2 and 4 have measured draw consumers here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDrawFlags(pub u32);

impl ImageDrawFlags {
    /// The serialized image/atlas resource path in native 833CB0 passes zero
    /// to Image constructor 957A50. Later caller mutations remain possible.
    pub const SERIALIZED_RESOURCE: Self = Self(0);

    /// Native 952800: image bit 2 becomes batch bit 2; an absent image bit 4
    /// enables batch bit 1. These are different bit domains.
    pub const fn batch_flags(self) -> u32 {
        let source_blend = self.0 & 2;
        let linear_filter = if self.0 & 4 == 0 { 1 } else { 0 };
        source_blend | linear_filter
    }

    pub const fn draw_state(self) -> UiDrawState {
        UiDrawState::for_textured_batch(self.batch_flags())
    }
}

/// Original shader selection in native 82E630. The caller must establish
/// whether the source configuration enables scissoring for the current draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiClipPath {
    Shader,
    Scissor,
}

impl UiClipPath {
    pub const fn shader_id(self) -> u32 {
        match self {
            Self::Shader => 1,
            Self::Scissor => 2,
        }
    }

    pub const fn shader_name(self) -> &'static str {
        match self {
            Self::Shader => "UTFWin_UIShader",
            Self::Scissor => "UTFWin_UIShader_NoClip",
        }
    }

    pub const fn scissor_enabled(self) -> bool {
        matches!(self, Self::Scissor)
    }
}

/// Numeric Direct3D9 state/value pair, with no Windows dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeState {
    pub id: u32,
    pub value: u32,
}

/// Explicit state applied to each native UI texture unit. Address W, sRGB,
/// anisotropy and other inherited sampler settings are outside this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSamplerState {
    pub unit: u32,
    /// D3DTEXF_POINT = 1 or D3DTEXF_LINEAR = 2, for both MAG and MIN.
    pub min_mag_filter: u32,
}

impl UiSamplerState {
    pub const fn states(self) -> [NativeState; 5] {
        [
            NativeState { id: 1, value: 3 }, // ADDRESSU, CLAMP
            NativeState { id: 2, value: 3 }, // ADDRESSV, CLAMP
            NativeState {
                id: 5,
                value: self.min_mag_filter,
            },
            NativeState {
                id: 6,
                value: self.min_mag_filter,
            },
            NativeState { id: 7, value: 0 }, // MIPFILTER, NONE
        ]
    }
}

/// Measured base UI state after per-batch updates with a texture bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiDrawState {
    /// Retained incoming batch word; only bits 1 and 2 affect this subset.
    pub batch_flags: u32,
    pub samplers: [UiSamplerState; 2],
    /// Exactly the seven render states explicitly configured in 9630A0.
    /// This does not supply unmeasured ZWRITE, separate alpha or sRGB values.
    pub render_states: [NativeState; 7],
}

impl UiDrawState {
    /// Requires a batch with at least one bound texture. Native 82E990 skips
    /// filter updates when both units are unbound, retaining earlier state.
    pub const fn for_textured_batch(batch_flags: u32) -> Self {
        let filter = if batch_flags & 1 != 0 { 2 } else { 1 };
        let source_blend = if batch_flags & 2 != 0 { 2 } else { 5 };
        Self {
            batch_flags,
            samplers: [
                UiSamplerState {
                    unit: 0,
                    min_mag_filter: filter,
                },
                UiSamplerState {
                    unit: 1,
                    min_mag_filter: filter,
                },
            ],
            render_states: [
                NativeState {
                    id: 19,
                    value: source_blend,
                }, // SRCBLEND
                NativeState { id: 20, value: 6 }, // DESTBLEND, INVSRCALPHA
                NativeState { id: 27, value: 1 }, // ALPHABLENDENABLE
                NativeState { id: 22, value: 1 }, // CULLMODE, NONE
                NativeState { id: 25, value: 8 }, // ALPHAFUNC, ALWAYS
                NativeState { id: 7, value: 0 },  // ZENABLE
                NativeState { id: 28, value: 0 }, // FOGENABLE
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_value(state: UiDrawState, id: u32) -> u32 {
        state
            .render_states
            .iter()
            .find(|s| s.id == id)
            .unwrap()
            .value
    }

    #[test]
    fn foreground_inheritance_and_fade_use_native_quantization_not_rounding() {
        assert_eq!(multiply_argb(0x8064_c832, 0x8040_80c0), 0x4019_6425);
        assert_eq!(multiply_argb(0x78ab_cd03, u32::MAX), 0x78ab_cd03);
        assert_eq!(multiply_argb(0x78ab_cd03, 0), 0);
        assert_eq!(fade_white_argb(0.).unwrap(), 0xffff_ffff);
        assert_eq!(fade_white_argb(0.25).unwrap(), 0xbfff_ffff);
        assert_eq!(fade_white_argb(0.5).unwrap(), 0x7fff_ffff);
        assert_eq!(fade_white_argb(0.75).unwrap(), 0x3fff_ffff);
        assert_eq!(fade_white_argb(1.).unwrap(), 0x00ff_ffff);
        assert!(fade_white_argb(-0.1).is_err());
        assert!(fade_white_argb(f32::NAN).is_err());
    }

    #[test]
    fn native_viewport_keeps_pixel_origin_half_pixel_and_row_composition_order() {
        let viewport = UiViewport::new([10., 20., 266., 148.], true).unwrap();
        assert_eq!(
            viewport.matrix.rows,
            [
                [0.0078125, 0., 0., 0.],
                [0., -0.015625, 0., 0.],
                [0., 0., 0., 0.],
                [-1.08203125, 1.3203125, 0.5, 1.]
            ]
        );
        let local = UiMatrix {
            rows: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [30., 40., 3., 1.],
            ],
            kind: 1,
        };
        let rows = viewport
            .object_registers(
                0xffff_ffff,
                local,
                [10., 20., 266., 148.],
                UiClipPath::Shader,
            )
            .unwrap();
        assert_eq!(rows[0], [1.; 4]);
        assert_eq!(rows[4], [-0.84765625, 0.6953125, 0.5, 1.]);
        assert_eq!(rows[5], [-1.00390625, -0.9921875, 0.99609375, 1.0078125]);
        let identity_marker = UiMatrix {
            rows: [[0.; 4]; 4],
            kind: 0,
        };
        assert_eq!(
            identity_marker.then(viewport.matrix).unwrap(),
            viewport.matrix
        );
        assert_eq!(
            viewport.matrix.then(identity_marker).unwrap(),
            viewport.matrix
        );
        assert_eq!(
            viewport
                .shader_clip([10., 20., 266., 148.], UiClipPath::Scissor)
                .unwrap(),
            [-10000., -10000., 10000., 10000.]
        );
        let tint = viewport
            .object_registers(
                0xff03_0607,
                identity_marker,
                [10., 20., 266., 148.],
                UiClipPath::Shader,
            )
            .unwrap()[0];
        assert_ne!(tint[0].to_bits(), (3.0f32 / 255.).to_bits());
    }

    #[test]
    fn native_batch_capacity_and_malformed_geometry_are_bounded() {
        assert_eq!(std::mem::size_of::<UiVertex>(), 24);
        assert_eq!(std::mem::offset_of!(UiVertex, control), 12);
        assert_eq!(std::mem::offset_of!(UiVertex, uv), 16);
        let vertex = UiVertex::new([12., 13.], 0x7856_3412, [0.25, 0.75], 15, true).unwrap();
        assert_eq!(vertex.control, 0x5aff);
        assert_eq!(vertex.argb, 0x7856_3412);
        assert_eq!(
            UiVertex::new([0.; 2], 0, [0.; 2], 16, true),
            Err(UiGeometryError::ObjectIndex(16))
        );
        assert!(UiVertex::new([f32::NAN, 0.], 0, [0.; 2], 0, false).is_err());
        assert!(UiViewport::new([1., 0., 1., 1.], false).is_err());
        assert!(UiViewport::new([0., 0., f32::INFINITY, 1.], false).is_err());
        let no_half = UiViewport::new([0., 0., 256., 128.], false).unwrap();
        assert_eq!(no_half.matrix.rows[3], [-1., 1., 0.5, 1.]);
    }

    #[test]
    fn serialized_image_is_linear_and_uses_source_alpha() {
        let state = ImageDrawFlags::SERIALIZED_RESOURCE.draw_state();
        assert_eq!(state.batch_flags, 1);
        assert!(state.samplers.iter().all(|s| s.min_mag_filter == 2));
        assert_eq!(render_value(state, 19), 5);
        assert_eq!(render_value(state, 20), 6);
        // Only measured U/V and MAG/MIN/MIP are exposed, with no invented W.
        assert_eq!(state.samplers[0].states().map(|s| s.id), [1, 2, 5, 6, 7]);
    }

    #[test]
    fn image_bits_control_independent_batch_domains() {
        let point = ImageDrawFlags(4).draw_state();
        assert_eq!(point.batch_flags, 0);
        assert!(point.samplers.iter().all(|s| s.min_mag_filter == 1));
        assert_eq!(render_value(point, 19), 5);

        let one = ImageDrawFlags(2).draw_state();
        assert_eq!(one.batch_flags, 3);
        assert!(one.samplers.iter().all(|s| s.min_mag_filter == 2));
        assert_eq!(render_value(one, 19), 2);

        let point_one = ImageDrawFlags(6).draw_state();
        assert_eq!(point_one.batch_flags, 2);
        assert!(point_one.samplers.iter().all(|s| s.min_mag_filter == 1));
        assert_eq!(render_value(point_one, 19), 2);
    }

    #[test]
    fn ignored_bits_are_retained_without_selecting_unestablished_states() {
        let flags = ImageDrawFlags(0xffff_fff9);
        assert_eq!(flags.0, 0xffff_fff9);
        assert_eq!(flags.batch_flags(), 1);
        let known = UiDrawState::for_textured_batch(3);
        let extra = UiDrawState::for_textured_batch(0xffff_ffff);
        assert_eq!(extra.batch_flags, 0xffff_ffff);
        assert_eq!(known.samplers, extra.samplers);
        assert_eq!(known.render_states, extra.render_states);
        assert_eq!(UiClipPath::Shader.shader_id(), 1);
        assert_eq!(UiClipPath::Scissor.shader_id(), 2);
        assert!(!UiClipPath::Shader.scissor_enabled());
        assert!(UiClipPath::Scissor.scissor_enabled());
    }
}
