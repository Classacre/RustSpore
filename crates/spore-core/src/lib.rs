//! Engine-agnostic core types for RustSpore.
//!
//! This crate deliberately has no Bevy dependency. Anything that should be
//! testable without a running renderer, or reused by a headless tool, belongs
//! here. Gameplay snapshots use bounded, versioned binary serialization. Resource
//! parsers accept payload slices independently of the DBPF container format.

use std::fmt;

pub mod anim;
pub mod anim_curve;
pub mod anim_binding;
pub mod anim_body_pose;
pub mod anim_body_capabilities;
pub mod anim_body_finalizer_fields;
pub mod anim_body_optional_feet;
pub mod anim_body_source;
pub mod anim_body_source_anchors;
pub mod anim_body_source_fields;
pub mod anim_body_topology;
pub mod anim_deform;
pub mod anim_order;
pub mod anim_palette;
pub mod anim_quaternion;
pub mod anim_rotation_curve;
pub mod audio_event;
pub mod bem;
pub mod cell;
pub mod cell_capability_curve;
pub mod cell_capability_geometry;
pub mod cell_capability_scalar;
pub mod cell_definition;
pub mod cell_globals;
pub mod cell_flow_field;
pub mod cell_movement;
pub mod cell_movement_application;
pub mod cell_movement_source;
pub mod cell_movement_scaling;
pub mod cell_part_geometry;
pub mod cell_part_classification;
pub mod cell_runtime_data;
pub mod cell_runtime_curves;
pub mod cell_runtime_modifiers;
pub mod cell_swimming;
pub mod civilization;
pub mod blockdata;
pub mod content_manifest;
pub mod creature_data;
pub mod creature_game;
pub mod editor;
pub mod gmdl;
pub mod gmdl_skin;
pub mod gmsh;
pub mod hash_pairs;
pub mod mesh;
pub mod locale_table;
pub mod native_crt_acos;
pub mod native_sse;
pub mod native_x87;
pub mod material_links;
pub mod part_capabilities;
pub mod prop;
pub mod raster;
pub mod rw4;
pub mod rw4_animation;
pub mod shader_fragments;
pub mod shader_binding;
pub mod shader_codegen;
pub mod shader_pack;
pub mod shader_selection;
pub mod skin;
pub mod space;
pub mod spui;
pub mod spui_draw;
pub mod spui_effect;
pub mod spui_font;
pub mod spui_font_outline;
pub mod spui_font_scaling;
pub mod spui_font_scaling_quotient;
pub mod spui_font_hint_program;
pub mod spui_glyph_atlas;
pub mod spui_layout;
pub mod string_table;
pub mod tribe;

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
