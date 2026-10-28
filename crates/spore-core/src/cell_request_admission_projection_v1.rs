//! Bounded, snapshot-based projection of E6CC30 row admission.
//!
//! Part and predicate results are supplied, not derived from a live actor.
//! The trace records logical helper calls and short-circuiting; a part probe
//! abstracts the source's manager resolution and optional part-list scan.
//! It is not a transcript of those internal calls. RNG, x87 scaling, selection,
//! and all live-world behavior are outside this projection.

#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub state: u32,
    pub mode: u32,
    pub part_code: u32,
    pub weight_bits: u32,
    /// Supplied result for E5C300(actor, row+18).
    pub predicate1: bool,
    /// Supplied result for E5C2B0(actor, row+1C).
    pub predicate2: bool,
}

/// Immutable logical outcomes. `context_flags == None` means the context
/// argument is null; otherwise each source E52770 read yields this value.
/// `parts` is the immutable part-code set returned by a successful source
/// lookup; an unavailable part list is represented by an empty slice.
pub struct Snapshot<'a> {
    pub state: u32,
    /// Source's entry comparison: 0 for the player actor, 1 otherwise.
    pub actor_mode: u32,
    pub context_flags: Option<u32>,
    pub parts: &'a [u32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Part { row: usize, code: u32 },
    ContextFlags { row: usize },
    Predicate1 { row: usize },
    Predicate2 { row: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Projection {
    /// Ordered indexes, including any duplicates introduced by fallback.
    pub candidates: Vec<usize>,
    pub sum_bits: u32,
    pub fallback_used: bool,
    pub checks: Vec<Check>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedArchitecture,
    UnsupportedMxcsr,
    InvalidMode,
    InvalidWeight,
    TooManyRows,
    CandidateCapacity,
    NonFiniteSum,
}

/// Requires masked SSE exceptions, round-to-nearest-even, and DAZ/FTZ off
/// (MXCSR controls 1F80). Finite nonnegative weights, including subnormals
/// and signed zero, are admitted. A nonfinite running sum is rejected as a
/// policy; source behavior for unordered totals is outside this projection.
/// Status bits may change. Validates every row before executing any checks.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn project(rows: &[Row], snapshot: &Snapshot<'_>) -> Result<Projection, Error> {
    if !std::is_x86_feature_detected!("sse") {
        return Err(Error::UnsupportedArchitecture);
    }
    let mut csr = 0u32;
    // SAFETY: SSE was detected and the pointer refers to a writable u32.
    unsafe { std::arch::asm!("stmxcsr [{p}]", p = in(reg) &mut csr, options(nostack, preserves_flags)); }
    if csr & !0x3f != 0x1f80 { return Err(Error::UnsupportedMxcsr); }
    if snapshot.actor_mode > 1 { return Err(Error::InvalidMode); }
    if rows.len() > 131 { return Err(Error::TooManyRows); }
    if rows.iter().any(|row| {
        let w = f32::from_bits(row.weight_bits);
        !w.is_finite() || w < 0.0
    }) { return Err(Error::InvalidWeight); }
    // SAFETY: SSE was detected and the caller's controls were admitted.
    unsafe { project_inner(rows, snapshot) }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn project(_: &[Row], _: &Snapshot<'_>) -> Result<Projection, Error> {
    Err(Error::UnsupportedArchitecture)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse")]
unsafe fn project_inner(rows: &[Row], s: &Snapshot<'_>) -> Result<Projection, Error> {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut checks = Vec::new();
    let mut candidates = Vec::new();
    let mut sum = _mm_setzero_ps();
    let mut part = |row: usize, code: u32, checks: &mut Vec<Check>| {
        checks.push(Check::Part { row, code });
        s.parts.contains(&code)
    };
    for (i, row) in rows.iter().enumerate() {
        if row.state != s.state || (row.mode != s.actor_mode && row.mode != 2) { continue; }
        if row.part_code != 1 && !part(i, row.part_code, &mut checks) { continue; }
        if row.part_code == 8 || row.part_code == 9 {
            let flags = s.context_flags.map(|f| { checks.push(Check::ContextFlags { row: i }); f });
            match flags {
                Some(f) if f != 0 && f != 8 => { if f & 4 != 0 { continue; } }
                _ => { if part(i, 10, &mut checks) { continue; } }
            }
            if row.part_code == 8 {
                let flags = s.context_flags.map(|f| { checks.push(Check::ContextFlags { row: i }); f });
                match flags {
                    Some(f) if f != 0 && f != 8 => { if f & 1 != 0 { continue; } }
                    _ => { if part(i, 9, &mut checks) { continue; } }
                }
            }
        }
        checks.push(Check::Predicate1 { row: i });
        if !row.predicate1 { continue; }
        checks.push(Check::Predicate2 { row: i });
        if !row.predicate2 { continue; }
        if candidates.len() == 131 { return Err(Error::CandidateCapacity); }
        candidates.push(i);
        sum = _mm_add_ss(_mm_set_ss(f32::from_bits(row.weight_bits)), sum);
        if !_mm_cvtss_f32(sum).is_finite() { return Err(Error::NonFiniteSum); }
    }
    // Under the admitted finite inputs, source UCOMISS's unordered case is
    // unreachable. The source retains already appended zero-weight rows.
    let fallback_used = _mm_cvtss_f32(sum) == 0.0;
    if fallback_used {
        for (i, row) in rows.iter().enumerate() {
            if row.state != s.state || (row.mode != s.actor_mode && row.mode != 2) || row.part_code != 0 { continue; }
            if candidates.len() == 131 { return Err(Error::CandidateCapacity); }
            candidates.push(i);
            sum = _mm_add_ss(_mm_set_ss(f32::from_bits(row.weight_bits)), sum);
            if !_mm_cvtss_f32(sum).is_finite() { return Err(Error::NonFiniteSum); }
        }
    }
    Ok(Projection { candidates, sum_bits: _mm_cvtss_f32(sum).to_bits(), fallback_used, checks })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(code: u32, weight: f32) -> Row {
        Row { state: 0, mode: 2, part_code: code, weight_bits: weight.to_bits(), predicate1: true, predicate2: true }
    }
    fn snap<'a>(parts: &'a [u32], flags: Option<u32>) -> Snapshot<'a> {
        Snapshot { state: 0, actor_mode: 0, context_flags: flags, parts }
    }
    #[test]
    fn code_eight_queries_twice_and_short_circuits_predicates() {
        let p = project(&[row(8, 5.0)], &snap(&[8], Some(2))).unwrap();
        assert_eq!(p.candidates, vec![0]);
        assert_eq!(p.checks, vec![Check::Part { row: 0, code: 8 }, Check::ContextFlags { row: 0 }, Check::ContextFlags { row: 0 }, Check::Predicate1 { row: 0 }, Check::Predicate2 { row: 0 }]);
        let p = project(&[row(8, 5.0)], &snap(&[8], Some(4))).unwrap();
        assert_eq!(p.checks, vec![Check::Part { row: 0, code: 8 }, Check::ContextFlags { row: 0 }]);
        assert!(p.candidates.is_empty());
    }
    #[test]
    fn fallback_retains_zero_weight_first_pass_rows_and_skips_predicates() {
        let rows = [row(1, 0.0), Row { predicate1: false, ..row(0, 2.0) }];
        let p = project(&rows, &snap(&[], None)).unwrap();
        assert_eq!(p.candidates, vec![0, 1]);
        assert!(p.fallback_used);
        assert_eq!(p.sum_bits, 2.0f32.to_bits());
        assert_eq!(p.checks, vec![Check::Predicate1 { row: 0 }, Check::Predicate2 { row: 0 }, Check::Part { row: 1, code: 0 }]);
    }
    #[test]
    fn code_nine_null_context_probes_part_ten() {
        let p = project(&[row(9, 3.0)], &snap(&[9, 10], None)).unwrap();
        assert_eq!(p.checks, vec![Check::Part { row: 0, code: 9 }, Check::Part { row: 0, code: 10 }]);
        assert!(p.candidates.is_empty());
    }
    #[test]
    fn predicate_and_mode_short_circuit_and_addss_rounding() {
        let rows = [Row { mode: 1, ..row(1, 1.0) }, row(1, 16_777_216.0), row(1, 1.0), Row { predicate1: false, ..row(1, 10.0) }];
        let p = project(&rows, &snap(&[], None)).unwrap();
        assert_eq!(p.candidates, vec![1, 2]);
        assert_eq!(p.sum_bits, 16_777_216.0f32.to_bits());
        assert!(!p.checks.contains(&Check::Predicate2 { row: 3 }));
    }

    #[test]
    fn part_ten_requires_membership_but_has_no_extra_flag_check() {
        let absent = project(&[row(10, 2.0)], &snap(&[], Some(4))).unwrap();
        assert_eq!(absent.checks, vec![Check::Part { row: 0, code: 10 }]);
        let present = project(&[row(10, 2.0)], &snap(&[10], Some(4))).unwrap();
        assert_eq!(present.candidates, vec![0]);
        assert_eq!(present.checks, vec![Check::Part { row: 0, code: 10 }, Check::Predicate1 { row: 0 }, Check::Predicate2 { row: 0 }]);
    }

    #[test]
    fn part_eight_missing_or_special_context_uses_part_conflicts() {
        for flags in [None, Some(0), Some(8)] {
            let p = project(&[row(8, 1.0)], &snap(&[8], flags)).unwrap();
            assert_eq!(p.candidates, vec![0]);
            let part_calls: Vec<_> = p.checks.iter().filter_map(|c| match c { Check::Part { code, .. } => Some(*code), _ => None }).collect();
            assert_eq!(part_calls, vec![8, 10, 9]);
            assert_eq!(p.checks.iter().filter(|c| matches!(c, Check::ContextFlags { .. })).count(), if flags.is_some() { 2 } else { 0 });

            let ten = project(&[row(8, 1.0)], &snap(&[8, 10, 9], flags)).unwrap();
            assert!(ten.candidates.is_empty());
            assert!(!ten.checks.contains(&Check::Part { row: 0, code: 9 }));
            let nine = project(&[row(8, 1.0)], &snap(&[8, 9], flags)).unwrap();
            assert!(nine.candidates.is_empty());
            assert!(nine.checks.contains(&Check::Part { row: 0, code: 9 }));
            assert!(!nine.checks.contains(&Check::Predicate1 { row: 0 }));
        }
    }

    #[test]
    fn part_eight_bit_one_and_both_flags_take_distinct_paths() {
        let bit_one = project(&[row(8, 1.0)], &snap(&[8], Some(1))).unwrap();
        assert!(bit_one.candidates.is_empty());
        assert_eq!(bit_one.checks.iter().filter(|c| matches!(c, Check::ContextFlags { .. })).count(), 2);
        let both = project(&[row(8, 1.0)], &snap(&[8], Some(5))).unwrap();
        assert!(both.candidates.is_empty());
        assert_eq!(both.checks.iter().filter(|c| matches!(c, Check::ContextFlags { .. })).count(), 1);
        assert!(!both.checks.contains(&Check::Predicate1 { row: 0 }));
    }

    #[test]
    fn part_nine_uses_bit_four_or_part_ten_but_not_bit_one() {
        let accepted = project(&[row(9, 1.0)], &snap(&[9, 10], Some(1))).unwrap();
        assert_eq!(accepted.candidates, vec![0]);
        assert!(!accepted.checks.contains(&Check::Part { row: 0, code: 10 }));
        let blocked = project(&[row(9, 1.0)], &snap(&[9], Some(4))).unwrap();
        assert!(blocked.candidates.is_empty());
        assert!(!blocked.checks.contains(&Check::Predicate1 { row: 0 }));
        for flags in [Some(0), Some(8)] {
            let fallback = project(&[row(9, 1.0)], &snap(&[9, 10], flags)).unwrap();
            assert!(fallback.candidates.is_empty());
            assert!(fallback.checks.contains(&Check::Part { row: 0, code: 10 }));
        }
    }

    #[test]
    fn fallback_appends_duplicate_part_zero_rows_in_source_order() {
        let rows = [row(0, 0.0), row(1, 0.0), row(0, 0.0)];
        let p = project(&rows, &snap(&[0], None)).unwrap();
        assert!(p.fallback_used);
        assert_eq!(p.candidates, vec![0, 1, 2, 0, 2]);
        assert_eq!(p.sum_bits, 0.0f32.to_bits());
    }

    #[test]
    fn fixed_candidate_capacity_is_an_explicit_policy() {
        let rows = vec![row(1, 1.0); 131];
        let p = project(&rows, &snap(&[], None)).unwrap();
        assert_eq!(p.candidates, (0..131).collect::<Vec<_>>());
        let too_many = vec![row(1, 1.0); 132];
        assert_eq!(project(&too_many, &snap(&[], None)), Err(Error::TooManyRows));
        let duplicate_overflow = vec![row(0, 0.0); 131];
        assert_eq!(project(&duplicate_overflow, &snap(&[0], None)), Err(Error::CandidateCapacity));
    }

    #[test]
    fn rejects_nonfinite_negative_and_overflow_but_accepts_subnormal_and_signed_zero() {
        assert_eq!(project(&[row(1, -1.0)], &snap(&[], None)), Err(Error::InvalidWeight));
        assert_eq!(project(&[row(1, f32::NAN)], &snap(&[], None)), Err(Error::InvalidWeight));
        assert_eq!(project(&[row(1, f32::INFINITY)], &snap(&[], None)), Err(Error::InvalidWeight));
        assert_eq!(project(&[row(1, f32::MAX), row(1, f32::MAX)], &snap(&[], None)), Err(Error::NonFiniteSum));
        let p = project(&[row(1, -0.0), row(1, f32::from_bits(1))], &snap(&[], None)).unwrap();
        assert_eq!(p.candidates, vec![0, 1]);
        assert_eq!(p.sum_bits, 1);
        assert!(!p.fallback_used);
    }
}
