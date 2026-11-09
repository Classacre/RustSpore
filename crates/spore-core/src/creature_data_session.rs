//! Reversible edits to an existing, complete CreatureData payload.
//!
//! This keeps the serialized source bytes as the document and records only the
//! 48 transform bytes changed by each edit. It does not regenerate a baked
//! model, validate editor attachment rules, or produce a playable creation.

use crate::creature_data::{self, CreatureDataError, PART_ROW_SIZE};
use crate::creature_data_edit::{patch_rest_transform, EditError, RestTransform};
use std::{fmt, ops::Range};

/// Maximum payload accepted by this local editing session.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
/// Maximum count of stored transform edits, independently of payload size.
pub const MAX_HISTORY_EDITS: usize = 4096;
const TRANSFORM_OFFSET: usize = 48;
const TRANSFORM_BYTES: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    PayloadTooLarge,
    InvalidSource(CreatureDataError),
    TrailingBytes,
    InvalidRowExtent,
    InvalidHistoryLimit,
    PartOutOfRange { index: usize, count: usize },
    Edit(EditError),
    RevisionExhausted,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge => write!(f, "CreatureData exceeds the session size limit"),
            Self::InvalidSource(error) => write!(f, "invalid CreatureData: {error}"),
            Self::TrailingBytes => write!(f, "CreatureData has unconsumed trailing bytes"),
            Self::InvalidRowExtent => write!(f, "CreatureData row extent is inconsistent"),
            Self::InvalidHistoryLimit => {
                write!(f, "history limit must be in 1..={MAX_HISTORY_EDITS}")
            }
            Self::PartOutOfRange { index, count } => {
                write!(f, "part index {index} is outside {count} source rows")
            }
            Self::Edit(error) => write!(f, "{error}"),
            Self::RevisionExhausted => write!(f, "edit revision exhausted"),
        }
    }
}
impl std::error::Error for SessionError {}

#[derive(Debug, Clone)]
struct Change {
    range: Range<usize>,
    before: [u8; TRANSFORM_BYTES],
    after: [u8; TRANSFORM_BYTES],
}

/// An in-memory source-byte document; history and revision are session-local.
#[derive(Debug, Clone)]
pub struct CreatureDataSession {
    bytes: Vec<u8>,
    row_start: usize,
    part_count: usize,
    history_limit: usize,
    past: Vec<Change>,
    future: Vec<Change>,
    revision: u64,
}

impl CreatureDataSession {
    /// Admit a complete version-9/10 payload without modifying it.
    pub fn new(bytes: Vec<u8>, history_limit: usize) -> Result<Self, SessionError> {
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(SessionError::PayloadTooLarge);
        }
        if !(1..=MAX_HISTORY_EDITS).contains(&history_limit) {
            return Err(SessionError::InvalidHistoryLimit);
        }
        let parsed = creature_data::parse(&bytes).map_err(SessionError::InvalidSource)?;
        if !parsed.exact || parsed.consumed != bytes.len() {
            return Err(SessionError::TrailingBytes);
        }
        let counts_bytes = if parsed.version == 9 {
            12usize
        } else {
            16usize
        };
        let row_start = 8usize
            .checked_add(counts_bytes)
            .and_then(|value| value.checked_add(parsed.compressed_buffer.len()))
            .ok_or(SessionError::InvalidRowExtent)?;
        let end = row_start
            .checked_add(parsed.data0.len())
            .ok_or(SessionError::InvalidRowExtent)?;
        let expected_len = parsed
            .parts
            .len()
            .checked_mul(PART_ROW_SIZE)
            .ok_or(SessionError::InvalidRowExtent)?;
        if expected_len != parsed.data0.len()
            || bytes.get(row_start..end) != Some(parsed.data0.as_slice())
        {
            return Err(SessionError::InvalidRowExtent);
        }
        let part_count = parsed.parts.len();
        Ok(Self {
            bytes,
            row_start,
            part_count,
            history_limit,
            past: Vec::new(),
            future: Vec::new(),
            revision: 0,
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn part_count(&self) -> usize {
        self.part_count
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn undo_len(&self) -> usize {
        self.past.len()
    }
    pub fn redo_len(&self) -> usize {
        self.future.len()
    }
    pub fn history_limit(&self) -> usize {
        self.history_limit
    }

    fn range(&self, ordinal: usize) -> Result<Range<usize>, SessionError> {
        if ordinal >= self.part_count {
            return Err(SessionError::PartOutOfRange {
                index: ordinal,
                count: self.part_count,
            });
        }
        let start = ordinal
            .checked_mul(PART_ROW_SIZE)
            .and_then(|n| self.row_start.checked_add(n))
            .and_then(|n| n.checked_add(TRANSFORM_OFFSET))
            .ok_or(SessionError::InvalidRowExtent)?;
        let end = start
            .checked_add(TRANSFORM_BYTES)
            .ok_or(SessionError::InvalidRowExtent)?;
        if self.bytes.get(start..end).is_none() {
            return Err(SessionError::InvalidRowExtent);
        }
        Ok(start..end)
    }

    /// Read the selected row's absolute source transform, without normalizing it.
    pub fn transform(&self, ordinal: usize) -> Result<RestTransform, SessionError> {
        let range = self.range(ordinal)?;
        let data = &self.bytes[range];
        let word =
            |index: usize| f32::from_le_bytes(data[index * 4..index * 4 + 4].try_into().unwrap());
        Ok(RestTransform {
            rotation: std::array::from_fn(|row| std::array::from_fn(|col| word(row * 3 + col))),
            translation: std::array::from_fn(|axis| word(9 + axis)),
        })
    }

    /// Replace one row's transform. Invalid edits are atomic. Byte-identical
    /// edits return false and leave the revision and redo branch untouched.
    pub fn set_transform(
        &mut self,
        ordinal: usize,
        transform: RestTransform,
    ) -> Result<bool, SessionError> {
        let range = self.range(ordinal)?;
        let candidate =
            patch_rest_transform(&self.bytes, ordinal, transform).map_err(SessionError::Edit)?;
        let before: [u8; TRANSFORM_BYTES] = self.bytes[range.clone()].try_into().unwrap();
        let after: [u8; TRANSFORM_BYTES] = candidate[range.clone()].try_into().unwrap();
        if before == after {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(SessionError::RevisionExhausted)?;
        self.push_past(Change {
            range,
            before,
            after,
        });
        self.future.clear();
        self.bytes = candidate;
        self.revision = revision;
        Ok(true)
    }

    /// Undo the most recent transform, preserving its exact original bits.
    pub fn undo(&mut self) -> Result<bool, SessionError> {
        if self.past.is_empty() {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(SessionError::RevisionExhausted)?;
        let change = self.past.pop().unwrap();
        self.bytes[change.range.clone()].copy_from_slice(&change.before);
        self.future.push(change);
        self.revision = revision;
        Ok(true)
    }

    /// Redo the most recently undone transform.
    pub fn redo(&mut self) -> Result<bool, SessionError> {
        if self.future.is_empty() {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(SessionError::RevisionExhausted)?;
        let change = self.future.pop().unwrap();
        self.bytes[change.range.clone()].copy_from_slice(&change.after);
        self.push_past(change);
        self.revision = revision;
        Ok(true)
    }

    fn push_past(&mut self, change: Change) {
        self.past.push(change);
        if self.past.len() > self.history_limit {
            self.past.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(version: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&creature_data::HEADER.to_le_bytes());
        bytes.extend_from_slice(&version.to_le_bytes());
        for count in [2u32, 0, 0] {
            bytes.extend_from_slice(&count.to_le_bytes());
        }
        if version == 10 {
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        bytes.extend_from_slice(&3u16.to_le_bytes());
        bytes.extend_from_slice(&[0, 0xfe, 127]);
        for index in 0..2u16 {
            let mut row = [0u8; PART_ROW_SIZE];
            row[0..2].copy_from_slice(&index.to_le_bytes());
            row[4..8].copy_from_slice(&[0xff; 4]);
            row[20..24].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
            bytes.extend_from_slice(&row);
        }
        bytes.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes
    }

    fn changed(session: &CreatureDataSession, ordinal: usize, value: f32) -> RestTransform {
        let mut transform = session.transform(ordinal).unwrap();
        transform.translation[0] = value;
        transform
    }

    #[test]
    fn undo_redo_preserves_exact_bytes_for_multiple_rows_and_versions() {
        for version in [9, 10] {
            let original = source(version);
            let mut session = CreatureDataSession::new(original.clone(), 8).unwrap();
            assert!(session
                .set_transform(0, changed(&session, 0, 3.25))
                .unwrap());
            let first = session.bytes().to_vec();
            assert!(session
                .set_transform(1, changed(&session, 1, -7.0))
                .unwrap());
            let second = session.bytes().to_vec();
            assert!(session.undo().unwrap());
            assert_eq!(session.bytes(), first);
            assert!(session.undo().unwrap());
            assert_eq!(session.bytes(), original);
            assert!(!session.undo().unwrap());
            assert!(session.redo().unwrap());
            assert_eq!(session.bytes(), first);
            assert!(session.redo().unwrap());
            assert_eq!(session.bytes(), second);
            assert_eq!(session.revision(), 6);
        }
    }

    #[test]
    fn invalid_and_byte_identical_edits_preserve_redo_and_revision() {
        let mut session = CreatureDataSession::new(source(9), 8).unwrap();
        assert!(session.set_transform(0, changed(&session, 0, 1.0)).unwrap());
        session.undo().unwrap();
        let before = session.bytes().to_vec();
        let revision = session.revision();
        assert!(!session
            .set_transform(0, session.transform(0).unwrap())
            .unwrap());
        let mut bad = session.transform(0).unwrap();
        bad.translation[1] = f32::NAN;
        assert!(session.set_transform(0, bad).is_err());
        assert!(session.set_transform(9, bad).is_err());
        assert_eq!(session.bytes(), before);
        assert_eq!(session.revision(), revision);
        assert_eq!(session.redo_len(), 1);
        assert!(session.redo().unwrap());
    }

    #[test]
    fn signed_zero_is_an_edit_and_new_branch_discards_redo() {
        let mut session = CreatureDataSession::new(source(10), 2).unwrap();
        assert!(session
            .set_transform(0, changed(&session, 0, -0.0))
            .unwrap());
        assert_eq!(
            session.transform(0).unwrap().translation[0].to_bits(),
            (-0.0f32).to_bits()
        );
        session.undo().unwrap();
        assert!(session.set_transform(1, changed(&session, 1, 2.0)).unwrap());
        assert_eq!(session.redo_len(), 0);
        assert!(!session.redo().unwrap());
    }

    #[test]
    fn multi_chunk_v10_with_owned_arrays_keeps_every_unedited_byte() {
        let mut source = Vec::new();
        source.extend_from_slice(&creature_data::HEADER.to_le_bytes());
        source.extend_from_slice(&10u32.to_le_bytes());
        for count in [2u32, 1, 1, 1] {
            source.extend_from_slice(&count.to_le_bytes());
        }
        source.extend_from_slice(&1u16.to_le_bytes());
        source.push(0);
        source.extend_from_slice(&2u16.to_le_bytes());
        source.extend_from_slice(&[0xfe, 127]);
        let row_start = source.len();
        for index in 0..2u16 {
            let mut row = [0u8; PART_ROW_SIZE];
            row[0..2].copy_from_slice(&index.to_le_bytes());
            row[4..8].copy_from_slice(&[0xff; 4]);
            if index == 0 {
                row[16] = 1;
                row[17] = 1;
            } else {
                row[12..14].copy_from_slice(&1u16.to_le_bytes());
                row[14..16].copy_from_slice(&1u16.to_le_bytes());
            }
            source.extend_from_slice(&row);
        }
        source.extend_from_slice(b"ABCD");
        source.push(7);
        source.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        source.extend_from_slice(&0.25f32.to_le_bytes());
        source.extend_from_slice(&0.75f32.to_le_bytes());
        source.extend_from_slice(&0x8765_4321u32.to_le_bytes());
        source.extend_from_slice(&[0xa5; 0x38]);
        let mut session = CreatureDataSession::new(source.clone(), 4).unwrap();
        assert!(session.set_transform(1, changed(&session, 1, 5.0)).unwrap());
        let range_start = row_start + PART_ROW_SIZE + TRANSFORM_OFFSET;
        let range_end = range_start + TRANSFORM_BYTES;
        assert_eq!(&session.bytes()[..range_start], &source[..range_start]);
        assert_eq!(&session.bytes()[range_end..], &source[range_end..]);
        let parsed = creature_data::parse(session.bytes()).unwrap();
        assert_eq!(
            (
                parsed.capabilities.len(),
                parsed.morphs.len(),
                parsed.data3.len()
            ),
            (1, 1, 0x38)
        );
        session.undo().unwrap();
        assert_eq!(session.bytes(), source);
    }

    #[test]
    fn unchanged_nonfinite_source_transform_is_rejected_without_mutation() {
        let mut original = source(9);
        let row_start = 8 + 12 + 5;
        original[row_start + TRANSFORM_OFFSET..row_start + TRANSFORM_OFFSET + 4]
            .copy_from_slice(&f32::NAN.to_le_bytes());
        let mut session = CreatureDataSession::new(original.clone(), 4).unwrap();
        let existing = session.transform(0).unwrap();
        assert!(existing.rotation[0][0].is_nan());
        assert_eq!(
            session.set_transform(0, existing),
            Err(SessionError::Edit(EditError::NonFiniteTransform))
        );
        assert_eq!(session.bytes(), original);
        assert_eq!(
            (session.revision(), session.undo_len(), session.redo_len()),
            (0, 0, 0)
        );
    }

    #[test]
    fn history_is_bounded_and_revision_overflow_is_transactional() {
        let mut session = CreatureDataSession::new(source(9), 2).unwrap();
        for value in [1.0, 2.0, 3.0] {
            assert!(session
                .set_transform(0, changed(&session, 0, value))
                .unwrap());
        }
        assert_eq!(session.undo_len(), 2);
        session.undo().unwrap();
        session.undo().unwrap();
        assert!(!session.undo().unwrap());
        assert_eq!(session.transform(0).unwrap().translation[0], 1.0);
        session.revision = u64::MAX;
        let before = session.bytes().to_vec();
        let future = session.redo_len();
        assert_eq!(session.redo(), Err(SessionError::RevisionExhausted));
        assert_eq!(
            session.set_transform(0, changed(&session, 0, 5.0)),
            Err(SessionError::RevisionExhausted)
        );
        assert_eq!(session.bytes(), before);
        assert_eq!(session.redo_len(), future);
    }

    #[test]
    fn malformed_source_and_invalid_limits_are_rejected() {
        assert!(matches!(
            CreatureDataSession::new(source(9), 0),
            Err(SessionError::InvalidHistoryLimit)
        ));
        assert!(matches!(
            CreatureDataSession::new(source(9), MAX_HISTORY_EDITS + 1),
            Err(SessionError::InvalidHistoryLimit)
        ));
        let mut trailing = source(9);
        trailing.push(0);
        assert!(matches!(
            CreatureDataSession::new(trailing, 1),
            Err(SessionError::TrailingBytes)
        ));
        assert!(matches!(
            CreatureDataSession::new(vec![], 1),
            Err(SessionError::InvalidSource(_))
        ));
    }

    #[test]
    #[ignore = "requires an explicitly supplied local CreatureData payload"]
    fn installed_source_edit_undo_and_redo_are_byte_exact() {
        let path = std::env::var_os("RUSTSPORE_CREATUREDATA_FIXTURE").expect("fixture path");
        let original = std::fs::read(path).unwrap();
        let mut session = CreatureDataSession::new(original.clone(), 4).unwrap();
        assert_eq!(session.part_count(), 10);
        let mut moved = session.transform(9).unwrap();
        moved.translation[0] += 1.0;
        assert!(session.set_transform(9, moved).unwrap());
        let edited = session.bytes().to_vec();
        session.undo().unwrap();
        assert_eq!(session.bytes(), original);
        session.redo().unwrap();
        assert_eq!(session.bytes(), edited);
        assert_eq!(
            creature_data::parse(&edited).unwrap().parts[9].translation,
            moved.translation
        );
    }
}
