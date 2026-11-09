#![allow(dead_code,unused_imports)]
#[cfg(test)]
#[path="../src/cell_source_part_provider_v1.rs"] mod cell_source_part_provider_v1;
#[cfg(test)]
#[path="../src/cell_source_player_v1.rs"] mod cell_source_player_v1;
#[path="../src/cell_source_capability_v1.rs"] mod cell_source_capability_v1;
#[path="../src/cell_source_feature_geometry_v1.rs"] mod feature_geometry;
#[path="../src/cell_source_contact_call_v1.rs"] mod contact_call;
#[path="../src/cell_source_contact_record_v1.rs"] mod contact_record;
#[path="../src/cell_source_player_pool_fixture_v1.rs"] mod pool_fixture;
use std::io::{Read,Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut input=Vec::new(); std::io::stdin().read_to_end(&mut input)?;
 if input.len()!=168 { return Err("expected seven 24-byte cases".into()); }
 let mut output=Vec::new();
 for row in input.chunks_exact(24) {
  let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();
  if w[0]>1 {return Err("bad equal flag".into());}
  let values=pool_fixture::coordinator_oracle(w[0]!=0,[w[1],w[2]],[w[3] as i32,w[4] as i32],w[5])?;
  if values.len()!=43{return Err("bad output width".into());}
  for value in values {output.extend_from_slice(&value.to_le_bytes());}
 }
 std::io::stdout().write_all(&output)?; Ok(())
}
