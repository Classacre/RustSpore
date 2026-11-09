//! Bounded StdDrawable pointer selection at source 987A00.
//! Caller-supplied state and slot occupancy do not establish runtime UI state.
use crate::spui::{SpuiElement, SpuiValue};
use crate::spui_layout::LayoutError;

pub const STD_DRAWABLE_CLASS: u32 = 0xb03c_196f;
pub const STD_DRAWABLE_SLOTS: u32 = 0x0f3a_c75e;

/// Serialized reference words; mapping them to native pointers requires an
/// independently supplied resolution outcome for every slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdDrawableSlots(pub [u16; 8]);

impl StdDrawableSlots {
    pub fn from_element(element: &SpuiElement) -> Result<Self, LayoutError> {
        if element.proxy_id != STD_DRAWABLE_CLASS {
            return Err(LayoutError::UnsupportedClass(element.proxy_id));
        }
        let property = element.property(STD_DRAWABLE_SLOTS).ok_or(LayoutError::InvalidProperty(STD_DRAWABLE_SLOTS))?;
        let bytes = match &property.value {
            SpuiValue::Scalars { width: 2, bytes }
                if property.type_code == 19 && property.count == 8 && bytes.len() == 16 => bytes,
            _ => return Err(LayoutError::InvalidProperty(STD_DRAWABLE_SLOTS)),
        };
        Ok(Self(std::array::from_fn(|i| u16::from_le_bytes([bytes[2*i], bytes[2*i+1]]))))
    }

    /// `occupied` must reflect resolved native-like pointers; no assumption is
    /// made about a serialized sentinel or whether a referenced object loaded.
    pub fn selected_reference(self, state: u32, occupied: [bool; 8]) -> Option<u16> {
        select_slot(state, occupied).map(|slot| self.0[slot])
    }
}

/// Return the selected non-null slot index, or `None` for the null fallback.
/// Slots correspond to source object offsets +10, +14, ... +2C. State bits
/// other than the low nibble are ignored by the observed selector.
pub fn select_slot(state: u32, occupied: [bool; 8]) -> Option<usize> {
    let modified = state & 8 != 0;
    let candidates: &[usize] = match (state & 7, modified) {
        (0, true) => &[4, 0],
        (1, true) => &[5, 1, 0],
        (2, true) => &[6, 4, 2, 0],
        (3, true) => &[7, 4, 3, 0],
        (1, false) => &[1, 0],
        (2, false) => &[2, 0],
        (3, false) => &[3, 0],
        _ => &[0],
    };
    candidates.iter().copied().find(|&index| occupied[index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spui::SpuiProperty;

    #[test]
    fn selects_modified_fallbacks_and_null() {
        assert_eq!(select_slot(10, [true, false, true, false, true, false, true, false]), Some(6));
        assert_eq!(select_slot(10, [true, false, true, false, true, false, false, false]), Some(4));
        assert_eq!(select_slot(10, [true, false, true, false, false, false, false, false]), Some(2));
        assert_eq!(select_slot(15, [false; 8]), None);
        assert_eq!(select_slot(4, [true; 8]), Some(0));
        assert_eq!(select_slot(0xffff_fff9, [true; 8]), Some(5));
    }

    #[test]
    fn preserves_serialized_words_and_requires_explicit_resolution() {
        let e = SpuiElement { index: 3, proxy_id: STD_DRAWABLE_CLASS, root: false,
            properties: vec![SpuiProperty { id: STD_DRAWABLE_SLOTS, type_code: 19, count: 8,
                value: SpuiValue::Scalars { width: 2, bytes: [132u16, 65535, 65535, 65535, 65535, 65535, 65535, 65535].into_iter().flat_map(u16::to_le_bytes).collect() } }] };
        let slots = StdDrawableSlots::from_element(&e).unwrap();
        assert_eq!(slots.selected_reference(0, [true, false, false, false, false, false, false, false]), Some(132));
        assert_eq!(slots.selected_reference(0, [false; 8]), None);
        let mut bad = e;
        bad.properties[0].count = 7;
        assert!(StdDrawableSlots::from_element(&bad).is_err());
    }
}
