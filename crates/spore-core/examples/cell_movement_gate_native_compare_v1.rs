//! Binary adapter: 10 little-endian u32 inputs and 3 u32 outputs per row.
use spore_core::{cell_movement_animation_gate_v1::{self as m, Environment, Snapshot}, native_x87::Context, native_sse::Policy};
use std::io::{self, Read, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
 let mut data=Vec::new(); io::stdin().read_to_end(&mut data)?;
 if data.len()%40!=0 { return Err("partial request".into()); }
 let mut out=io::BufWriter::new(io::stdout().lock());
 for bytes in data.chunks_exact(40) {
  let v: Vec<u32>=bytes.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();
  if v[..8].iter().any(|x| x & 0x7f800000 == 0x7f800000) || v[8]>0xffff || v[8]&!0xf00!=0x7f || !matches!(v[8]&0x300,0x200|0x300) || v[9]!=0x1f80 { return Err("invalid input".into()); }
  let f=|i| f32::from_bits(v[i]);
  let mut s=Snapshot {position_08:[f(0),f(1),f(2)],target_4c:[f(3),f(4),f(5)],timer_368:f(6),state_18c:0};
  let result=m::gate(&mut s,f(7),Environment { x87:Context::from_control_word(v[8] as u16)?,sse:Policy::Mxcsr1f80 });
  let (status,al)=match result {Ok(b)=>(0u32,b as u32),Err(_)=>(1,0)};
  for w in [status,al,s.timer_368.to_bits()] {out.write_all(&w.to_le_bytes())?;}
 }
 out.flush()?; Ok(())
}
