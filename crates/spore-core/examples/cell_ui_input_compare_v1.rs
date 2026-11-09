//! Fixed-record adapter for reviewed E5BE20/selector host output.
use spore_core::{cell_progression_v1::ProgressStateV1,cell_ui_progress_input_v1::from_snapshot,native_x87::Context};
use std::{error::Error,io::{self,Read,Write}};
fn main()->Result<(),Box<dyn Error>> {
 let mut bytes=Vec::new();io::stdin().take(4097).read_to_end(&mut bytes)?;
 if bytes.is_empty()||bytes.len()>4096||bytes.len()%32!=0{return Err("bad host extent".into())}
 let mut out=io::BufWriter::new(io::stdout().lock());
 for row in bytes.chunks_exact(32) {
  let word=|i|u32::from_le_bytes(row[i..i+4].try_into().unwrap());let cw=word(0);let progress=word(4);
  if !matches!(cw,0x027f|0x037f)||progress>1000{return Err("input outside fixture".into())}
  let state=ProgressStateV1 { progress_1c:progress as i32,counter_20:0,counter_24:0,counter_28:0 };
  let v=from_snapshot(state,Context::from_control_word(cw as u16)?).map_err(|e|format!("model {e:?}"))?;
  writeln!(out,"{cw:08X} {progress:08X} {:08X} {:08X} {:08X} {:08X}",v.value.to_bits(),v.selection.segment,v.selection.bucket,v.selection.fraction.to_bits())?;
 }
 out.flush()?;Ok(())
}
