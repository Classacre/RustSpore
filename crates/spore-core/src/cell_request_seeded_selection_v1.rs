//! Source-derived, bounded E6CC30 snapshot selection with a supplied RNG state.
//!
//! Static q783: E6CF18 loads the committed binary32 candidate sum, stores it
//! exactly as binary64, calls 935D40, multiplies in x87, clamps against the sum
//! then zero, and E6D162 stores binary32 using x87 rounding. No binary32 RNG
//! intermediate exists. This candidate has not been tested
//! against the original executable. Inputs are synthetic/supplied snapshots;
//! neither the live RNG state nor actual actor outcomes are established here.
//!
//! Only nonzero seeds and 53/64-bit x87 precision are supported. The RNG
//! fraction abstraction loses negative zero on exact cancellation. Therefore
//! numerator zero with round-down is rejected, even though the numeric
//! selection could be the same. Finite, nonnegative weights and the SSE MXCSR
//! policy are enforced by admission and suffix. Status flags, unmasked x87
//! exceptions, live concurrency and the zero-seed RDTSC path are not modeled.
//! Like the source, success consumes one RNG word even for no candidates.
//! Unlike the source, this API is transactional on model errors: it commits
//! the supplied RNG only after all steps succeed; validation and error paths
//! are bounded-model policy, not source behavior.

use crate::{
    cell_anim_state_table_v1 as table,
    cell_request_admission_projection_v1 as admission,
    cell_request_rng_v1 as rng,
    cell_request_snapshot_selection_v1 as snapshot_selection,
    cell_request_weighted_suffix_v1 as suffix,
    native_x87::{Context, Extended, Precision, Rounding, X87Error},
};
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    PredicateCount { rows: usize, predicates: usize },
    Admission(admission::Error),
    Suffix(suffix::Error),
    UnsupportedPrecision,
    UnrepresentedNegativeZero,
    X87(X87Error),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SeededSelection {
    pub admission: admission::Projection,
    pub fraction: rng::Fraction,
    pub draw_bits: u32,
    pub selection: suffix::Selection,
}

/// Advance a nonzero-state RNG after successful projection. `context` is an
/// explicit arithmetic policy, not a claim about the original process CW.
pub fn select_snapshot_seeded(
    rows: &[table::Record],
    predicates: &[snapshot_selection::Predicates],
    snapshot: &admission::Snapshot<'_>,
    rng: &mut rng::Rng,
    context: Context,
) -> Result<SeededSelection, Error> {
    if rows.len() != predicates.len() {
        return Err(Error::PredicateCount { rows: rows.len(), predicates: predicates.len() });
    }
    if rows.len() > table::RECORD_COUNT {
        return Err(Error::Admission(admission::Error::TooManyRows));
    }
    if context.precision == Precision::Bits24 { return Err(Error::UnsupportedPrecision); }
    let input: Vec<_> = rows.iter().zip(predicates).map(|(row, p)| admission::Row {
        state: row.state_word(), mode: row.words[3], part_code: row.words[4],
        weight_bits: row.words[2], predicate1: p.predicate1, predicate2: p.predicate2,
    }).collect();
    let admitted = admission::project(&input, snapshot).map_err(Error::Admission)?;
    let mut next = *rng;
    let fraction = next.next_fraction();
    if fraction.numerator == 0 && context.rounding == Rounding::Down {
        return Err(Error::UnrepresentedNegativeZero);
    }
    let sum = Extended::from_f32(f32::from_bits(admitted.sum_bits)).map_err(Error::X87)?;
    let fraction_ext = Extended::from_f64(fraction.as_f64()).map_err(Error::X87)?;
    let product = context.mul(sum, fraction_ext).map_err(Error::X87)?;
    // E6CF32 selects sum when sum <= product. Otherwise E6CF3E selects
    // zero when zero > product. Retain the chosen x87 value until FSTP.
    let chosen = if context.compare(sum, product).map_err(Error::X87)? != Ordering::Greater {
        sum
    } else if context.compare(Extended::from_i32(0), product).map_err(Error::X87)? == Ordering::Greater {
        Extended::from_i32(0)
    } else {
        product
    };
    let draw_bits = context.store_f32(chosen).map_err(Error::X87)?.to_bits();
    let candidate_rows: Vec<_> = admitted.candidates.iter().map(|&i| {
        let row = &rows[i];
        suffix::AdmittedRow { key: row.request_key(), weight_bits: row.words[2],
            first_byte: row.words[5] as u8, second_byte: row.words[8] as u8 }
    }).collect();
    let selection = suffix::select(&candidate_rows, draw_bits).map_err(Error::Suffix)?;
    *rng = next;
    Ok(SeededSelection { admission: admitted, fraction, draw_bits, selection })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(rounding: Rounding) -> Context { Context { precision: Precision::Bits64, rounding } }
    fn row(weight: f32) -> table::Record {
        table::Record { words: [7, 42, weight.to_bits(), 2, 1, 0x12, 0, 0, 0x34] }
    }
    fn snap() -> admission::Snapshot<'static> {
        admission::Snapshot { state: 7, actor_mode: 0, context_flags: None, parts: &[] }
    }
    fn pred() -> snapshot_selection::Predicates {
        snapshot_selection::Predicates { predicate1: true, predicate2: true }
    }
    #[test]
    fn retained_fraction_and_directed_store() {
        // Next numerator is FFFFFFFF: an f32 intermediate would become 1.
        for (rounding, expected) in [(Rounding::NearestEven, 1.0f32.to_bits()),
            (Rounding::Down, 0x3f7f_ffff), (Rounding::TowardZero, 0x3f7f_ffff),
            (Rounding::Up, 1.0f32.to_bits())] {
            let mut rng = rng::Rng::new(0x313f_249b).unwrap();
            let result = select_snapshot_seeded(&[row(1.0)], &[pred()], &snap(), &mut rng, ctx(rounding)).unwrap();
            assert_eq!(result.fraction.numerator, u32::MAX);
            assert_eq!(result.draw_bits, expected);
            assert_eq!(result.selection.key, 42);
            assert_eq!(rng.state(), 0x7fff_ffff);
        }
    }
    #[test]
    fn empty_admission_still_advances_and_selects_nothing() {
        let mut rng = rng::Rng::new(1).unwrap();
        let result = select_snapshot_seeded(&[], &[], &snap(), &mut rng, ctx(Rounding::NearestEven)).unwrap();
        assert!(result.admission.candidates.is_empty());
        assert_eq!(result.draw_bits, 0);
        assert_eq!(result.selection.second_byte, suffix::SecondByte::Unchanged);
        assert_eq!(rng.state(), 0x278d_de6d);
    }
    #[test]
    fn zero_sign_and_errors_are_transactional() {
        assert!(rng::Rng::new(0).is_err());
        let mut rng = rng::Rng::new(0x8000_0000).unwrap();
        assert_eq!(select_snapshot_seeded(&[row(1.0)], &[pred()], &snap(), &mut rng, ctx(Rounding::Down)), Err(Error::UnrepresentedNegativeZero));
        assert_eq!(rng.state(), 0x8000_0000);
        let ok = select_snapshot_seeded(&[row(1.0)], &[pred()], &snap(), &mut rng, ctx(Rounding::NearestEven)).unwrap();
        assert_eq!(ok.draw_bits, 0);
        assert_eq!(ok.selection.key, 42);
        let mut rng = rng::Rng::new(1).unwrap();
        assert_eq!(select_snapshot_seeded(&[row(-1.0)], &[pred()], &snap(), &mut rng, ctx(Rounding::NearestEven)), Err(Error::Admission(admission::Error::InvalidWeight)));
        assert_eq!(rng.state(), 1);
        assert_eq!(select_snapshot_seeded(&[], &[], &snap(), &mut rng, Context { precision: Precision::Bits24, rounding: Rounding::NearestEven }), Err(Error::UnsupportedPrecision));
        assert_eq!(rng.state(), 1);
    }
    #[test]
    fn both_supported_precisions_store_subnormal_sum_under_each_rounding_mode() {
        // Seed 1 produces 0xa78dde6d / 2^32, strictly between one half and one.
        // The exact product with the smallest binary32 subnormal is therefore
        // between one half and one unit in the binary32 subnormal lattice.
        for precision in [Precision::Bits53, Precision::Bits64] {
            for (rounding, expected) in [(Rounding::NearestEven, 1),
                (Rounding::Down, 0), (Rounding::Up, 1), (Rounding::TowardZero, 0)] {
                let mut rng = rng::Rng::new(1).unwrap();
                let result = select_snapshot_seeded(&[row(f32::from_bits(1))], &[pred()],
                    &snap(), &mut rng, Context { precision, rounding }).unwrap();
                assert_eq!(result.admission.sum_bits, 1);
                assert_eq!(result.draw_bits, expected);
                assert_eq!(result.selection.key, 42);
            }
        }
    }
    #[test]
    fn fallback_duplicates_preserve_order_and_selected_output_bytes() {
        let first = table::Record { words: [7, 11, 0, 2, 0, 0xaabb_cc12, 0, 0, 0xddee_ff34] };
        let second = table::Record { words: [7, 22, 2.0f32.to_bits(), 2, 0, 0xaabb_cc56, 0, 0, 0xddee_ff78] };
        let snapshot = admission::Snapshot { state: 7, actor_mode: 0, context_flags: None, parts: &[0] };
        let mut rng = rng::Rng::new(1).unwrap();
        let result = select_snapshot_seeded(&[first, second],
            &[pred(), snapshot_selection::Predicates { predicate1: false, predicate2: false }],
            &snapshot, &mut rng, ctx(Rounding::NearestEven)).unwrap();
        assert!(result.admission.fallback_used);
        assert_eq!(result.admission.candidates, vec![0, 0, 1]);
        assert_eq!(result.admission.sum_bits, 2.0f32.to_bits());
        assert_eq!(result.selection, suffix::Selection { key: 22, first_byte: 0x56,
            second_byte: suffix::SecondByte::Written(0x78) });
        assert_eq!(rng.state(), 0x278d_de6d);
    }
}
