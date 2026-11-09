use std::io::Write;
use spore_core::cell_e780_root_unlink_v1::{project,Snapshot};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut out=Vec::new();
 for (count,head) in [(1,0),(7,55)] {
  let x=Snapshot{actor_word:0x10000,fresh_slot_word:0x10000,actor_token:11,container_token:22,manager_token:33,node_token:44,container_root:44,manager_count:count,old_free_head:head,stable_disjoint_owned:true};
  let p=project(x).map_err(|e|format!("projection: {e:?}"))?;
  for word in [p.manager_count,p.container_root as u32,u32::from(p.node_next==55),u32::from(p.manager_free_head==44)] {out.extend_from_slice(&word.to_le_bytes())}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
