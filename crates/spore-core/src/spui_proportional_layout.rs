//! Bounded ProportionalLayout edge helpers from the pinned source executable.
//! This does not run WinProc dispatch, clamp edges, floor, or compose transforms.
//! By-value arguments exclude native pointer aliasing. Float results assume the
//! host's ordinary IEEE environment; native MXCSR rounding, FTZ and DAZ are not
//! established. Rejecting nonfinite values is model admission, not source behavior.
use crate::spui::{SpuiElement, SpuiValue};
use crate::spui_layout::LayoutError;

pub const PROPORTIONAL_LAYOUT_CLASS: u32 = 0xaf3d_f411;
pub const PROPORTIONAL_LAYOUT_EDGES: u32 = 0x0f3d_0000;

/// Per-edge coefficients in left, top, right, bottom order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProportionalLayout {
    pub edges: [f32; 4],
}

impl ProportionalLayout {
    /// Decode the explicit source property; missing properties are not defaulted.
    pub fn from_element(element: &SpuiElement) -> Result<Self, LayoutError> {
        if element.proxy_id != PROPORTIONAL_LAYOUT_CLASS {
            return Err(LayoutError::UnsupportedClass(element.proxy_id));
        }
        let p = element.property(PROPORTIONAL_LAYOUT_EDGES)
            .ok_or(LayoutError::InvalidProperty(PROPORTIONAL_LAYOUT_EDGES))?;
        let bytes = match &p.value {
            SpuiValue::Scalars { width: 4, bytes }
                if p.type_code == 11 && p.count == 4 && bytes.len() == 16 => bytes,
            _ => return Err(LayoutError::InvalidProperty(PROPORTIONAL_LAYOUT_EDGES)),
        };
        let edges = std::array::from_fn(|i| {
            f32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap())
        });
        if edges.iter().any(|v| !v.is_finite()) {
            return Err(LayoutError::InvalidProperty(PROPORTIONAL_LAYOUT_EDGES));
        }
        Ok(Self { edges })
    }

    /// Native callback 97EA70; parent origin is used only to calculate spans.
    pub fn apply(self, parent: [f32; 4], child: [f32; 4]) -> Result<[f32; 4], LayoutError> {
        self.adjust(parent, child, false)
    }

    /// Native callback 97EAE0; rounding means this is not an exact inverse.
    pub fn unapply(self, parent: [f32; 4], child: [f32; 4]) -> Result<[f32; 4], LayoutError> {
        self.adjust(parent, child, true)
    }

    fn adjust(self, parent: [f32; 4], mut child: [f32; 4], subtract: bool) -> Result<[f32; 4], LayoutError> {
        if parent.iter().chain(&child).chain(&self.edges).any(|x| !x.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        // Source visits left, right, top, bottom; each operation is rounded
        // separately as SUBSS, MULSS, then ADDSS or SUBSS.
        for i in [0, 2, 1, 3] {
            let span = if i == 0 || i == 2 { parent[2] - parent[0] } else { parent[3] - parent[1] };
            let delta = span * self.edges[i];
            child[i] = if subtract { child[i] - delta } else { delta + child[i] };
        }
        if child.iter().any(|x| !x.is_finite()) {
            return Err(LayoutError::NonfiniteRectangle);
        }
        Ok(child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spui::SpuiProperty;

    fn element(values: [f32; 4]) -> SpuiElement {
        SpuiElement { index: 0, proxy_id: PROPORTIONAL_LAYOUT_CLASS, root: false,
            properties: vec![SpuiProperty { id: PROPORTIONAL_LAYOUT_EDGES, type_code: 11, count: 4,
                value: SpuiValue::Scalars { width: 4, bytes: values.into_iter().flat_map(f32::to_le_bytes).collect() } }] }
    }

    #[test]
    fn decodes_and_adjusts_each_edge_by_its_parent_span() {
        let layout = ProportionalLayout::from_element(&element([0.5, -0.25, 1., 2.])).unwrap();
        let parent = [10., 20., 30., 60.];
        let child = [1., 2., 3., 4.];
        assert_eq!(layout.apply(parent, child).unwrap(), [11., -8., 23., 84.]);
        assert_eq!(layout.unapply(parent, child).unwrap(), [-9., 12., -17., -76.]);
    }

    #[test]
    fn keeps_multiply_and_add_separate_and_does_not_clamp() {
        let layout = ProportionalLayout { edges: [1.0000001, 0., 0., 0.] };
        let actual = layout.apply([0., 0., 1.0000001, 1.], [-1.0000002, 5., -3., 1.]).unwrap();
        assert_eq!(actual[0].to_bits(), 0.0f32.to_bits());
        assert!(1.0000001f32.mul_add(1.0000001, -1.0000002) > 0.);
        assert_eq!(&actual[1..], &[5., -3., 1.]);
    }

    #[test]
    fn rejects_bad_shapes_nonfinite_inputs_and_overflow() {
        let mut e = element([0.; 4]);
        e.properties[0].count = 3;
        assert!(ProportionalLayout::from_element(&e).is_err());
        assert!(ProportionalLayout::from_element(&element([0., f32::NAN, 0., 0.])).is_err());
        let layout = ProportionalLayout { edges: [1.; 4] };
        assert!(layout.apply([0., 0., f32::INFINITY, 1.], [0.; 4]).is_err());
        assert!(layout.apply([-f32::MAX, 0., f32::MAX, 1.], [0.; 4]).is_err());
    }
}
