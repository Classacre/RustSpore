use spore_core::{creature_generated_skin_grid_pack_v1::{filter_and_pack, CandidateAdmission, CellGeometry},creature_generated_skin_kernel_v1::Lane, native_sse::Policy};
fn main() {
 let rows = [(0.,1.,0.,1.),(1.,1.,0.,1.),(2.,4.,0.25,2.),(3.,9.,0.0625,1.),(4.,16.,0.03125,0.5),(5.,25.,0.,1.),(6.,36.,0.,1.),(7.,49.,0.,1.)];
 let lanes: Vec<_> = rows.into_iter().enumerate().map(|(i,(x,cutoff,scale,coefficient))| Lane {center:[x,0.,0.],cutoff,scale,coefficient,key:i as u32}).collect();
 let cases = [(0.,0.,0.),(0.,0.,1.),(0.,0.,f32::from_bits(1f32.to_bits()+1)),(0.,1.,0.),(2.,1.,0.),(2.,1.,3.)];
 for (center,extent,threshold) in cases {
   let cells=[CellGeometry{center:[center,0.,0.],half_extent:[extent,0.,0.]};512];
   let grid=filter_and_pack(&lanes,&cells,threshold,Policy::Mxcsr1f80,CandidateAdmission).unwrap();
   let binding=grid.cells();
   let out: Vec<_> = binding[0].unwrap_or(&[]).iter().flat_map(|r|r.iter()).filter(|l|l.cutoff!=0.).map(|l|l.key.to_string()).collect();
   println!("{}",out.join(","));
 }
}
