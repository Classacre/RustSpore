//! Bounded native linear FadeEffect state, driven by explicit UI event inputs.
//!
//! Native 963740 handles attachment11, flag-change0E and tick0C; 9635F0 sets
//! the target and 96EFB0 supplies the white/alpha callback. The caller supplies
//! elapsed seconds and applies returned callbacks/completion signals. Generic
//! Clock, Window visibility holds, message dispatch and other effect modes
//! remain separate. This does not establish a retail screen's active phase.

use crate::{spui::SpuiElement, spui_draw::fade_white_argb};
use std::fmt;

pub const FADE_EFFECT_CLASS: u32 = 0x6f2b_6d2c;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeError {
    UnsupportedClass(u32),
    InvalidProperty(u32),
    UnsupportedInterpolation(u32),
    UnsupportedTrigger(u32),
    InvalidDuration,
    InvalidElapsedTime,
    ZeroDurationWithoutElapsedTime,
    Detached,
}
impl fmt::Display for FadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native linear UI fade: {self:?}")
    }
}
impl std::error::Error for FadeError {}

/// Properties consumed by the measured linear visibility branch. The native
/// constructor defaults are duration0, trigger1 and interpolation0. Ease and
/// damping fields do not enter interpolation0; other modes are rejected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeParameters {
    pub duration_seconds: f32,
    pub trigger: u32,
    pub interpolation: u32,
}
impl Default for FadeParameters {
    fn default() -> Self {
        Self {
            duration_seconds: 0.,
            trigger: 1,
            interpolation: 0,
        }
    }
}
impl FadeParameters {
    pub fn from_element(element: &SpuiElement) -> Result<Self, FadeError> {
        if element.proxy_id != FADE_EFFECT_CLASS {
            return Err(FadeError::UnsupportedClass(element.proxy_id));
        }
        let mut result = Self::default();
        if let Some(p) = element.property(0x4f2b_0000) {
            if p.type_code != 11 || p.count != 1 {
                return Err(FadeError::InvalidProperty(p.id));
            }
            let bytes = p
                .scalar_bytes()
                .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
                .ok_or(FadeError::InvalidProperty(p.id))?;
            result.duration_seconds = f32::from_le_bytes(bytes);
        }
        for (id, value) in [
            (0x4f2b_0001, &mut result.trigger),
            (0x4f2b_0002, &mut result.interpolation),
        ] {
            if let Some(p) = element.property(id) {
                if p.type_code != 9 {
                    return Err(FadeError::InvalidProperty(id));
                }
                *value = p.u32_value().ok_or(FadeError::InvalidProperty(id))?;
            }
        }
        Ok(result)
    }

    pub fn linear(self) -> Result<LinearFade, FadeError> {
        if self.interpolation != 0 {
            return Err(FadeError::UnsupportedInterpolation(self.interpolation));
        }
        if self.trigger > 1 {
            return Err(FadeError::UnsupportedTrigger(self.trigger));
        }
        if !self.duration_seconds.is_finite() || self.duration_seconds < 0. {
            return Err(FadeError::InvalidDuration);
        }
        Ok(LinearFade {
            parameters: self,
            current: None,
            target: 0.,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeTransition {
    pub target: f32,
    /// Native 9635F0 restarts the clock and signals transition start whenever
    /// target differs from current, even when target itself did not change.
    pub started: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeTick {
    pub progress: f32,
    /// The callback runs only when current progress changed on this tick.
    pub shade_color: Option<u32>,
    /// Native convergence on this delivered tick. The caller can stop/reset
    /// its clock, update tick subscriptions and send the native completion
    /// message. This is a condition, not an edge-triggered notification.
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearFade {
    parameters: FadeParameters,
    current: Option<f32>,
    target: f32,
}
impl LinearFade {
    pub fn from_element(element: &SpuiElement) -> Result<Self, FadeError> {
        FadeParameters::from_element(element)?.linear()
    }

    pub fn parameters(&self) -> FadeParameters {
        self.parameters
    }

    pub fn progress(&self) -> Option<f32> {
        self.current
    }

    fn target_for_visibility(&self, visible: bool) -> f32 {
        let active = if self.parameters.trigger == 0 {
            visible
        } else {
            !visible
        };
        if active { 1. } else { 0. }
    }

    /// Native event11 initializes target, previous and current together, then
    /// always calls FadeEffect. Trigger0 follows visible; trigger1 inverts it.
    pub fn attach(&mut self, visible: bool) -> u32 {
        let progress = self.target_for_visibility(visible);
        self.current = Some(progress);
        self.target = progress;
        fade_white_argb(progress).expect("visibility supplies a unit endpoint")
    }

    /// Native event12 removes the Window association. Reattachment explicitly
    /// initializes the phase again; detached ticks are outside this runner.
    pub fn detach(&mut self) {
        self.current = None;
    }

    /// Native flag-change0E -> 9635F0. This does not advance time or apply the
    /// new color immediately. Window hold/event/Clock side effects are caller
    /// responsibilities, indicated by the returned start condition.
    pub fn visibility_changed(&mut self, visible: bool) -> Result<FadeTransition, FadeError> {
        let current = self.current.ok_or(FadeError::Detached)?;
        self.target = self.target_for_visibility(visible);
        Ok(FadeTransition {
            target: self.target,
            started: current != self.target,
        })
    }

    /// Native tick0C, interpolation0, using the elapsed seconds since the
    /// caller's last native-style reset. Arithmetic steps retain float32
    /// rounding; the convergence comparison retains the x87 subtraction.
    /// Negative/nonfinite times and a moving zero-duration/zero-time step
    /// are rejected before mutation. The latter would produce native NaN and
    /// leave this runner's finite unit-progress domain.
    pub fn tick(&mut self, elapsed_seconds: f32) -> Result<FadeTick, FadeError> {
        let before = self.current.ok_or(FadeError::Detached)?;
        if !elapsed_seconds.is_finite() || elapsed_seconds < 0. {
            return Err(FadeError::InvalidElapsedTime);
        }
        let duration = self.parameters.duration_seconds;
        if duration == 0. && elapsed_seconds == 0. && before != self.target {
            return Err(FadeError::ZeroDurationWithoutElapsedTime);
        }
        let mut current = before;
        if current != self.target {
            if elapsed_seconds > duration {
                current = self.target;
            } else {
                let step = elapsed_seconds / duration;
                current = if self.target > current {
                    (current + step).min(self.target)
                } else {
                    (current - step).max(self.target)
                };
            }
        }
        // In the linear branch previous is copied from current before this
        // test, so abs(current-previous) is zero. Compare before f32 rounding
        // of the difference, matching 963B60's x87 compare at the threshold.
        let completed = (f64::from(current) - f64::from(self.target)).abs() < f64::from(0.001f32);
        if completed {
            current = self.target;
        }
        self.current = Some(current);
        Ok(FadeTick {
            progress: current,
            shade_color: if current != before {
                Some(fade_white_argb(current).expect("bounded linear step retains unit progress"))
            } else {
                None
            },
            completed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spui::{SpuiProperty, SpuiValue};

    fn parameters(duration_seconds: f32) -> FadeParameters {
        FadeParameters {
            duration_seconds,
            trigger: 1,
            interpolation: 0,
        }
    }
    fn word(value: &str) -> u32 {
        u32::from_str_radix(value, 16).unwrap()
    }
    fn runtime_at(progress: f32) -> LinearFade {
        let mut fade = parameters(1.).linear().unwrap();
        fade.attach(progress == 1.);
        if progress != 0. && progress != 1. {
            fade.attach(true);
            fade.visibility_changed(false).unwrap();
            fade.tick(progress).unwrap();
        } else {
            fade.attach(progress == 0.);
        }
        assert_eq!(fade.progress().unwrap().to_bits(), progress.to_bits());
        fade
    }
    fn assert_tick(fade: &mut LinearFade, row: &[&str], start: usize) {
        let result = fade.tick(f32::from_bits(word(row[start]))).unwrap();
        assert_eq!(result.progress.to_bits(), word(row[start + 1]), "{row:?}");
        assert_eq!(result.completed, row[start + 2] == "1", "{row:?}");
        let expected = if row[start + 3] == "-" {
            None
        } else {
            Some(word(row[start + 3]))
        };
        assert_eq!(result.shade_color, expected, "{row:?}");
    }

    #[test]
    fn independent_native_110_linear_tick_fixtures() {
        let mut count = 0;
        for line in include_str!("../tests/fixtures/spui_linear_fade_ticks.tsv").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let row = line.split('\t').collect::<Vec<_>>();
            let mut fade = runtime_at(f32::from_bits(word(row[0])));
            fade.visibility_changed(row[1] == "0").unwrap();
            assert_tick(&mut fade, &row, 2);
            count += 1;
        }
        assert_eq!(count, 110);
    }

    #[test]
    fn independent_native_ten_linear_frame_sequences() {
        let mut active_name = "";
        let mut fade = parameters(1.).linear().unwrap();
        let (mut sequences, mut frames) = (0, 0);
        for line in include_str!("../tests/fixtures/spui_linear_fade_sequences.tsv").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let row = line.split('\t').collect::<Vec<_>>();
            if row[0] != active_name {
                active_name = row[0];
                fade = runtime_at(f32::from_bits(word(row[1])));
                fade.visibility_changed(row[2] == "0").unwrap();
                sequences += 1;
            }
            assert_tick(&mut fade, &row, 4);
            frames += 1;
        }
        assert_eq!((sequences, frames), (10, 156));
    }

    #[test]
    fn attach_visibility_reversal_and_repeated_target_are_explicit() {
        let mut fade = parameters(1.).linear().unwrap();
        assert_eq!(fade.attach(true), 0xffff_ffff);
        assert_eq!(fade.visibility_changed(false).unwrap().target, 1.);
        assert_eq!(fade.tick(0.25).unwrap().progress, 0.25);
        assert!(fade.visibility_changed(false).unwrap().started);
        assert!(fade.visibility_changed(true).unwrap().started);
        assert_eq!(fade.tick(0.125).unwrap().progress, 0.125);
        let end = fade.tick(0.125).unwrap();
        assert!(end.completed);
        assert_eq!(end.shade_color, Some(0xffff_ffff));
        assert!(!fade.visibility_changed(true).unwrap().started);
        fade.detach();
        assert_eq!(fade.tick(0.25), Err(FadeError::Detached));
        assert_eq!(fade.visibility_changed(true), Err(FadeError::Detached));
        assert_eq!(fade.attach(false), 0x00ff_ffff);
        let mut visible_trigger = FadeParameters {
            trigger: 0,
            ..parameters(1.)
        }
        .linear()
        .unwrap();
        assert_eq!(visible_trigger.attach(true), 0x00ff_ffff);
        assert_eq!(visible_trigger.attach(false), 0xffff_ffff);
    }

    #[test]
    fn zero_duration_and_rejected_time_steps_preserve_the_bounded_domain() {
        let mut fade = parameters(0.).linear().unwrap();
        fade.attach(true);
        assert!(fade.tick(0.).unwrap().completed);
        fade.visibility_changed(false).unwrap();
        assert_eq!(
            fade.tick(0.),
            Err(FadeError::ZeroDurationWithoutElapsedTime)
        );
        for delta in [-1., f32::INFINITY, f32::NAN] {
            assert_eq!(fade.tick(delta), Err(FadeError::InvalidElapsedTime));
            assert_eq!(fade.progress(), Some(0.));
        }
        let end = fade.tick(f32::MIN_POSITIVE).unwrap();
        assert_eq!(end.shade_color, Some(0x00ff_ffff));
        assert!(end.completed);
        for duration in [-1., f32::INFINITY, f32::NAN] {
            assert_eq!(
                parameters(duration).linear(),
                Err(FadeError::InvalidDuration)
            );
        }
        assert_eq!(
            FadeParameters {
                interpolation: 1,
                ..parameters(1.)
            }
            .linear(),
            Err(FadeError::UnsupportedInterpolation(1))
        );
        assert_eq!(
            FadeParameters {
                trigger: 2,
                ..parameters(1.)
            }
            .linear(),
            Err(FadeError::UnsupportedTrigger(2))
        );
    }

    #[test]
    fn source_fields_defaults_and_malformed_shapes_remain_distinct() {
        let mut element = SpuiElement {
            index: 2,
            proxy_id: FADE_EFFECT_CLASS,
            root: false,
            properties: vec![],
        };
        assert_eq!(
            FadeParameters::from_element(&element).unwrap(),
            FadeParameters::default()
        );
        element.properties.push(SpuiProperty {
            id: 0x4f2b_0000,
            type_code: 11,
            count: 1,
            value: SpuiValue::Scalars {
                width: 4,
                bytes: 1.0f32.to_le_bytes().to_vec(),
            },
        });
        assert_eq!(
            LinearFade::from_element(&element).unwrap().parameters(),
            parameters(1.)
        );
        element.properties[0].count = 2;
        assert_eq!(
            FadeParameters::from_element(&element),
            Err(FadeError::InvalidProperty(0x4f2b_0000))
        );
        element.proxy_id = 0;
        assert_eq!(
            FadeParameters::from_element(&element),
            Err(FadeError::UnsupportedClass(0))
        );
    }
}
