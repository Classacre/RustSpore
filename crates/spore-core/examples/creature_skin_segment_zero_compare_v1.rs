use spore_core::{creature_skin_segment_zero_threshold_v1::{sample,MathAdmission},native_sse::Policy,native_x87::{Context,Precision,Rounding}};
fn main() {
 let ctx=Context{precision:Precision::Bits53,rounding:Rounding::NearestEven};
 let cases=[([0.,0.,0.],[1.,0.,0.],1.,0.3,0x80000001),([2.,-1.,1.],[2.,-1.,1.4],0.2,0.3,0x80000002),([0.,0.,0.],[0.1,0.,0.],1.,0.3,7),([-1.,2.,0.],[0.5,2.,0.],0.5,0.3,9)];
 for (a,b,r,s,key) in cases {let lanes=sample(a,b,r,s,key,Policy::Mxcsr1f80,ctx,MathAdmission).unwrap(); print!("{}",lanes.len());for l in lanes {for v in l.center.map(f32::to_bits).into_iter().chain([l.cutoff.to_bits(),l.scale.to_bits(),l.coefficient.to_bits(),l.key]) {print!(",{v}");}}println!();}
}
