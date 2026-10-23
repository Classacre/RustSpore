//! Compare typed globals with native metadata, preserving the old-layout exclusion.
use spore_core::cell_globals::{self, CellGlobals};
use std::io::Read;

fn semantic_digest(value: CellGlobals<'_>) -> u64 {
    let mut rows: Vec<(u32, u32, u32)> = Vec::new();
    for (at, kind, word) in [
        (0x00, 5, value.game_mode()),
        (0x04, 3, value.world_1()),
        (0x08, 3, value.world_2()),
        (0x0c, 3, value.world_3()),
        (0x10, 3, value.world_4()),
        (0x14, 3, value.world_5()),
        (0x18, 3, value.world_background_1()),
        (0x1c, 3, value.world_background_2()),
        (0x20, 3, value.world_background_3()),
        (0x24, 3, value.world_background_4()),
        (0x28, 3, value.world_background_5()),
        (0x2c, 3, value.world_random()),
        (0x30, 3, value.world_random_bg()),
        (0x34, 3, value.start_cell()),
        (0x38, 7, value.starting_cell_key()),
        (0x3c, 3, value.effect_map_entry()),
        (0x40, 3, value.background_map_entry()),
        (0x84, 3, value.key_look_algorithm()),
        (0xc0, 1, value.num_high_lod_fg() as u32),
        (0xc4, 1, value.num_high_lod_bg() as u32),
        (0xd4, 5, value.control_method()),
        (0xd8, 5, value.editor_method()),
        (0xdc, 5, value.tutorial_method()),
        (0xe0, 5, value.ending_method()),
        (0xe4, 5, value.eye_method()),
    ] {
        rows.push((at, kind, word));
    }
    for (at, number) in [
        (0x44, value.flow_multiplier()),
        (0x48, value.npc_speed_multiplier()),
        (0x4c, value.npc_turn_speed_multiplier_jet()),
        (0x50, value.npc_turn_speed_multiplier_flagella()),
        (0x54, value.npc_turn_speed_multiplier_cilia()),
        (0x58, value.density_rock()),
        (0x5c, value.density_solid()),
        (0x60, value.density_liquid()),
        (0x64, value.density_air()),
        (0x68, value.background_distance()),
        (0x6c, value.min_drag_collision_speed()),
        (0x70, value.min_impact_collision_speed()),
        (0x74, value.cilia_speed_as_jet()),
        (0x78, value.flagella_speed_as_jet()),
        (0x7c, value.flagella_speed_as_cilia()),
        (0x80, value.cilia_speed_as_flagella()),
        (0x88, value.beach_distance()),
        (0x8c, value.finish_line_distance()),
        (0x90, value.no_part_speed()),
        (0x94, value.flagella_ramp_min_factor()),
        (0x98, value.flagella_ramp_time()),
        (0x9c, value.flagella_ramp_reset_angle()),
        (0xa0, value.flagella_turn_speed_ramp_start()),
        (0xa4, value.flagella_turn_speed_ramp_end()),
        (0xa8, value.flagella_turn_speed_min()),
        (0xac, value.flagella_turn_speed_max()),
        (0xb0, value.cilia_turn_speed()),
        (0xb4, value.jet_turn_speed()),
        (0xb8, value.start_level_no_creature_radius()),
        (0xbc, value.start_level_no_anything_radius()),
        (0xc8, value.percent_animal_food()),
        (0xcc, value.percent_plant_food()),
        (0xe8, value.time_to_goldy_cinematic()),
        (0xec, value.mission_time()),
        (0xf0, value.mission_reset_time()),
        (0xf4, value.escape_min_distance()),
        (0xf8, value.escape_max_distance()),
        (0xfc, value.escape_delay_medium()),
        (0x100, value.escape_timer_hard()),
        (0x104, value.non_animating_cilia_movement_factor()),
        (0x108, value.non_animating_jet_movement_factor()),
        (0x10c, value.mate_trigger_distance()),
        (0x110, value.mate_spawn_distance()),
    ] {
        rows.push((at, 2, number.to_bits()));
    }
    rows.sort_by_key(|row| row.0);
    assert_eq!(rows.len(), 68);
    let mut hash = 0xcbf29ce484222325u64;
    for (offset, kind, word) in rows {
        for byte in offset
            .to_le_bytes()
            .into_iter()
            .chain(kind.to_le_bytes())
            .chain(4u32.to_le_bytes())
            .chain(word.to_le_bytes())
        {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    hash
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: cell_globals_verify <RSCELLG1 fixture>")?;
    let mut input = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut header = [0; 12];
    input.read_exact(&mut header)?;
    if &header[..8] != b"RSCELLG1" {
        return Err("wrong globals fixture magic".into());
    }
    let count = u32::from_le_bytes(header[8..].try_into()?);
    if count == 0 || count > 10000 {
        return Err("invalid globals fixture count".into());
    }
    let (mut accepted, mut excluded) = (0, 0);
    for ordinal in 0..count {
        let mut record = [0; 13];
        input.read_exact(&mut record)?;
        let length = u32::from_le_bytes(record[..4].try_into()?) as usize;
        let supported = match record[4] {
            0 => false,
            1 => true,
            _ => return Err("invalid globals support flag".into()),
        };
        let expected = u64::from_le_bytes(record[5..].try_into()?);
        if length > 1024 * 1024 {
            return Err("globals fixture payload exceeds bound".into());
        }
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes)?;
        match cell_globals::parse(&bytes) {
            Ok(value) if supported => {
                if semantic_digest(value) != expected {
                    return Err(format!("globals row {ordinal}: fields differ").into());
                }
                accepted += 1;
            }
            Err(error) if !supported && error.actual_size == length => excluded += 1,
            _ => return Err(format!("globals row {ordinal}: support state differs").into()),
        }
    }
    if input.read(&mut [0])? != 0 {
        return Err("trailing globals fixture bytes".into());
    }
    println!(
        "CELL_GLOBALS_VERIFY records={count} supported={accepted} excluded={excluded} exact=true"
    );
    Ok(())
}
