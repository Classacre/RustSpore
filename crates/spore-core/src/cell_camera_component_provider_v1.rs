//! Controlled E51760 component projection joined to the camera dispatch.
use crate::{cell_camera_dispatch_v1 as dispatch,cell_camera_actor_branch_v1::Slot};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnreadableComponent, Dispatch(dispatch::Error) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub projected_component:Option<usize>, pub dispatch:dispatch::Outcome }

/// Caller supplies fresh actor +248/+FC snapshots at E51760, stable disjoint
/// component-pool storage and valid nonwrapping base/stride addressing, plus
/// fresh post-provider state and post-reader actor lookup snapshots. A boolean
/// component entry denotes whether its supplied +24 request pointer is nonzero;
/// the corresponding aggregate is separately supplied. Source B71D00 uses
/// only the low 16 bits and does not check the stored generation. Other source
/// dispatch targets, real ownership and the next camera provider remain pending.
pub fn project(state_b:[u32;3],state_c:[u32;3],alpha:u32,player_xy:Option<[u32;2]>,mode:u8,component_handle:u32,actor_fc:u32,components:&[bool],actor_state_after_provider:u32,aggregate:&[u8],target_handle_after_reader:u32,high_water:i32,slots:&[Slot],cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let mut projected=None;let mut provider=false;
 if mode==0&&player_xy.is_some()&&component_handle!=0&&actor_fc!=0{
  let index=(component_handle&0xffff) as usize;
  provider=*components.get(index).ok_or(Error::UnreadableComponent)?;projected=Some(index);
 }
 let d=dispatch::project(state_b,state_c,alpha,player_xy,mode,provider,actor_state_after_provider,aggregate,target_handle_after_reader,high_water,slots,cw,mxcsr,admitted).map_err(Error::Dispatch)?;
 Ok(Outcome{projected_component:projected,dispatch:d})
}
#[cfg(test)]mod tests{use super::*;#[test]fn gate_and_unchecked_alias(){let b=[10f32.to_bits(),20f32.to_bits(),30f32.to_bits()];let c=[3f32.to_bits(),4f32.to_bits(),5f32.to_bits()];let actor=Some([2f32.to_bits(),4f32.to_bits()]);
 let run=|h,fc,components:&[bool]|project(b,c,0.5f32.to_bits(),actor,0,h,fc,components,0x29,&[0;4],0,0,&[],0x027f,0x1f80,true);
 assert_eq!(run(0,1,&[]).unwrap().projected_component,None);assert_eq!(run(0x10000,0,&[]).unwrap().projected_component,None);
 assert_eq!(run(0x10000,1,&[true]).unwrap().projected_component,Some(0));assert_eq!(run(0x20000,1,&[true]).unwrap().projected_component,Some(0));
 assert_eq!(run(0x10001,1,&[true]),Err(Error::UnreadableComponent));
}}
