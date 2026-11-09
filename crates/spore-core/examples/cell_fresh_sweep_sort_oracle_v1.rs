use std::io::{Read,Write};use spore_core::cell_fresh_sweep_registration_v1::FreshCollection;
fn main()->Result<(),Box<dyn std::error::Error>>{let mut input=Vec::new();std::io::stdin().read_to_end(&mut input)?;if input.len()!=144{return Err("expected three 48-byte cases".into())}
 let mut out=Vec::new();for row in input.chunks_exact(48){let mut c=FreshCollection::new(2).map_err(|e|format!("{e:?}"))?;
 for actor in row.chunks_exact(24){let w:Vec<u32>=actor.chunks_exact(4).map(|b|u32::from_le_bytes(b.try_into().unwrap())).collect();c.append(w[0],w[1],[w[2],w[3]],[w[4],w[5]]).map_err(|e|format!("{e:?}"))?;}
 let s=c.sorted_fresh().map_err(|e|format!("{e:?}"))?;for slot in s.slots{out.extend(slot.actor.to_le_bytes());out.extend(slot.tag.to_le_bytes());for v in slot.endpoint_links{out.extend(v.to_le_bytes())}}
 for axis in s.axes {for e in axis{out.extend(e.opening.to_le_bytes());out.extend(e.slot.to_le_bytes());out.extend(e.value_bits.to_le_bytes())}}
 for _ in 0..2{out.extend(4u32.to_le_bytes())}}
 std::io::stdout().write_all(&out)?;Ok(())}
