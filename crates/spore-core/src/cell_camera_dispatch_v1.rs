//! Bounded E60750 -> E60818 -> selected E60A60 actor/camera path.
use crate::{cell_camera_earlier_vector_v1 as earlier,cell_camera_actor_branch_v1 as actor,cell_runtime_data::{CellRuntimeData,CellRuntimeDataError},native_x87::Context};
pub const SELECTED_ID:u32=0x98c9_42b2;
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error{Earlier(earlier::Error),Reader(CellRuntimeDataError),Actor(actor::Error)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Branch{ModeOrActorSkip,ProviderNull,StateOutOfRange,OtherSelector(u8),Selected}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome{pub earlier:earlier::Outcome,pub b:[u32;3],pub branch:Branch,pub provider_called:bool,pub scalar:Option<u32>,pub resolved_index:Option<usize>}
/// Explicit snapshots: `actor_state_after_provider` is read after the scripted
/// E51760 return; `target_handle_after_reader` and slots are fresh after the
/// selected reader. The provider object's aggregate and all lifetimes, stable
/// source frame, and callback effects are caller admissions. Other dispatch
/// targets and E60B0D are pending. No live player or provider is identified.
pub fn project(state_b:[u32;3],state_c:[u32;3],alpha:u32,player_xy:Option<[u32;2]>,mode:u8,provider_nonzero:bool,actor_state_after_provider:u32,aggregate:&[u8],target_handle_after_reader:u32,high_water:i32,slots:&[actor::Slot],cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 let e=earlier::project(state_b,state_c,alpha,player_xy,cw,mxcsr,admitted).map_err(Error::Earlier)?;
 let base=Outcome{earlier:e,b:e.b,branch:Branch::ModeOrActorSkip,provider_called:false,scalar:None,resolved_index:None};
 if mode!=0||player_xy.is_none(){return Ok(base)}
 if !provider_nonzero{return Ok(Outcome{branch:Branch::ProviderNull,provider_called:true,..base})}
 let idx=actor_state_after_provider.wrapping_sub(7);if idx>0x27{return Ok(Outcome{branch:Branch::StateOutOfRange,provider_called:true,..base})}
 let selector=match idx{0..=2|36..=37=>0,34=>1,39=>2,_=>3};
 if selector!=1{return Ok(Outcome{branch:Branch::OtherSelector(selector),provider_called:true,..base})}
 let ctx=Context::from_control_word(cw).map_err(|x|Error::Reader(CellRuntimeDataError::X87(x)))?;
 let sample=CellRuntimeData::from_aggregate_a0(aggregate).map_err(Error::Reader)?.lookup(SELECTED_ID,ctx).map_err(Error::Reader)?;
 let selected=actor::project(target_handle_after_reader,high_water,slots,e.b,sample.value.to_bits(),mxcsr,admitted).map_err(Error::Actor)?;
 Ok(Outcome{branch:Branch::Selected,provider_called:true,scalar:Some(sample.value.to_bits()),resolved_index:selected.resolved_index,b:selected.vector,..base})
}
#[cfg(test)]mod tests{use super::*;#[test]fn dispatch_guards(){let b=[10f32.to_bits(),20f32.to_bits(),30f32.to_bits()];let c=[3f32.to_bits(),4f32.to_bits(),5f32.to_bits()];let p=Some([2f32.to_bits(),4f32.to_bits()]);
 let run=|s,provider|project(b,c,0.5f32.to_bits(),p,0,provider,s,&[0;4],0,0,&[],0x027f,0x1f80,true).unwrap();
 assert_eq!(run(0x29,false).branch,Branch::ProviderNull);assert_eq!(run(0x30,true).branch,Branch::StateOutOfRange);assert_eq!(run(7,true).branch,Branch::OtherSelector(0));assert_eq!(run(0x29,true).branch,Branch::Selected);
}}
