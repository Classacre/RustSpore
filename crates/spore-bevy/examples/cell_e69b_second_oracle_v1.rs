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
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut input=Vec::new();std::io::stdin().read_to_end(&mut input)?;
 if input.len()!=12{return Err("expected four 3-byte cases".into())}
 let mut out=Vec::new();for row in input.chunks_exact(3){if row[0]>1{return Err("invalid presence".into())}for v in pool_fixture::e69b_second_oracle(row[0]!=0,row[1],row[2])?{out.extend(v.to_le_bytes())}}
 std::io::stdout().write_all(&out)?;Ok(())
}
