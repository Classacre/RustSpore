//! Native ANIM component-slot preparation using an explicit PCTP dictionary.
//!
//! This implements the measured descriptor fixup in native 0x99D010 and layout
//! selection in 0x9FC3E0. It does not evaluate authored curves, finalize a body
//! hierarchy or turn component slots into mesh-bone indices. The parser's saved
//! descriptors stay unchanged. The initial native fallback is empty; callers
//! can also supply an explicit fallback without inventing one from a name.

use crate::anim::{Anim, AnimComponent};
use crate::anim_binding::SourceSelector;
use crate::part_capabilities::{DeformChannel, PartCapabilities};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorBindingError {
    DuplicateCapability(u32),
    TooManyDeformSlots {
        channel: Option<usize>,
        count: usize,
    },
    /// The native template persists across channels. A stale matching ID would
    /// make the native loader read beyond the current layout's parameter array.
    StaleDeformSlot {
        channel: usize,
        component: usize,
        slot: usize,
        layout_count: usize,
    },
}
impl fmt::Display for DescriptorBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ANIM descriptor binding: {self:?}")
    }
}
impl std::error::Error for DescriptorBindingError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutSource {
    Capability(u32),
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedChannel {
    pub selector: SourceSelector,
    pub layout_source: LayoutSource,
    /// Current PCTP layout order, independent of source component order.
    pub deforms: Vec<DeformChannel>,
    /// Source descriptors with only native flags and slot fields rewritten.
    /// An unmatched descriptor has slot FFFFFFFF and enable bit 0x10 cleared.
    pub components: Vec<AnimComponent>,
}
impl PreparedChannel {
    /// Return the exact PCTP entry assigned to a deformation component. This
    /// does not imply that its enable bit is set or that its curve is evaluated.
    pub fn deform_for_component(&self, component: usize) -> Option<&DeformChannel> {
        let descriptor = self.components.get(component)?;
        let ordinal = descriptor.slot.checked_sub(3)? as usize;
        self.deforms.get(ordinal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedComponents {
    pub channels: Vec<PreparedChannel>,
}

/// Select the capability used by the native deformation-layout lookup. Root
/// mode uses the literal fourcc; unsupported lookup modes go to the fallback.
/// Spatial/hierarchy bits only select the alternate in the three measured cases.
pub fn deformation_capability(selector: SourceSelector) -> Option<u32> {
    let mut capability = match selector.flags & 0x100003 {
        0 => selector.capability,
        1 => u32::from_le_bytes(*b"root"),
        _ => return None,
    };
    if matches!(selector.flags & 0x801C00, 0x1800 | 0x1C00 | 0x800400) {
        capability = selector.alternate;
    }
    (capability != 0).then_some(capability)
}

/// Use the empty fallback allocated by the original animation subsystem's
/// initializer. Native 0xA0F204 passes object 0x166D038 to 0x9A9730, which stores
/// a zero-length descriptor vector at +0x24 (global 0x166D05C). This establishes
/// initial state; callers with a later replacement can use `prepare` directly.
pub fn prepare_native_initial(
    animation: &Anim,
    dictionary: &PartCapabilities,
) -> Result<PreparedComponents, DescriptorBindingError> {
    prepare(animation, dictionary, &[])
}

/// Prepare all channels in source order with the caller's explicit dictionary
/// and fallback. The native nine-slot template is initialized once per clip;
/// only IDs covered by each current layout replace its six deformation IDs.
/// Layouts larger than six and stale out-of-range parameter reads are rejected.
pub fn prepare(
    animation: &Anim,
    dictionary: &PartCapabilities,
    fallback: &[DeformChannel],
) -> Result<PreparedComponents, DescriptorBindingError> {
    if fallback.len() > 6 {
        return Err(DescriptorBindingError::TooManyDeformSlots {
            channel: None,
            count: fallback.len(),
        });
    }
    let mut layouts = BTreeMap::new();
    for layout in &dictionary.deforms {
        if layouts
            .insert(layout.capability, &layout.channels)
            .is_some()
        {
            return Err(DescriptorBindingError::DuplicateCapability(
                layout.capability,
            ));
        }
    }
    let mut ids = [0u32; 9];
    ids[0] = u32::from_le_bytes(*b"INFO");
    ids[1] = u32::from_le_bytes(*b"POS\0");
    ids[2] = u32::from_le_bytes(*b"ROT\0");
    let mut channels = Vec::with_capacity(animation.bones.len());
    for (channel_index, channel) in animation.bones.iter().enumerate() {
        let selector = SourceSelector::primary(channel);
        let (layout_source, layout) = match deformation_capability(selector)
            .and_then(|key| layouts.get(&key).map(|layout| (key, layout.as_slice())))
        {
            Some((key, layout)) => (LayoutSource::Capability(key), layout),
            None => (LayoutSource::Fallback, fallback),
        };
        if layout.len() > 6 {
            return Err(DescriptorBindingError::TooManyDeformSlots {
                channel: Some(channel_index),
                count: layout.len(),
            });
        }
        for (slot, deform) in layout.iter().enumerate() {
            ids[slot + 3] = deform.id;
        }
        let mut components = channel.components.clone();
        for (component_index, component) in components.iter_mut().enumerate() {
            let kind = component.flags & 0xF;
            let tag = u32::from_le_bytes(component.tag);
            let found = ids.iter().enumerate().position(|(slot, id)| {
                kind == slot.min(3) as u32 && *id != 0 && (*id == tag || tag == u32::MAX)
            });
            let Some(slot) = found else {
                component.slot = u32::MAX;
                component.flags &= !0x10;
                continue;
            };
            component.slot = slot as u32;
            if slot >= 3 {
                let deform =
                    layout
                        .get(slot - 3)
                        .ok_or(DescriptorBindingError::StaleDeformSlot {
                            channel: channel_index,
                            component: component_index,
                            slot,
                            layout_count: layout.len(),
                        })?;
                let flags = deform.raw_parameters[2];
                component.flags =
                    (component.flags & !0xC0) | ((flags & 1) << 7) | ((flags & 2) << 5);
            }
        }
        channels.push(PreparedChannel {
            selector,
            layout_source,
            deforms: layout.to_vec(),
            components,
        });
    }
    Ok(PreparedComponents { channels })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::AnimBone;
    use crate::part_capabilities::CapabilityDeform;

    fn selector(flags: u32, capability: u32, alternate: u32) -> SourceSelector {
        SourceSelector {
            flags,
            capability,
            alternate,
            reserved: 0xAABBCCDD,
        }
    }
    fn descriptor(tag: u32, flags: u32) -> AnimComponent {
        AnimComponent {
            flags,
            tag: tag.to_le_bytes(),
            offset: 0x1234,
            stride: 0x5678,
            slot: 0xDEADBEEF,
            reserved: [1, 2, 3],
        }
    }
    fn channel(selector: SourceSelector, components: Vec<AnimComponent>) -> AnimBone {
        let mut selector_words = [0u32; 19];
        selector_words[1..5].copy_from_slice(&[
            selector.flags,
            selector.capability,
            selector.alternate,
            selector.reserved,
        ]);
        AnimBone {
            name: "ignored channel label".into(),
            keyframe_count: 0,
            has_position: false,
            has_rotation: false,
            channel_flags: 0,
            selector_words,
            components,
            keyframe_stride: 0,
            keyframes: vec![],
            raw_keyframes: vec![],
        }
    }
    fn animation(bones: Vec<AnimBone>) -> Anim {
        Anim {
            path: String::new(),
            hash: 0,
            header_flags_280: 0,
            frame_duration: 1.0,
            duration: 0.0,
            last_frame: 0,
            bone_count: bones.len() as u32,
            header_field_304: 0,
            anim_name: String::new(),
            bones,
        }
    }
    fn dictionary(groups: Vec<(u32, Vec<(u32, u32)>)>) -> PartCapabilities {
        PartCapabilities {
            version: 4,
            priority_bits: None,
            names: vec![],
            remaps: vec![],
            aggregates: vec![],
            deforms: groups
                .into_iter()
                .map(|(capability, channels)| CapabilityDeform {
                    capability,
                    channels: channels
                        .into_iter()
                        .map(|(id, flags)| DeformChannel {
                            id,
                            raw_parameters: [0x3F000000, 0x3F800000, flags],
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn lookup_masks_preserve_native_root_alternate_and_fallback_order() {
        assert_eq!(deformation_capability(selector(0, 7, 8)), Some(7));
        assert_eq!(
            deformation_capability(selector(1, 7, 8)),
            Some(u32::from_le_bytes(*b"root"))
        );
        for flags in [0x1800, 0x1C00, 0x800400, 0x1801] {
            assert_eq!(deformation_capability(selector(flags, 7, 8)), Some(8));
        }
        for flags in [2, 3, 0x100000, 0x101800] {
            assert_eq!(deformation_capability(selector(flags, 7, 8)), None);
        }
        assert_eq!(deformation_capability(selector(0x800, 7, 8)), Some(7));
        assert_eq!(deformation_capability(selector(0, 0, 8)), None);
        assert_eq!(deformation_capability(selector(0x1800, 7, 0)), None);
    }

    #[test]
    fn slots_follow_layout_order_flags_are_replaced_without_enabling_and_source_is_unchanged() {
        let dictionary = dictionary(vec![(7, vec![(0xA, 1), (0xB, 2), (0xC, 3)])]);
        let original = vec![
            descriptor(0xB, 0x80000083),
            descriptor(0xA, 0x800000D3),
            descriptor(0xC, 0x80000013),
            descriptor(0xD, 0x800000D3),
        ];
        let anim = animation(vec![channel(selector(0, 7, 0), original.clone())]);
        let prepared = prepare(&anim, &dictionary, &[]).unwrap();
        let c = &prepared.channels[0];
        assert_eq!(c.layout_source, LayoutSource::Capability(7));
        assert_eq!(
            c.components.iter().map(|d| d.slot).collect::<Vec<_>>(),
            [4, 3, 5, u32::MAX]
        );
        assert_eq!(
            c.components.iter().map(|d| d.flags).collect::<Vec<_>>(),
            [0x80000043, 0x80000093, 0x800000D3, 0x800000C3]
        );
        assert_eq!(c.deform_for_component(0).unwrap().id, 0xB);
        assert!(c.deform_for_component(3).is_none());
        for descriptor in &c.components {
            assert_eq!(
                (descriptor.offset, descriptor.stride, descriptor.reserved),
                (0x1234, 0x5678, [1, 2, 3])
            );
        }
        assert_eq!(anim.bones[0].components, original);
    }

    #[test]
    fn tag_and_low_type_must_both_match_and_wildcard_selects_first_nonzero_slot() {
        let dictionary = dictionary(vec![(7, vec![(0, 0), (0xB, 0), (0xB, 3)])]);
        let anim = animation(vec![channel(
            selector(0, 7, 0),
            vec![
                descriptor(u32::from_le_bytes(*b"ROT\0"), 0x12),
                descriptor(u32::from_le_bytes(*b"ROT\0"), 0x11),
                descriptor(u32::MAX, 0x13),
                descriptor(u32::MAX, 0x10),
            ],
        )]);
        let prepared = prepare(&anim, &dictionary, &[]).unwrap();
        assert_eq!(
            prepared.channels[0]
                .components
                .iter()
                .map(|d| d.slot)
                .collect::<Vec<_>>(),
            [2, u32::MAX, 4, 0]
        );
        assert_eq!(
            prepared.channels[0].deform_for_component(2).unwrap().id,
            0xB
        );
    }

    #[test]
    fn fallback_is_explicit_and_stale_template_reads_are_rejected() {
        let dictionary = dictionary(vec![(7, vec![(0xA, 0), (0xB, 0)]), (8, vec![(0xC, 0)])]);
        let fallback = [DeformChannel {
            id: 0xD,
            raw_parameters: [0, 0, 1],
        }];
        let anim = animation(vec![channel(
            selector(2, 7, 0),
            vec![descriptor(0xD, 0x13)],
        )]);
        let prepared = prepare(&anim, &dictionary, &fallback).unwrap();
        assert_eq!(prepared.channels[0].layout_source, LayoutSource::Fallback);
        assert_eq!(prepared.channels[0].components[0].flags, 0x93);
        let initial = prepare_native_initial(&anim, &dictionary).unwrap();
        assert_eq!(initial.channels[0].layout_source, LayoutSource::Fallback);
        assert_eq!(initial.channels[0].components[0].slot, u32::MAX);
        assert_eq!(initial.channels[0].components[0].flags, 3);
        let stale = animation(vec![
            channel(selector(0, 7, 0), vec![]),
            channel(selector(0, 8, 0), vec![descriptor(0xB, 0x13)]),
        ]);
        assert_eq!(
            prepare(&stale, &dictionary, &[]),
            Err(DescriptorBindingError::StaleDeformSlot {
                channel: 1,
                component: 0,
                slot: 4,
                layout_count: 1,
            })
        );
    }

    #[test]
    fn excessive_and_ambiguous_layouts_are_explicit_errors() {
        let a = animation(vec![channel(selector(0, 7, 0), vec![])]);
        let excessive = dictionary(vec![(7, vec![(1, 0); 7])]);
        assert_eq!(
            prepare(&a, &excessive, &[]),
            Err(DescriptorBindingError::TooManyDeformSlots {
                channel: Some(0),
                count: 7,
            })
        );
        let duplicate = dictionary(vec![(7, vec![]), (7, vec![])]);
        assert_eq!(
            prepare(&a, &duplicate, &[]),
            Err(DescriptorBindingError::DuplicateCapability(7))
        );
        let fallback = [DeformChannel {
            id: 1,
            raw_parameters: [0; 3],
        }; 7];
        assert_eq!(
            prepare(&a, &dictionary(vec![]), &fallback),
            Err(DescriptorBindingError::TooManyDeformSlots {
                channel: None,
                count: 7
            })
        );
    }
}
