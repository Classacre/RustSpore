//! Bounded provider-to-camera continuation using explicitly supplied state.
use crate::{cell_camera_provider_join_v1 as provider,cell_camera_continuation_v1 as continuation};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Provider(provider::Error), Continuation(continuation::Error) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub provider:provider::Outcome, pub continuation:continuation::Outcome }

/// Both stages use the same admitted stable aggregate, vectors and control
/// environment. The provider's third output word is the continuation input,
/// matching the source S+34 handoff. The original native fixture uses a
/// capture thunk at that boundary. Live provider/global/camera identity and
/// the next handoff remain unestablished.
pub fn project(aggregate:&[u8],old:[u32;3],global:[u32;3],flags:u32,old_direction:[u32;3],cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 let p=provider::project(aggregate,old,global,cw,mxcsr,admitted).map_err(Error::Provider)?;
 let c=continuation::project(flags,old_direction,p.vector[2],cw,mxcsr,admitted).map_err(Error::Continuation)?;
 Ok(Outcome{provider:p,continuation:c})
}
#[cfg(test)]mod tests{use super::*;#[test]fn source_handoff_and_admission(){
 let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let global=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];let v=[2f32.to_bits(),(-3f32).to_bits(),4f32.to_bits()];
 let x=project(&[0;4],old,global,0,v,0x027f,0x1f80,true).unwrap();assert_eq!(x.provider.vector,old);assert_eq!(x.continuation.vector,[0,0xbf000000,0]);
 assert!(matches!(project(&[0;4],old,global,0,v,0x027f,0x1f80,false),Err(Error::Provider(provider::Error::Unadmitted))));
}}
