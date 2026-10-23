//! Current native `globals` (`0x2A3CE5B7`) CellGame tuning, before fixups.
//!
//! Descriptor 16B1B3C declares 0x114 bytes and 68 named four-byte fields.
//! The installed 264-byte older copy is a distinct unsupported layout; it is
//! never padded or assigned current fields silently. Runtime values come from
//! the installed resource, not constants embedded in this module.

use std::fmt;

pub const GLOBALS_SIZE: usize = 0x114;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellGlobalsError {
    pub actual_size: usize,
}

impl fmt::Display for CellGlobalsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported Cell globals extent {}, expected {GLOBALS_SIZE}",
            self.actual_size
        )
    }
}
impl std::error::Error for CellGlobalsError {}

#[derive(Debug, Clone, Copy)]
pub struct CellGlobals<'a> {
    bytes: &'a [u8; GLOBALS_SIZE],
}

pub fn parse(bytes: &[u8]) -> Result<CellGlobals<'_>, CellGlobalsError> {
    let bytes = bytes.try_into().map_err(|_| CellGlobalsError {
        actual_size: bytes.len(),
    })?;
    Ok(CellGlobals { bytes })
}

macro_rules! float_fields {
    ($($name:ident = $at:expr),* $(,)?) => { $(
        pub fn $name(self) -> f32 { f32::from_bits(self.word($at)) }
    )* };
}

macro_rules! word_fields {
    ($($name:ident = $at:expr),* $(,)?) => { $(
        pub fn $name(self) -> u32 { self.word($at) }
    )* };
}

impl<'a> CellGlobals<'a> {
    pub fn raw(self) -> &'a [u8; GLOBALS_SIZE] {
        self.bytes
    }
    fn word(self, at: usize) -> u32 {
        u32::from_le_bytes(self.bytes[at..at + 4].try_into().unwrap())
    }
    /// Preserved word absent from this native field descriptor list.
    pub fn unnamed_d0(self) -> u32 {
        self.word(0xd0)
    }
    pub fn num_high_lod_fg(self) -> i32 {
        self.word(0xc0) as i32
    }
    pub fn num_high_lod_bg(self) -> i32 {
        self.word(0xc4) as i32
    }
    /// Native type 7 word; its string/hash conversion is not inferred here.
    pub fn starting_cell_key(self) -> u32 {
        self.word(0x38)
    }

    word_fields! {
        game_mode = 0x00,
        world_1 = 0x04, world_2 = 0x08, world_3 = 0x0c, world_4 = 0x10, world_5 = 0x14,
        world_background_1 = 0x18, world_background_2 = 0x1c, world_background_3 = 0x20,
        world_background_4 = 0x24, world_background_5 = 0x28,
        world_random = 0x2c, world_random_bg = 0x30, start_cell = 0x34,
        effect_map_entry = 0x3c, background_map_entry = 0x40, key_look_algorithm = 0x84,
        control_method = 0xd4, editor_method = 0xd8, tutorial_method = 0xdc,
        ending_method = 0xe0, eye_method = 0xe4,
    }
    float_fields! {
        flow_multiplier = 0x44, npc_speed_multiplier = 0x48,
        npc_turn_speed_multiplier_jet = 0x4c, npc_turn_speed_multiplier_flagella = 0x50,
        npc_turn_speed_multiplier_cilia = 0x54,
        density_rock = 0x58, density_solid = 0x5c, density_liquid = 0x60, density_air = 0x64,
        background_distance = 0x68, min_drag_collision_speed = 0x6c,
        min_impact_collision_speed = 0x70,
        cilia_speed_as_jet = 0x74, flagella_speed_as_jet = 0x78,
        flagella_speed_as_cilia = 0x7c, cilia_speed_as_flagella = 0x80,
        beach_distance = 0x88, finish_line_distance = 0x8c, no_part_speed = 0x90,
        flagella_ramp_min_factor = 0x94, flagella_ramp_time = 0x98,
        flagella_ramp_reset_angle = 0x9c, flagella_turn_speed_ramp_start = 0xa0,
        flagella_turn_speed_ramp_end = 0xa4, flagella_turn_speed_min = 0xa8,
        flagella_turn_speed_max = 0xac, cilia_turn_speed = 0xb0, jet_turn_speed = 0xb4,
        start_level_no_creature_radius = 0xb8, start_level_no_anything_radius = 0xbc,
        percent_animal_food = 0xc8, percent_plant_food = 0xcc,
        time_to_goldy_cinematic = 0xe8, mission_time = 0xec, mission_reset_time = 0xf0,
        escape_min_distance = 0xf4, escape_max_distance = 0xf8,
        escape_delay_medium = 0xfc, escape_timer_hard = 0x100,
        non_animating_cilia_movement_factor = 0x104, non_animating_jet_movement_factor = 0x108,
        mate_trigger_distance = 0x10c, mate_spawn_distance = 0x110,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_extent_is_explicit_and_old_layout_is_not_padded() {
        assert_eq!(parse(&[0; 264]).unwrap_err().actual_size, 264);
        assert_eq!(parse(&[0; 277]).unwrap_err().actual_size, 277);
        assert!(parse(&[0; GLOBALS_SIZE]).is_ok());
    }

    #[test]
    fn signed_bits_unknown_word_and_new_tail_fields_remain_exact() {
        let mut bytes = [0; GLOBALS_SIZE];
        bytes[0x44..0x48].copy_from_slice(&0x8000_0000u32.to_le_bytes());
        bytes[0x48..0x4c].copy_from_slice(&0x7fc1_2345u32.to_le_bytes());
        bytes[0xc0..0xc4].copy_from_slice(&(-3i32).to_le_bytes());
        bytes[0xd0..0xd4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes[0x108..0x10c].copy_from_slice(&(-7.5f32).to_bits().to_le_bytes());
        let value = parse(&bytes).unwrap();
        assert_eq!(value.flow_multiplier().to_bits(), 0x8000_0000);
        assert_eq!(value.npc_speed_multiplier().to_bits(), 0x7fc1_2345);
        assert_eq!(value.num_high_lod_fg(), -3);
        assert_eq!(value.unnamed_d0(), 0x1234_5678);
        assert_eq!(value.non_animating_jet_movement_factor(), -7.5);
        assert_eq!(value.raw(), &bytes);
    }
}
