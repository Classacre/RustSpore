use std::io::Write;
use spore_core::{cell_effect_pool_route_v1::Route,cell_effect_placement_prefix_v1::{project,Input},native_sse::Policy,native_x87::Context};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let table=[0x41200000,0x41f00000,0x42c80000,0x43960000,0x447a0000,0x453b8000,0x461c4000,0x46ea6000,0x47c35000,0x48927c00,0x49742400,0x4a371b00,0x4b189680,0x4be4e1c0,0x4cbebc20,0x4d8f0d18];
 let cases=[(3,[2,-3],0x3f800000,0x447a0000,false),(11,[-1,4],0x40000000,0x459c4000,true),(3,[16777217,-16777217],0x3f000000,0x461c4000,true),(11,[0,0],0x3f800000,0x3f800000,false)];
 let ctx=Context::from_control_word(0x027f).map_err(|e|format!("context: {e:?}"))?;let mut out=Vec::new();
 for (level,grid,helper,global,present) in cases{
  let x=Input{route:Route::PendingNonzeroPlacement,level,grid,spacing_bits:&table,helper_return_bits:helper,global_514c_bits:global,effect_present:present,stable_disjoint:true};
  let p=project(x,Policy::Mxcsr1f80,ctx).map_err(|e|format!("placement: {e:?}"))?;
  for v in [u32::from(p.pending_existing)].into_iter().chain(p.corner_words).chain([level]){out.extend_from_slice(&v.to_le_bytes())}
 }
 std::io::stdout().write_all(&out)?;Ok(())
}
