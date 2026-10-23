//! Source-part candidate inspection for capability-driven ANIM channels.
//!
//! This is the initial creature-row stage observed in the native loader, not a
//! completed runtime rig. Native finalization can insert/reorder/reroot blocks.
//! Results carry that unsupported boundary and must not be used as bone indices.
//! See `re/anim-rig-native.md`; no proprietary implementation was copied.

use crate::creature_data::{CreatureCapability, CreatureData, CreatureDataError, CreatureMorph};
use crate::part_capabilities::PartCapabilities;
use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingError {
    InvalidRanges(CreatureDataError),
    MissingPartView(usize),
    TooManyParts(usize),
    InvalidTopology {
        part: usize,
        field: &'static str,
    },
    NonFinite {
        part: usize,
        field: &'static str,
    },
    InvalidOrientation(usize),
    CapabilityOverflow(usize),
    DuplicateAggregate(u32),
    UnsupportedSelector {
        flags: u32,
        alternate: u32,
        reserved: u32,
    },
    NativeFinalizationRequired,
}
impl fmt::Display for BindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRanges(e) => write!(f, "{e}"),
            Self::MissingPartView(i) => write!(f, "source part {i} has no checked row view"),
            Self::TooManyParts(n) => write!(f, "{n} source parts exceed the native supported limit of 255"),
            Self::InvalidTopology { part, field } => write!(f, "source part {part} has invalid {field}"),
            Self::NonFinite { part, field } => write!(f, "source part {part} has nonfinite {field}"),
            Self::InvalidOrientation(i) => write!(f, "source part {i} has no supported proper rotation"),
            Self::CapabilityOverflow(i) => write!(f, "source part {i} exceeds 32 expanded capabilities"),
            Self::DuplicateAggregate(id) => write!(f, "ambiguous duplicate capability aggregate {id:08X}"),
            Self::UnsupportedSelector { flags, alternate, reserved } => write!(f, "unsupported source selector flags={flags:08X} alternate={alternate:08X} reserved={reserved:08X}"),
            Self::NativeFinalizationRequired => f.write_str("native generated-block, root and ordering finalization is not implemented"),
        }
    }
}
impl std::error::Error for BindingError {}

/// Four raw selector words. Names do not participate in candidate selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSelector {
    pub flags: u32,
    pub capability: u32,
    pub alternate: u32,
    pub reserved: u32,
}
impl SourceSelector {
    pub fn from_words(words: [u32; 4]) -> Self {
        Self {
            flags: words[0],
            capability: words[1],
            alternate: words[2],
            reserved: words[3],
        }
    }
    pub fn primary(channel: &crate::anim::AnimBone) -> Self {
        Self::from_words(channel.selector_words[1..5].try_into().unwrap())
    }
    pub fn secondary(channel: &crate::anim::AnimBone) -> Self {
        Self::from_words(channel.selector_words[5..9].try_into().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityValue {
    pub id: u32,
    pub number: u8,
}

/// A source row's initial body-block values, before native finalization.
#[derive(Debug, Clone)]
pub struct SourcePartBlock {
    pub source_row: usize,
    pub source_part_id: u16,
    pub part_instance: u64,
    pub parent_source_row: Option<usize>,
    pub symmetric_source_row: Option<usize>,
    pub capability_range: Range<usize>,
    pub morph_range: Range<usize>,
    pub capabilities: Vec<CreatureCapability>,
    pub morphs: Vec<CreatureMorph>,
    /// PCTP-expanded identifiers; original records remain above.
    pub expanded_capabilities: Vec<CapabilityValue>,
    pub raw_row: [u8; 0x8C],
    pub scale: f32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub stored_rotation: [[f32; 3]; 3],
    /// xyzw from the native loader's matrix input convention; not a mesh pose.
    pub orientation_xyzw: [f32; 4],
    pub part_translation: [f32; 3],
    /// Transformed scaled bounds center used by the initial spatial selector.
    pub selection_position: [f32; 3],
    pub half_extents: [f32; 3],
    pub body_origin_local: [f32; 3],
    /// Exact source triple at +96, before scale/origin conversion.
    pub source_effects_origin: [f32; 3],
    pub effects_origin_local: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedPreparation {
    NativeFinalizationRequired,
}

/// Candidates in source row coordinates. No runtime-block/bone IDs are exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCandidates {
    pub source_rows: Vec<usize>,
    pub unsupported_preparation: UnsupportedPreparation,
}

#[derive(Debug, Clone)]
pub struct SourceRig {
    blocks: Vec<SourcePartBlock>,
}
impl SourceRig {
    pub fn blocks(&self) -> &[SourcePartBlock] {
        &self.blocks
    }

    pub fn from_creature(
        c: &CreatureData,
        dictionary: &PartCapabilities,
    ) -> Result<Self, BindingError> {
        c.validate_part_ranges()
            .map_err(BindingError::InvalidRanges)?;
        if c.parts.len() > 255 {
            return Err(BindingError::TooManyParts(c.parts.len()));
        }
        for (i, p) in c.parts.iter().enumerate() {
            if usize::from(p.index) != i {
                return Err(BindingError::InvalidTopology {
                    part: i,
                    field: "ordinal",
                });
            }
            for (field, target) in [
                ("parent", p.parent_index()),
                ("symmetry", p.symmetric_index()),
            ] {
                if target.is_some_and(|j| j >= c.parts.len() || j == i) {
                    return Err(BindingError::InvalidTopology { part: i, field });
                }
            }
            if let Some(j) = p.symmetric_index() {
                if c.parts[j].symmetric_index() != Some(i) {
                    return Err(BindingError::InvalidTopology {
                        part: i,
                        field: "reciprocal symmetry",
                    });
                }
            }
            let mut at = p.parent_index();
            let mut visited = 0;
            while let Some(j) = at {
                if visited >= c.parts.len() {
                    return Err(BindingError::InvalidTopology {
                        part: i,
                        field: "parent cycle",
                    });
                }
                at = c
                    .parts
                    .get(j)
                    .ok_or(BindingError::InvalidTopology {
                        part: i,
                        field: "parent",
                    })?
                    .parent_index();
                visited += 1;
            }
        }
        if c.parts
            .iter()
            .filter(|p| p.parent_index().is_none())
            .count()
            != 1
        {
            return Err(BindingError::InvalidTopology {
                part: 0,
                field: "root count",
            });
        }
        let mut aggregates = std::collections::BTreeMap::new();
        for aggregate in &dictionary.aggregates {
            if aggregates
                .insert(aggregate.capability, &aggregate.members)
                .is_some()
            {
                return Err(BindingError::DuplicateAggregate(aggregate.capability));
            }
        }
        let mut blocks = Vec::with_capacity(c.parts.len());
        for (i, p) in c.parts.iter().enumerate() {
            let view = c.part_view(i).ok_or(BindingError::MissingPartView(i))?;
            let raw_row: [u8; 0x8C] = view
                .raw_row
                .try_into()
                .map_err(|_| BindingError::MissingPartView(i))?;
            let float_at = |at| f32::from_le_bytes(raw_row[at..at + 4].try_into().unwrap());
            let scale = float_at(44);
            let source_effects_origin = [float_at(96), float_at(100), float_at(104)];
            for (field, values) in [
                ("scale", &[scale][..]),
                ("bounds min", &p.bounds_min[..]),
                ("bounds max", &p.bounds_max[..]),
                ("translation", &p.translation[..]),
                ("effects origin", &source_effects_origin[..]),
            ] {
                if values.iter().any(|v| !v.is_finite()) {
                    return Err(BindingError::NonFinite { part: i, field });
                }
            }
            let orientation_xyzw =
                orientation_from_source(p.rotation).ok_or(BindingError::InvalidOrientation(i))?;
            let center = std::array::from_fn(|j| {
                ((p.bounds_min[j] * scale) + (p.bounds_max[j] * scale)) * 0.5
            });
            let rotated = rotate(orientation_xyzw, center);
            let selection_position = std::array::from_fn(|j| p.translation[j] + rotated[j]);
            let origin_delta = std::array::from_fn(|j| p.translation[j] - selection_position[j]);
            let mut inverse = orientation_xyzw;
            for v in &mut inverse[..3] {
                *v = -*v;
            }
            let body_origin_local = rotate(inverse, origin_delta);
            let effects_origin_local =
                std::array::from_fn(|j| source_effects_origin[j] * scale + body_origin_local[j]);
            let half_extents = std::array::from_fn(|j| {
                let value = (p.bounds_max[j] * scale - p.bounds_min[j] * scale) * 0.5;
                if value <= 0.0 {
                    0.1
                } else {
                    value
                }
            });
            if selection_position
                .iter()
                .chain(&body_origin_local)
                .chain(&effects_origin_local)
                .chain(&half_extents)
                .any(|v| !v.is_finite())
            {
                return Err(BindingError::NonFinite {
                    part: i,
                    field: "derived block geometry",
                });
            }
            let mut expanded: Vec<CapabilityValue> = Vec::new();
            for cap in view.capabilities {
                merge_capability(
                    &mut expanded,
                    CapabilityValue {
                        id: cap.packed_id(),
                        number: cap.number,
                    },
                    i,
                )?;
            }
            let mut changed = true;
            while changed {
                changed = false;
                for cap in expanded.clone() {
                    if let Some(members) = aggregates.get(&cap.id) {
                        for id in *members {
                            changed |= merge_capability(
                                &mut expanded,
                                CapabilityValue {
                                    id: *id,
                                    number: cap.number,
                                },
                                i,
                            )?;
                        }
                    }
                }
            }
            blocks.push(SourcePartBlock {
                source_row: i,
                source_part_id: p.index,
                part_instance: p.part_instance,
                parent_source_row: p.parent_index(),
                symmetric_source_row: p.symmetric_index(),
                capability_range: p.capability_range(),
                morph_range: p.morph_range(),
                capabilities: view.capabilities.to_vec(),
                morphs: view.morphs.to_vec(),
                expanded_capabilities: expanded,
                raw_row,
                scale,
                bounds_min: p.bounds_min,
                bounds_max: p.bounds_max,
                stored_rotation: p.rotation,
                orientation_xyzw,
                part_translation: p.translation,
                selection_position,
                half_extents,
                body_origin_local,
                source_effects_origin,
                effects_origin_local,
            });
        }
        Ok(Self { blocks })
    }

    /// Initial-stage capability/X-side/extremum candidate inspection only.
    /// Hierarchy/class/root/index/unknown flags and alternate words are rejected.
    /// Native finalization is explicitly still required for every result.
    pub fn candidate_parts(
        &self,
        selector: SourceSelector,
    ) -> Result<SourceCandidates, BindingError> {
        let supported = 0x2000C | 0xE000;
        let side = selector.flags & 0x2000C;
        let extremum = selector.flags & 0xE000;
        if selector.flags & !supported != 0
            || selector.alternate != 0
            || selector.reserved != 0
            || extremum == 0xE000
            || !matches!(side, 0 | 4 | 8 | 12 | 0x20000 | 0x20004 | 0x20008)
        {
            return Err(BindingError::UnsupportedSelector {
                flags: selector.flags,
                alternate: selector.alternate,
                reserved: selector.reserved,
            });
        }
        let matching: Vec<_> = self
            .blocks
            .iter()
            .filter(|b| {
                b.expanded_capabilities
                    .iter()
                    .any(|c| c.id == selector.capability && c.number != 0)
            })
            .collect();
        let low = matching
            .iter()
            .map(|b| b.selection_position[0])
            .fold(f32::INFINITY, f32::min);
        let high = matching
            .iter()
            .map(|b| b.selection_position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let mut selected: Vec<usize> = Vec::new();
        for b in matching {
            let x = b.selection_position[0];
            let threshold = b.half_extents.iter().sum::<f32>() * 0.15;
            let excluded = match side {
                4 => x < -threshold,
                8 => x > threshold,
                12 => x.abs() > threshold,
                0x20000 => (x * 100.0).floor() < ((high - (high - low) * 0.525) * 100.0).floor(),
                0x20004 => (x * 100.0).floor() > (((high - low) * 0.525 + low) * 100.0).floor(),
                0x20008 => ((x - ((high - low) * 0.5 + low)) * 100.0).floor().abs() > 10.0,
                _ => false,
            };
            if excluded {
                continue;
            }
            if extremum == 0 {
                selected.push(b.source_row);
                continue;
            }
            let axis = match extremum {
                0x2000 | 0x4000 => 0,
                0x6000 | 0x8000 => 1,
                _ => 2,
            };
            let maximize = matches!(extremum, 0x2000 | 0x8000 | 0xA000);
            if let Some(first) = selected.first_mut() {
                let a = self.blocks[*first].selection_position[axis];
                let v = b.selection_position[axis];
                if (maximize && v > a) || (!maximize && v < a) {
                    *first = b.source_row;
                }
            } else {
                selected.push(b.source_row);
            }
        }
        Ok(SourceCandidates {
            source_rows: selected,
            unsupported_preparation: UnsupportedPreparation::NativeFinalizationRequired,
        })
    }

    /// Full final binding intentionally has no successful path yet.
    pub fn require_finalized(&self) -> Result<(), BindingError> {
        Err(BindingError::NativeFinalizationRequired)
    }
}

fn merge_capability(
    values: &mut Vec<CapabilityValue>,
    cap: CapabilityValue,
    part: usize,
) -> Result<bool, BindingError> {
    if cap.id == 0 {
        return Ok(false);
    }
    if let Some(existing) = values.iter_mut().find(|v| v.id == cap.id) {
        let previous = existing.number;
        existing.number = existing.number.max(cap.number);
        return Ok(previous != existing.number);
    } else {
        if values.len() >= 32 {
            return Err(BindingError::CapabilityOverflow(part));
        }
        values.push(cap);
    }
    Ok(true)
}

/// Standard quaternion rotation. The source helper does not renormalize its
/// matrix-derived quaternion, so this initial-stage observation keeps its value.
fn rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let [x, y, z, w] = q;
    [
        (1.0 - 2.0 * (y * y + z * z)) * v[0]
            + 2.0 * (x * y - z * w) * v[1]
            + 2.0 * (x * z + y * w) * v[2],
        2.0 * (x * y + z * w) * v[0]
            + (1.0 - 2.0 * (x * x + z * z)) * v[1]
            + 2.0 * (y * z - x * w) * v[2],
        2.0 * (x * z - y * w) * v[0]
            + 2.0 * (y * z + x * w) * v[1]
            + (1.0 - 2.0 * (x * x + y * y)) * v[2],
    ]
}

fn orientation_from_source(source: [[f32; 3]; 3]) -> Option<[f32; 4]> {
    if source.iter().flatten().any(|v| !v.is_finite()) {
        return None;
    }
    for i in 0..3 {
        for j in 0..3 {
            let dot: f32 = (0..3).map(|k| source[i][k] * source[j][k]).sum();
            if (dot - if i == j { 1.0 } else { 0.0 }).abs() > 0.01 {
                return None;
            }
        }
    }
    let determinant = source[0][0] * (source[1][1] * source[2][2] - source[1][2] * source[2][1])
        - source[0][1] * (source[1][0] * source[2][2] - source[1][2] * source[2][0])
        + source[0][2] * (source[1][0] * source[2][1] - source[1][1] * source[2][0]);
    if (determinant - 1.0).abs() > 0.02 {
        return None;
    }
    let m: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| source[j][i]));
    let trace = m[0][0] + m[1][1] + m[2][2];
    let mut q = [0.0; 4];
    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        q = [
            (m[2][1] - m[1][2]) / s,
            (m[0][2] - m[2][0]) / s,
            (m[1][0] - m[0][1]) / s,
            s * 0.25,
        ];
    } else {
        let i = if m[0][0] >= m[1][1] && m[0][0] >= m[2][2] {
            0
        } else if m[1][1] >= m[2][2] {
            1
        } else {
            2
        };
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        let s = (1.0 + m[i][i] - m[j][j] - m[k][k]).sqrt() * 2.0;
        q[i] = s * 0.25;
        q[j] = (m[i][j] + m[j][i]) / s;
        q[k] = (m[i][k] + m[k][i]) / s;
        q[3] = (m[k][j] - m[j][k]) / s;
    }
    q.iter().all(|v| v.is_finite()).then_some(q)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (CreatureData, PartCapabilities) {
        let mut bytes = Vec::new();
        for value in [crate::creature_data::HEADER, 9, 3, 3, 0] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&(crate::creature_data::HEADER_BUFFER_SIZE as u16).to_le_bytes());
        bytes.extend_from_slice(&[0u8; crate::creature_data::HEADER_BUFFER_SIZE]);
        for i in 0..3u16 {
            let mut row = [0u8; 140];
            row[0..2].copy_from_slice(&i.to_le_bytes());
            row[4..6].copy_from_slice(&(if i == 0 { u16::MAX } else { 0 }).to_le_bytes());
            row[6..8].copy_from_slice(&(if i == 0 { u16::MAX } else { 3 - i }).to_le_bytes());
            row[12..14].copy_from_slice(&i.to_le_bytes());
            row[16] = 1;
            for (offset, value) in [
                (20, -0.2f32),
                (24, -0.2),
                (28, -0.2),
                (32, 0.2),
                (36, 0.2),
                (40, 0.2),
                (44, 1.0),
                (48, 1.0),
                (64, 1.0),
                (80, 1.0),
                (
                    84,
                    if i == 1 {
                        1.0
                    } else if i == 2 {
                        -1.0
                    } else {
                        0.0
                    },
                ),
            ] {
                row[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
            bytes.extend_from_slice(&row);
        }
        bytes.extend_from_slice(b"spinfootfoot");
        bytes.extend_from_slice(&[1, 1, 1]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let c = crate::creature_data::parse(&bytes).unwrap();
        let dictionary = PartCapabilities {
            version: 4,
            priority_bits: None,
            names: vec![],
            remaps: vec![],
            aggregates: vec![],
            deforms: vec![],
        };
        (c, dictionary)
    }
    fn selector(flags: u32) -> SourceSelector {
        SourceSelector {
            flags,
            capability: u32::from_le_bytes(*b"foot"),
            alternate: 0,
            reserved: 0,
        }
    }
    #[test]
    fn capability_side_relative_and_extremum_source_candidates() {
        let (c, d) = fixture();
        let rig = SourceRig::from_creature(&c, &d).unwrap();
        for (flags, expected) in [
            (0, vec![1, 2]),
            (4, vec![1]),
            (8, vec![2]),
            (12, vec![]),
            (0x20000, vec![1]),
            (0x20004, vec![2]),
            (0x20008, vec![]),
            (0x2000, vec![1]),
            (0x4000, vec![2]),
        ] {
            let result = rig.candidate_parts(selector(flags)).unwrap();
            assert_eq!(result.source_rows, expected, "flags {flags:X}");
            assert_eq!(
                result.unsupported_preparation,
                UnsupportedPreparation::NativeFinalizationRequired
            );
        }
        assert_eq!(
            rig.require_finalized(),
            Err(BindingError::NativeFinalizationRequired)
        );
    }
    #[test]
    fn hierarchy_root_class_and_unknown_selectors_are_explicitly_unsupported() {
        let (c, d) = fixture();
        let rig = SourceRig::from_creature(&c, &d).unwrap();
        for flags in [
            1, 3, 0x100000, 0x400, 0x1800, 0x100, 0x40000, 0xE000, 0x80000000,
        ] {
            assert!(matches!(
                rig.candidate_parts(selector(flags)),
                Err(BindingError::UnsupportedSelector { .. })
            ));
        }
        let mut s = selector(0);
        s.alternate = 1;
        assert!(matches!(
            rig.candidate_parts(s),
            Err(BindingError::UnsupportedSelector { .. })
        ));
    }
    #[test]
    fn source_orientation_convention_and_scaled_bounds_center_are_distinct() {
        let (mut c, d) = fixture();
        let p = &mut c.parts[1];
        p.rotation = [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        p.bounds_min = [0.0, 0.0, 0.0];
        p.bounds_max = [2.0, 0.0, 0.0];
        p.translation = [3.0, 4.0, 5.0];
        let at = 140 + 44;
        c.data0[at..at + 4].copy_from_slice(&2.0f32.to_le_bytes());
        let rig = SourceRig::from_creature(&c, &d).unwrap();
        let p = &rig.blocks()[1];
        for (a, b) in p.selection_position.iter().zip([3.0, 2.0, 5.0]) {
            assert!((a - b).abs() < 0.00001);
        }
        assert_eq!(p.half_extents, [2.0, 0.1, 0.1]);
        assert_eq!(p.part_translation, [3.0, 4.0, 5.0]);
    }
    #[test]
    fn cyclic_aggregates_are_bounded_and_duplicate_numbers_take_maximum() {
        use crate::part_capabilities::CapabilityAggregate;
        let (c, mut d) = fixture();
        let foot = u32::from_le_bytes(*b"foot");
        let alias = 7;
        d.aggregates = vec![
            CapabilityAggregate {
                capability: foot,
                members: vec![alias],
            },
            CapabilityAggregate {
                capability: alias,
                members: vec![foot],
            },
        ];
        let rig = SourceRig::from_creature(&c, &d).unwrap();
        assert_eq!(rig.blocks()[1].expanded_capabilities.len(), 2);
        let mut s = selector(0);
        s.capability = alias;
        assert_eq!(rig.candidate_parts(s).unwrap().source_rows, vec![1, 2]);
        d.aggregates.push(d.aggregates[0].clone());
        assert!(matches!(
            SourceRig::from_creature(&c, &d),
            Err(BindingError::DuplicateAggregate(_))
        ));
    }
    #[test]
    fn invalid_topology_geometry_and_capability_expansion_fail_safely() {
        let (mut c, mut d) = fixture();
        c.parts[1].translation[0] = f32::NAN;
        assert!(matches!(
            SourceRig::from_creature(&c, &d),
            Err(BindingError::NonFinite { .. })
        ));
        c.parts[1].translation[0] = 0.0;
        c.parts[1].flags_a = 2;
        c.parts[2].flags_a = 1;
        assert!(matches!(
            SourceRig::from_creature(&c, &d),
            Err(BindingError::InvalidTopology { .. })
        ));
        c.parts[1].flags_a = 0;
        c.parts[2].flags_a = 0;
        c.parts[1].rotation[0][0] = 2.0;
        assert!(matches!(
            SourceRig::from_creature(&c, &d),
            Err(BindingError::InvalidOrientation(_))
        ));
        c.parts[1].rotation[0][0] = 1.0;
        d.aggregates
            .push(crate::part_capabilities::CapabilityAggregate {
                capability: u32::from_le_bytes(*b"foot"),
                members: (1..=33).collect(),
            });
        assert!(matches!(
            SourceRig::from_creature(&c, &d),
            Err(BindingError::CapabilityOverflow(_))
        ));
    }
}
