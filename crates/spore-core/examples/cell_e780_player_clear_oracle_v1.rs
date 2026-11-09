use std::io::Write;
use spore_core::cell_e780_player_clear_v1::{project,Boundary};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let slots=[0x10000,0x20001];let cases=[(0,0x10000),(0x10000,0x10000),(0x10000,0x20001),(0x30000,0x30000),(0x20001,0x20001),(0x30002,0x30002)];let mut out=Vec::new();
 for (actor,player) in cases {
  let result=project(actor,player,1,&slots,true,true).map_err(|e|format!("projection: {e:?}"))?;
  let continued=u32::from(matches!(result.boundary,Boundary::FreshLookupPending{..}));
  for word in [result.player_word,continued,actor] {out.extend_from_slice(&word.to_le_bytes())}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
