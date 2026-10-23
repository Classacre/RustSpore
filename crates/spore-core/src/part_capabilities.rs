//! PartCapabilities (`0x7C19AA7A`, `pctp`) dictionary measured from shipped bytes.
//!
//! Version 3/4 scalar words and string lengths are big endian. Capability IDs
//! pack up to four ASCII bytes in little-endian character order, so their
//! on-disk bytes often look reversed. These IDs identify capabilities, not bones.
//! Deformation parameters remain raw: decoding does not establish retargeting,
//! interpolation, or the effect of a channel on a particular assembled creature.

use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartCapabilitiesError {
    BadMagic,
    UnsupportedVersion(u32),
    Truncated,
    CountOutOfRange,
    TrailingData { remaining: usize },
}

impl fmt::Display for PartCapabilitiesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => f.write_str("bad pctp magic"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported PartCapabilities version {v}"),
            Self::Truncated => f.write_str("truncated PartCapabilities resource"),
            Self::CountOutOfRange => f.write_str("PartCapabilities count exceeds remaining bytes"),
            Self::TrailingData { remaining } => {
                write!(f, "{remaining} trailing PartCapabilities bytes")
            }
        }
    }
}
impl std::error::Error for PartCapabilitiesError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityName {
    /// Exact name bytes, preserving unrecognised encodings and embedded bytes.
    pub name_bytes: Vec<u8>,
    pub capability: u32,
}

impl CapabilityName {
    pub fn name(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.name_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRemap {
    /// Key in the remap table. Named entries match lowercase FNV-1 hashes;
    /// the table also includes aliases absent from the name list.
    pub key: u32,
    pub capability: u32,
    /// Retained mapping index; not an assembled-creature row or RW4 bone ID.
    pub index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityAggregate {
    pub capability: u32,
    /// Source-listed relationships; no implicit resolution direction is applied.
    pub members: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeformChannel {
    pub id: u32,
    /// Exact remaining words from this 16-byte entry. The first two words have
    /// finite float-shaped values in the corpus, the last has values 0/1/2.
    /// Their runtime semantics are not established by parsing alone.
    pub raw_parameters: [u32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDeform {
    pub capability: u32,
    pub channels: Vec<DeformChannel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartCapabilities {
    pub version: u32,
    /// Version 4 adds this float-shaped priority word. Preserve its exact bits.
    pub priority_bits: Option<u32>,
    pub names: Vec<CapabilityName>,
    pub remaps: Vec<CapabilityRemap>,
    pub aggregates: Vec<CapabilityAggregate>,
    pub deforms: Vec<CapabilityDeform>,
}

impl PartCapabilities {
    pub fn priority(&self) -> Option<f32> {
        self.priority_bits.map(f32::from_bits)
    }

    /// Return every matching descriptor rather than silently choosing a duplicate.
    pub fn deforms_for(&self, capability: u32) -> impl Iterator<Item = &CapabilityDeform> {
        self.deforms
            .iter()
            .filter(move |d| d.capability == capability)
    }
}

/// Capability character bytes before any display/encoding interpretation.
pub fn capability_bytes(id: u32) -> [u8; 4] {
    id.to_le_bytes()
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], PartCapabilitiesError> {
        let end = self
            .at
            .checked_add(count)
            .ok_or(PartCapabilitiesError::Truncated)?;
        let value = self
            .bytes
            .get(self.at..end)
            .ok_or(PartCapabilitiesError::Truncated)?;
        self.at = end;
        Ok(value)
    }
    fn word(&mut self) -> Result<u32, PartCapabilitiesError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, minimum_size: usize) -> Result<usize, PartCapabilitiesError> {
        let count = self.word()? as usize;
        if count > (self.bytes.len() - self.at) / minimum_size {
            return Err(PartCapabilitiesError::CountOutOfRange);
        }
        Ok(count)
    }
}

pub fn parse(bytes: &[u8]) -> Result<PartCapabilities, PartCapabilitiesError> {
    let mut r = Reader { bytes, at: 0 };
    if r.take(4)? != b"pctp" {
        return Err(PartCapabilitiesError::BadMagic);
    }
    let version = r.word()?;
    if !matches!(version, 3 | 4) {
        return Err(PartCapabilitiesError::UnsupportedVersion(version));
    }
    let priority_bits = if version == 4 { Some(r.word()?) } else { None };
    let count = r.count(8)?;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        let length = r.word()? as usize;
        let name_bytes = r.take(length)?.to_vec();
        names.push(CapabilityName {
            name_bytes,
            capability: r.word()?,
        });
    }
    let count = r.count(12)?;
    let mut remaps = Vec::with_capacity(count);
    for _ in 0..count {
        remaps.push(CapabilityRemap {
            key: r.word()?,
            capability: r.word()?,
            index: r.word()?,
        });
    }
    let count = r.count(8)?;
    let mut aggregates = Vec::with_capacity(count);
    for _ in 0..count {
        let capability = r.word()?;
        let count = r.count(4)?;
        let mut members = Vec::with_capacity(count);
        for _ in 0..count {
            members.push(r.word()?);
        }
        aggregates.push(CapabilityAggregate {
            capability,
            members,
        });
    }
    let count = r.count(8)?;
    let mut deforms = Vec::with_capacity(count);
    for _ in 0..count {
        let capability = r.word()?;
        let count = r.count(16)?;
        let mut channels = Vec::with_capacity(count);
        for _ in 0..count {
            channels.push(DeformChannel {
                id: r.word()?,
                raw_parameters: [r.word()?, r.word()?, r.word()?],
            });
        }
        deforms.push(CapabilityDeform {
            capability,
            channels,
        });
    }
    if r.at != bytes.len() {
        return Err(PartCapabilitiesError::TrailingData {
            remaining: bytes.len() - r.at,
        });
    }
    Ok(PartCapabilities {
        version,
        priority_bits,
        names,
        remaps,
        aggregates,
        deforms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(out: &mut Vec<u8>, value: u32) {
        out.extend(value.to_be_bytes());
    }
    fn fixture(version: u32) -> Vec<u8> {
        let mut out = b"pctp".to_vec();
        word(&mut out, version);
        if version == 4 {
            word(&mut out, 2.0f32.to_bits());
        }
        word(&mut out, 1);
        word(&mut out, 4);
        out.extend(b"test");
        word(&mut out, 0x74736574);
        word(&mut out, 1);
        word(&mut out, 0x01234567);
        word(&mut out, 0x74736574);
        word(&mut out, 7);
        word(&mut out, 1);
        word(&mut out, 0x74736574);
        word(&mut out, 2);
        word(&mut out, 11);
        word(&mut out, 12);
        word(&mut out, 1);
        word(&mut out, 0x74736574);
        word(&mut out, 1);
        word(&mut out, 0x10203040);
        word(&mut out, 0.5f32.to_bits());
        word(&mut out, 1.0f32.to_bits());
        word(&mut out, 0xDEADBEEF);
        out
    }

    #[test]
    fn both_measured_versions_decode_exactly() {
        for version in [3, 4] {
            let p = parse(&fixture(version)).unwrap();
            assert_eq!(p.version, version);
            assert_eq!(p.priority(), if version == 4 { Some(2.0) } else { None });
            assert_eq!(p.names[0].name(), "test");
            assert_eq!(capability_bytes(p.names[0].capability), *b"test");
            assert_eq!(
                p.remaps[0],
                CapabilityRemap {
                    key: 0x01234567,
                    capability: 0x74736574,
                    index: 7
                }
            );
            assert_eq!(p.aggregates[0].members, [11, 12]);
            assert_eq!(
                p.deforms[0].channels[0].raw_parameters,
                [0.5f32.to_bits(), 1.0f32.to_bits(), 0xDEADBEEF]
            );
            assert_eq!(p.deforms_for(0x74736574).count(), 1);
            assert_eq!(p.deforms_for(0).count(), 0);
        }
    }

    #[test]
    fn every_truncated_prefix_is_rejected_without_panicking() {
        for version in [3, 4] {
            let bytes = fixture(version);
            for end in 0..bytes.len() {
                assert!(parse(&bytes[..end]).is_err(), "prefix {end}");
            }
        }
    }

    #[test]
    fn huge_counts_and_string_lengths_are_bounded_before_allocation() {
        let base = fixture(3);
        // Names, name length, remaps, aggregates, aggregate members, deforms,
        // deform entries; every field is independently checked against input.
        for at in [8, 12, 24, 40, 48, 60, 68] {
            let mut bytes = base.clone();
            bytes[at..at + 4].copy_from_slice(&u32::MAX.to_be_bytes());
            assert!(parse(&bytes).is_err(), "count field {at}");
        }
    }

    #[test]
    fn bad_magic_version_and_trailing_bytes_are_explicit() {
        let mut bytes = fixture(3);
        bytes[0] = 0;
        assert_eq!(parse(&bytes), Err(PartCapabilitiesError::BadMagic));
        let mut bytes = fixture(3);
        bytes[4..8].copy_from_slice(&2u32.to_be_bytes());
        assert_eq!(
            parse(&bytes),
            Err(PartCapabilitiesError::UnsupportedVersion(2))
        );
        let mut bytes = fixture(3);
        bytes.push(0);
        assert_eq!(
            parse(&bytes),
            Err(PartCapabilitiesError::TrailingData { remaining: 1 })
        );
    }

    #[test]
    fn unknown_bits_names_and_duplicate_descriptors_are_preserved() {
        let mut bytes = fixture(4);
        bytes[8..12].copy_from_slice(&0x7FC01234u32.to_be_bytes());
        bytes[20] = 0xFF;
        let p = parse(&bytes).unwrap();
        assert_eq!(p.priority_bits, Some(0x7FC01234));
        assert_eq!(p.names[0].name_bytes[0], 0xFF);
        assert_eq!(p.deforms[0].channels[0].raw_parameters[2], 0xDEADBEEF);
        let mut p = p;
        p.deforms.push(p.deforms[0].clone());
        assert_eq!(p.deforms_for(0x74736574).count(), 2);
    }
}
