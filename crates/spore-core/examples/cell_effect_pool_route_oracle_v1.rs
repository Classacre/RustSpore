use std::io::Write;
use spore_core::{cell_effect_curve_v1::Curve,cell_effect_pool_route_v1::{route,from_installed_curve,Route},native_x87::Context};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let c=Context::from_control_word(0x027f).map_err(|e|format!("context: {e:?}"))?;
 let mut r=Vec::new();
 for (kind,level,wanted,weight) in [(0,3,3,0x3f800000),(1,4,3,0x3f800000),(1,3,3,0),(1,3,3,0x80000000),(1,3,3,0x3f800000),(1,3,3,0xbf800000)]{r.push(route(kind,level,wanted,weight,true).map_err(|e|format!("route: {e:?}"))?);}
 for (curve,level,scalar) in [(Curve::Near,3,0.),(Curve::Near,3,24000.),(Curve::Far,11,14950.)]{r.push(from_installed_curve(curve,level,scalar,c,1,level,true).map_err(|e|format!("curve: {e:?}"))?.1);}
 let mut out=Vec::new();for v in r{let n:u32=match v{Route::NoMatch=>0,Route::PendingZeroEffectInspection=>1,Route::PendingNonzeroPlacement=>2};out.extend_from_slice(&n.to_le_bytes());}std::io::stdout().write_all(&out)?;Ok(())
}
