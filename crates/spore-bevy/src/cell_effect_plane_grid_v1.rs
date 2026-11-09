//! Controlled source-derived plane placement; all runtime state is supplied.
use spore_core::{cell_effect_placement_prefix_v1::Input,cell_effect_pool_route_v1::Route,cell_effect_tier_v1::{placement_with_helper,TierRow},cell_effect_transform_v1,native_sse::Policy,native_x87::Context};
/// Format: level,progress,grid_x,grid_y,+514C binary32 hex word.
/// The source tier/spacing table words are the bounded audited PE snapshot.
/// Identity basis and stable snapshots are explicit inspection admissions.
pub fn parse_and_project(text:&str)->Result<[f32;4],String>{
 let f:Vec<_>=text.split(',').collect();if f.len()!=5||f[4].len()!=8||!f[4].bytes().all(|b|b.is_ascii_hexdigit()){return Err("effect plane grid requires level,progress,grid_x,grid_y,514c_hex".into())}
 let level=f[0].parse::<u32>().map_err(|_|"invalid level")?;let progress=f[1].parse::<i32>().map_err(|_|"invalid progress")?;let gx=f[2].parse::<i32>().map_err(|_|"invalid grid x")?;let gy=f[3].parse::<i32>().map_err(|_|"invalid grid y")?;let scalar=u32::from_str_radix(f[4],16).map_err(|_|"invalid 514c")?;
 let rows:Vec<_>=(0..22).map(|i|TierRow{tier:if i==0{0}else{(((i+1)/2) as u32).min(10)},threshold:if i<2{0}else{(i as i32-1)*50}}).collect();
 let spacing=[10f32,30.,100.,300.,1000.,3000.,10000.,30000.,100000.,300000.,1000000.,3000000.,10000000.,30000000.,100000000.,300000000.].map(f32::to_bits);
 let input=Input{route:Route::PendingNonzeroPlacement,level,grid:[gx,gy],spacing_bits:&spacing,helper_return_bits:0,global_514c_bits:scalar,effect_present:false,stable_disjoint:true};
 let ctx=Context::from_control_word(0x027f).map_err(|e|format!("x87 context: {e:?}"))?;let p=placement_with_helper(input,progress,&rows,10,true,Policy::Mxcsr1f80,ctx).map_err(|e|format!("effect placement rejected: {e:?}"))?;
 let one=1f32.to_bits();let basis=[one,0,0,0,one,0,0,0,one];let t=cell_effect_transform_v1::project(p.corner_words,basis,Policy::Mxcsr1f80,true).map_err(|e|format!("effect transform rejected: {e:?}"))?;let r=[f32::from_bits(t[1]),f32::from_bits(t[2]),f32::from_bits(t[4]),f32::from_bits(t[4])];
 if r.iter().any(|v|!v.is_finite())||r[0].abs()>1000.||r[1].abs()>1000.||r[2]<=0.||r[2]>1000.{return Err("effect plane grid projection outside admitted bounds".into())}Ok(r)
}
#[cfg(test)]mod tests{use super::*;#[test]fn controlled_and_reject(){let x=parse_and_project("3,0,0,0,3f0ccccd").unwrap();assert!(x[2]>57.&&x[2]<59.);assert_eq!(x[0].to_bits(),(x[2]*0.5).to_bits());assert_eq!(x[2],x[3]);for bad in ["3,0,0,0,00000000","4,0,0,0,3f0ccccd","3,1001,0,0,3f0ccccd","3,0,2147483647,0,3f0ccccd","3,0,0,0,nothex"]{assert!(parse_and_project(bad).is_err(),"{bad}");}}}
#[cfg(test)]mod mapping_tests{use super::*;#[test]fn unit_quad_maps_to_source_grid_corners(){
 // The pinned mesh is [-0.5,+0.5] on both axes; the source transform's
 // midpoint and uniform x-span map this square to these controlled corners.
 let r=parse_and_project("3,0,0,0,3f0ccccd").unwrap();let left=r[0]-0.5*r[2];let right=r[0]+0.5*r[2];let bottom=r[1]-0.5*r[3];let top=r[1]+0.5*r[3];assert!(left.abs()<1e-5&&bottom.abs()<1e-5);assert!((right-r[2]).abs()<1e-5&&(top-r[3]).abs()<1e-5);
 let r=parse_and_project("3,0,-1,-1,3f0ccccd").unwrap();assert!((r[0]+0.5*r[2]).abs()<1e-5&&(r[1]+0.5*r[3]).abs()<1e-5);
}}
