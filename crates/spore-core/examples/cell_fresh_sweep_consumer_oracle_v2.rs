use std::io::{Read,Write};use spore_core::{cell_fresh_sweep_registration_v1::FreshCollection,cell_source_sweep_pairs_v1 as _};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut input=Vec::new();std::io::stdin().read_to_end(&mut input)?;if input.len()!=400{return Err("expected four 100-byte cases".into())}
 let mut out=Vec::new();for row in input.chunks_exact(100){let capacity=u32::from_le_bytes(row[..4].try_into()?) as usize;if capacity>6{return Err("capacity exceeds fixture".into())}let mut c=FreshCollection::new(4).map_err(|e|format!("{e:?}"))?;
 for actor in row[4..].chunks_exact(24){let w:Vec<u32>=actor.chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap())).collect();c.append(w[0],w[1],[w[2],w[3]],[w[4],w[5]]).map_err(|e|format!("{e:?}"))?;}
 let consumer=c.consume_fresh(capacity).map_err(|e|format!("{e:?}"))?;let sorted=consumer.sorted;let pairs=consumer.pairs;
 for v in [pairs.stored_order().len() as u32,8,0]{out.extend(v.to_le_bytes())}
 for slot in sorted.slots{out.extend(slot.actor.to_le_bytes());out.extend(slot.tag.to_le_bytes());for v in slot.endpoint_links{out.extend(v.to_le_bytes())}}
 for axis in sorted.axes{for e in axis{out.extend(e.opening.to_le_bytes());out.extend(e.slot.to_le_bytes());out.extend(e.value_bits.to_le_bytes())}}
 for i in 0..6 {if let Some(&(a,b))=pairs.stored_order().get(i){out.extend((a as u32).to_le_bytes());out.extend((b as u32).to_le_bytes())}else{out.extend([0xcc;8])}}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
