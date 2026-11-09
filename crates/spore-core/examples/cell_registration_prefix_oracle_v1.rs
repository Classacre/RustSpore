use std::io::{Read,Write};
use spore_core::{cell_actor_geometry_projection_v1::Output,cell_registration_prefix_v1::{self as prefix,Input,ResultV1}};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut input=Vec::new();std::io::stdin().read_to_end(&mut input)?;
 if input.len()!=256{return Err("expected eight 32-byte cases".into())}
 let mut out=Vec::new();for row in input.chunks_exact(32){let w:Vec<u32>=row.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();
 let input=Input{original_manager:w[0],current_manager:w[1],argument10:w[2],definition_b4:w[3],actor_handle:w[4],geometry:Output{center:[w[5],w[6],0],scalar:w[7]},post_e86960_stable_disjoint:true};
 let result=match prefix::evaluate(input).map_err(|e|format!("{e:?}"))?{ResultV1::Skipped=>[0,0xcccccccc,0xcccccccc,0xcccccccc,0xcccccccc,0xcccccccc],ResultV1::Register{tag,min,max}=>[1,tag,min[0],min[1],max[0],max[1]]};
 for word in result{out.extend(word.to_le_bytes())}}
 std::io::stdout().write_all(&out)?;Ok(())
}
