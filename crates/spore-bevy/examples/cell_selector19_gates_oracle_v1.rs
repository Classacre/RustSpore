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
use std::io::Write;
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut out=Vec::new();
 for bytes in [[0,0],[1,0],[0,1],[2,2]] {
  for value in pool_fixture::selector19_oracle(bytes)? {out.extend_from_slice(&value.to_le_bytes());}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
