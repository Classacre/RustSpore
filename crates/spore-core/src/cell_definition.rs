//! Source Cell definitions (`0xDFAD9F51`), before native resource fixups.
//!
//! The executable's `cell`, `cAIData`, `cEatData` and `cLocalizedString`
//! descriptors establish these names, offsets and fixed extents. References
//! remain source IDs; native `E4AAD0` replaces them with runtime handles.
//! This module decodes tuning data and retains the bounded numeric portion of
//! native eating. Swimming, combat damage, growth, events and stage transitions
//! require their remaining runtime consumers.

use std::fmt;

pub const DEFINITION_SIZE: usize = 0x31c;
pub const AI_SIZE: usize = 0xb4;
pub const LABEL_UNITS: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellDefinitionError {
    pub actual_size: usize,
}

impl fmt::Display for CellDefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cell definition has {} bytes, expected {DEFINITION_SIZE}",
            self.actual_size
        )
    }
}
impl std::error::Error for CellDefinitionError {}

/// Preserve the source byte, including a noncanonical boolean representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellBool(pub u8);

impl CellBool {
    /// Shipped definitions use 0/1. Keep other bytes uninterpreted: native
    /// consumers include both equality-to-one and comparison-to-zero branches.
    pub fn canonical_value(self) -> Option<bool> {
        match self.0 {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

/// Exact source field names. Runtime difficulty-to-field selection is separate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellAiVariant {
    Ai,
    AiHard,
    AiEasy,
}

impl CellAiVariant {
    pub const fn source_offset(self) -> usize {
        match self {
            Self::Ai => 0xe0,
            Self::AiHard => 0x194,
            Self::AiEasy => 0x248,
        }
    }
}

/// Borrowed immutable view; source padding and unclassified words stay available.
#[derive(Debug, Clone, Copy)]
pub struct CellDefinition<'a> {
    bytes: &'a [u8; DEFINITION_SIZE],
}

pub fn parse(bytes: &[u8]) -> Result<CellDefinition<'_>, CellDefinitionError> {
    let bytes = bytes.try_into().map_err(|_| CellDefinitionError {
        actual_size: bytes.len(),
    })?;
    Ok(CellDefinition { bytes })
}

impl<'a> CellDefinition<'a> {
    pub fn raw(self) -> &'a [u8; DEFINITION_SIZE] {
        self.bytes
    }
    pub fn structure_id(self) -> u32 {
        word(self.bytes, 0)
    }

    /// The complete native 0xA0-byte text member, excluding its trailing word.
    pub fn label_units(self) -> [u16; LABEL_UNITS] {
        std::array::from_fn(|i| {
            u16::from_le_bytes(self.bytes[4 + i * 2..6 + i * 2].try_into().unwrap())
        })
    }

    pub fn embedded_label(self) -> Result<String, std::string::FromUtf16Error> {
        let units = self.label_units();
        let end = units.iter().position(|u| *u == 0).unwrap_or(units.len());
        String::from_utf16(&units[..end])
    }

    /// Unnamed integer in the native cLocalizedString descriptor, at local A0.
    pub fn label_trailing_word(self) -> u32 {
        word(self.bytes, 0xa4)
    }
    pub fn hp(self) -> i32 {
        word(self.bytes, 0xa8) as i32
    }
    pub fn fixed_orientation(self) -> CellBool {
        CellBool(self.bytes[0xac])
    }
    pub fn flags(self) -> u32 {
        word(self.bytes, 0xb0)
    }
    pub fn cell_type(self) -> u32 {
        word(self.bytes, 0xb4)
    }
    pub fn unlock_type(self) -> u32 {
        word(self.bytes, 0xb8)
    }
    pub fn density(self) -> u32 {
        word(self.bytes, 0xbc)
    }
    pub fn sound(self) -> u32 {
        word(self.bytes, 0xc0)
    }
    pub fn break_id(self) -> u32 {
        word(self.bytes, 0xc4)
    }
    pub fn pieces_id(self) -> u32 {
        word(self.bytes, 0xc8)
    }
    pub fn leak_id(self) -> u32 {
        word(self.bytes, 0xcc)
    }
    pub fn expel_id(self) -> u32 {
        word(self.bytes, 0xd0)
    }
    pub fn loot_id(self) -> u32 {
        word(self.bytes, 0xd4)
    }
    pub fn explosion_table_id(self) -> u32 {
        word(self.bytes, 0xd8)
    }
    pub fn poison_id(self) -> u32 {
        word(self.bytes, 0xdc)
    }

    pub fn ai(self, variant: CellAiVariant) -> CellAiData<'a> {
        let at = variant.source_offset();
        CellAiData {
            bytes: self.bytes[at..at + AI_SIZE].try_into().unwrap(),
        }
    }

    /// Native E573E0 selects easy at 0 and hard at 2, inheriting the base AI
    /// when the selected variant's unnamed first word is FFFFFFFF.
    pub fn selected_ai_variant(self, difficulty_code: i32) -> CellAiVariant {
        let candidate = match difficulty_code {
            0 => CellAiVariant::AiEasy,
            2 => CellAiVariant::AiHard,
            _ => return CellAiVariant::Ai,
        };
        if self.ai(candidate).unnamed_enum() == u32::MAX {
            CellAiVariant::Ai
        } else {
            candidate
        }
    }

    /// Exact final E573E0 predicate. The unnamed code's additional meanings
    /// remain open; zero rejects a nonplayer actor and still accepts the player.
    pub fn passes_native_ai_gate(self, difficulty_code: i32, actor_is_player: bool) -> bool {
        self.ai(self.selected_ai_variant(difficulty_code))
            .unnamed_enum()
            != 0
            || actor_is_player
    }

    pub fn friend_group(self) -> i32 {
        word(self.bytes, 0x2fc) as i32
    }
    pub fn wont_attack_player(self) -> CellBool {
        CellBool(self.bytes[0x300])
    }
    pub fn wont_attack_player_when_small(self) -> CellBool {
        CellBool(self.bytes[0x301])
    }
    /// Native type 11 occupies eight bytes. Its numerical consumer is separate.
    pub fn size_bits(self) -> [u32; 2] {
        [word(self.bytes, 0x304), word(self.bytes, 0x308)]
    }
    /// Native E5D100 reads this pair as floats before random interpolation.
    /// Sampling, stage scaling and native x87 rounding are not applied here.
    pub fn size_range(self) -> [f32; 2] {
        self.size_bits().map(f32::from_bits)
    }
    pub fn eat(self) -> CellEatData<'a> {
        CellEatData {
            bytes: self.bytes[0x30c..0x318].try_into().unwrap(),
        }
    }
    pub fn triggers_escape_mission(self) -> CellBool {
        CellBool(self.bytes[0x318])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellAiData<'a> {
    bytes: &'a [u8; AI_SIZE],
}

macro_rules! ai_float_accessors {
    ($($name:ident = $offset:expr),* $(,)?) => { $(
        pub fn $name(self) -> f32 { float(self.bytes, $offset) }
    )* };
}

macro_rules! ai_bool_accessors {
    ($($name:ident = $offset:expr),* $(,)?) => { $(
        pub fn $name(self) -> CellBool { CellBool(self.bytes[$offset]) }
    )* };
}

impl<'a> CellAiData<'a> {
    pub fn raw(self) -> &'a [u8; AI_SIZE] {
        self.bytes
    }
    /// This descriptor has no field name. Do not infer an AI state from its value.
    pub fn unnamed_enum(self) -> u32 {
        word(self.bytes, 0)
    }
    pub fn movement_style(self) -> u32 {
        word(self.bytes, 0x10)
    }
    pub fn spawn_output_id(self) -> u32 {
        word(self.bytes, 0x50)
    }
    pub fn num_arcs(self) -> i32 {
        word(self.bytes, 0x5c) as i32
    }
    pub fn transformation_id(self) -> u32 {
        word(self.bytes, 0x60)
    }
    pub fn projectile_id(self) -> u32 {
        word(self.bytes, 0x64)
    }
    pub fn food(self) -> u32 {
        word(self.bytes, 0x68)
    }
    pub fn grow_count(self) -> i32 {
        word(self.bytes, 0x6c) as i32
    }
    pub fn digestion_count(self) -> i32 {
        word(self.bytes, 0x70) as i32
    }
    pub fn digestion_output_id(self) -> u32 {
        word(self.bytes, 0x78)
    }
    pub fn grow_amount(self) -> i32 {
        word(self.bytes, 0x9c) as i32
    }

    ai_float_accessors! {
        awareness_radius = 0x04, awareness_radius_food = 0x08,
        awareness_radius_predator = 0x0c, speed = 0x18,
        chase_speed = 0x1c, wander_speed = 0x20, flee_speed = 0x24,
        fears_nearby_damage_radius = 0x28, fears_nearby_death_radius = 0x2c,
        fears_nearby_damage_time = 0x30, fears_nearby_death_time = 0x34,
        protect_radius = 0x38, protect_time = 0x3c, turn_factor = 0x40,
        spawn_time = 0x48, spawn_rest_time = 0x4c,
        arc_length = 0x54, arc_length_secondary = 0x58,
        digestion_time = 0x74, flee_time = 0x7c, flee_rest_time = 0x80,
        chase_time = 0x84, chase_rest_time = 0x88,
        awake_time = 0x94, sleep_time = 0x98, hatch_duration = 0xa0,
        poison_recharge = 0xa4, electric_recharge = 0xa8,
        electric_recharge_vs_small = 0xac, electric_discharge = 0xb0,
    }
    ai_bool_accessors! {
        flocking = 0x14, axial_movement = 0x44, chases_damage = 0x8c,
        fears_mouths = 0x8d, fears_weapons = 0x8e, fears_electric = 0x8f,
        fears_poison = 0x90, fears_damage = 0x91, ignores_food = 0x92,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellEatData<'a> {
    bytes: &'a [u8; 12],
}

impl<'a> CellEatData<'a> {
    pub fn raw(self) -> &'a [u8; 12] {
        self.bytes
    }
    pub fn food_value(self) -> i32 {
        word(self.bytes, 0) as i32
    }
    pub fn hp_value(self) -> i32 {
        word(self.bytes, 4) as i32
    }
    pub fn bomb(self) -> CellBool {
        CellBool(self.bytes[8])
    }
    pub fn poison_nova(self) -> CellBool {
        CellBool(self.bytes[9])
    }
}

/// The numeric/effect-request subset of native E7D0A0. Applying damage, visual
/// effects, progression, stats and event/audio output remains the caller's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellEatingNumerics {
    pub health_after: i32,
    /// None is the source zero-food early exit, after healing/effect requests.
    pub food_value: Option<i32>,
    pub trigger_bomb: bool,
    pub trigger_poison_nova: bool,
}

/// Evaluate source healing and the food-value early-exit branch.
///
/// `None` selects the native absent/special-definition path (health +2, food 10).
/// `difficulty_code` is the raw +7C save-state value; this function only relies
/// on the measured zero-versus-nonzero branch. No missing-asset fallback is
/// implied. The caller must resolve the source resource/special-ID condition.
pub fn eating_numerics(
    health_before: i32,
    consumed: Option<CellDefinition<'_>>,
    difficulty_code: i32,
) -> CellEatingNumerics {
    let eat = consumed.map(CellDefinition::eat);
    let mut health = health_before;
    if health < 6 {
        match eat {
            None => health = health.wrapping_add(2),
            Some(eat) if eat.hp_value() > 0 => {
                let gain = if difficulty_code == 0 {
                    eat.hp_value().wrapping_mul(2)
                } else {
                    eat.hp_value()
                };
                health = health.wrapping_add(gain);
            }
            Some(_) => {}
        }
    }
    let food = eat.map_or(10, CellEatData::food_value);
    CellEatingNumerics {
        health_after: health.min(6),
        food_value: if food == 0 { None } else { Some(food) },
        trigger_bomb: eat.is_some_and(|eat| eat.bomb().0 != 0),
        trigger_poison_nova: eat.is_some_and(|eat| eat.poison_nova().0 != 0),
    }
}

fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn float(bytes: &[u8], at: usize) -> f32 {
    f32::from_bits(word(bytes, at))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_label_handles_surrogates_and_never_consumes_numeric_tail() {
        let mut bytes = [0; DEFINITION_SIZE];
        let text = format!("{}\u{1f30d}", "A".repeat(78));
        for (i, unit) in text.encode_utf16().enumerate() {
            bytes[4 + i * 2..6 + i * 2].copy_from_slice(&unit.to_le_bytes());
        }
        bytes[0xa4..0xa8].copy_from_slice(&0x0042_0042u32.to_le_bytes());
        let value = parse(&bytes).unwrap();
        assert_eq!(value.embedded_label().unwrap(), text);
        assert_eq!(value.label_trailing_word(), 0x0042_0042);
        assert_eq!(
            parse(&bytes[..DEFINITION_SIZE - 1])
                .unwrap_err()
                .actual_size,
            DEFINITION_SIZE - 1
        );
        assert!(parse(&[0; DEFINITION_SIZE + 1]).is_err());
    }

    #[test]
    fn source_variants_keep_native_offsets_float_bits_and_byte_flags() {
        let mut bytes = [0xA5; DEFINITION_SIZE];
        for (variant, bits) in [
            (CellAiVariant::Ai, 0x8000_0000u32),
            (CellAiVariant::AiHard, 0x7fc1_2345),
            (CellAiVariant::AiEasy, 0x3f40_0000),
        ] {
            let at = variant.source_offset() + 0x18;
            bytes[at..at + 4].copy_from_slice(&bits.to_le_bytes());
        }
        bytes[0x30c..0x310].copy_from_slice(&(-12i32).to_le_bytes());
        bytes[0x310..0x314].copy_from_slice(&7i32.to_le_bytes());
        bytes[0x314] = 0;
        bytes[0x315] = 1;
        let value = parse(&bytes).unwrap();
        assert_eq!(value.ai(CellAiVariant::Ai).speed().to_bits(), 0x8000_0000);
        assert_eq!(
            value.ai(CellAiVariant::AiHard).speed().to_bits(),
            0x7fc1_2345
        );
        assert_eq!(
            value.ai(CellAiVariant::AiEasy).speed().to_bits(),
            0x3f40_0000
        );
        assert_eq!(value.ai(CellAiVariant::Ai).flocking(), CellBool(0xA5));
        assert_eq!(value.eat().food_value(), -12);
        assert_eq!(value.eat().hp_value(), 7);
        assert_eq!(value.eat().bomb().canonical_value(), Some(false));
        assert_eq!(value.eat().poison_nova().canonical_value(), Some(true));
        assert_eq!(
            value.ai(CellAiVariant::Ai).flocking().canonical_value(),
            None
        );
        assert_eq!(value.raw(), &bytes);
    }

    #[test]
    fn eating_keeps_native_health_cap_difficulty_and_zero_food_order() {
        let mut bytes = [0; DEFINITION_SIZE];
        bytes[0x310..0x314].copy_from_slice(&1i32.to_le_bytes());
        bytes[0x314] = 2;
        bytes[0x315] = 1;
        let value = parse(&bytes).unwrap();
        // The source runs effects/healing before its zero-food early return.
        assert_eq!(
            eating_numerics(1, Some(value), 0),
            CellEatingNumerics {
                health_after: 3,
                food_value: None,
                trigger_bomb: true,
                trigger_poison_nova: true,
            }
        );
        assert_eq!(eating_numerics(1, Some(value), 1).health_after, 2);
        assert_eq!(eating_numerics(1, Some(value), 2).health_after, 2);
        assert_eq!(eating_numerics(5, Some(value), 0).health_after, 6);
        assert_eq!(eating_numerics(8, Some(value), 0).health_after, 6);
        assert_eq!(
            eating_numerics(0, None, 0),
            CellEatingNumerics {
                health_after: 2,
                food_value: Some(10),
                trigger_bomb: false,
                trigger_poison_nova: false,
            }
        );
        bytes[0x310..0x314].copy_from_slice(&(-1i32).to_le_bytes());
        bytes[0x30c..0x310].copy_from_slice(&7i32.to_le_bytes());
        let value = parse(&bytes).unwrap();
        assert_eq!(eating_numerics(3, Some(value), 0).health_after, 3);
        assert_eq!(eating_numerics(3, Some(value), 0).food_value, Some(7));
    }

    #[test]
    fn ai_difficulty_inherits_only_the_native_sentinel_and_preserves_player_gate() {
        let mut bytes = [0; DEFINITION_SIZE];
        bytes[0xe0..0xe4].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x194..0x198].copy_from_slice(&u32::MAX.to_le_bytes());
        let value = parse(&bytes).unwrap();
        assert_eq!(value.selected_ai_variant(0), CellAiVariant::AiEasy);
        assert_eq!(value.selected_ai_variant(1), CellAiVariant::Ai);
        assert_eq!(value.selected_ai_variant(2), CellAiVariant::Ai);
        assert_eq!(value.selected_ai_variant(-1), CellAiVariant::Ai);
        assert!(!value.passes_native_ai_gate(0, false));
        assert!(value.passes_native_ai_gate(0, true));
        assert!(value.passes_native_ai_gate(2, false));
        bytes[0x194..0x198].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(
            parse(&bytes).unwrap().selected_ai_variant(2),
            CellAiVariant::AiHard
        );
    }
}
