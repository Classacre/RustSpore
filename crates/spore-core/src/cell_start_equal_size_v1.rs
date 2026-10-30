//! Candidate-specific pre-allocation size transition from E74A50.
//!
//! Bounded model. The general 4BA080 sampler stores binary64 intermediates;
//! native_x87 currently has no binary64 store. This module therefore admits
//! only identical, positive normal binary32 size endpoints. In that domain the
//! sampled numerical value is exactly the endpoint, but 935D40 is still called
//! once and its state advances. It does not model orientation, cache, actor
//! allocation, resource fixup, side effects, gameplay or original FP flags.
use crate::{cell_definition::CellDefinition, cell_globals::CellGlobals,
    cell_request_rng_v1::Rng, native_x87::{Context, Extended, Precision, X87Error}};

pub const CANDIDATE_ID: u32 = 0xcbcd_1287;
pub const CELL_TYPE: u32 = 0xdfad_9f51;

/// Caller-admitted canonical resource key. This does not authenticate the
/// supplied bytes, package provenance, native fixup, or runtime selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedCellKey { pub instance: u32, pub type_id: u32, pub scope: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartSizeError {
    UnsupportedCandidate,
    UnsupportedRange,
    UnsupportedInput,
    UnsupportedControl,
    UnsupportedRng,
    Arithmetic(X87Error),
}

impl From<X87Error> for StartSizeError {
    fn from(value: X87Error) -> Self { Self::Arithmetic(value) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartSizeResult {
    pub sampled_bits: u32,
    pub actor_scale_bits: u32,
    pub rng_state_after: u32,
}

/// Model only the `E751C0` call's argument-5 = -1 / argument-7 = +0 branch.
/// The caller explicitly admits a canonical CELL key; raw globals.startCell and
/// definition.structure_id must also match. These checks authenticate neither
/// source bytes nor package provenance, fixup, or live selection.
/// The structure_id check is an extra restriction specific to this candidate;
/// it is not proof of the definition's resource identity.
/// `global_514c` and argument 6 are separately supplied observed inputs. Require
/// positive normal operands, masked x87 exceptions and PC53/PC64; inputs outside
/// this restricted model are rejected. This pure API returns no advanced RNG
/// state on error, even if its local draw occurred; native RNG may already have
/// advanced before a later fault or exceptional result. Atomic rejection is a
/// safety policy and must not be mistaken for the native error transition.
///
/// The RNG fraction model does not preserve sign of zero; reject that single
/// fraction rather than asserting exceptional/sign-sensitive equivalence. The
/// returned sampled value is derived for equal endpoints, not a general sampler.
pub fn candidate_start_size(
    key: AdmittedCellKey, definition: CellDefinition<'_>, globals: CellGlobals<'_>,
    global_514c: f32, argument_6: f32, rng_state: u32, control_word: u16,
) -> Result<StartSizeResult, StartSizeError> {
    if key != (AdmittedCellKey { instance: CANDIDATE_ID, type_id: CELL_TYPE, scope: 0 }) ||
       globals.start_cell() != CANDIDATE_ID || definition.structure_id() != CANDIDATE_ID {
        return Err(StartSizeError::UnsupportedCandidate);
    }
    let [low, high] = definition.size_range();
    if low.to_bits() != high.to_bits() || !low.is_normal() || low <= 0.0 {
        return Err(StartSizeError::UnsupportedRange);
    }
    if !global_514c.is_normal() || global_514c <= 0.0 ||
       !argument_6.is_normal() || argument_6 <= 0.0 {
        return Err(StartSizeError::UnsupportedInput);
    }
    if control_word & 0x003f != 0x003f { return Err(StartSizeError::UnsupportedControl); }
    let x87 = Context::from_control_word(control_word).map_err(|_| StartSizeError::UnsupportedControl)?;
    if x87.precision == Precision::Bits24 { return Err(StartSizeError::UnsupportedControl); }
    let mut rng = Rng::new(rng_state).map_err(|_| StartSizeError::UnsupportedRng)?;
    let draw = rng.next_fraction();
    if draw.numerator == 0 { return Err(StartSizeError::UnsupportedRng); }
    // Equal, finite positive binary32 endpoints make the rounded double
    // difference zero and clamp result exactly `low` under the admitted modes.
    // The native x87 evaluation after 4DF9A0 is (sample * (1 / global)) * arg6.
    let inverse = x87.div(Extended::from_f32(1.0)?, Extended::from_f32(global_514c)?)?;
    let sampled = Extended::from_f32(low)?;
    let scaled = x87.mul(sampled, inverse)?;
    let scaled = x87.mul(scaled, Extended::from_f32(argument_6)?)?;
    let result = x87.store_f32(scaled)?;
    if !result.is_normal() || result <= 0.0 { return Err(StartSizeError::UnsupportedInput); }
    Ok(StartSizeResult { sampled_bits: low.to_bits(), actor_scale_bits: result.to_bits(), rng_state_after: rng.state() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cell_definition, cell_globals};
    const KEY: AdmittedCellKey = AdmittedCellKey { instance: CANDIDATE_ID, type_id: CELL_TYPE, scope: 0 };

    fn records() -> (Vec<u8>, Vec<u8>) {
        let mut definition = vec![0; cell_definition::DEFINITION_SIZE];
        definition[..4].copy_from_slice(&CANDIDATE_ID.to_le_bytes());
        definition[0x304..0x308].copy_from_slice(&0x3e8c_cccdu32.to_le_bytes());
        definition[0x308..0x30c].copy_from_slice(&0x3e8c_cccdu32.to_le_bytes());
        let mut globals = vec![0; cell_globals::GLOBALS_SIZE];
        globals[0x34..0x38].copy_from_slice(&CANDIDATE_ID.to_le_bytes());
        (definition, globals)
    }
    #[test]
    fn equal_candidate_consumes_rng_and_reports_scale() {
        let (d,g) = records();
        let result = candidate_start_size(KEY, cell_definition::parse(&d).unwrap(), cell_globals::parse(&g).unwrap(),
            1.0, 1.0, 1, 0x027f).unwrap();
        assert_eq!(result.sampled_bits, 0x3e8c_cccd);
        assert_eq!(result.actor_scale_bits, 0x3e8c_cccd);
        assert_eq!(result.rng_state_after, 0x278d_de6d);
    }
    #[test]
    fn explicit_scale_and_seed_are_respected() {
        let (d,g) = records();
        let result = candidate_start_size(KEY, cell_definition::parse(&d).unwrap(), cell_globals::parse(&g).unwrap(),
            2.0, 4.0, 1, 0x037f).unwrap();
        assert_eq!(f32::from_bits(result.actor_scale_bits), f32::from_bits(0x3e8c_cccd) * 2.0);
        assert_eq!(result.rng_state_after, 0x278d_de6d);
    }
    #[test]
    fn caller_must_admit_the_exact_key_separately_from_structure_id() {
        let (d,g) = records();
        let wrong = AdmittedCellKey { instance: CANDIDATE_ID + 1, ..KEY };
        assert_eq!(candidate_start_size(wrong, cell_definition::parse(&d).unwrap(),
            cell_globals::parse(&g).unwrap(), 1., 1., 1, 0x027f),
            Err(StartSizeError::UnsupportedCandidate));
        let wrong_type = AdmittedCellKey { type_id: CELL_TYPE ^ 1, ..KEY };
        assert_eq!(candidate_start_size(wrong_type, cell_definition::parse(&d).unwrap(),
            cell_globals::parse(&g).unwrap(), 1., 1., 1, 0x027f),
            Err(StartSizeError::UnsupportedCandidate));
        let wrong_scope = AdmittedCellKey { scope: 1, ..KEY };
        assert_eq!(candidate_start_size(wrong_scope, cell_definition::parse(&d).unwrap(),
            cell_globals::parse(&g).unwrap(), 1., 1., 1, 0x027f),
            Err(StartSizeError::UnsupportedCandidate));
    }
    #[test]
    fn directed_rounding_straddles_a_binary32_boundary_for_both_precisions() {
        let (d,g) = records();
        for (down, nearest, up) in [(0x067f, 0x027f, 0x0a7f), (0x077f, 0x037f, 0x0b7f)] {
            let evaluate = |cw| candidate_start_size(KEY, cell_definition::parse(&d).unwrap(),
                cell_globals::parse(&g).unwrap(), 5., 1., 1, cw).unwrap().actor_scale_bits;
            let (lo, mid, hi) = (evaluate(down), evaluate(nearest), evaluate(up));
            assert!(lo <= mid && mid <= hi);
            assert_eq!(hi, lo + 1);
        }
    }
    #[test]
    fn finite_inputs_with_overflow_or_zero_result_reject_atomically() {
        let (d,g) = records();
        let eval = |global, arg| candidate_start_size(KEY, cell_definition::parse(&d).unwrap(),
            cell_globals::parse(&g).unwrap(), global, arg, 1, 0x027f);
        assert_eq!(eval(f32::MIN_POSITIVE, f32::MAX),
                   Err(StartSizeError::Arithmetic(X87Error::NonFiniteResult)));
        assert_eq!(eval(f32::MAX, f32::MIN_POSITIVE), Err(StartSizeError::UnsupportedInput));
        // The input seed is a value, not mutated on either failed call.
        assert_eq!(eval(1., 1.).unwrap().rng_state_after, 0x278d_de6d);
    }
    #[test]
    fn unsupported_ranges_states_and_control_are_not_guessed() {
        let (mut d,mut g) = records();
        let call = |d: &[u8],g: &[u8],global,arg,seed,cw| candidate_start_size(
            KEY,cell_definition::parse(d).unwrap(),cell_globals::parse(g).unwrap(),global,arg,seed,cw);
        assert_eq!(call(&d,&g,1.,1.,0,0x027f),Err(StartSizeError::UnsupportedRng));
        assert_eq!(call(&d,&g,1.,1.,0x8000_0000,0x027f),Err(StartSizeError::UnsupportedRng));
        assert_eq!(call(&d,&g,0.,1.,1,0x027f),Err(StartSizeError::UnsupportedInput));
        assert_eq!(call(&d,&g,1.,1.,1,0x007f),Err(StartSizeError::UnsupportedControl));
        assert_eq!(call(&d,&g,1.,1.,1,0x027e),Err(StartSizeError::UnsupportedControl));
        d[0x308..0x30c].copy_from_slice(&1.0f32.to_le_bytes());
        assert_eq!(call(&d,&g,1.,1.,1,0x027f),Err(StartSizeError::UnsupportedRange));
        g[0x34..0x38].fill(0);
        assert_eq!(call(&d,&g,1.,1.,1,0x027f),Err(StartSizeError::UnsupportedCandidate));
    }
}
