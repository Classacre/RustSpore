//! Synthetic, checked composition of the finite E5B160 input-target stages.
//!
//! Based on SporeApp.exe SHA-256 17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a;
//! see `re/cell-indexed-input-player-target-static-v1.md`. The frozen indexed
//! state, four AL values, baseline and actor are caller-supplied. This is not a
//! live controller, actor-pool lookup, callback dispatch or gameplay step.
//! Native sticky FP flags, partial writes and later updates are not modeled.
use crate::{
    cell_indexed_input_vector_v1 as input, cell_input_candidate_normalize_v1 as norm,
    cell_input_target_angle_v1 as angle, cell_input_target_early_v1 as early,
    native_sse::{self, Policy, SseError},
    native_x87::Context,
};

/// Admit valid, nonaliasing source storage, the intended actor-pool identity,
/// sufficient masked x87 stack/copy semantics, and stable callback/global
/// state across this frozen fixture. No actual callback or lookup is executed.
#[derive(Debug, Clone, Copy)]
pub struct StableSourceAdmission;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub direction_cc: [f32; 3],
    pub selected_c4: u32,
    pub active_c8: u8,
    pub indicator_e0: u8,
    pub actor_position_4c: [f32; 3],
    pub actor_target_08: [f32; 3],
    pub actor_flag_04: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    Inactive,
    InactiveReset,
    Active(early::EarlyBranch),
    ActiveAngle(angle::Selection),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    pub state: State,
    pub branch: Branch,
    pub side_effect_requests: [bool; 4],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Input(input::Error),
    Normalize(norm::Error),
    Early(early::Error),
    Angle(angle::Error),
    LongPathUnsupported,
    NonFiniteInput,
    NonFiniteIntermediate,
    UnsupportedArchitecture,
}
impl From<SseError> for Error {
    fn from(_: SseError) -> Self {
        Self::UnsupportedArchitecture
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "synthetic input-target composition: {self:?}")
    }
}
impl std::error::Error for Error {}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn mul(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("mulss {left}, {right}", left=inout(xmm_reg) left, right=in(xmm_reg) right, options(nostack,preserves_flags));
    }
    left
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn add(mut left: f32, right: f32) -> f32 {
    unsafe {
        std::arch::asm!("addss {left}, {right}", left=inout(xmm_reg) left, right=in(xmm_reg) right, options(nostack,preserves_flags));
    }
    left
}

/// Compose only the finite inactive and E5AF20 early-return paths. A zero or
/// negative scalar is not substituted for the unmodeled angle path: scalar
/// does not affect the two early returns. The scalar is still admitted finite.
/// All actor and direction words are checked finite even if a branch would not
/// consume them. Errors are transactional and differ from native partial writes.
pub fn update(
    state: State,
    bits: [u32; 8],
    layout_al: [u8; 4],
    baseline: [f32; 3],
    scalar: f32,
    policy: Policy,
    context: Context,
    admit: StableSourceAdmission,
) -> Result<Output, Error> {
    update_inner(
        state, bits, layout_al, baseline, scalar, None, policy, context, admit,
    )
}

/// Caller-supplied committed CRT results and period for an optional controlled
/// long-path fixture. These inputs are not derived from a live CRT or runtime.
#[derive(Debug, Clone, Copy)]
pub struct LongPathInputs {
    pub current_asin: f32,
    pub candidate_asin: f32,
    pub period: f32,
    pub max_wrap_iterations: usize,
}

/// As `update`, but permit the long branch using explicitly supplied CRT words.
/// For a CRT-bound interpretation the caller must separately establish that
/// they correspond to current and normalized X inputs. Synthetic substitutes
/// are also useful for isolated tests. This does not execute `_CIasin` or prove
/// live actor/input binding.
pub fn update_with_angles(
    state: State,
    bits: [u32; 8],
    layout_al: [u8; 4],
    baseline: [f32; 3],
    scalar: f32,
    long: LongPathInputs,
    policy: Policy,
    context: Context,
    admit: StableSourceAdmission,
) -> Result<Output, Error> {
    update_inner(
        state,
        bits,
        layout_al,
        baseline,
        scalar,
        Some(LongPathMode::Supplied(long)),
        policy,
        context,
        admit,
    )
}

/// Explicit source-state inputs and admissions for the conditional legacy CRT
/// branch. This remains a synthetic fixture, not live CRT dispatch evidence.
#[derive(Debug, Clone, Copy)]
pub struct LegacyPathInputs {
    pub period: f32,
    pub max_wrap_iterations: usize,
    pub admission: angle::LegacyClosedAdmission,
}

/// Permit the long branch using the separately admitted legacy x87 finite closed path.
/// The source runtime period, active dispatch and return policy are not derived
/// by this function. The x87 control must be 027F; the admitted finite closed
/// domain includes exact endpoints, with no native FP-status claim.
pub fn update_with_legacy_closed(
    state: State,
    bits: [u32; 8],
    layout_al: [u8; 4],
    baseline: [f32; 3],
    scalar: f32,
    legacy: LegacyPathInputs,
    policy: Policy,
    context: Context,
    admit: StableSourceAdmission,
) -> Result<Output, Error> {
    update_inner(
        state,
        bits,
        layout_al,
        baseline,
        scalar,
        Some(LongPathMode::Legacy(legacy)),
        policy,
        context,
        admit,
    )
}

#[derive(Debug, Clone, Copy)]
enum LongPathMode {
    Supplied(LongPathInputs),
    Legacy(LegacyPathInputs),
}

fn update_inner(
    state: State,
    bits: [u32; 8],
    layout_al: [u8; 4],
    baseline: [f32; 3],
    scalar: f32,
    long: Option<LongPathMode>,
    policy: Policy,
    context: Context,
    _admit: StableSourceAdmission,
) -> Result<Output, Error> {
    if !state
        .direction_cc
        .into_iter()
        .chain(state.actor_position_4c)
        .chain(state.actor_target_08)
        .chain(baseline)
        .chain([scalar])
        .all(f32::is_finite)
    {
        return Err(Error::NonFiniteInput);
    }
    let assembled = input::assemble(bits, layout_al, baseline, policy).map_err(Error::Input)?;
    let normalized =
        norm::normalize(assembled.vector, policy, context).map_err(Error::Normalize)?;
    let mut next = state;
    if assembled.side_effect_requests.into_iter().any(|v| v) {
        next.selected_c4 = 0;
        next.indicator_e0 = 1;
    }
    match normalized {
        norm::Outcome::Inactive { .. } => {
            let branch = if next.active_c8 == 1 {
                next.active_c8 = 0;
                next.actor_flag_04 = 1;
                next.direction_cc = baseline;
                Branch::InactiveReset
            } else {
                Branch::Inactive
            };
            Ok(Output {
                state: next,
                branch,
                side_effect_requests: assembled.side_effect_requests,
            })
        }
        norm::Outcome::Active { candidate, .. } => {
            let (branch, direction) = match early::early(
                state.direction_cc,
                candidate,
                policy,
                early::X87CopyAdmission,
            )
            .map_err(Error::Early)?
            {
                early::Outcome::Early { branch, candidate } => (Branch::Active(branch), candidate),
                early::Outcome::NotEarly => {
                    let copy = crate::cell_input_angle_wrap_v1::X87CopyAdmission;
                    let result = match long.ok_or(Error::LongPathUnsupported)? {
                        LongPathMode::Supplied(long) => angle::long_path(
                            angle::Input {
                                current_y: state.direction_cc[1],
                                candidate,
                                current_asin: long.current_asin,
                                candidate_asin: long.candidate_asin,
                                step: scalar,
                                period: long.period,
                                baseline,
                                max_wrap_iterations: long.max_wrap_iterations,
                            },
                            policy,
                            context,
                            copy,
                        ),
                        LongPathMode::Legacy(legacy) => angle::long_path_with_legacy_closed(
                            angle::LegacyClosedInput {
                                current: state.direction_cc,
                                candidate,
                                step: scalar,
                                period: legacy.period,
                                baseline,
                                max_wrap_iterations: legacy.max_wrap_iterations,
                            },
                            policy,
                            context,
                            copy,
                            legacy.admission,
                        ),
                    }
                    .map_err(Error::Angle)?;
                    (Branch::ActiveAngle(result.selection), result.vector)
                }
            };
            native_sse::scoped(policy, || {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    // E5B493..C7: component * 20, then actor position as
                    // ADDSS destination for each component, in Y/X/Z order.
                    let y = mul(direction[1], 20.0);
                    let x = mul(direction[0], 20.0);
                    let z = mul(direction[2], 20.0);
                    let tx = add(state.actor_position_4c[0], x);
                    let ty = add(state.actor_position_4c[1], y);
                    let tz = add(state.actor_position_4c[2], z);
                    if ![y, x, z, tx, ty, tz].into_iter().all(f32::is_finite) {
                        return Err(Error::NonFiniteIntermediate);
                    }
                    next.direction_cc = direction;
                    next.actor_target_08 = [tx, ty, tz];
                    next.actor_flag_04 = 0;
                    next.active_c8 = 1;
                    Ok(Output {
                        state: next,
                        branch,
                        side_effect_requests: assembled.side_effect_requests,
                    })
                }
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                {
                    let _ = (branch, direction);
                    Err(Error::UnsupportedArchitecture)
                }
            })
        }
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};
    fn ctx() -> Context {
        Context {
            precision: Precision::Bits53,
            rounding: Rounding::NearestEven,
        }
    }
    fn state() -> State {
        State {
            direction_cc: [0.; 3],
            selected_c4: 7,
            active_c8: 0,
            indicator_e0: 0,
            actor_position_4c: [10., 20., 30.],
            actor_target_08: [3., 4., 5.],
            actor_flag_04: 9,
        }
    }
    fn bits(codes: &[usize]) -> [u32; 8] {
        let mut w = [0; 8];
        for &c in codes {
            w[c >> 5] |= 1u32 << (c & 31);
        }
        w
    }
    fn run(s: State, codes: &[usize], base: [f32; 3]) -> Result<Output, Error> {
        update(
            s,
            bits(codes),
            [0; 4],
            base,
            0.5,
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
    }
    #[test]
    fn active_from_zero_updates_target_and_side_effects() {
        let o = run(state(), &[0x57], [0.; 3]).unwrap();
        assert_eq!(o.branch, Branch::Active(early::EarlyBranch::SmallCurrent));
        assert_eq!(o.state.direction_cc, [0., 1., 0.]);
        assert_eq!(o.state.actor_target_08, [10., 40., 30.]);
        assert_eq!(
            (
                o.state.selected_c4,
                o.state.indicator_e0,
                o.state.active_c8,
                o.state.actor_flag_04
            ),
            (0, 1, 1, 0)
        );
        assert_eq!(o.side_effect_requests, [true, false, false, false]);
    }
    #[test]
    fn admitted_key_press_and_release_reach_the_target_fixture() {
        use crate::cell_virtual_key_snapshot_v1::{DownGate, Event, Snapshot};
        let mut keys = Snapshot::default();
        keys.apply(Event::Down {
            code: 0x57,
            gate: DownGate::Passed,
        })
        .unwrap();
        let pressed = update(
            state(),
            keys.words(),
            [0; 4],
            [0.; 3],
            0.5,
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
        .unwrap();
        assert_eq!(
            pressed.branch,
            Branch::Active(early::EarlyBranch::SmallCurrent)
        );
        assert_eq!(pressed.state.actor_target_08, [10., 40., 30.]);

        keys.apply(Event::Up { code: 0x57 }).unwrap();
        let released = update(
            pressed.state,
            keys.words(),
            [0; 4],
            [0.; 3],
            0.5,
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
        .unwrap();
        assert_eq!(released.branch, Branch::InactiveReset);
        assert_eq!(
            (released.state.active_c8, released.state.actor_flag_04),
            (0, 1)
        );
        assert_eq!(
            released.state.actor_target_08,
            pressed.state.actor_target_08
        );
    }
    #[test]
    fn inactive_latch_and_no_latch_preserve_target() {
        let mut s = state();
        s.active_c8 = 1;
        s.direction_cc = [3., 4., 5.];
        let base = [-0.0, 0.0, -0.0];
        let o = run(s, &[], base).unwrap();
        assert_eq!(o.branch, Branch::InactiveReset);
        assert_eq!(
            o.state.direction_cc.map(f32::to_bits),
            base.map(f32::to_bits)
        );
        assert_eq!((o.state.active_c8, o.state.actor_flag_04), (0, 1));
        assert_eq!(o.state.actor_target_08, s.actor_target_08);
        s.active_c8 = 2;
        let o = run(s, &[], base).unwrap();
        assert_eq!(o.branch, Branch::Inactive);
        assert_eq!(o.state, s);
    }
    #[test]
    fn opposing_groups_clear_selection_even_when_inactive() {
        let o = run(state(), &[0x57, 0x53], [0.; 3]).unwrap();
        assert_eq!(o.branch, Branch::Inactive);
        assert_eq!(o.side_effect_requests, [true, true, false, false]);
        assert_eq!((o.state.selected_c4, o.state.indicator_e0), (0, 1));
    }
    #[test]
    fn unsupported_long_path_and_invalid_inputs_are_transactional() {
        let mut s = state();
        s.direction_cc = [0., -1., 0.];
        assert_eq!(run(s, &[0x57], [0.; 3]), Err(Error::LongPathUnsupported));
        assert_eq!(s.selected_c4, 7);
        assert_eq!(
            run(state(), &[], [f32::NAN, 0., 0.]),
            Err(Error::NonFiniteInput)
        );
        assert_eq!(
            run(state(), &[], [f32::MAX, 0., 0.]),
            Err(Error::Normalize(norm::Error::NonFiniteIntermediate))
        );
    }
    #[test]
    fn supplied_angle_words_compose_into_player_target() {
        let mut s = state();
        s.direction_cc = [1., 1., 0.];
        // Deliberate substitutes, not the asin of either source X word.
        let long = LongPathInputs {
            current_asin: 0.,
            candidate_asin: 0.5,
            period: f32::from_bits(0x40c9_0fdb),
            max_wrap_iterations: 8,
        };
        let o = update_with_angles(
            s,
            bits(&[0x57]),
            [0; 4],
            [0.; 3],
            0.25,
            long,
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
        .unwrap();
        assert_eq!(o.branch, Branch::ActiveAngle(angle::Selection::Stepped));
        assert_eq!(
            o.state.direction_cc.map(f32::to_bits),
            [0x3e7d5777, 0x3f780aa5, 0]
        );
        assert!((o.state.actor_target_08[0] - 14.948079).abs() < 0.00001);
        assert!((o.state.actor_target_08[1] - 39.37825).abs() < 0.00001);
        assert_eq!(
            (
                o.state.selected_c4,
                o.state.indicator_e0,
                o.state.active_c8,
                o.state.actor_flag_04
            ),
            (0, 1, 1, 0)
        );
        let invalid = LongPathInputs { period: 0., ..long };
        assert_eq!(
            update_with_angles(
                s,
                bits(&[0x57]),
                [0; 4],
                [0.; 3],
                0.25,
                invalid,
                Policy::Mxcsr1f80,
                ctx(),
                StableSourceAdmission
            ),
            Err(Error::Angle(angle::Error::Wrap(
                crate::cell_input_angle_wrap_v1::Error::InvalidPeriod
            )))
        );
        assert_eq!(s.direction_cc, [1., 1., 0.]);
    }
    #[test]
    fn admitted_legacy_interior_steers_from_real_candidate_words() {
        let mut s = state();
        s.direction_cc = [0., 1., 0.];
        let legacy = LegacyPathInputs {
            period: f32::from_bits(0x40c9_0fdb),
            max_wrap_iterations: 8,
            admission: angle::LegacyClosedAdmission,
        };
        let o = update_with_legacy_closed(
            s,
            bits(&[0x57, 0x44]),
            [0; 4],
            [0.; 3],
            0.125,
            legacy,
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
        .unwrap();
        assert_eq!(o.branch, Branch::ActiveAngle(angle::Selection::Stepped));
        assert_eq!(
            o.state.direction_cc.map(f32::to_bits),
            [0x3dff5577, 0x3f7e00ab, 0]
        );
        assert_eq!(
            o.state.actor_target_08.map(f32::to_bits),
            [0x4147e55a, 0x421f6036, 0x41f00000]
        );
        assert_eq!(
            (o.state.selected_c4, o.state.indicator_e0, o.state.active_c8),
            (0, 1, 1)
        );
        let mut endpoint = s;
        endpoint.direction_cc[0] = f32::from_bits(0x3f800001);
        assert_eq!(
            update_with_legacy_closed(
                endpoint,
                bits(&[0x57, 0x44]),
                [0; 4],
                [0.; 3],
                0.125,
                legacy,
                Policy::Mxcsr1f80,
                ctx(),
                StableSourceAdmission
            ),
            Err(Error::Angle(angle::Error::LegacyAsin(
                crate::cell_legacy_ciasin_interior_v1::Error::OutsideInterior
            )))
        );
        assert_eq!(endpoint.direction_cc, [f32::from_bits(0x3f800001), 1., 0.]);
    }
    #[test]
    fn admitted_legacy_closed_domain_handles_pure_horizontal_input() {
        let mut s = state();
        s.direction_cc = [0., 1., 0.];
        let o = update_with_legacy_closed(
            s,
            bits(&[0x44]),
            [0; 4],
            [0.; 3],
            0.125,
            LegacyPathInputs {
                period: f32::from_bits(0x40c9_0fdb),
                max_wrap_iterations: 8,
                admission: angle::LegacyClosedAdmission,
            },
            Policy::Mxcsr1f80,
            ctx(),
            StableSourceAdmission,
        )
        .unwrap();
        assert_eq!(
            o.branch,
            Branch::ActiveAngle(angle::Selection::CopyCandidate)
        );
        assert_eq!(o.state.direction_cc.map(f32::to_bits), [0x3f800000, 0, 0]);
        assert_eq!(
            o.state.actor_target_08.map(f32::to_bits),
            [0x41f00000, 0x41a00000, 0x41f00000]
        );
    }
    #[test]
    fn hostile_mxcsr_is_restored() {
        fn mxcsr() -> u32 {
            let mut x = 0u32;
            unsafe {
                std::arch::asm!("stmxcsr [{p}]",p=in(reg) &mut x,options(nostack,preserves_flags));
            }
            x
        }
        struct Restore(u32);
        impl Drop for Restore {
            fn drop(&mut self) {
                unsafe {
                    std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &self.0,options(nostack,preserves_flags));
                }
            }
        }
        let _restore = Restore(mxcsr());
        let hostile = 0xdf81u32;
        unsafe {
            std::arch::asm!("ldmxcsr [{p}]",p=in(reg) &hostile,options(nostack,preserves_flags));
        }
        let o = run(state(), &[0x57], [f32::from_bits(1), 0., 0.]).unwrap();
        assert_eq!(o.state.direction_cc[0].to_bits(), 1);
        assert_eq!(mxcsr(), hostile);
        assert!(run(state(), &[], [f32::MAX, 0., 0.]).is_err());
        assert_eq!(mxcsr(), hostile);
    }
}
