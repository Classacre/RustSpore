//! Declarative source fragment selection, recovered from native reader/consumer
//! evidence and independently checked against installed builder records.
//!
//! Callers must supply actual shader data, current vertex-declaration bits and available
//! samplers. No game-specific defaults are manufactured here. This module covers
//! the eight condition codes observed in all installed header-7 builders.
use crate::shader_pack::FragmentSelector;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionError {
    UnsupportedCondition(u8),
    MissingRequiredData(u16),
    ShortData {
        id: u16,
        required: usize,
        actual: usize,
    },
    TooManyFragments,
}
impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source shader selection: {self:?}")
    }
}
impl std::error::Error for SelectionError {}

/// Bindings for one draw. Presence of a zero-byte data item is distinct from an
/// absent item; only conditions that inspect its contents need a payload.
pub struct SelectionContext<'a> {
    /// Native current vertex declaration +0x10, not a guessed quality mask.
    pub declaration_features: u32,
    pub shader_data: &'a BTreeMap<u16, Vec<u8>>,
    /// Units for which both native sampler pointer and availability flag are set.
    pub available_samplers: &'a BTreeSet<u16>,
}
impl SelectionContext<'_> {
    fn bytes(&self, id: u16, count: usize) -> Result<Option<&[u8]>, SelectionError> {
        match self.shader_data.get(&id) {
            None => Ok(None),
            Some(data) if data.len() < count => Err(SelectionError::ShortData {
                id,
                required: count,
                actual: data.len(),
            }),
            Some(data) => Ok(Some(data)),
        }
    }
    fn first(&self, id: u16) -> Result<Option<u8>, SelectionError> {
        Ok(self.bytes(id, 1)?.map(|b| b[0]))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// Ordered bytes emitted by the selector consumer. Zero can occur after
    /// wrapping addition and retains its native recipe-terminator meaning.
    pub fragment_ids: Vec<u8>,
    pub accepted_selectors: Vec<usize>,
    pub accumulated_flags: u32,
}

pub fn select(
    selectors: &[FragmentSelector],
    context: &SelectionContext<'_>,
) -> Result<Selection, SelectionError> {
    let mut result = Selection {
        fragment_ids: vec![],
        accepted_selectors: vec![],
        accumulated_flags: 0,
    };
    for (index, selector) in selectors.iter().enumerate() {
        let [declaration_mask, required_flags, forbidden_flags, added_flags] = selector.words();
        if context.declaration_features & declaration_mask != declaration_mask
            || result.accumulated_flags & required_flags != required_flags
            || result.accumulated_flags & forbidden_flags != 0
        {
            continue;
        }
        let threshold = selector.trailing_byte() as i8;
        if threshold >= 0 {
            // Native shaderdata237 is a pointer to a signed byte. An absent
            // pointer accepts only threshold0, rather than a guessed quality.
            let pass = match context.first(0x237)? {
                Some(value) => value as i8 <= threshold,
                None => threshold == 0,
            };
            if !pass {
                continue;
            }
        }
        let [a, b, _] = selector.short_words();
        let present = |id| context.shader_data.contains_key(&id);
        let offset = match selector.condition_code() {
            0 => Some(0),
            1 => present(a).then_some(0),
            4 => context.first(a)?,
            5 => (present(a) && present(b)).then_some(0),
            7 => context
                .first(a)?
                .filter(|value| u16::from(*value) == b)
                .map(|_| 0),
            9 => {
                if present(b) {
                    context.first(a)?
                } else {
                    None
                }
            }
            10 => context.available_samplers.contains(&a).then_some(0),
            12 => {
                let bytes = context
                    .bytes(0x24E, 2)?
                    .ok_or(SelectionError::MissingRequiredData(0x24E))?;
                let value = u16::from_le_bytes(bytes[..2].try_into().unwrap());
                (value & a == b).then_some(0)
            }
            other => return Err(SelectionError::UnsupportedCondition(other)),
        };
        if let Some(offset) = offset {
            result.accumulated_flags |= added_flags;
            result.accepted_selectors.push(index);
            let base = selector.fragment_id();
            if base != 0 {
                if result.fragment_ids.len() == 32 {
                    return Err(SelectionError::TooManyFragments);
                }
                result.fragment_ids.push(base.wrapping_add(offset));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selector(
        code: u8,
        args: [u16; 3],
        words: [u32; 4],
        threshold: i8,
        base: u8,
    ) -> FragmentSelector {
        let mut raw = [0; 25];
        raw[0] = code;
        for (i, value) in args.into_iter().enumerate() {
            raw[1 + i * 2..3 + i * 2].copy_from_slice(&value.to_be_bytes());
        }
        for (i, value) in words.into_iter().enumerate() {
            raw[7 + i * 4..11 + i * 4].copy_from_slice(&value.to_be_bytes());
        }
        raw[23] = threshold as u8;
        raw[24] = base;
        FragmentSelector { raw }
    }
    #[test]
    fn cell_material_byte_selects_offset_and_equality_without_reading_padding() {
        let samplers = BTreeSet::new();
        // Native condition4 reads one byte; independently vary all three
        // following bytes so an accidental u32 comparison cannot pass.
        for padding in [[0, 0, 0], [0x5C, 0x44, 0x26], [255, 255, 255]] {
            let data = BTreeMap::from([(0x210, vec![4, padding[0], padding[1], padding[2]])]);
            let context = SelectionContext {
                declaration_features: 0,
                shader_data: &data,
                available_samplers: &samplers,
            };
            let chosen = select(
                &[
                    selector(4, [0x210, 0, 0], [0; 4], -1, 13),
                    selector(7, [0x210, 4, 0], [0; 4], -1, 27),
                    selector(7, [0x210, 5, 0], [0; 4], -1, 28),
                ],
                &context,
            )
            .unwrap();
            assert_eq!(chosen.fragment_ids, [17, 27]);
            assert_eq!(chosen.accepted_selectors, [0, 1]);
        }
    }
    #[test]
    fn declaration_masks_and_ordered_groups_require_all_bits_and_reject_any_forbidden_bit() {
        let data = BTreeMap::new();
        let samplers = BTreeSet::new();
        let context = SelectionContext {
            declaration_features: 0b0101,
            shader_data: &data,
            available_samplers: &samplers,
        };
        let selectors = [
            selector(0, [0; 3], [0b0101, 0, 0, 1], -1, 0), // flags-only acceptance
            selector(0, [0; 3], [0, 1, 2, 2], -1, 4),
            selector(0, [0; 3], [0, 3, 0, 0], -1, 5),
            selector(0, [0; 3], [0, 0, 1, 0], -1, 6),
            selector(0, [0; 3], [0b0011, 0, 0, 0], -1, 7),
        ];
        let result = select(&selectors, &context).unwrap();
        assert_eq!(result.fragment_ids, [4, 5]);
        assert_eq!(result.accepted_selectors, [0, 1, 2]);
        assert_eq!(result.accumulated_flags, 3);
    }
    #[test]
    fn thresholds_use_signed_bytes_and_absent_pointer_has_native_zero_exception() {
        let samplers = BTreeSet::new();
        let selectors = [
            selector(0, [0; 3], [0; 4], 0, 1),
            selector(0, [0; 3], [0; 4], 1, 2),
            selector(0, [0; 3], [0; 4], -1, 3),
        ];
        for (quality, expected) in [
            (None, vec![1, 3]),
            (Some(0), vec![1, 2, 3]),
            (Some(1), vec![2, 3]),
            (Some(2), vec![3]),
            (Some(255), vec![1, 2, 3]),
        ] {
            let data = quality.map_or_else(BTreeMap::new, |v| BTreeMap::from([(0x237, vec![v])]));
            assert_eq!(
                select(
                    &selectors,
                    &SelectionContext {
                        declaration_features: 0,
                        shader_data: &data,
                        available_samplers: &samplers
                    }
                )
                .unwrap()
                .fragment_ids,
                expected
            );
        }
    }
    #[test]
    fn presence_pairs_sampler_gate_masked_word_and_short_data_are_explicit() {
        let data = BTreeMap::from([(8, vec![]), (9, vec![2]), (0x24E, vec![5, 0])]);
        let samplers = BTreeSet::from([2]);
        let context = SelectionContext {
            declaration_features: 0,
            shader_data: &data,
            available_samplers: &samplers,
        };
        let selectors = [
            selector(1, [8, 0, 0], [0; 4], -1, 1),
            selector(5, [8, 9, 0], [0; 4], -1, 2),
            selector(9, [9, 8, 0], [0; 4], -1, 3),
            selector(10, [2, 0, 0], [0; 4], -1, 6),
            selector(10, [3, 0, 0], [0; 4], -1, 7),
            selector(12, [7, 5, 0], [0; 4], -1, 8),
        ];
        assert_eq!(
            select(&selectors, &context).unwrap().fragment_ids,
            [1, 2, 5, 6, 8]
        );
        assert!(matches!(
            select(&[selector(4, [8, 0, 0], [0; 4], -1, 1)], &context),
            Err(SelectionError::ShortData { id: 8, .. })
        ));
        let empty = BTreeMap::new();
        let empty = SelectionContext {
            shader_data: &empty,
            ..context
        };
        assert_eq!(
            select(&[selectors[5].clone()], &empty),
            Err(SelectionError::MissingRequiredData(0x24E))
        );
    }
    #[test]
    fn unsupported_conditions_fragment_capacity_and_wrapped_terminator_are_visible() {
        let data = BTreeMap::from([(1, vec![255])]);
        let samplers = BTreeSet::new();
        let context = SelectionContext {
            declaration_features: 0,
            shader_data: &data,
            available_samplers: &samplers,
        };
        assert_eq!(
            select(&[selector(2, [0; 3], [0; 4], -1, 1)], &context),
            Err(SelectionError::UnsupportedCondition(2))
        );
        assert_eq!(
            select(&vec![selector(0, [0; 3], [0; 4], -1, 1); 33], &context),
            Err(SelectionError::TooManyFragments)
        );
        assert_eq!(
            select(&[selector(4, [1, 0, 0], [0; 4], -1, 1)], &context)
                .unwrap()
                .fragment_ids,
            [0]
        );
    }
}
