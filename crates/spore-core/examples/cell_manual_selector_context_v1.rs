//! Explicit hypothetical E6CC30 selector context over caller-supplied table bytes.
//! No actor, source creature, RNG state, installed clip, or playback is inferred.
use spore_core::{cell_anim_state_table_v1 as table, cell_request_admission_projection_v1 as admission,
                 cell_request_snapshot_selection_v1::{self as selection, Predicates}};
use std::{error::Error, fs::File, io::Read};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 { return Err("usage: cell_manual_selector_context_v1 <source-table-file>".into()); }
    let mut input = Vec::new();
    File::open(&args[0])?.take((table::TABLE_BYTES + 1) as u64).read_to_end(&mut input)?;
    let rows = table::parse_table(&input).ok_or("table extent mismatch")?;
    let predicates = vec![Predicates { predicate1: true, predicate2: true }; rows.len()];
    let context = admission::Snapshot { state: 0, actor_mode: 0, context_flags: None, parts: &[] };
    println!("CONTEXT state=0 actor_mode=0 parts=empty context=null predicates=all_true source_creature_association=none rng=explicit_draw");
    for draw in [0.0f32, 5.5, 10.5] {
        let result = selection::select_snapshot(&rows, &predicates, &context, draw.to_bits())
            .map_err(|error| format!("projection failed: {error:?}"))?;
        let indices = result.admission.candidates.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
        println!("DRAW bits={:08X} rows={} sum_bits={:08X} fallback={} key={:08X}", draw.to_bits(), indices,
                 result.admission.sum_bits, result.admission.fallback_used, result.selection.key);
    }
    Ok(())
}
