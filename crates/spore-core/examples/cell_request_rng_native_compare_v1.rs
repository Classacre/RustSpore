use spore_core::cell_request_rng_v1::Rng;
use std::io::{self, Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Vec::new(); io::stdin().read_to_end(&mut data)?;
    if data.len() % 8 != 0 { return Err("partial request".into()); }
    let mut out = io::BufWriter::new(io::stdout().lock());
    for row in data.chunks_exact(8) {
        let state = u32::from_le_bytes(row[0..4].try_into()?);
        let cw = u32::from_le_bytes(row[4..8].try_into()?);
        if state == 0 || cw & !0xf00 != 0x7f || !matches!(cw & 0x300, 0x200 | 0x300) { return Err("invalid request".into()); }
        let mut rng = Rng::new(state).map_err(|_| "zero seed")?;
        let value = rng.next_fraction().as_f64();
        out.write_all(&rng.state().to_le_bytes())?;
        out.write_all(&value.to_le_bytes())?;
    }
    out.flush()?; Ok(())
}
