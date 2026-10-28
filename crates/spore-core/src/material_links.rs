//! Header-zero SMT material records and their companion RW4 state libraries.
//!
//! Framing and state-count association are measured independently from installed
//! bytes. Texture words are instance/group, both big-endian. The opaque compiled
//! payload is retained; unresolved render-state and saved-pointer words are not
//! interpreted as functioning GPU bindings.
use crate::rw4::{self, Rw4};
use std::{fmt, ops::Range};

const MAX_RECORDS: usize = 65_536;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialError {
    Truncated,
    UnsupportedHeader(u32),
    TooManyRecords,
    InvalidMaterial,
    MissingTerminator,
    TrailingBytes,
    StateCountMismatch { expected: usize, actual: usize },
    WrongSectionType { section: usize, type_code: u32 },
    MalformedState { section: usize },
    UnsupportedStateAlignment { section: usize },
    UnsupportedBindingFlags,
    InvalidBinding,
    TextureBindingCount { expected: usize, actual: usize },
}
impl fmt::Display for MaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source materials: {self:?}")
    }
}
impl std::error::Error for MaterialError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureReference {
    pub instance_id: u32,
    pub group_id: u32,
}
impl TextureReference {
    /// Representation used by this repository's wide DBPF index reader.
    pub fn packed_instance(self) -> u64 {
        (u64::from(self.group_id) << 32) | u64::from(self.instance_id)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialLink {
    pub material_id: u32,
    pub state_count: u8,
    pub textures: Vec<TextureReference>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialLinks {
    pub materials: Vec<MaterialLink>,
}
impl MaterialLinks {
    pub fn state_count(&self) -> usize {
        self.materials
            .iter()
            .map(|m| usize::from(m.state_count))
            .sum()
    }
    /// Each source record consumes this consecutive range of companion RW4
    /// sections. A two-state record advances by two; it is not a state index.
    pub fn state_ranges(&self) -> Vec<Range<usize>> {
        let mut at = 0;
        self.materials
            .iter()
            .map(|m| {
                let start = at;
                at += usize::from(m.state_count);
                start..at
            })
            .collect()
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, MaterialError> {
        if self.materials.len() > MAX_RECORDS {
            return Err(MaterialError::TooManyRecords);
        }
        if self
            .materials
            .iter()
            .any(|m| m.material_id == u32::MAX || m.textures.len() > u8::MAX as usize)
        {
            return Err(MaterialError::InvalidMaterial);
        }
        let mut out = Vec::new();
        out.extend(0u32.to_be_bytes());
        for m in &self.materials {
            out.extend(m.material_id.to_be_bytes());
            out.push(m.state_count);
            out.push(m.textures.len() as u8);
            for t in &m.textures {
                out.extend(t.instance_id.to_be_bytes());
                out.extend(t.group_id.to_be_bytes());
            }
        }
        out.extend(u32::MAX.to_be_bytes());
        Ok(out)
    }
    /// Strictly associate the complete companion state library. Other RW4
    /// section layouts are rejected rather than filtered and silently shifted.
    pub fn bind_states(&self, rw: &Rw4, data: &[u8]) -> Result<MaterialStates, MaterialError> {
        let expected = self.state_count();
        if expected != rw.sections.len() {
            return Err(MaterialError::StateCountMismatch {
                expected,
                actual: rw.sections.len(),
            });
        }
        let states = (0..rw.sections.len())
            .map(|i| compiled_state(rw, data, i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MaterialStates {
            states,
            material_ranges: self.state_ranges(),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledState {
    pub section_index: usize,
    pub primitive_type: u32,
    pub soft_dirty: u32,
    pub soft_delta: u32,
    pub hard_dirty: u32,
    pub hard_delta: u32,
    pub shader_id: u32,
    /// The four bytes between the 28-byte header and its aligned payload. Source
    /// files contain uninitialized-looking bytes here; retain, do not use them.
    pub alignment_padding: [u8; 4],
    pub payload: Vec<u8>,
}
impl CompiledState {
    /// Decode supported variable layouts, including the bounded opaque 0x8 region.
    /// Other flag combinations remain explicitly unsupported; `payload` still
    /// retains their bytes. Saved raster words are not live resource pointers.
    pub fn bindings(&self) -> Result<CompiledBindings, MaterialError> {
        bindings(self)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for v in [
            (32 + self.payload.len()) as u32,
            self.primitive_type,
            self.soft_dirty,
            self.soft_delta,
            self.hard_dirty,
            self.hard_delta,
            self.shader_id,
        ] {
            out.extend(v.to_le_bytes());
        }
        out.extend(self.alignment_padding);
        out.extend(&self.payload);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingRegion {
    pub flag: u32,
    pub bytes: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateValue {
    pub state: u32,
    pub value: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateGroup {
    /// Observed IDs are 0 and 5. Their render-domain meaning is not assigned.
    pub id: u32,
    pub values: Vec<StateValue>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureStage {
    pub stage: u32,
    /// Material libraries use 0x00400000 as an injection slot and
    /// 0xFFFFFFFF as absent. Embedded RW4 states can contain other saved
    /// words; their interpretation and runtime pointer fixup are separate.
    pub raster_reference: u32,
    pub list_a_flags: u32,
    pub list_a: Vec<StateValue>,
    pub list_b_flags: u32,
    pub list_b: Vec<StateValue>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledBindings {
    /// Variable vertex-declaration and fixed-size regions, in source order.
    /// Pointer-looking/padding words within these regions stay opaque.
    pub prefix: Vec<BindingRegion>,
    pub groups: Option<Vec<StateGroup>>,
    pub texture_leading_word: u32,
    pub textures: Vec<TextureStage>,
}
impl CompiledBindings {
    pub fn to_bytes(&self) -> Result<Vec<u8>, MaterialError> {
        fn put(out: &mut Vec<u8>, v: u32) { out.extend(v.to_le_bytes()); }
        fn pairs(out: &mut Vec<u8>, values: &[StateValue]) -> Result<(), MaterialError> {
            for p in values {
                if p.state == u32::MAX { return Err(MaterialError::InvalidBinding); }
                put(out, p.state);
                put(out, p.value);
            }
            put(out, u32::MAX);
            Ok(())
        }
        let mut out = Vec::new();
        for region in &self.prefix { out.extend(&region.bytes); }
        if let Some(groups) = &self.groups {
            for group in groups {
                if group.id == u32::MAX { return Err(MaterialError::InvalidBinding); }
                put(&mut out, group.id);
                pairs(&mut out, &group.values)?;
            }
            put(&mut out, u32::MAX);
        }
        put(&mut out, self.texture_leading_word);
        for t in &self.textures {
            if t.stage == u32::MAX
                || (t.list_a_flags == 0 && !t.list_a.is_empty())
                || (t.list_b_flags == 0 && !t.list_b.is_empty()) {
                return Err(MaterialError::InvalidBinding);
            }
            put(&mut out, t.stage);
            put(&mut out, t.raster_reference);
            put(&mut out, t.list_a_flags);
            if t.list_a_flags != 0 { pairs(&mut out, &t.list_a)?; }
            put(&mut out, t.list_b_flags);
            if t.list_b_flags != 0 { pairs(&mut out, &t.list_b)?; }
        }
        put(&mut out, u32::MAX);
        Ok(out)
    }
}

/// A serialized raster placeholder. Source cardinality matches the associated
/// texture-key vector, but key-to-slot ordering still needs native corroboration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureSlot {
    pub state_index: usize,
    pub stage: u32,
}
impl MaterialStates {
    /// Enumerate 0x00400000 placeholders in serialized order and validate their
    /// cardinality against the source key vector. This does not assign keys.
    pub fn texture_slots(
        &self, material: usize, link: &MaterialLink,
    ) -> Result<Vec<TextureSlot>, MaterialError> {
        let range = self.material_ranges.get(material).ok_or(MaterialError::InvalidBinding)?;
        let mut slots = Vec::new();
        for state_index in range.clone() {
            let s = self.states.get(state_index).ok_or(MaterialError::InvalidBinding)?;
            for t in s.bindings()?.textures {
                if t.raster_reference == 0x00400000 { slots.push(TextureSlot { state_index, stage: t.stage }); }
                else if t.raster_reference != u32::MAX { return Err(MaterialError::InvalidBinding); }
            }
        }
        if slots.len() != link.textures.len() {
            return Err(MaterialError::TextureBindingCount { expected: link.textures.len(), actual: slots.len() });
        }
        Ok(slots)
    }
}

fn bindings(state: &CompiledState) -> Result<CompiledBindings, MaterialError> {
    struct Reader<'a> { data: &'a [u8], at: usize }
    impl Reader<'_> {
        fn take(&mut self, n: usize) -> Result<Vec<u8>, MaterialError> {
            let end = self.at.checked_add(n).ok_or(MaterialError::Truncated)?;
            let b = self.data.get(self.at..end).ok_or(MaterialError::Truncated)?;
            self.at = end;
            Ok(b.to_vec())
        }
        fn word(&mut self) -> Result<u32, MaterialError> {
            Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
        }
        fn pairs(&mut self) -> Result<Vec<StateValue>, MaterialError> {
            let mut result = Vec::new();
            loop {
                let state = self.word()?;
                if state == u32::MAX { return Ok(result); }
                if result.len() == MAX_RECORDS { return Err(MaterialError::TooManyRecords); }
                result.push(StateValue { state, value: self.word()? });
            }
        }
    }
    if state.soft_dirty & !0x10803c != 0 || state.hard_delta != 0 {
        return Err(MaterialError::UnsupportedBindingFlags);
    }
    let mut r = Reader { data: &state.payload, at: 0 };
    let mut prefix = Vec::new();
    if state.soft_dirty & 0x100000 != 0 {
        let b = r.data.get(12..14).ok_or(MaterialError::Truncated)?;
        let count = usize::from(u16::from_le_bytes(b.try_into().unwrap()));
        prefix.push(BindingRegion { flag: 0x100000, bytes: r.take(12 * (count + 2))? });
    }
    if state.soft_dirty & 0x8 != 0 {
        // 11EE650 and 11EE810 advance past each eight-byte header and,
        // for nonzero tags, a four-byte body when the third u16 is zero,
        // otherwise the sum of the second and third u16 fields. The full
        // terminator header is included; contents remain uninterpreted.
        let start = r.at;
        let mut count = 0;
        loop {
            let header = r.take(8)?;
            let tag = u16::from_le_bytes([header[0], header[1]]);
            if tag == 0 { break; }
            if count == MAX_RECORDS { return Err(MaterialError::TooManyRecords); }
            count += 1;
            let a = usize::from(u16::from_le_bytes([header[2], header[3]]));
            let b = usize::from(u16::from_le_bytes([header[4], header[5]]));
            r.take(if b == 0 { 4 } else { a + b })?;
        }
        prefix.push(BindingRegion { flag: 0x8, bytes: r.data[start..r.at].to_vec() });
    }
    for (flag, size) in [(0x10, 16), (0x20, 12), (0x8000, 17)] {
        if state.soft_dirty & flag != 0 {
            prefix.push(BindingRegion { flag, bytes: r.take(size)? });
        }
    }
    let groups = if state.hard_dirty & 0x20000 != 0 {
        let mut groups = Vec::new();
        loop {
            let id = r.word()?;
            if id == u32::MAX { break; }
            if groups.len() == MAX_RECORDS { return Err(MaterialError::TooManyRecords); }
            groups.push(StateGroup { id, values: r.pairs()? });
        }
        Some(groups)
    } else { None };
    let texture_leading_word = r.word()?;
    let mut textures = Vec::new();
    loop {
        let stage = r.word()?;
        if stage == u32::MAX { break; }
        if textures.len() == MAX_RECORDS { return Err(MaterialError::TooManyRecords); }
        let raster_reference = r.word()?;
        let list_a_flags = r.word()?;
        let list_a = if list_a_flags != 0 { r.pairs()? } else { Vec::new() };
        let list_b_flags = r.word()?;
        let list_b = if list_b_flags != 0 { r.pairs()? } else { Vec::new() };
        textures.push(TextureStage { stage, raster_reference, list_a_flags, list_a, list_b_flags, list_b });
    }
    if r.at != r.data.len() { return Err(MaterialError::TrailingBytes); }
    Ok(CompiledBindings { prefix, groups, texture_leading_word, textures })
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialStates {
    pub states: Vec<CompiledState>,
    pub material_ranges: Vec<Range<usize>>,
}

fn word(data: &[u8], at: &mut usize) -> Result<u32, MaterialError> {
    let end = at.checked_add(4).ok_or(MaterialError::Truncated)?;
    let b = data.get(*at..end).ok_or(MaterialError::Truncated)?;
    *at = end;
    Ok(u32::from_be_bytes(b.try_into().unwrap()))
}
pub fn parse(data: &[u8]) -> Result<MaterialLinks, MaterialError> {
    let mut at = 0;
    let version = word(data, &mut at)?;
    if version != 0 {
        return Err(MaterialError::UnsupportedHeader(version));
    }
    let mut materials = Vec::new();
    loop {
        if at == data.len() {
            return Err(MaterialError::MissingTerminator);
        }
        let material_id = word(data, &mut at)?;
        if material_id == u32::MAX {
            break;
        }
        if materials.len() == MAX_RECORDS {
            return Err(MaterialError::TooManyRecords);
        }
        let header = data.get(at..at + 2).ok_or(MaterialError::Truncated)?;
        at += 2;
        let state_count = header[0];
        let count = usize::from(header[1]);
        if count > data.len().saturating_sub(at) / 8 {
            return Err(MaterialError::Truncated);
        }
        let mut textures = Vec::with_capacity(count);
        for _ in 0..count {
            textures.push(TextureReference {
                instance_id: word(data, &mut at)?,
                group_id: word(data, &mut at)?,
            });
        }
        materials.push(MaterialLink {
            material_id,
            state_count,
            textures,
        });
    }
    if at != data.len() {
        return Err(MaterialError::TrailingBytes);
    }
    Ok(MaterialLinks { materials })
}
pub fn compiled_state(
    rw: &Rw4,
    data: &[u8],
    section: usize,
) -> Result<CompiledState, MaterialError> {
    let s = rw
        .sections
        .get(section)
        .ok_or(MaterialError::MalformedState { section })?;
    if s.info.type_code != rw4::object_types::COMPILED_STATE {
        return Err(MaterialError::WrongSectionType {
            section,
            type_code: s.info.type_code,
        });
    }
    if s.start % 16 != 0 {
        return Err(MaterialError::UnsupportedStateAlignment { section });
    }
    let b = rw
        .object(data, section as u32)
        .ok_or(MaterialError::MalformedState { section })?;
    if b.len() < 32 {
        return Err(MaterialError::MalformedState { section });
    }
    let words: Vec<_> = b[..28]
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
        .collect();
    if words[0] as usize != b.len() {
        return Err(MaterialError::MalformedState { section });
    }
    Ok(CompiledState {
        section_index: section,
        primitive_type: words[1],
        soft_dirty: words[2],
        soft_delta: words[3],
        hard_dirty: words[4],
        hard_delta: words[5],
        shader_id: words[6],
        alignment_padding: b[28..32].try_into().unwrap(),
        payload: b[32..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn links() -> MaterialLinks {
        MaterialLinks {
            materials: vec![
                MaterialLink {
                    material_id: 0xAABBCCDD,
                    state_count: 1,
                    textures: vec![TextureReference {
                        instance_id: 0x12345678,
                        group_id: 0x40002A00,
                    }],
                },
                MaterialLink {
                    material_id: 0x10203040,
                    state_count: 2,
                    textures: vec![],
                },
            ],
        }
    }
    fn rw_states() -> (Rw4, Vec<u8>) {
        let mut data = Vec::new();
        let mut sections = Vec::new();
        for index in 0..3 {
            let state = CompiledState {
                section_index: index,
                primitive_type: 4,
                soft_dirty: 0x00108004,
                soft_delta: 0x8004,
                hard_dirty: 0xA0003,
                hard_delta: 0,
                shader_id: 0x70000060 + index as u32,
                alignment_padding: [0xCD; 4],
                payload: vec![index as u8; 16],
            };
            let bytes = state.to_bytes();
            let start = data.len();
            data.extend(bytes);
            sections.push(rw4::Section {
                start,
                data_size: 48,
                info: rw4::SectionInfo {
                    p_data: start as i32,
                    field_04: 0,
                    data_size: 48,
                    alignment: 16,
                    type_code_index: 0,
                    type_code: rw4::object_types::COMPILED_STATE,
                },
            });
        }
        (
            Rw4 {
                rw_type_code: 0xCAFED00D,
                p_section_infos: 0,
                p_buffer_data: data.len(),
                sections,
            },
            data,
        )
    }
    #[test]
    fn source_frames_keep_big_endian_keys_state_counts_and_texture_groups() {
        let source = links();
        let d = source.to_bytes().unwrap();
        let parsed = parse(&d).unwrap();
        assert_eq!(parsed, source);
        assert_eq!(&d[4..8], &[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(
            parsed.materials[0].textures[0].packed_instance(),
            0x40002A0012345678
        );
        assert_eq!(parsed.state_ranges(), vec![0..1, 1..3]);
        assert_eq!(parsed.state_count(), 3);
    }
    #[test]
    fn empty_table_and_zero_texture_record_roundtrip() {
        let empty = MaterialLinks { materials: vec![] };
        assert_eq!(parse(&empty.to_bytes().unwrap()), Ok(empty));
        assert_eq!(
            parse(&links().to_bytes().unwrap()).unwrap().materials[1]
                .textures
                .len(),
            0
        );
    }
    #[test]
    fn every_prefix_unknown_version_and_trailing_bytes_rejected() {
        let d = links().to_bytes().unwrap();
        for end in 0..d.len() {
            assert!(parse(&d[..end]).is_err(), "prefix {end}");
        }
        let mut bad = d.clone();
        bad[..4].copy_from_slice(&1u32.to_be_bytes());
        assert_eq!(parse(&bad), Err(MaterialError::UnsupportedHeader(1)));
        let mut extra = d;
        extra.push(0);
        assert_eq!(parse(&extra), Err(MaterialError::TrailingBytes));
    }
    #[test]
    fn bogus_texture_count_is_bounded_before_allocation() {
        let mut d = links().to_bytes().unwrap();
        d[9] = 255;
        assert_eq!(parse(&d), Err(MaterialError::Truncated));
    }
    #[test]
    fn two_state_material_consumes_next_two_sections_without_default_shift() {
        let (rw, d) = rw_states();
        let bound = links().bind_states(&rw, &d).unwrap();
        assert_eq!(bound.material_ranges, vec![0..1, 1..3]);
        assert_eq!(bound.states[1].shader_id, 0x70000061);
        for (state, s) in bound.states.iter().zip(&rw.sections) {
            assert_eq!(state.to_bytes(), d[s.start..s.start + s.data_size]);
        }
    }
    #[test]
    fn encoder_rejects_reserved_id_and_oversize_texture_vector() {
        let mut source = links();
        source.materials[0].material_id = u32::MAX;
        assert_eq!(source.to_bytes(), Err(MaterialError::InvalidMaterial));
        let mut source = links();
        source.materials[0].textures = vec![
            TextureReference {
                instance_id: 1,
                group_id: 0
            };
            256
        ];
        assert_eq!(source.to_bytes(), Err(MaterialError::InvalidMaterial));
    }
    #[test]
    fn missing_wrong_type_misaligned_and_size_mismatched_states_rejected() {
        let (mut rw, mut d) = rw_states();
        rw.sections.pop();
        assert!(matches!(
            links().bind_states(&rw, &d),
            Err(MaterialError::StateCountMismatch { .. })
        ));
        let (mut rw, _) = rw_states();
        rw.sections[1].info.type_code = rw4::object_types::MESH;
        assert!(matches!(
            links().bind_states(&rw, &d),
            Err(MaterialError::WrongSectionType { section: 1, .. })
        ));
        let (mut rw, _) = rw_states();
        rw.sections[1].start += 1;
        assert!(matches!(
            links().bind_states(&rw, &d),
            Err(MaterialError::UnsupportedStateAlignment { section: 1 })
        ));
        let (rw, _) = rw_states();
        d[0..4].copy_from_slice(&99u32.to_le_bytes());
        assert!(matches!(
            links().bind_states(&rw, &d),
            Err(MaterialError::MalformedState { section: 0 })
        ));
    }
    fn binding_state() -> CompiledState {
        let (rw, data) = rw_states();
        let mut state = compiled_state(&rw, &data, 0).unwrap();
        state.soft_dirty = 0x8004;
        state.hard_dirty = 0x20000;
        state.hard_delta = 0;
        let mut payload = vec![0xCD; 17];
        // Two independent state groups, followed by one absent raster and one
        // injection placeholder. Both conditional lists exercise termination.
        for v in [0, 7, 1, u32::MAX, 5, 171, 1, u32::MAX, u32::MAX,
            u32::MAX, 0, u32::MAX, 0, 0,
            3, 0x00400000, 2, 9, 123, u32::MAX, 0x33, 1, 4, 6, 2, u32::MAX,
            u32::MAX] {
            payload.extend(v.to_le_bytes());
        }
        state.payload = payload;
        state
    }
    #[test]
    fn binding_regions_keep_unaligned_state_lists_and_absent_rasters() {
        let state = binding_state();
        let b = state.bindings().unwrap();
        assert_eq!(b.prefix[0].bytes.len(), 17);
        assert_eq!(b.groups.as_ref().unwrap()[1].id, 5);
        assert_eq!(b.textures[0].raster_reference, u32::MAX);
        assert_eq!(b.textures[1].stage, 3);
        assert_eq!(b.textures[1].list_a, vec![StateValue { state: 9, value: 123 }]);
        assert_eq!(b.textures[1].list_b[1], StateValue { state: 6, value: 2 });
        assert_eq!(b.to_bytes().unwrap(), state.payload);
    }
    #[test]
    fn opaque_eight_region_keeps_both_record_lengths_and_full_terminator() {
        let mut state = binding_state();
        state.soft_dirty |= 0x100008;
        // Declaration count zero: 24 bytes. Its presence tests prefix order.
        let mut declaration = vec![0x5a; 24];
        declaration[12..14].copy_from_slice(&0u16.to_le_bytes());
        let mut region = vec![1, 0, 9, 0, 0, 0, 0xaa, 0xbb];
        region.extend([1, 2, 3, 4]); // third u16 zero: exactly four bytes
        region.extend([2, 0, 2, 0, 3, 0, 0xcc, 0xdd]);
        region.extend([5, 6, 7, 8, 9]); // otherwise second plus third u16
        region.extend([0, 0, 9, 8, 7, 6, 5, 4]); // entire terminator retained
        declaration.extend(&region);
        declaration.extend(&state.payload);
        state.payload = declaration;
        let parsed = state.bindings().unwrap();
        assert_eq!(parsed.prefix.iter().map(|r| r.flag).collect::<Vec<_>>(), vec![0x100000, 8, 0x8000]);
        assert_eq!(parsed.prefix[1].bytes, region);
        assert_eq!(parsed.to_bytes().unwrap(), state.payload);
        for end in 0..state.payload.len() {
            let mut truncated = state.clone();
            truncated.payload.truncate(end);
            assert!(truncated.bindings().is_err(), "binding prefix {end}");
        }
    }
    #[test]
    fn opaque_eight_region_rejects_oversize_record_and_missing_terminator() {
        let mut state = binding_state();
        state.soft_dirty = 8;
        state.hard_dirty = 0;
        state.payload = vec![1, 0, 0xff, 0xff, 0xff, 0xff, 0, 0];
        assert_eq!(state.bindings(), Err(MaterialError::Truncated));
        state.payload = vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4];
        assert_eq!(state.bindings(), Err(MaterialError::Truncated));
    }
    #[test]
    fn every_binding_prefix_unknown_extent_and_missing_terminator_rejected() {
        let source = binding_state();
        for end in 0..source.payload.len() {
            let mut state = source.clone();
            state.payload.truncate(end);
            assert!(state.bindings().is_err(), "binding prefix {end}");
        }
        let mut state = source.clone();
        state.soft_dirty |= 1;
        assert_eq!(state.bindings(), Err(MaterialError::UnsupportedBindingFlags));
        let mut state = source.clone();
        state.hard_delta = 1;
        assert_eq!(state.bindings(), Err(MaterialError::UnsupportedBindingFlags));
        let mut state = source;
        state.payload.extend([0; 4]);
        assert_eq!(state.bindings(), Err(MaterialError::TrailingBytes));
    }
    #[test]
    fn binding_slots_validate_material_cardinality_without_assigning_keys() {
        let states = MaterialStates { states: vec![binding_state()], material_ranges: vec![0..1] };
        let mut link = links().materials.remove(0);
        assert_eq!(states.texture_slots(0, &link).unwrap(), vec![TextureSlot { state_index: 0, stage: 3 }]);
        link.textures.clear();
        assert_eq!(states.texture_slots(0, &link), Err(MaterialError::TextureBindingCount { expected: 0, actual: 1 }));
    }
    #[test]
    fn binding_encoder_rejects_sentinels_and_values_in_disabled_lists() {
        let mut b = binding_state().bindings().unwrap();
        b.textures[1].list_a_flags = 0;
        assert_eq!(b.to_bytes(), Err(MaterialError::InvalidBinding));
        let mut b = binding_state().bindings().unwrap();
        b.groups.as_mut().unwrap()[0].id = u32::MAX;
        assert_eq!(b.to_bytes(), Err(MaterialError::InvalidBinding));
    }
}
