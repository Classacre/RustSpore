//! Compare typed Cell fields with independently extracted native-schema fixtures.
use spore_core::cell_definition::{self, CellAiVariant, DEFINITION_SIZE};
use std::io::Read;

fn semantic_digest(value: cell_definition::CellDefinition<'_>) -> (u64, usize) {
    let mut fields: Vec<(u32, u32, Vec<u8>)> = Vec::new();
    let mut add = |offset: usize, kind: u32, bytes: Vec<u8>| {
        fields.push((offset as u32, kind, bytes));
    };
    add(
        4,
        10,
        value
            .label_units()
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect(),
    );
    macro_rules! word {
        ($offset:expr, $kind:expr, $value:expr) => {
            add($offset, $kind, $value.to_le_bytes().to_vec());
        };
    }
    macro_rules! flag {
        ($offset:expr, $value:expr) => {
            add($offset, 6, vec![$value.0]);
        };
    }
    word!(0, 3, value.structure_id());
    word!(0xa4, 1, value.label_trailing_word());
    word!(0xa8, 1, value.hp());
    flag!(0xac, value.fixed_orientation());
    word!(0xb0, 1, value.flags());
    word!(0xb4, 5, value.cell_type());
    word!(0xb8, 5, value.unlock_type());
    word!(0xbc, 5, value.density());
    word!(0xc0, 5, value.sound());
    word!(0xc4, 3, value.break_id());
    word!(0xc8, 3, value.pieces_id());
    word!(0xcc, 3, value.leak_id());
    word!(0xd0, 3, value.expel_id());
    word!(0xd4, 3, value.loot_id());
    word!(0xd8, 3, value.explosion_table_id());
    word!(0xdc, 3, value.poison_id());
    for variant in [
        CellAiVariant::Ai,
        CellAiVariant::AiHard,
        CellAiVariant::AiEasy,
    ] {
        let ai = value.ai(variant);
        let base = variant.source_offset();
        for (at, kind, bits) in [
            (0x00, 5, ai.unnamed_enum()),
            (0x10, 5, ai.movement_style()),
            (0x50, 3, ai.spawn_output_id()),
            (0x5c, 1, ai.num_arcs() as u32),
            (0x60, 3, ai.transformation_id()),
            (0x64, 3, ai.projectile_id()),
            (0x68, 5, ai.food()),
            (0x6c, 1, ai.grow_count() as u32),
            (0x70, 1, ai.digestion_count() as u32),
            (0x78, 3, ai.digestion_output_id()),
            (0x9c, 1, ai.grow_amount() as u32),
        ] {
            word!(base + at, kind, bits);
        }
        for (at, number) in [
            (0x04, ai.awareness_radius()),
            (0x08, ai.awareness_radius_food()),
            (0x0c, ai.awareness_radius_predator()),
            (0x18, ai.speed()),
            (0x1c, ai.chase_speed()),
            (0x20, ai.wander_speed()),
            (0x24, ai.flee_speed()),
            (0x28, ai.fears_nearby_damage_radius()),
            (0x2c, ai.fears_nearby_death_radius()),
            (0x30, ai.fears_nearby_damage_time()),
            (0x34, ai.fears_nearby_death_time()),
            (0x38, ai.protect_radius()),
            (0x3c, ai.protect_time()),
            (0x40, ai.turn_factor()),
            (0x48, ai.spawn_time()),
            (0x4c, ai.spawn_rest_time()),
            (0x54, ai.arc_length()),
            (0x58, ai.arc_length_secondary()),
            (0x74, ai.digestion_time()),
            (0x7c, ai.flee_time()),
            (0x80, ai.flee_rest_time()),
            (0x84, ai.chase_time()),
            (0x88, ai.chase_rest_time()),
            (0x94, ai.awake_time()),
            (0x98, ai.sleep_time()),
            (0xa0, ai.hatch_duration()),
            (0xa4, ai.poison_recharge()),
            (0xa8, ai.electric_recharge()),
            (0xac, ai.electric_recharge_vs_small()),
            (0xb0, ai.electric_discharge()),
        ] {
            word!(base + at, 2, number.to_bits());
        }
        for (at, byte) in [
            (0x14, ai.flocking()),
            (0x44, ai.axial_movement()),
            (0x8c, ai.chases_damage()),
            (0x8d, ai.fears_mouths()),
            (0x8e, ai.fears_weapons()),
            (0x8f, ai.fears_electric()),
            (0x90, ai.fears_poison()),
            (0x91, ai.fears_damage()),
            (0x92, ai.ignores_food()),
        ] {
            flag!(base + at, byte);
        }
    }
    word!(0x2fc, 1, value.friend_group());
    flag!(0x300, value.wont_attack_player());
    flag!(0x301, value.wont_attack_player_when_small());
    add(
        0x304,
        11,
        value
            .size_range()
            .into_iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect(),
    );
    word!(0x30c, 1, value.eat().food_value());
    word!(0x310, 1, value.eat().hp_value());
    flag!(0x314, value.eat().bomb());
    flag!(0x315, value.eat().poison_nova());
    flag!(0x318, value.triggers_escape_mission());
    fields.sort_by_key(|field| field.0);
    let mut hash = 0xcbf29ce484222325u64;
    let mut end = 0;
    for (offset, kind, bytes) in &fields {
        assert!(*offset >= end, "overlapping typed fields");
        end = offset + bytes.len() as u32;
        for byte in offset
            .to_le_bytes()
            .into_iter()
            .chain(kind.to_le_bytes())
            .chain((bytes.len() as u32).to_le_bytes())
            .chain(bytes.iter().copied())
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    (hash, fields.len())
}

fn rules_digest(value: cell_definition::CellDefinition<'_>) -> u64 {
    let mut bytes = Vec::new();
    for difficulty in [-1i32, 0, 1, 2, 3] {
        bytes.extend(difficulty.to_le_bytes());
        bytes.extend((value.selected_ai_variant(difficulty).source_offset() as u32).to_le_bytes());
        bytes.extend([
            value.passes_native_ai_gate(difficulty, false) as u8,
            value.passes_native_ai_gate(difficulty, true) as u8,
        ]);
        for present in [false, true] {
            for health in [-1i32, 0, 1, 4, 5, 6, 7] {
                let outcome = cell_definition::eating_numerics(
                    health,
                    if present { Some(value) } else { None },
                    difficulty,
                );
                bytes.extend(health.to_le_bytes());
                bytes.push(present as u8);
                bytes.extend(outcome.health_after.to_le_bytes());
                bytes.push(outcome.food_value.is_some() as u8);
                bytes.extend(outcome.food_value.unwrap_or(0).to_le_bytes());
                bytes.extend([
                    outcome.trigger_bomb as u8,
                    outcome.trigger_poison_nova as u8,
                ]);
            }
        }
    }
    bytes.into_iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_definition_verify <RSCELLD1 fixture>")?;
    let rules = std::env::args_os()
        .nth(2)
        .map(
            |path| -> Result<Vec<(u64, u64)>, Box<dyn std::error::Error>> {
                let content = std::fs::read_to_string(path)?;
                content
                    .lines()
                    .enumerate()
                    .map(|(ordinal, line)| {
                        let values: Vec<_> = line.split('\t').collect();
                        if values.len() != 3 || values[0].parse::<usize>()? != ordinal {
                            return Err("invalid rule fixture row".into());
                        }
                        Ok((
                            u64::from_str_radix(values[1], 16)?,
                            u64::from_str_radix(values[2], 16)?,
                        ))
                    })
                    .collect()
            },
        )
        .transpose()?;
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..8] != b"RSCELLD1" {
        return Err("wrong Cell fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 100_000 {
        return Err("invalid Cell fixture count".into());
    }
    if rules.as_ref().is_some_and(|r| r.len() != count as usize) {
        return Err("rule fixture count differs from resource fixture".into());
    }
    let mut total_fields = 0;
    for ordinal in 0..count {
        let mut record = [0; 20];
        input.read_exact(&mut record)?;
        let id = u64::from_le_bytes(record[..8].try_into()?);
        let length = u32::from_le_bytes(record[8..12].try_into()?) as usize;
        let expected = u64::from_le_bytes(record[12..].try_into()?);
        if length != DEFINITION_SIZE {
            return Err(format!("wrong Cell record size at {ordinal}").into());
        }
        let mut bytes = [0; DEFINITION_SIZE];
        input.read_exact(&mut bytes)?;
        let value = cell_definition::parse(&bytes)?;
        value.embedded_label()?;
        let (actual, fields) = semantic_digest(value);
        if actual != expected {
            return Err(format!("Cell {id:016X}: typed fields differ").into());
        }
        if let Some(rules) = &rules {
            let (expected_id, expected_digest) = rules[ordinal as usize];
            if expected_id != id || rules_digest(value) != expected_digest {
                return Err(format!("Cell {id:016X}: native rule branches differ").into());
            }
        }
        total_fields += fields;
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing Cell fixture bytes".into());
    }
    println!("CELL_DEFINITION_VERIFY resources={count} fields={total_fields} rule_resources={} exact=true",
        if rules.is_some() { count } else { 0 });
    Ok(())
}
