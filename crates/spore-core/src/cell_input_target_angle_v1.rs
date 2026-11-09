//! Bounded composition of the E5AF20 long angle path.
//!
//! The two already-committed `_CIasin` words, the runtime period and baseline,
//! scalar, and candidate vector are explicit inputs. Their live producers are
//! not established. The caller must separately establish that the two early
//! branches did not return, and admit native x87 finite-copy semantics. This
//! model is not a live player, a CRT implementation, or gameplay.
//!
//! Nonfinite/iteration checks and transactional errors are model restrictions;
//! native sticky FP flags, traps, aliasing and partial writes are excluded.

use crate::cell_input_angle_vector_v1::{self as vector, Branch as VectorBranch};
use crate::cell_input_angle_wrap_v1::{self as wrap, X87CopyAdmission};
use crate::cell_legacy_ciasin_interior_v1 as legacy_asin;
use crate::native_sse::{self, Policy, SseError};
use crate::native_x87::Context;

const PI: f32 = f32::from_bits(0x4049_0fdb);
const HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);
const QUARTER_PI: f32 = f32::from_bits(0x3f49_0fdb);
const FACTOR: f32 = f32::from_bits(0x3f8c_cccd);

#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub current_y: f32,
    pub candidate: [f32; 3],
    /// Caller-supplied outputs after each native `_CIasin` result is stored as f32.
    pub current_asin: f32,
    pub candidate_asin: f32,
    pub step: f32,
    pub period: f32,
    pub baseline: [f32; 3],
    pub max_wrap_iterations: usize,
}

/// Inputs for the separately admitted legacy CRT finite closed path.
#[derive(Debug, Clone, Copy)]
pub struct LegacyClosedInput {
    pub current: [f32; 3],
    pub candidate: [f32; 3],
    pub step: f32,
    pub period: f32,
    pub baseline: [f32; 3],
    pub max_wrap_iterations: usize,
}

/// Admit legacy dispatch with SSE2 disabled, CW 027F, stable CRT globals,
/// a valid source stack, no pending unmasked x87 exception, and the finite
/// finite closed return paths. This marker does not establish which CRT or dispatch
/// a running game actually selected.
#[derive(Debug, Clone, Copy)]
pub struct LegacyClosedAdmission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    CopyCandidate,
    Stepped,
    CandidateAngle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    pub vector: [f32; 3],
    pub selection: Selection,
    pub current_angle: f32,
    pub candidate_angle: f32,
    pub selected_angle: Option<f32>,
    pub vector_branch: Option<VectorBranch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NonFiniteInput,
    AsinOutsideAdmittedRange,
    NonFiniteIntermediate,
    LegacyControlWord,
    LegacyAsin(legacy_asin::Error),
    Wrap(wrap::Error),
    Vector(vector::Error),
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl From<wrap::Error> for Error {
    fn from(value: wrap::Error) -> Self {
        Self::Wrap(value)
    }
}
impl From<vector::Error> for Error {
    fn from(value: vector::Error) -> Self {
        Self::Vector(value)
    }
}
impl From<legacy_asin::Error> for Error {
    fn from(value: legacy_asin::Error) -> Self {
        Self::LegacyAsin(value)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bounded target-angle path: {self:?}")
    }
}
impl std::error::Error for Error {}

#[cfg(target_arch = "x86_64")]
fn add(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {a}, {b}", a=inout(xmm_reg) a, b=in(xmm_reg) b, options(nostack,preserves_flags));
    }
    a
}
#[cfg(target_arch = "x86_64")]
fn sub(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("subss {a}, {b}", a=inout(xmm_reg) a, b=in(xmm_reg) b, options(nostack,preserves_flags));
    }
    a
}
#[cfg(target_arch = "x86_64")]
fn mul(mut a: f32, b: f32) -> f32 {
    unsafe {
        std::arch::asm!("mulss {a}, {b}", a=inout(xmm_reg) a, b=in(xmm_reg) b, options(nostack,preserves_flags));
    }
    a
}

#[cfg(target_arch = "x86_64")]
fn angle(y: f32, asin: f32) -> f32 {
    if y > 0.0 {
        asin
    } else {
        // COMISS treats both zero signs as equal; XORPS flips only PI's sign.
        let signed_pi = if asin < 0.0 { -PI } else { PI };
        sub(signed_pi, asin)
    }
}

/// Model the finite long-path slice after `_CIasin` results have been supplied.
/// Those words are restricted to the image half-pi range as a fixture admission;
/// they are not computed or checked against the input vectors here.
pub fn long_path(
    input: Input,
    policy: Policy,
    context: Context,
    copy: X87CopyAdmission,
) -> Result<Output, Error> {
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = (input, policy, context, copy);
        return Err(Error::UnsupportedArchitecture);
    }
    #[cfg(target_arch = "x86_64")]
    {
        if ![
            input.current_y,
            input.current_asin,
            input.candidate_asin,
            input.step,
            input.period,
        ]
        .into_iter()
        .chain(input.candidate)
        .chain(input.baseline)
        .all(f32::is_finite)
        {
            return Err(Error::NonFiniteInput);
        }
        if input.current_asin.abs() > HALF_PI || input.candidate_asin.abs() > HALF_PI {
            return Err(Error::AsinOutsideAdmittedRange);
        }
        native_sse::scoped(policy, || {
            let a = angle(input.current_y, input.current_asin);
            let b = angle(input.candidate[1], input.candidate_asin);
            if !a.is_finite() || !b.is_finite() {
                return Err(Error::NonFiniteIntermediate);
            }
            let initial =
                wrap::difference(a, b, input.period, input.max_wrap_iterations, policy, copy)?;
            let threshold = mul(QUARTER_PI, FACTOR);
            if initial.magnitude >= threshold {
                return Ok(Output {
                    vector: input.candidate,
                    selection: Selection::CopyCandidate,
                    current_angle: a,
                    candidate_angle: b,
                    selected_angle: None,
                    vector_branch: None,
                });
            }
            let scaled = mul(initial.sign, input.step);
            // The source keeps sign*step as the ADDSS destination and adds A.
            let stepped = add(scaled, a);
            if !scaled.is_finite() || !stepped.is_finite() {
                return Err(Error::NonFiniteIntermediate);
            }
            let stepped = wrap::wrap(
                stepped,
                input.period,
                input.max_wrap_iterations,
                policy,
                copy,
            )?
            .value;
            let remaining = wrap::difference(
                stepped,
                b,
                input.period,
                input.max_wrap_iterations,
                policy,
                copy,
            )?;
            // For ordered finite +/-1 signs, FUCOMPI followed by LAHF/TEST/JNP
            // keeps the stepped angle exactly when both signs compare equal.
            let (selection, selected) = if remaining.sign == initial.sign {
                (Selection::Stepped, stepped)
            } else {
                (Selection::CandidateAngle, b)
            };
            let result = vector::angle_vector(selected, input.baseline, policy, context)?;
            Ok(Output {
                vector: result.vector,
                selection,
                current_angle: a,
                candidate_angle: b,
                selected_angle: Some(selected),
                vector_branch: Some(result.branch),
            })
        })
    }
}

/// Compose the long path with separately admitted legacy finite x87
/// calculation, including exact endpoints. The caller still admits that E5AF20's early branches were not
/// taken, that its runtime period/baseline are supplied correctly, and that
/// this specific legacy CRT path and CW 027F were active. The model does not
/// reproduce native sticky flags or claim the default SSE2 CRT behavior.
pub fn long_path_with_legacy_closed(
    input: LegacyClosedInput,
    policy: Policy,
    context: Context,
    copy: X87CopyAdmission,
    _legacy: LegacyClosedAdmission,
) -> Result<Output, Error> {
    if context.control_word() != 0x027f {
        return Err(Error::LegacyControlWord);
    }
    if !input.current.into_iter().all(f32::is_finite) {
        return Err(Error::NonFiniteInput);
    }
    let current_asin = legacy_asin::calculate_finite_closed(
        input.current[0],
        context,
        legacy_asin::FiniteClosedAdmission,
    )?;
    let candidate_asin = legacy_asin::calculate_finite_closed(
        input.candidate[0],
        context,
        legacy_asin::FiniteClosedAdmission,
    )?;
    long_path(
        Input {
            current_y: input.current[1],
            candidate: input.candidate,
            current_asin,
            candidate_asin,
            step: input.step,
            period: input.period,
            baseline: input.baseline,
            max_wrap_iterations: input.max_wrap_iterations,
        },
        policy,
        context,
        copy,
    )
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn input() -> Input {
        Input {
            current_y: 1.,
            candidate: [0.5, 1., -0.0],
            current_asin: 0.,
            candidate_asin: 0.5,
            step: 0.125,
            period: f32::from_bits(0x40c9_0fdb),
            baseline: [2., 3., -7.],
            max_wrap_iterations: 8,
        }
    }
    fn run(i: Input) -> Result<Output, Error> {
        long_path(
            i,
            Policy::Mxcsr1f80,
            Context {
                precision: Precision::Bits53,
                rounding: Rounding::NearestEven,
            },
            X87CopyAdmission,
        )
    }
    #[test]
    fn direct_copy_preserves_candidate_bits() {
        let mut i = input();
        i.candidate_asin = 1.0;
        let o = run(i).unwrap();
        assert_eq!(o.selection, Selection::CopyCandidate);
        assert_eq!(o.vector.map(f32::to_bits), i.candidate.map(f32::to_bits));
        assert_eq!(o.selected_angle, None);
    }
    #[test]
    fn stepping_and_overshoot_follow_sign_comparison() {
        let mut i = input();
        let stepped = run(i).unwrap();
        assert_eq!(stepped.selection, Selection::Stepped);
        assert_eq!(
            stepped.selected_angle.unwrap().to_bits(),
            0.125f32.to_bits()
        );
        i.step = 1.0;
        let overshoot = run(i).unwrap();
        assert_eq!(overshoot.selection, Selection::CandidateAngle);
        assert_eq!(
            overshoot.selected_angle.unwrap().to_bits(),
            0.5f32.to_bits()
        );
    }
    #[test]
    fn negative_turn_and_exact_target_keep_native_sign_rules() {
        let mut i = input();
        i.candidate_asin = -0.5;
        i.step = 0.125;
        let partial = run(i).unwrap();
        assert_eq!(partial.selection, Selection::Stepped);
        assert_eq!(
            partial.selected_angle.unwrap().to_bits(),
            (-0.125f32).to_bits()
        );
        i.step = 0.5;
        let exact = run(i).unwrap();
        // E50B10 returns +1 for a zero difference. The initial direction was
        // -1, so E5AF20 selects the candidate angle even though both agree.
        assert_eq!(exact.selection, Selection::CandidateAngle);
        assert_eq!(exact.selected_angle.unwrap().to_bits(), (-0.5f32).to_bits());
    }
    #[test]
    fn both_zero_asins_select_positive_pi_for_nonpositive_y() {
        for asin in [0.0, -0.0] {
            let mut i = input();
            i.current_y = -0.0;
            i.current_asin = asin;
            assert_eq!(run(i).unwrap().current_angle.to_bits(), PI.to_bits());
        }
    }
    #[test]
    fn nonpositive_y_uses_signed_pi_minus_asin() {
        let mut i = input();
        i.current_y = -0.0;
        i.candidate[1] = 0.0;
        i.current_asin = -0.5;
        i.candidate_asin = -0.25;
        i.step = 0.125;
        let o = run(i).unwrap();
        assert_eq!(o.current_angle.to_bits(), (-PI + 0.5).to_bits());
        assert_eq!(o.candidate_angle.to_bits(), (-PI + 0.25).to_bits());
    }
    #[test]
    fn invalid_values_and_unbounded_wrap_are_rejected() {
        let mut i = input();
        i.current_asin = f32::NAN;
        assert_eq!(run(i), Err(Error::NonFiniteInput));
        i = input();
        i.current_asin = 2.;
        assert_eq!(run(i), Err(Error::AsinOutsideAdmittedRange));
        i = input();
        i.period = 0.;
        assert_eq!(run(i), Err(Error::Wrap(wrap::Error::InvalidPeriod)));
    }
    #[test]
    fn legacy_interior_composes_real_input_words_under_explicit_control() {
        let i = LegacyClosedInput {
            current: [0., 1., 0.],
            candidate: [0.5, 0.8660254, 0.],
            step: 0.125,
            period: f32::from_bits(0x40c9_0fdb),
            baseline: [2., 3., -7.],
            max_wrap_iterations: 8,
        };
        let context = Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        };
        let o = long_path_with_legacy_closed(
            i,
            Policy::Mxcsr1f80,
            context,
            X87CopyAdmission,
            LegacyClosedAdmission,
        )
        .unwrap();
        assert_eq!(o.candidate_angle.to_bits(), 0x3f060a92);
        assert_eq!(o.selection, Selection::Stepped);
        assert_eq!(o.selected_angle.unwrap().to_bits(), 0.125f32.to_bits());
        let mut endpoint = i;
        endpoint.candidate[0] = f32::from_bits(0x3f800001);
        assert_eq!(
            long_path_with_legacy_closed(
                endpoint,
                Policy::Mxcsr1f80,
                context,
                X87CopyAdmission,
                LegacyClosedAdmission
            ),
            Err(Error::LegacyAsin(legacy_asin::Error::OutsideInterior))
        );
        let wrong_control = Context {
            precision: Precision::Bits64,
            ..context
        };
        assert_eq!(
            long_path_with_legacy_closed(
                i,
                Policy::Mxcsr1f80,
                wrong_control,
                X87CopyAdmission,
                LegacyClosedAdmission
            ),
            Err(Error::LegacyControlWord)
        );
    }
}
