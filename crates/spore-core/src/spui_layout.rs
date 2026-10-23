//! Measured source local layout operations, independent of rendering.
//! These are local layout coordinates; this is not a complete SPUI procedure
//! runner, global transform, hit tester, or class-default implementation.
use crate::spui::{SpuiElement, SpuiValue};
use std::fmt;

pub const SIMPLE_LAYOUT_CLASS: u32 = 0x2f41_8d72;
pub const SIMPLE_LAYOUT_FLAGS: u32 = 0x0f3d_0000;
pub const IMAGE_DRAWABLE_CLASS: u32 = 0xcf3c_4eae;
pub const IMAGE_CLASS: u32 = 0x01be_6b15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    UnsupportedClass(u32),
    MissingFlags,
    InvalidFlagsShape,
    InvalidProperty(u32),
    NonfiniteRectangle,
}
impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source UI local layout: {self:?}")
    }
}
impl std::error::Error for LayoutError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleLayout {
    /// Original byte, including high bits which the measured helper ignores.
    pub flags: u8,
}
impl SimpleLayout {
    pub fn from_element(element: &SpuiElement) -> Result<Self, LayoutError> {
        if element.proxy_id != SIMPLE_LAYOUT_CLASS {
            return Err(LayoutError::UnsupportedClass(element.proxy_id));
        }
        let p = element
            .property(SIMPLE_LAYOUT_FLAGS)
            .ok_or(LayoutError::MissingFlags)?;
        match &p.value {
            SpuiValue::Scalars { width: 1, bytes }
                if p.type_code == 7 && p.count == 1 && bytes.len() == 1 =>
            {
                Ok(Self { flags: bytes[0] })
            }
            _ => Err(LayoutError::InvalidFlagsShape),
        }
    }

    /// Native 97E920: add parent spans to selected child edges. Parent origin
    /// does not enter these local coordinates. Float32 operation order matters.
    pub fn apply(self, parent: [f32; 4], child: [f32; 4]) -> Result<[f32; 4], LayoutError> {
        self.adjust(parent, child, false)
    }
    /// Native 97E990 removes the same spans. This is the inverse of the local
    /// helper, before the caller's separate inverted-edge clamp.
    pub fn unapply(self, parent: [f32; 4], child: [f32; 4]) -> Result<[f32; 4], LayoutError> {
        self.adjust(parent, child, true)
    }
    fn adjust(
        self,
        parent: [f32; 4],
        mut child: [f32; 4],
        inverse: bool,
    ) -> Result<[f32; 4], LayoutError> {
        if parent.iter().chain(&child).any(|v| !v.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        if self.flags & 8 != 0 {
            let span = parent[2] - parent[0];
            child[2] = if inverse {
                child[2] - span
            } else {
                span + child[2]
            };
            if self.flags & 4 == 0 {
                child[0] = if inverse {
                    child[0] - span
                } else {
                    span + child[0]
                };
            }
        }
        if self.flags & 2 != 0 {
            let span = parent[3] - parent[1];
            child[3] = if inverse {
                child[3] - span
            } else {
                span + child[3]
            };
            if self.flags & 1 == 0 {
                child[1] = if inverse {
                    child[1] - span
                } else {
                    span + child[1]
                };
            }
        }
        if child.iter().any(|v| !v.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        Ok(child)
    }
}

/// Native Window layout event for a supplied list containing only SimpleLayout
/// procedures. Their priority is zero; the reverse source-array load and equal
/// priority insertion recover authored dispatch order. Each procedure clamps
/// separately, then 95A0F0 floors all edges once after dispatch. Other WinProcs,
/// layout invalidation, global transforms and event side effects are separate.
pub fn simple_window_layout(
    parent: [f32; 4],
    authored: [f32; 4],
    procedures: &[SimpleLayout],
) -> Result<[f32; 4], LayoutError> {
    if parent.iter().chain(&authored).any(|v| !v.is_finite()) {
        return Err(LayoutError::NonfiniteRectangle);
    }
    let mut rectangle = authored;
    for procedure in procedures {
        rectangle = clamp_inverted_edges(procedure.apply(parent, rectangle)?)?;
    }
    Ok(rectangle.map(f32::floor))
}

/// ImageDrawable's initial sizing pass in native 97D1F0, before effects,
/// tiling/slicing, clipping and painting. Defaults are from constructor 97D850.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageSizing {
    pub flags: u32,
    pub scale: f32,
    pub horizontal_alignment: u32,
    pub vertical_alignment: u32,
}
impl Default for ImageSizing {
    fn default() -> Self {
        Self {
            flags: 0,
            scale: 1.,
            horizontal_alignment: 3,
            vertical_alignment: 3,
        }
    }
}
impl ImageSizing {
    pub fn from_element(element: &SpuiElement) -> Result<Self, LayoutError> {
        if element.proxy_id != IMAGE_DRAWABLE_CLASS {
            return Err(LayoutError::UnsupportedClass(element.proxy_id));
        }
        let mut result = Self::default();
        for (id, target) in [
            (0x4f3c_4a27, &mut result.flags),
            (0x4f3c_4a29, &mut result.horizontal_alignment),
            (0x4f3c_4a28, &mut result.vertical_alignment),
        ] {
            if let Some(p) = element.property(id) {
                if p.type_code != 9 || p.count != 1 {
                    return Err(LayoutError::InvalidProperty(id));
                }
                *target = p.u32_value().ok_or(LayoutError::InvalidProperty(id))?;
            }
        }
        if let Some(p) = element.property(0x4f3c_4a2b) {
            if p.type_code != 11 || p.count != 1 {
                return Err(LayoutError::InvalidProperty(p.id));
            }
            let bytes: [u8; 4] = p
                .scalar_bytes()
                .and_then(|v| v.try_into().ok())
                .ok_or(LayoutError::InvalidProperty(p.id))?;
            result.scale = f32::from_le_bytes(bytes);
        }
        if !result.scale.is_finite() {
            return Err(LayoutError::InvalidProperty(0x4f3c_4a2b));
        }
        Ok(result)
    }

    pub fn rectangle(self, area: [f32; 4], intrinsic: [i32; 2]) -> Result<[f32; 4], LayoutError> {
        if !self.scale.is_finite() || area.iter().any(|v| !v.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        let mut rect = area;
        for (bit, first, last, size, alignment) in [
            (1, 0, 2, intrinsic[0], self.horizontal_alignment),
            (2, 1, 3, intrinsic[1], self.vertical_alignment),
        ] {
            if self.flags & bit == 0 {
                continue;
            }
            let span = size as f32 * self.scale;
            match alignment {
                2 => rect[first] = rect[last] - span,
                3 => rect[first] = ((rect[last] - area[first]) - span) * 0.5 + area[first],
                _ => {} // Native 1 and all other values retain the first edge.
            }
            rect[last] = rect[first] + span;
        }
        if rect.iter().any(|v| !v.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        Ok(rect)
    }
}

/// Serialized Image properties, before resource resolution. Native 957850
/// stores UV origin/spans and 955410 maps relative UV through that frame.
/// Source rectangles outside 0..1 and signed intrinsic dimensions are retained.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageRegion {
    pub resource_reference: u16,
    pub saved_uv: [f32; 4],
    pub intrinsic: [i32; 2],
    pub flags: crate::spui_draw::ImageDrawFlags,
}
impl ImageRegion {
    pub fn from_element(element: &SpuiElement) -> Result<Self, LayoutError> {
        if element.proxy_id != IMAGE_CLASS {
            return Err(LayoutError::UnsupportedClass(element.proxy_id));
        }
        let property = |id| element.property(id).ok_or(LayoutError::InvalidProperty(id));
        let resource_reference = property(0x01be_0001)?
            .reference()
            .ok_or(LayoutError::InvalidProperty(0x01be_0001))?;
        let saved_uv = property(0x01be_0002)?
            .rectangle()
            .ok_or(LayoutError::InvalidProperty(0x01be_0002))?;
        let dimensions = property(0x01be_0003)?;
        if dimensions.type_code != 15 || dimensions.count != 1 {
            return Err(LayoutError::InvalidProperty(dimensions.id));
        }
        let bytes = dimensions
            .scalar_bytes()
            .filter(|v| v.len() == 8)
            .ok_or(LayoutError::InvalidProperty(dimensions.id))?;
        let intrinsic = [
            i32::from_le_bytes(bytes[..4].try_into().unwrap()),
            i32::from_le_bytes(bytes[4..].try_into().unwrap()),
        ];
        let flags = property(0x01be_0004)?;
        if flags.type_code != 9 {
            return Err(LayoutError::InvalidProperty(flags.id));
        }
        let flags = flags
            .u32_value()
            .ok_or(LayoutError::InvalidProperty(flags.id))?;
        let image = Self {
            resource_reference,
            saved_uv,
            intrinsic,
            flags: crate::spui_draw::ImageDrawFlags(flags),
        };
        image.map_uv([0., 0., 1., 1.])?;
        Ok(image)
    }

    pub fn map_uv(self, relative: [f32; 4]) -> Result<[f32; 4], LayoutError> {
        if self
            .saved_uv
            .iter()
            .chain(&relative)
            .any(|v| !v.is_finite())
        {
            return Err(LayoutError::NonfiniteRectangle);
        }
        let width = self.saved_uv[2] - self.saved_uv[0];
        let height = self.saved_uv[3] - self.saved_uv[1];
        let result = [
            width * relative[0] + self.saved_uv[0],
            height * relative[1] + self.saved_uv[1],
            width * relative[2] + self.saved_uv[0],
            height * relative[3] + self.saved_uv[1],
        ];
        if result.iter().any(|v| !v.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippedImage {
    pub rectangle: [f32; 4],
    /// Relative image coordinates, before the image's saved atlas UV mapping.
    pub relative_uv: [f32; 4],
}

/// Native 95C1F0/634520. Fully contained draws retain the fast path even when
/// subpixel or inverted; partial draws require at least one unit in each source
/// span. A disjoint/touching intersection emits no draw. No global clip policy
/// or atlas resource is manufactured by this local operation.
pub fn clip_image(draw: [f32; 4], clip: [f32; 4]) -> Result<Option<ClippedImage>, LayoutError> {
    if draw.iter().chain(&clip).any(|v| !v.is_finite()) {
        return Err(LayoutError::NonfiniteRectangle);
    }
    if clip[0] <= draw[0] && draw[2] <= clip[2] && clip[1] <= draw[1] && draw[3] <= clip[3] {
        return Ok(Some(ClippedImage {
            rectangle: draw,
            relative_uv: [0., 0., 1., 1.],
        }));
    }
    let width = draw[2] - draw[0];
    let height = draw[3] - draw[1];
    if !width.is_finite() || !height.is_finite() {
        return Err(LayoutError::NonfiniteRectangle);
    }
    if width < 1. || height < 1. {
        return Ok(None);
    }
    if !(clip[0] < draw[2] && draw[0] < clip[2] && clip[1] < draw[3] && draw[1] < clip[3]) {
        return Ok(None);
    }
    // Explicit tie choices retain native signed-zero behavior.
    let rectangle = [
        if draw[0] <= clip[0] { clip[0] } else { draw[0] },
        if draw[1] <= clip[1] { clip[1] } else { draw[1] },
        if clip[2] <= draw[2] { clip[2] } else { draw[2] },
        if clip[3] <= draw[3] { clip[3] } else { draw[3] },
    ];
    let reciprocal_x = 1.0f32 / width;
    let reciprocal_y = 1.0f32 / height;
    Ok(Some(ClippedImage {
        rectangle,
        relative_uv: [
            (rectangle[0] - draw[0]) * reciprocal_x,
            (rectangle[1] - draw[1]) * reciprocal_y,
            (rectangle[2] - draw[0]) * reciprocal_x,
            (rectangle[3] - draw[1]) * reciprocal_y,
        ],
    }))
}

/// Native 97E8A0's separate post-layout clamp. It does not shift the other
/// edge or normalize against the parent origin, and is not an invertible step.
pub fn clamp_inverted_edges(mut rectangle: [f32; 4]) -> Result<[f32; 4], LayoutError> {
    if rectangle.iter().any(|v| !v.is_finite()) {
        return Err(LayoutError::NonfiniteRectangle);
    }
    if rectangle[0] > rectangle[2] {
        rectangle[2] = rectangle[0];
    }
    if rectangle[1] > rectangle[3] {
        rectangle[3] = rectangle[1];
    }
    Ok(rectangle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_regions_keep_outside_uvs_and_reject_malformed_typed_rectangles() {
        use crate::spui::SpuiProperty;
        let scalar = |id, type_code, width, bytes| SpuiProperty {
            id,
            type_code,
            count: 1,
            value: SpuiValue::Scalars { width, bytes },
        };
        let mut source = SpuiElement {
            index: 9,
            proxy_id: IMAGE_CLASS,
            root: false,
            properties: vec![
                scalar(0x01be_0004, 9, 4, 0x8000_0004u32.to_le_bytes().to_vec()),
                scalar(0x01be_0001, 19, 2, 7u16.to_le_bytes().to_vec()),
                scalar(
                    0x01be_0003,
                    15,
                    8,
                    [-329i32, 32]
                        .into_iter()
                        .flat_map(i32::to_le_bytes)
                        .collect(),
                ),
                scalar(
                    0x01be_0002,
                    16,
                    16,
                    [-0.25f32, 0.125, 1.25, 0.875]
                        .into_iter()
                        .flat_map(f32::to_le_bytes)
                        .collect(),
                ),
            ],
        };
        let image = ImageRegion::from_element(&source).unwrap();
        assert_eq!(image.resource_reference, 7);
        assert_eq!(image.flags.0, 0x8000_0004);
        assert_eq!(image.intrinsic, [-329, 32]);
        assert_eq!(
            image.map_uv([-0.25, 0.125, 1.25, 0.875]).unwrap(),
            [-0.625, 0.21875, 1.625, 0.78125]
        );
        source.properties[3].value = SpuiValue::Scalars {
            width: 16,
            bytes: vec![0; 12],
        };
        assert_eq!(source.properties[3].rectangle(), None);
        assert!(ImageRegion::from_element(&source).is_err());
        assert!(image.map_uv([0., 0., f32::NAN, 1.]).is_err());
    }

    #[test]
    fn measured_anchor_rules_position_title_rectangles_and_keep_parent_origin_local() {
        let parent = [10., 20., 810., 620.];
        let logo = [5., 90., -1., 213.];
        let unmodified = SimpleLayout { flags: 5 }.apply(parent, logo).unwrap();
        assert_eq!(unmodified, logo);
        let placed = SimpleLayout { flags: 13 }
            .apply(parent, unmodified)
            .unwrap();
        assert_eq!(placed, [5., 90., 799., 213.]);
        assert_eq!(
            SimpleLayout { flags: 13 }.unapply(parent, placed).unwrap(),
            logo
        );
        let caption = [6., -45., -3., -13.];
        assert_eq!(
            SimpleLayout { flags: 14 }.apply(parent, caption).unwrap(),
            [6., 555., 797., 587.]
        );
        assert_eq!(
            SimpleLayout { flags: 0xff }
                .apply(parent, [1., 2., 3., 4.])
                .unwrap(),
            [1., 2., 803., 604.]
        );
        assert_eq!(
            clamp_inverted_edges([5., 90., -1., -4.]).unwrap(),
            [5., 90., 5., 90.]
        );
        assert!(SimpleLayout { flags: 0 }
            .apply([f32::NAN; 4], logo)
            .is_err());
        assert!(clamp_inverted_edges([0., 0., f32::INFINITY, 1.]).is_err());
        // The first source layout clamps the negative authored right edge.
        // Deferring that clamp or reversing the two procedures gives 799,
        // which differs from the native event's actual 805 edge.
        assert_eq!(
            simple_window_layout(
                parent,
                logo,
                &[SimpleLayout { flags: 5 }, SimpleLayout { flags: 13 }]
            )
            .unwrap(),
            [5., 90., 805., 213.]
        );
        assert_eq!(
            simple_window_layout(parent, [0.9, -0.1, 12.9, 20.1], &[]).unwrap(),
            [0., -1., 12., 20.]
        );
    }

    #[test]
    fn native_image_size_flags_preserve_intrinsics_and_align_without_changing_other_axis() {
        let source = ImageSizing {
            flags: 3,
            ..Default::default()
        };
        assert_eq!(
            source.rectangle([5., 90., 799., 213.], [420, 166]).unwrap(),
            [192., 68.5, 612., 234.5]
        );
        assert_eq!(
            source.rectangle([6., 555., 797., 587.], [329, 32]).unwrap(),
            [237., 555., 566., 587.]
        );
        let default = ImageSizing::default();
        assert_eq!(
            default
                .rectangle([5., 90., 799., 213.], [420, 166])
                .unwrap(),
            [5., 90., 799., 213.]
        );
        let right = ImageSizing {
            flags: 1,
            scale: 0.5,
            horizontal_alignment: 2,
            ..default
        };
        assert_eq!(
            right.rectangle([5., 90., 799., 213.], [420, 166]).unwrap(),
            [589., 90., 799., 213.]
        );
        let native_other = ImageSizing {
            flags: 1,
            horizontal_alignment: 99,
            ..default
        };
        assert_eq!(
            native_other
                .rectangle([5., 90., 799., 213.], [420, 166])
                .unwrap(),
            [5., 90., 425., 213.]
        );
        assert!(source
            .rectangle([f32::INFINITY, 0., 1., 1.], [1, 1])
            .is_err());
    }

    #[test]
    fn native_image_clip_keeps_contained_subpixels_and_maps_partial_uvs() {
        let contained = clip_image([0., 0., 0.5, 0.5], [0., 0., 10., 10.])
            .unwrap()
            .unwrap();
        assert_eq!(contained.relative_uv, [0., 0., 1., 1.]);
        assert!(clip_image([-0.2, 0., 0.4, 0.5], [0., 0., 10., 10.])
            .unwrap()
            .is_none());
        let cropped = clip_image([0., 0., 4., 4.], [1., 1., 3., 3.])
            .unwrap()
            .unwrap();
        assert_eq!(cropped.rectangle, [1., 1., 3., 3.]);
        assert_eq!(cropped.relative_uv, [0.25, 0.25, 0.75, 0.75]);
        assert!(clip_image([0., 0., 4., 4.], [4., 0., 5., 4.])
            .unwrap()
            .is_none());
        assert!(clip_image([0., 0., 4., 4.], [5., 0., 6., 4.])
            .unwrap()
            .is_none());
        assert_eq!(
            clip_image([2., 2., 1., 1.], [0., 0., 4., 4.])
                .unwrap()
                .unwrap()
                .rectangle,
            [2., 2., 1., 1.]
        );
        assert!(clip_image([0., 0., f32::NAN, 4.], [0., 0., 1., 1.]).is_err());
    }
}
