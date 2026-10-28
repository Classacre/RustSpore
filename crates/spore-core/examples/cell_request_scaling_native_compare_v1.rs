//! Binary adapter for the bounded public seeded-selection model.
//! Input: state, nonnegative finite f32 sum bits, masked CW (three LE u32s).
//! Output: status, resulting state, draw bits (three LE u32s).
//! Status 1 means the model rejected the unrepresented negative-zero case.
use spore_core::{cell_anim_state_table_v1 as table, cell_request_admission_projection_v1 as admission,
    cell_request_rng_v1::Rng, cell_request_seeded_selection_v1 as selection,
    cell_request_snapshot_selection_v1 as snapshot_selection, native_x87::Context};
use std::io::{self, Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Vec::new(); io::stdin().read_to_end(&mut data)?;
    if data.len() % 12 != 0 { return Err("partial request".into()); }
    let mut out = io::BufWriter::new(io::stdout().lock());
    for row in data.chunks_exact(12) {
        let state = u32::from_le_bytes(row[0..4].try_into()?);
        let sum = u32::from_le_bytes(row[4..8].try_into()?);
        let cw = u32::from_le_bytes(row[8..12].try_into()?);
        if state == 0 || sum & 0x8000_0000 != 0 || sum & 0x7f80_0000 == 0x7f80_0000 ||
            cw & !0xf00 != 0x7f || !matches!(cw & 0x300, 0x200 | 0x300) { return Err("invalid request".into()); }
        let record = table::Record { words: [7, 42, sum, 2, 1, 0x12, 0, 0, 0x34] };
        let snap = admission::Snapshot { state: 7, actor_mode: 0, context_flags: None, parts: &[] };
        let predicate = snapshot_selection::Predicates { predicate1: true, predicate2: true };
        let mut rng = Rng::new(state).map_err(|_| "zero seed")?;
        let context = Context::from_control_word(cw as u16)?;
        let result = selection::select_snapshot_seeded(&[record], &[predicate], &snap, &mut rng, context);
        let (status, bits) = match result {
            Ok(value) => {
                if value.admission.candidates != [0] || value.admission.sum_bits != sum { return Err("fixture admission mismatch".into()); }
                (0u32, value.draw_bits)
            }
            Err(selection::Error::UnrepresentedNegativeZero) => (1u32, 0u32),
            Err(other) => return Err(format!("model error: {other:?}").into()),
        };
        out.write_all(&status.to_le_bytes())?;
        out.write_all(&rng.state().to_le_bytes())?;
        out.write_all(&bits.to_le_bytes())?;
    }
    out.flush()?; Ok(())
}
