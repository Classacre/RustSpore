//! Bounded ordinary primary-root record constructor (native 009A0E30).
//!
//! Explicit prepared state only. This does not admit clips or associate actors.
//! Mirror selection uses the two source-reached integer branches. Secondary selection is the
//! count-zero override path; growth, pseudo records and empty fallback are excluded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Descriptor {
    pub flags: u32,
    pub tag: u32,
    pub slot: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub primary_flags: u32,
    pub selected_root: u32,
    pub selected_ordinal: u32,
    pub channel_byte: u8,
    pub channel_index: u8,
    pub words: [u32; 2],
    pub mirror_argument: bool,
    pub owner_byte: u8,
    pub metadata_ac: u32,
    /// Native D10 is zero; D14 is -1 or this byte.
    pub secondary_override: Option<u8>,
    pub descriptors: Vec<Descriptor>,
    pub claims: [u8; 9],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    pub claims: [u8; 9],
    pub record: Option<[u32; 4]>,
    pub root_count_delta: u32,
    pub secondary_count_delta: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    UnsupportedSelector,
    DescriptorLimit,
    PseudoRecord,
    EmptyFallback,
    UnsupportedMirrorMode,
}

/// Construct a record with capacity already available. Errors leave input intact.
pub fn construct(input: &Input) -> Result<Projection, Error> {
    if !matches!(input.primary_flags, 1 | 5) || input.selected_root != 0 {
        return Err(Error::UnsupportedSelector);
    }
    if input.descriptors.len() > 24 {
        return Err(Error::DescriptorLimit);
    }
    // Exclude the pseudo descriptor even when disabled, so its separate path
    // cannot silently enter the supported API as metadata changes.
    if input.descriptors.iter().any(|d| d.flags & 15 == 3 && d.tag == 0x70e47545) {
        return Err(Error::PseudoRecord);
    }
    let mirror = match input.metadata_ac & 0x32 {
        0 => input.mirror_argument,
        0x20 => false,
        _ => return Err(Error::UnsupportedMirrorMode),
    };
    let mut claims = input.claims;
    let mut mask = 0u32;
    let mut any_admitted = false;
    let mut flags = if mirror { 2u8 } else { 0 };
    for (index, descriptor) in input.descriptors.iter().enumerate() {
        let Ok(slot) = usize::try_from(descriptor.slot) else { continue };
        if slot >= claims.len() || (claims[slot] != 255 && claims[slot] != input.owner_byte) {
            continue;
        }
        // Native sets this even for a disabled descriptor after slot/claim admission.
        any_admitted = true;
        if descriptor.flags & 0x10 == 0 {
            continue;
        }
        claims[slot] = input.owner_byte;
        mask |= 1u32 << index;
        if descriptor.flags & 15 == 1 {
            flags |= 0x20;
        }
    }
    if mask == 0 && any_admitted && input.metadata_ac & 8 != 0 {
        return Err(Error::EmptyFallback);
    }
    let secondary = input.secondary_override.unwrap_or(255);
    if input.secondary_override.is_some() {
        flags |= 0x10;
    }
    let record = (mask != 0).then_some([
        input.words[0], input.words[1], mask | (u32::from(input.channel_index) << 24),
        u32::from_le_bytes([0, input.channel_byte, secondary, flags]),
    ]);
    Ok(Projection {
        claims, root_count_delta: u32::from(record.is_some()),
        secondary_count_delta: u32::from(record.is_some() && secondary != 255), record,
    })
}
