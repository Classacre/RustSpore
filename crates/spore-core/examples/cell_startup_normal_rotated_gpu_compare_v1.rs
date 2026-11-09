//! Controlled yzx-rotation/positive-uniform-scale GPU normals versus MeshData.
use spore_core::{gmdl,mesh};
fn f32_at(b:&[u8],at:usize)->f32 { f32::from_le_bytes(b[at..at+4].try_into().unwrap()) }
fn main(){
 let a:Vec<_>=std::env::args_os().collect();assert_eq!(a.len(),3,"pinned model and float target required");
 let model=std::fs::read(&a[1]).unwrap();let target=std::fs::read(&a[2]).unwrap();assert_eq!(target.len(),66*43*16);
 let parsed=gmdl::parse(&model).unwrap();let decoded=mesh::from_gmdl(&model,&parsed).unwrap();assert_eq!(decoded.normals.len(),2555);assert!(decoded.normals_from_file);
 let mut rows=Vec::with_capacity(2555);
 for (i,n) in decoded.normals.iter().enumerate(){
  let x=1+i%64;let y=1+i/64;let at=(y*66+x)*16;
  let g=[f32_at(&target,at),f32_at(&target,at+4),f32_at(&target,at+8),f32_at(&target,at+12)];
  assert!(g.iter().all(|x|x.is_finite()) && g[3]==1.0);
  // Exact axis permutation of the already normalized CPU normal. Positive
  // uniform scale does not change direction mathematically.
  let diff=(g[0]-n.y).abs().max((g[1]-n.z).abs()).max((g[2]-n.x).abs());rows.push((diff,i));
 }
 let mean=rows.iter().map(|x|f64::from(x.0)).sum::<f64>()/rows.len() as f64;rows.sort_by(|a,b|a.0.total_cmp(&b.0));let (maximum,worst)=rows[2554];
 println!("candidate_count=2555 mean_max_component_delta={mean:.12} median={:.12} p95={:.12} max={maximum:.12} worst_index={worst} over_0_01={} sanitised={} scope=yzx_rotation_uniform_scale_2_no_bevy_gpu_or_lighting_claim",rows[1277].0,rows[2427].0,rows.iter().filter(|v|v.0>0.01).count(),decoded.sanitised);
}
