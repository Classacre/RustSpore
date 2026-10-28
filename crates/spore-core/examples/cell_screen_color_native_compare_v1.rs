//! Binary adapter: four little-endian float bit patterns in; status and word out.
use spore_core::cell_screen_quad_v1::pack_color_mxcsr1f80;
use std::io::{self, Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
 let mut input = Vec::new(); io::stdin().read_to_end(&mut input)?;
 if input.len() % 16 != 0 { return Err("partial request".into()); }
 let mut output = io::BufWriter::new(io::stdout().lock());
 for row in input.chunks_exact(16) {
  let components = std::array::from_fn(|i| f32::from_bits(u32::from_le_bytes(row[i*4..i*4+4].try_into().unwrap())));
  let (status, color) = match pack_color_mxcsr1f80(components) { Ok(v) => (0u32, v), Err(_) => (1, 0) };
  output.write_all(&status.to_le_bytes())?; output.write_all(&color.to_le_bytes())?;
 }
 output.flush()?; Ok(())
}
