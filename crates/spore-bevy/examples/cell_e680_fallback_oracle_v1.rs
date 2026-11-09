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
 if input.len()!=1120{return Err("expected ten 112-byte cases".into())}
 let mut out=Vec::new();for row in input.chunks_exact(112){let a=row[..56].try_into()?;let b=row[56..].try_into()?;out.extend(pool_fixture::fallback_oracle(a,b)?)}
 std::io::stdout().write_all(&out)?;Ok(())
}
