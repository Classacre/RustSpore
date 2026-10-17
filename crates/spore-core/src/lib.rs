//! Engine-agnostic core types for RustSpore.
//!
//! This crate deliberately has no Bevy dependency. Anything that should be
//! testable without a running renderer, or reused by a headless tool, belongs
//! here. Nothing in this crate performs serialization and nothing here depends
//! on the DBPF container format.

use std::fmt;

pub mod anim;
pub mod blockdata;
pub mod gmsh;
pub mod prop;
pub mod raster;
pub mod rw4;
pub mod string_table;

/// One of the five stages of SPORE, in progression order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GameStage {
    /// The single cell stage played in a tide pool.
    Cell,
    /// The creature stage played on the surface of the home planet.
    Creature,
    /// The tribal stage.
    Tribe,
    /// The civilization stage.
    Civilization,
    /// The space stage.
    Space,
}

impl GameStage {
    /// Every stage, in progression order.
    pub const ALL: [GameStage; 5] = [
        GameStage::Cell,
        GameStage::Creature,
        GameStage::Tribe,
        GameStage::Civilization,
        GameStage::Space,
    ];

    /// The lowercase name of the stage.
    pub const fn name(self) -> &'static str {
        match self {
            GameStage::Cell => "cell",
            GameStage::Creature => "creature",
            GameStage::Tribe => "tribe",
            GameStage::Civilization => "civilization",
            GameStage::Space => "space",
        }
    }

    /// The next stage in progression, or `None` after the space stage.
    pub const fn next(self) -> Option<GameStage> {
        match self {
            GameStage::Cell => Some(GameStage::Creature),
            GameStage::Creature => Some(GameStage::Tribe),
            GameStage::Tribe => Some(GameStage::Civilization),
            GameStage::Civilization => Some(GameStage::Space),
            GameStage::Space => None,
        }
    }
}

impl fmt::Display for GameStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An opaque identifier for a game resource.
///
/// This is a RustSpore level handle. It is not the same thing as the DBPF
/// `ResourceKey` triple; conversion between the two is a later concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

/// A plain three component float vector.
///
/// This exists so engine-agnostic code can do simple maths without pulling in
/// Bevy. It is not a replacement for `bevy::math::Vec3` at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::splat(0.0);
    pub const ONE: Vec3 = Vec3::splat(1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_progression_is_ordered_and_terminates() {
        assert_eq!(GameStage::ALL.len(), 5);
        assert_eq!(GameStage::Cell.next(), Some(GameStage::Creature));
        assert_eq!(GameStage::Civilization.next(), Some(GameStage::Space));
        assert_eq!(GameStage::Space.next(), None);
    }

    #[test]
    fn stage_display_matches_name() {
        for stage in GameStage::ALL {
            assert_eq!(stage.to_string(), stage.name());
        }
    }

    #[test]
    fn resource_id_display_is_hex() {
        assert_eq!(ResourceId(0x1234).to_string(), "0000000000001234");
    }

    #[test]
    fn vec3_basic_maths() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);
        assert_eq!(v.dot(Vec3::new(1.0, 0.0, 0.0)), 3.0);
    }
}
