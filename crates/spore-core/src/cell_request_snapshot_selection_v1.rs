//! Bounded composition of the E6CC30 admission and weighted suffix models.
//!
//! The actor snapshot, predicate outcomes, and committed binary32 draw are
//! caller-supplied. This does not obtain RNG state, inspect a live actor, or
//! establish a runtime clip. Source offsets below are hexadecimal; see the
//! q783 E6CC30 dossier and `re/cell-actor-animation-request-trace-v1.md`.

use crate::{cell_anim_state_table_v1 as table, cell_request_admission_projection_v1 as admission, cell_request_weighted_suffix_v1 as suffix};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Predicates {
    pub predicate1: bool,
    pub predicate2: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    PredicateCount { rows: usize, predicates: usize },
    Admission(admission::Error),
    Suffix(suffix::Error),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SelectionProjection {
    /// Includes fallback duplicates in the order used by the suffix.
    pub admission: admission::Projection,
    /// The returned request key and output bytes; `Unchanged` distinguishes
    /// no selection from selection of a row whose request key is zero.
    pub selection: suffix::Selection,
}

/// Project a bounded set of raw table records using immutable caller input.
/// `predicates[i]` supplies the E5C300/E5C2B0 outcomes for `rows[i]`.
/// `draw_bits` is already committed; this function does not compute it from
/// the sum or establish that it was attainable from the source RNG. Source
/// output locations are modeled as distinct and non-aliasing. The constituent
/// modules enforce their architecture, floating
/// point controls, capacity and numeric input policies.
pub fn select_snapshot(
    rows: &[table::Record],
    predicates: &[Predicates],
    snapshot: &admission::Snapshot<'_>,
    draw_bits: u32,
) -> Result<SelectionProjection, Error> {
    if rows.len() != predicates.len() {
        return Err(Error::PredicateCount { rows: rows.len(), predicates: predicates.len() });
    }
    // Avoid constructing an arbitrarily large intermediate before the bound.
    if rows.len() > table::RECORD_COUNT {
        return Err(Error::Admission(admission::Error::TooManyRows));
    }
    let input: Vec<_> = rows.iter().zip(predicates).map(|(row, p)| admission::Row {
        state: row.state_word(),
        mode: row.words[3],
        part_code: row.words[4],
        weight_bits: row.words[2],
        predicate1: p.predicate1,
        predicate2: p.predicate2,
    }).collect();
    let projection = admission::project(&input, snapshot).map_err(Error::Admission)?;
    let admitted: Vec<_> = projection.candidates.iter().map(|&i| {
        let row = &rows[i];
        suffix::AdmittedRow {
            key: row.request_key(),
            weight_bits: row.words[2],
            first_byte: row.words[5] as u8,  // row +0x14
            second_byte: row.words[8] as u8, // row +0x20
        }
    }).collect();
    let selection = suffix::select(&admitted, draw_bits).map_err(Error::Suffix)?;
    Ok(SelectionProjection { admission: projection, selection })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(key: u32, part: u32, weight: f32) -> table::Record {
        table::Record { words: [7, key, weight.to_bits(), 2, part, 0xaabb_cc12, 0, 0, 0xddee_ff34] }
    }
    fn pred(p1: bool, p2: bool) -> Predicates { Predicates { predicate1: p1, predicate2: p2 } }
    fn snapshot(parts: &[u32]) -> admission::Snapshot<'_> {
        admission::Snapshot { state: 7, actor_mode: 0, context_flags: None, parts }
    }
    #[test]
    fn selection_uses_raw_words_and_admitted_order() {
        let rows = [row(11, 1, 1.0), row(22, 1, 2.0)];
        let result = select_snapshot(&rows, &[pred(true, true); 2], &snapshot(&[]), 1.5f32.to_bits()).unwrap();
        assert_eq!(result.admission.candidates, vec![0, 1]);
        assert_eq!(result.selection, suffix::Selection { key: 22, first_byte: 0x12, second_byte: suffix::SecondByte::Written(0x34) });
    }
    #[test]
    fn fallback_bypasses_predicates_and_preserves_zero_weight_duplicates() {
        let rows = [row(10, 0, 0.0), row(20, 1, 0.0), row(30, 0, 2.0)];
        let predicates = [pred(true, true), pred(true, true), pred(false, false)];
        let result = select_snapshot(&rows, &predicates, &snapshot(&[0]), 1.0f32.to_bits()).unwrap();
        assert!(result.admission.fallback_used);
        assert_eq!(result.admission.candidates, vec![0, 1, 0, 2]);
        assert_eq!(result.admission.sum_bits, 2.0f32.to_bits());
        assert_eq!(result.selection.key, 30);
    }
    #[test]
    fn zero_draw_selects_first_zero_weight_candidate_even_with_duplicates() {
        let result = select_snapshot(&[row(0, 0, 0.0)], &[pred(true, true)], &snapshot(&[0]), 0).unwrap();
        assert_eq!(result.admission.candidates, vec![0, 0]);
        assert_eq!(result.selection, suffix::Selection { key: 0, first_byte: 0x12, second_byte: suffix::SecondByte::Written(0x34) });
    }
    #[test]
    fn no_selection_preserves_second_output() {
        let result = select_snapshot(&[row(4, 1, 1.0)], &[pred(true, true)], &snapshot(&[]), 2.0f32.to_bits()).unwrap();
        assert_eq!(result.selection, suffix::Selection { key: 0, first_byte: 0, second_byte: suffix::SecondByte::Unchanged });
    }
    #[test]
    fn propagates_bounds_admission_and_suffix_errors() {
        let rows = [row(4, 1, 1.0)];
        assert_eq!(select_snapshot(&rows, &[], &snapshot(&[]), 0), Err(Error::PredicateCount { rows: 1, predicates: 0 }));
        assert_eq!(select_snapshot(&rows, &[pred(true, true)], &snapshot(&[]), f32::NAN.to_bits()), Err(Error::Suffix(suffix::Error::InvalidDraw)));
        assert_eq!(select_snapshot(&[row(4, 1, -1.0)], &[pred(true, true)], &snapshot(&[]), 0), Err(Error::Admission(admission::Error::InvalidWeight)));
        let many = vec![row(1, 1, 1.0); 132];
        assert_eq!(select_snapshot(&many, &vec![pred(true, true); 132], &snapshot(&[]), 0), Err(Error::Admission(admission::Error::TooManyRows)));
        let dup = vec![row(1, 0, 0.0); 131];
        assert_eq!(select_snapshot(&dup, &vec![pred(true, true); 131], &snapshot(&[0]), 0), Err(Error::Admission(admission::Error::CandidateCapacity)));
    }
}
