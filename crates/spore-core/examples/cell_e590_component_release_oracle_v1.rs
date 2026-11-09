use std::io::Write;
use spore_core::cell_e590_component_release_v1::{project,Pool,Snapshot};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let records=[0x10000];let actors=[0x10000];let pool=|slot_words|Pool{signed_bound:0,slot_words,equal_bound_readable:true};let mut out=Vec::new();
 for (record_handle,record_actor_handle) in [(0x20000,0x10000),(0x10000,0x30000),(0x10000,0x10000)]{
  let x=Snapshot{record_handle,record_pool:pool(&records),record_slot_word:0x10000,record_actor_handle,actor_pool:Some(pool(&actors)),actor_36c:0x11111111,actor_370:0x22222222,record_free_head:u32::MAX,record_live:1,stable_disjoint_owned:true};
  let p=project(x).map_err(|e|format!("projection: {e:?}"))?;
  for word in [p.record_slot_word,p.record_free_head,p.record_live,p.actor_36c,p.actor_370]{out.extend_from_slice(&word.to_le_bytes())}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
