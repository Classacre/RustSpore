//! Bounded E56960 gate and the E6EF30 animation-state assignment.
//!
//! This is a supplied snapshot at the state dispatch, not the complete actor
//! integrator. The caller must account for all preceding calls, writes and mode
//! changes and for subsequent E557A0 effects. Finite values only; unmasked
//! exceptions, status flags, NaNs, infinities and aliased memory are excluded.
//! Errors leave the supplied snapshot unchanged, unlike a faulting native call.
//! Tests exercise this model; they are not a comparison with native execution.

use crate::{
    native_sse::{self, Policy, SseError},
    native_x87::{Context, Extended, X87Error},
};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Snapshot {
    pub position_08: [f32; 3],
    pub target_4c: [f32; 3],
    pub timer_368: f32,
    pub state_18c: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    X87(X87Error),
    Sse(SseError),
    NonFinite,
}
impl From<X87Error> for Error {
    fn from(e: X87Error) -> Self {
        Self::X87(e)
    }
}
impl From<SseError> for Error {
    fn from(e: SseError) -> Self {
        Self::Sse(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub x87: Context,
    pub sse: Policy,
}

/// Returns the native AL result and updates +368. Delta is the original gate
/// argument, not the movement speed scaled earlier in E6EF30.
pub fn gate(snapshot: &mut Snapshot, delta: f32, env: Environment) -> Result<bool, Error> {
    let mut next = *snapshot;
    let result = native_sse::scoped(env.sse, || gate_inner(&mut next, delta, env.x87))?;
    *snapshot = next;
    Ok(result)
}

fn gate_inner(s: &mut Snapshot, delta: f32, c: Context) -> Result<bool, Error> {
    let mut d = [Extended::from_i32(0); 3];
    for (i, slot) in d.iter_mut().enumerate() {
        *slot = c.sub(
            Extended::from_f32(s.target_4c[i])?,
            Extended::from_f32(s.position_08[i])?,
        )?;
    }
    // E56976..82: z*z, x*x, z²+x², y*y, sum, sqrt.
    let z2 = c.mul(d[2], d[2])?;
    let x2 = c.mul(d[0], d[0])?;
    let zx = c.add(z2, x2)?;
    let y2 = c.mul(d[1], d[1])?;
    let sum = c.add(zx, y2)?;
    let length = c.sqrt(sum)?;
    // FST commits binary32 for the later SSE comparison while x87 retains length.
    let stored_length = c.store_f32(length)?;
    if c.compare(Extended::from_i32(1), length)? == Ordering::Greater {
        s.timer_368 = 0.0;
    }
    if stored_length < 3.0 {
        return Ok(false);
    }
    if !s.timer_368.is_finite() {
        return Err(Error::NonFinite);
    }
    if s.timer_368 > 1.5 {
        return Ok(false);
    }
    if !delta.is_finite() {
        return Err(Error::NonFinite);
    }
    let next = s.timer_368 + delta;
    if !next.is_finite() {
        return Err(Error::NonFinite);
    }
    s.timer_368 = next;
    Ok(true)
}

/// Models only the dispatch immediately after the +17e guard. `mode` must be
/// the value after earlier E6EF30 helper calls, and `first_word_matches_player`
/// is the observed equality between the actor's first word and global +411c
/// for state 0x28. Other states and modes preserve +18c.
/// The complete function continues into external movement calls after this step.
pub fn dispatch(
    s: &mut Snapshot,
    mode: u32,
    first_word_matches_player: bool,
    delta: f32,
    env: Environment,
) -> Result<Option<bool>, Error> {
    if !matches!(s.state_18c, 0 | 0x12 | 0x30) && !(s.state_18c == 0x28 && first_word_matches_player) {
        return Ok(None);
    }
    match mode {
        0 => {
            let active = gate(s, delta, env)?;
            s.state_18c = if active { 0x30 } else { 0x12 };
            Ok(Some(active))
        }
        1..=4 => {
            s.state_18c = [0x14, 0x13, 0x32, 0x33][(mode - 1) as usize];
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Apply the later E557A0 false-result state reset, using a caller-supplied
/// post-call state. Call only when E6EF30 actually took its bit-4 branch and
/// bit 8 was clear; the helper and its potential aliasing are not modeled.
pub fn apply_e557a0_result(state_18c: &mut u32, returned_true: bool) {
    if !returned_true && matches!(*state_18c, 0x12 | 0x13 | 0x14 | 0x30) {
        *state_18c = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_x87::{Precision, Rounding};

    fn env() -> Environment {
        Environment {
            x87: Context {
                precision: Precision::Bits64,
                rounding: Rounding::NearestEven,
            },
            sse: Policy::Mxcsr1f80,
        }
    }
    fn snapshot(distance: f32, timer: f32, state: u32) -> Snapshot {
        Snapshot {
            position_08: [0.0; 3],
            target_4c: [distance, 0.0, 0.0],
            timer_368: timer,
            state_18c: state,
        }
    }
    #[test]
    fn thresholds_and_state_dispatch() {
        let mut s = snapshot(0.5, 4.0, 0);
        assert_eq!(dispatch(&mut s, 0, false, 0.25, env()), Ok(Some(false)));
        assert_eq!((s.timer_368, s.state_18c), (0.0, 0x12));
        s.target_4c[0] = 3.0;
        s.timer_368 = 1.5;
        assert_eq!(dispatch(&mut s, 0, false, 0.25, env()), Ok(Some(true)));
        assert_eq!((s.timer_368, s.state_18c), (1.75, 0x30));
        assert_eq!(dispatch(&mut s, 0, false, 0.25, env()), Ok(Some(false)));
        assert_eq!((s.timer_368, s.state_18c), (1.75, 0x12));
        apply_e557a0_result(&mut s.state_18c, false);
        assert_eq!(s.state_18c, 0);
    }
    #[test]
    fn nonparticipating_states_and_modes_do_not_read_gate_values() {
        let mut s = snapshot(f32::NAN, f32::NAN, 0x28);
        assert_eq!(dispatch(&mut s, 0, false, f32::NAN, env()), Ok(None));
        assert_eq!(s.state_18c, 0x28);
        assert_eq!(dispatch(&mut s, 3, true, f32::NAN, env()), Ok(None));
        assert_eq!(s.state_18c, 0x32);
        apply_e557a0_result(&mut s.state_18c, false);
        assert_eq!(s.state_18c, 0x32);
    }
    #[test]
    fn rejected_arithmetic_keeps_snapshot() {
        let mut s = snapshot(3.0, 1.0, 0);
        let before = s;
        assert_eq!(
            dispatch(&mut s, 0, false, f32::NAN, env()),
            Err(Error::NonFinite)
        );
        assert_eq!(s, before);
    }

    #[test]
    fn adjacent_distance_and_timer_thresholds() {
        let below_three = f32::from_bits(3.0f32.to_bits() - 1);
        let above_three = f32::from_bits(3.0f32.to_bits() + 1);
        for (distance, expected) in [(below_three, false), (3.0, true), (above_three, true)] {
            let mut s = snapshot(distance, 0.0, 0);
            assert_eq!(gate(&mut s, 0.5, env()), Ok(expected));
            assert_eq!(s.timer_368, if expected { 0.5 } else { 0.0 });
        }
        let below_timer = f32::from_bits(1.5f32.to_bits() - 1);
        let above_timer = f32::from_bits(1.5f32.to_bits() + 1);
        for (timer, expected) in [(below_timer, true), (1.5, true), (above_timer, false)] {
            let mut s = snapshot(3.0, timer, 0);
            assert_eq!(gate(&mut s, 0.0, env()), Ok(expected));
            assert_eq!(s.timer_368.to_bits(), timer.to_bits());
        }
    }

    #[test]
    fn signed_delta_and_rejected_overflow() {
        let mut s = snapshot(3.0, 0.25, 0);
        assert_eq!(gate(&mut s, -0.5, env()), Ok(true));
        assert_eq!(s.timer_368, -0.25);
        let mut s = snapshot(3.0, -f32::MAX, 0);
        let before = s;
        assert_eq!(gate(&mut s, -f32::MAX, env()), Err(Error::NonFinite));
        assert_eq!(s, before);
    }

    #[test]
    fn explicit_x87_precision_and_rounding_affect_committed_distance() {
        // The true length is just below 3 but above its binary32 midpoint.
        // A downward FST remains below 3; nearest and upward commit 3.
        for precision in [Precision::Bits53, Precision::Bits64] {
            for (rounding, expected) in [
                (Rounding::NearestEven, true),
                (Rounding::Up, true),
                (Rounding::Down, false),
                (Rounding::TowardZero, false),
            ] {
                let mut s = snapshot(f32::from_bits(3.0f32.to_bits() - 1), 0.0, 0);
                s.target_4c[1] = 0.001;
                let e = Environment {
                    x87: Context {
                        precision,
                        rounding,
                    },
                    ..env()
                };
                assert_eq!(
                    gate(&mut s, 0.5, e),
                    Ok(expected),
                    "{precision:?} {rounding:?}"
                );
            }
        }
    }

    #[test]
    fn every_dispatch_state_and_mode_and_later_reset() {
        for (state, same, eligible) in [
            (0, false, true),
            (0x12, false, true),
            (0x30, false, true),
            (0x28, true, true),
            (0x28, false, false),
            (0x13, true, false),
            (0x14, true, false),
            (0x32, true, false),
            (0x33, true, false),
            (u32::MAX, true, false),
        ] {
            for (mode, requested) in [
                (0, 0x30),
                (1, 0x14),
                (2, 0x13),
                (3, 0x32),
                (4, 0x33),
                (5, state),
                (u32::MAX, state),
            ] {
                let mut s = snapshot(3.0, 0.0, state);
                let result = dispatch(&mut s, mode, same, 0.5, env());
                assert_eq!(
                    result,
                    Ok(if eligible && mode == 0 {
                        Some(true)
                    } else {
                        None
                    })
                );
                assert_eq!(s.state_18c, if eligible { requested } else { state });
                assert_eq!(s.timer_368, if eligible && mode == 0 { 0.5 } else { 0.0 });
            }
        }
        for state in [0, 0x12, 0x13, 0x14, 0x28, 0x30, 0x32, 0x33] {
            let mut unchanged = state;
            apply_e557a0_result(&mut unchanged, true);
            assert_eq!(unchanged, state);
            let mut result = state;
            apply_e557a0_result(&mut result, false);
            assert_eq!(
                result,
                if matches!(state, 0x12 | 0x13 | 0x14 | 0x30) {
                    0
                } else {
                    state
                }
            );
        }
    }
}
