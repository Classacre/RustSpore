//! Controlled provider-to-matrix camera composition, with an explicit gap.
use crate::{cell_camera_provider_continuation_v1 as upstream,cell_camera_handoff_v1 as handoff,cell_camera_matrix_v1 as matrix};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Upstream(upstream::Error), Handoff(handoff::Error), Matrix(matrix::Error) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub upstream:upstream::Outcome, pub handoff:handoff::Handoff, pub matrix:[u32;16] }

/// `earlier_vector` is separately supplied S+1C/20/24 state; the upstream
/// chain does not produce it. The caller admits stable/disjoint snapshots,
/// scripted manager/setters, a matching host PC53 arithmetic environment and
/// no intervening mutation. The original fixture has two capture thunks.
/// No live actor/camera, actual setter side effects or auxiliary update follows.
pub fn project(aggregate:&[u8],old:[u32;3],global:[u32;3],flags:u32,direction:[u32;3],earlier_vector:[u32;3],up:[u32;3],cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 let a=upstream::project(aggregate,old,global,flags,direction,cw,mxcsr,admitted).map_err(Error::Upstream)?;
 let h=handoff::project(a.continuation.output,earlier_vector,a.provider.vector,up,mxcsr,admitted).map_err(Error::Handoff)?;
 let m=matrix::project(h.second,h.first,up,cw,mxcsr,admitted).map_err(Error::Matrix)?;
 Ok(Outcome{upstream:a,handoff:h,matrix:m})
}
#[cfg(test)]mod tests{use super::*;#[test]fn explicit_gap_and_admission(){
 let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let global=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];let dir=[2f32.to_bits(),(-3f32).to_bits(),4f32.to_bits()];let up=[0,1f32.to_bits(),0];
 let a=project(&[0;4],old,global,0,dir,old,up,0x027f,0x1f80,true).unwrap();let mut other=old;other[0]=2f32.to_bits();let b=project(&[0;4],old,global,0,dir,other,up,0x027f,0x1f80,true).unwrap();assert_ne!(a.handoff.second,b.handoff.second);
 assert!(matches!(project(&[0;4],old,global,0,dir,old,up,0x027f,0x1f80,false),Err(Error::Upstream(_))));
}}
