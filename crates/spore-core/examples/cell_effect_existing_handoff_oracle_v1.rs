use std::io::Write;
use spore_core::{cell_effect_existing_handoff_v1::project,cell_effect_placement_prefix_v1::Projection,cell_effect_pool_route_v1::Route};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut out=Vec::new();
 for weight in [0x3f800000,0x3f40009f,0x3362b819,0x3f800000] {
  let p=project(Route::PendingNonzeroPlacement,Projection{corner_words:[0;6],pending_existing:true},1,2,weight,true).map_err(|e|format!("{e:?}"))?;
  // The last two words describe the comparator's inert method stub, not live callbacks.
  for v in [p.first_argument,p.weight_bits,p.last_argument,1,1] {out.extend_from_slice(&v.to_le_bytes());}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
