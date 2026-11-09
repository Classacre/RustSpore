use std::io::Write;
use spore_core::cell_effect_zero_handoff_v1::{project,FirstEffect};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let cases=[(None,None),(Some(1),Some(FirstEffect::Unchanged)),(Some(1),Some(FirstEffect::Cleared)),(Some(1),Some(FirstEffect::Replaced(2)))];let mut out=Vec::new();
 for (initial,effect) in cases {
  let p=project(initial,effect,true).map_err(|e|format!("projection: {e:?}"))?;
  // These counts/observations describe the native comparator's inert stubs;
  // the projection itself stops at a pending second callback.
  let first=u32::from(p.first_object.is_some());let second=p.pending_second_object.unwrap_or(0) as u32;let called=u32::from(second!=0);
  for v in [first,second,called,first,called]{out.extend_from_slice(&v.to_le_bytes());}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
