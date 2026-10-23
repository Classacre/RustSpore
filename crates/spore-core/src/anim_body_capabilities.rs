//! Native32-slot body capabilities before root selection and runtime ordering.
//!
//! `9FC210` appends dictionary aggregate members; `9FC2F0` combines duplicate
//! numbers and fills holes from the tail; `9B2E20` builds separate presence
//! and nonzero-number masks. Source row order and all stale number bytes are
//! preserved. This does not construct or finalize the generated hierarchy.

use crate::{creature_data::CreatureData, part_capabilities::PartCapabilities};
use std::{collections::BTreeMap, fmt};

pub const CAPABILITY_SLOTS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySlots {
    pub ids: [u32; CAPABILITY_SLOTS],
    pub numbers: [u8; CAPABILITY_SLOTS],
}
impl Default for CapabilitySlots {
    fn default() -> Self {
        Self {
            ids: [0; CAPABILITY_SLOTS],
            numbers: [0; CAPABILITY_SLOTS],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyCapabilitiesError {
    InvalidSourcePart(usize),
    SourceCapacityExceeded {
        part: usize,
        count: usize,
    },
    MissingInitialZero,
    DuplicateAggregate(u32),
    ExpansionCapacityExceeded {
        source_slot: usize,
        capability: u32,
        append_at: usize,
        members: usize,
    },
}
impl fmt::Display for BodyCapabilitiesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native body capability preparation: {self:?}")
    }
}
impl std::error::Error for BodyCapabilitiesError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedBodyCapabilities {
    slots: CapabilitySlots,
    presence_mask: u32,
    nonzero_mask: u32,
}
impl PreparedBodyCapabilities {
    /// All32 slots, including number bytes left behind when an ID is cleared.
    pub fn slots(&self) -> &CapabilitySlots {
        &self.slots
    }
    pub fn presence_mask(&self) -> u32 {
        self.presence_mask
    }
    pub fn nonzero_mask(&self) -> u32 {
        self.nonzero_mask
    }
    pub fn len(&self) -> usize {
        self.slots
            .ids
            .iter()
            .position(|id| *id == 0)
            .unwrap_or(CAPABILITY_SLOTS)
    }
    pub fn is_empty(&self) -> bool {
        self.slots.ids[0] == 0
    }
}

// 9B2DA0: these eight literal IDs are the native mask categories.
fn mask(id: u32) -> u32 {
    match id {
        0x746f_6f72 => 1,   // root
        0x626d_696c => 2,   // limb
        0x746f_6f66 => 4,   // foot
        0x6e69_7073 => 8,   // spin
        0x7466_7370 => 16,  // psft
        0x7475_6f6d => 32,  // mout
        0x7073_7267 => 64,  // grsp
        0x7274_736e => 128, // nstr
        _ => 0,
    }
}

/// Prepare the exact native slot array with an explicit aggregate dictionary.
///
/// Expansion starts appending at the first initial zero ID, then scans the
/// changing array once, including appended IDs, until zero or32 processed
/// slots. It does not repeatedly merge a mathematical set. Deduplication keeps
/// the first ID and maximum number; compaction pulls the last occupied slot
/// into each hole. Clearing an ID never clears its number byte.
///
/// The native full-initial-array path loads an uninitialized append position
/// at9FC230. This bounded API rejects that input, and rejects any expansion
/// that would escape32 slots. It also rejects ambiguous duplicate aggregate
/// keys. These are explicit interface bounds, not native error diagnostics.
pub fn prepare_capabilities(
    mut slots: CapabilitySlots,
    dictionary: &PartCapabilities,
) -> Result<PreparedBodyCapabilities, BodyCapabilitiesError> {
    let mut append_at = slots
        .ids
        .iter()
        .position(|id| *id == 0)
        .ok_or(BodyCapabilitiesError::MissingInitialZero)?;
    let mut aggregates = BTreeMap::new();
    for aggregate in &dictionary.aggregates {
        if aggregates
            .insert(aggregate.capability, &aggregate.members)
            .is_some()
        {
            return Err(BodyCapabilitiesError::DuplicateAggregate(
                aggregate.capability,
            ));
        }
    }
    for source_slot in 0..CAPABILITY_SLOTS {
        let capability = slots.ids[source_slot];
        if capability == 0 {
            break;
        }
        let Some(members) = aggregates.get(&capability) else {
            continue;
        };
        if members.len() > CAPABILITY_SLOTS - append_at {
            return Err(BodyCapabilitiesError::ExpansionCapacityExceeded {
                source_slot,
                capability,
                append_at,
                members: members.len(),
            });
        }
        for id in members.iter().copied() {
            slots.ids[append_at] = id;
            slots.numbers[append_at] = slots.numbers[source_slot];
            append_at += 1;
        }
    }
    for i in 0..CAPABILITY_SLOTS {
        if slots.ids[i] == 0 {
            continue;
        }
        for j in i + 1..CAPABILITY_SLOTS {
            if slots.ids[j] == slots.ids[i] {
                slots.ids[j] = 0;
                slots.numbers[i] = slots.numbers[i].max(slots.numbers[j]);
            }
        }
    }
    let mut tail = CAPABILITY_SLOTS - 1;
    let mut i = 0;
    loop {
        if slots.ids[i] == 0 && i < tail {
            while i < tail && slots.ids[tail] == 0 {
                tail -= 1;
            }
            if slots.ids[tail] != 0 {
                slots.ids[i] = slots.ids[tail];
                slots.numbers[i] = slots.numbers[tail];
                slots.ids[tail] = 0;
                tail -= 1;
            }
        }
        i += 1;
        if i >= tail {
            break;
        }
    }
    let (mut presence_mask, mut nonzero_mask) = (0, 0);
    for (id, number) in slots.ids.into_iter().zip(slots.numbers) {
        if id == 0 {
            break;
        }
        presence_mask |= mask(id);
        if number != 0 {
            nonzero_mask |= mask(id);
        }
    }
    Ok(PreparedBodyCapabilities {
        slots,
        presence_mask,
        nonzero_mask,
    })
}

/// Copy one checked CreatureData row's original capability records into the
/// initially zeroed native array, then run the exact preparation operation.
pub fn prepare_source_capabilities(
    creature: &CreatureData,
    part: usize,
    dictionary: &PartCapabilities,
) -> Result<PreparedBodyCapabilities, BodyCapabilitiesError> {
    let view = creature
        .part_view(part)
        .ok_or(BodyCapabilitiesError::InvalidSourcePart(part))?;
    if view.capabilities.len() > CAPABILITY_SLOTS {
        return Err(BodyCapabilitiesError::SourceCapacityExceeded {
            part,
            count: view.capabilities.len(),
        });
    }
    let mut slots = CapabilitySlots::default();
    for (i, capability) in view.capabilities.iter().enumerate() {
        slots.ids[i] = capability.packed_id();
        slots.numbers[i] = capability.number;
    }
    prepare_capabilities(slots, dictionary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::part_capabilities::CapabilityAggregate;
    const SPIN: u32 = 0x6e69_7073;
    const LIMB: u32 = 0x626d_696c;
    const FOOT: u32 = 0x746f_6f66;

    fn dictionary(aggregates: Vec<CapabilityAggregate>) -> PartCapabilities {
        PartCapabilities {
            version: 4,
            priority_bits: Some(0x4000_0000),
            names: vec![],
            remaps: vec![],
            aggregates,
            deforms: vec![],
        }
    }
    fn initial(values: &[(u32, u8)]) -> CapabilitySlots {
        let mut slots = CapabilitySlots::default();
        for (i, (id, number)) in values.iter().copied().enumerate() {
            slots.ids[i] = id;
            slots.numbers[i] = number;
        }
        slots
    }

    #[test]
    fn duplicates_take_maximum_and_holes_pull_from_tail_without_clearing_numbers() {
        let dictionary = dictionary(vec![
            CapabilityAggregate {
                capability: 10,
                members: vec![SPIN, LIMB],
            },
            CapabilityAggregate {
                capability: 20,
                members: vec![FOOT],
            },
        ]);
        let result =
            prepare_capabilities(initial(&[(10, 1), (20, 2), (10, 7)]), &dictionary).unwrap();
        assert_eq!(
            &result.slots().ids[..8],
            &[10, 20, FOOT, SPIN, LIMB, 0, 0, 0]
        );
        assert_eq!(&result.slots().numbers[..8], &[7, 2, 2, 7, 7, 2, 7, 7]);
        assert_eq!(
            (result.len(), result.presence_mask(), result.nonzero_mask()),
            (5, 14, 14)
        );
    }

    #[test]
    fn zero_member_stops_expansion_before_tail_compaction_without_restarting() {
        let dictionary = dictionary(vec![
            CapabilityAggregate {
                capability: 10,
                members: vec![0],
            },
            CapabilityAggregate {
                capability: 20,
                members: vec![SPIN],
            },
        ]);
        let result =
            prepare_capabilities(initial(&[(10, 1), (0, 9), (20, 2), (30, 3)]), &dictionary)
                .unwrap();
        assert_eq!(&result.slots().ids[..5], &[10, 30, 20, 0, 0]);
        assert_eq!(&result.slots().numbers[..5], &[1, 3, 2, 3, 0]);
        assert_eq!(result.presence_mask(), 0);
    }

    #[test]
    fn complete_expanded_array_is_supported_but_full_initial_and_cycles_are_bounded() {
        let dictionary = dictionary(vec![CapabilityAggregate {
            capability: 10,
            members: (100..131).collect(),
        }]);
        let result = prepare_capabilities(initial(&[(10, 5)]), &dictionary).unwrap();
        assert_eq!(result.len(), 32);
        assert_eq!(result.slots().ids[31], 130);
        assert_eq!(result.slots().numbers, [5; 32]);
        assert_eq!(
            prepare_capabilities(
                CapabilitySlots {
                    ids: [10; 32],
                    numbers: [1; 32]
                },
                &dictionary
            ),
            Err(BodyCapabilitiesError::MissingInitialZero)
        );
        let cyclic = self::dictionary(vec![CapabilityAggregate {
            capability: 10,
            members: vec![10],
        }]);
        assert!(matches!(
            prepare_capabilities(initial(&[(10, 1)]), &cyclic),
            Err(BodyCapabilitiesError::ExpansionCapacityExceeded {
                source_slot: 31,
                append_at: 32,
                ..
            })
        ));
    }

    #[test]
    fn presence_and_nonzero_masks_are_distinct_and_dictionary_ambiguity_is_explicit() {
        let result =
            prepare_capabilities(initial(&[(SPIN, 0), (FOOT, 2)]), &dictionary(vec![])).unwrap();
        assert_eq!((result.presence_mask(), result.nonzero_mask()), (12, 4));
        let duplicate = CapabilityAggregate {
            capability: 10,
            members: vec![],
        };
        assert_eq!(
            prepare_capabilities(
                initial(&[]),
                &dictionary(vec![duplicate.clone(), duplicate])
            ),
            Err(BodyCapabilitiesError::DuplicateAggregate(10))
        );
    }
}
