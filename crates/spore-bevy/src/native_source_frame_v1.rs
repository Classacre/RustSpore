//! Explicit per-frame inputs for a future persistent source-program renderer.
//! Wired only to the opt-in second-frame diagnostic, not the Cell stage. It does
//! not choose a model, infer camera conventions, or claim retail scene fidelity.
use bevy::math::{Mat4, Vec3};

pub const MAX_VIEWPORT: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError { Viewport, NonFiniteInput, NonFiniteClip }

/// Caller-provided D3D-oriented matrices and camera position; all are explicit.
/// `model` maps source-local to world, `view` maps world to D3D view, and
/// `projection` maps view to D3D clip. No axis, handedness or depth conversion
/// is performed here. The caller must use consistent conventions and units.
#[derive(Debug, Clone, Copy)]
pub struct FrameInput {
    pub width: u32,
    pub height: u32,
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_world: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct ValidatedFrame {
    pub width: u32,
    pub height: u32,
    pub model: Mat4,
    pub clip: Mat4,
    pub camera_world: Vec3,
}

impl FrameInput {
    /// Validate extent and finite values; this does not prove invertibility,
    /// correct coordinate conventions, perspective visibility or source pose.
    pub fn validate(self) -> Result<ValidatedFrame, FrameError> {
        if self.width == 0 || self.height == 0 || self.width > MAX_VIEWPORT || self.height > MAX_VIEWPORT {
            return Err(FrameError::Viewport);
        }
        if !self.model.is_finite() || !self.view.is_finite() || !self.projection.is_finite()
            || !self.camera_world.is_finite() {
            return Err(FrameError::NonFiniteInput);
        }
        let clip = self.projection * self.view * self.model;
        if !clip.is_finite() { return Err(FrameError::NonFiniteClip); }
        Ok(ValidatedFrame { width: self.width, height: self.height, model: self.model, clip, camera_world: self.camera_world })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> FrameInput {
        FrameInput { width: 640, height: 480, model: Mat4::IDENTITY, view: Mat4::IDENTITY,
                     projection: Mat4::IDENTITY, camera_world: Vec3::ZERO }
    }
    #[test]
    fn preserves_explicit_matrix_order_and_camera() {
        let mut f=input();
        f.model=Mat4::from_translation(Vec3::new(1.,2.,3.));
        f.view=Mat4::from_scale(Vec3::new(2.,3.,4.));
        f.projection=Mat4::from_translation(Vec3::new(4.,5.,6.));
        f.camera_world=Vec3::new(7.,8.,9.);
        let v=f.validate().unwrap();
        assert_eq!(v.clip, f.projection*f.view*f.model);
        assert_eq!(v.camera_world, f.camera_world);
    }
    #[test]
    fn rejects_invalid_extent_and_nonfinite_inputs() {
        let mut f=input(); f.width=0; assert_eq!(f.validate().unwrap_err(), FrameError::Viewport);
        f=input(); f.height=MAX_VIEWPORT+1; assert_eq!(f.validate().unwrap_err(), FrameError::Viewport);
        f=input(); f.camera_world.x=f32::NAN; assert_eq!(f.validate().unwrap_err(), FrameError::NonFiniteInput);
        f=input(); f.model.x_axis.x=f32::INFINITY; assert_eq!(f.validate().unwrap_err(), FrameError::NonFiniteInput);
    }
    #[test]
    fn rejects_finite_inputs_that_overflow_clip() {
        let mut f=input(); f.model=Mat4::from_scale(Vec3::splat(f32::MAX));
        f.view=Mat4::from_scale(Vec3::splat(2.));
        assert_eq!(f.validate().unwrap_err(), FrameError::NonFiniteClip);
    }
}
