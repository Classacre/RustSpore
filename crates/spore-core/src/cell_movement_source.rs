//! Original source-AI selection and cache getters before steering.
//!
//! `E52770` selects the same source AI variant as the checked definition API.
//! `E67CB0` resolves its style override; `E67D20` copies base speed to four
//! slots. Live definition/cache/actor lookup remains the caller's operation.

use crate::cell_definition::CellDefinition;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellMovementSourceError {
    MissingDefinition,
    MissingCachedStyle,
    MissingCachedSpeed,
    NonFiniteCachedSpeed,
}
impl fmt::Display for CellMovementSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native Cell movement source: {self:?}")
    }
}
impl std::error::Error for CellMovementSourceError {}

/// Complete finite/typed source of the original E67CB0 style getter.
///
/// Zero actor FC returns one without reading a definition or cache. Otherwise
/// E52770 selects easy for raw difficulty0, hard for2 and base for every other
/// code, with FFFFFFFF first-word inheritance. A nonzero source style wins
/// before the cache is inspected. Unknown nonzero style/mask bits are retained.
/// Missing reached references are checked API errors, not native fallbacks.
pub fn resolve_definition_movement_style(
    actor_word_fc: u32,
    definition: Option<CellDefinition<'_>>,
    difficulty_code: i32,
    cached_style: Option<u32>,
) -> Result<u32, CellMovementSourceError> {
    if actor_word_fc == 0 {
        return Ok(1);
    }
    let definition = definition.ok_or(CellMovementSourceError::MissingDefinition)?;
    let selected = definition.ai(definition.selected_ai_variant(difficulty_code));
    let source_style = selected.movement_style();
    if source_style != 0 {
        Ok(source_style)
    } else {
        cached_style.ok_or(CellMovementSourceError::MissingCachedStyle)
    }
}

/// E67D20's four literal-one or cached1258 outputs, without authored-AI scaling.
///
/// Native finite FLD/FSTP binary32 copies are exact under every explicit x87
/// precision/rounding combination, including signed zero and subnormals. The
/// zero-FC path never reads the cache. Nonfinite reached cache words are an
/// explicit typed boundary; no fabricated speed is substituted for them.
pub fn creation_base_speeds(
    actor_word_fc: u32,
    cached_speed: Option<f32>,
) -> Result<[f32; 4], CellMovementSourceError> {
    if actor_word_fc == 0 {
        return Ok([1.; 4]);
    }
    let speed = cached_speed.ok_or(CellMovementSourceError::MissingCachedSpeed)?;
    if !speed.is_finite() {
        return Err(CellMovementSourceError::NonFiniteCachedSpeed);
    }
    Ok([speed; 4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_definition::{self, DEFINITION_SIZE};

    #[test]
    fn native_default_and_source_override_skip_unused_references() {
        assert_eq!(resolve_definition_movement_style(0, None, 2, None), Ok(1));
        assert_eq!(creation_base_speeds(0, None), Ok([1.; 4]));
        assert_eq!(creation_base_speeds(0, Some(f32::NAN)), Ok([1.; 4]));
        let mut raw = [0u8; DEFINITION_SIZE];
        raw[0x194 + 0x10..0x194 + 0x14].copy_from_slice(&0x8000_0008u32.to_le_bytes());
        raw[0x248..0x24c].copy_from_slice(&u32::MAX.to_le_bytes());
        let definition = cell_definition::parse(&raw).unwrap();
        assert_eq!(
            resolve_definition_movement_style(1, Some(definition), 2, None),
            Ok(0x8000_0008)
        );
        assert_eq!(
            resolve_definition_movement_style(1, Some(definition), 0, Some(7)),
            Ok(7)
        );
        assert_eq!(
            resolve_definition_movement_style(1, Some(definition), -1, None),
            Err(CellMovementSourceError::MissingCachedStyle)
        );
    }

    #[test]
    fn finite_speed_copies_preserve_bits_and_reached_errors_are_explicit() {
        for bits in [0, 0x8000_0000, 1, 0x8000_0001, 0x007f_ffff, 0x7f7f_ffff] {
            assert_eq!(
                creation_base_speeds(1, Some(f32::from_bits(bits)))
                    .unwrap()
                    .map(f32::to_bits),
                [bits; 4]
            );
        }
        assert_eq!(
            creation_base_speeds(1, None),
            Err(CellMovementSourceError::MissingCachedSpeed)
        );
        assert_eq!(
            creation_base_speeds(1, Some(f32::INFINITY)),
            Err(CellMovementSourceError::NonFiniteCachedSpeed)
        );
        assert_eq!(
            resolve_definition_movement_style(1, None, 1, Some(1)),
            Err(CellMovementSourceError::MissingDefinition)
        );
    }
}
