//! Checked B1E280 mode-manager prefix. No live manager or service is accessed.
pub const CREATURE_MODE:u32=0x01654c01;
pub const PREVIOUS_MODE:u32=0xfffffffe;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Changed {pub old:u32,pub requested:u32,pub restored_argument:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Projection {pub return_al:u8,pub manager_28:u8,pub changed:Option<Changed>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unstable,MissingServiceByte,UnusedServiceByte}
/// Caller admits stable/disjoint manager and service lifetimes, normal return
/// from all scripted callbacks, and no callback mutation of +20. Changed
/// projects call inputs and the +28 store, but does not execute the selector,
/// service method, state setter or establish a completed mode switch.
/// `service_byte` is supplied only if the changed path reads it. The final
/// setter argument preserves upper bits of the original request stack slot.
pub fn project(request:u32,current:u32,previous:u32,initial_28:u8,service_byte:Option<u8>,stable_disjoint:bool)->Result<Projection,Error>{
 if !stable_disjoint{return Err(Error::Unstable)}
 let requested=if request==PREVIOUS_MODE{previous}else{request};
 if requested==current {if service_byte.is_some(){return Err(Error::UnusedServiceByte)}return Ok(Projection{return_al:1,manager_28:initial_28,changed:None})}
 let byte=service_byte.ok_or(Error::MissingServiceByte)?;
 Ok(Projection{return_al:1,manager_28:1,changed:Some(Changed{old:current,requested,restored_argument:(request&0xffffff00)|u32::from(byte)})})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn exact_and_previous_requests(){assert_eq!(project(CREATURE_MODE,CREATURE_MODE,0,0x5a,None,true).unwrap().changed,None);let p=project(PREVIOUS_MODE,0,CREATURE_MODE,0x5a,Some(0xa5),true).unwrap();assert_eq!(p.changed,Some(Changed{old:0,requested:CREATURE_MODE,restored_argument:0xffffffa5}));assert_eq!(p.manager_28,1);}
 #[test]fn requires_only_admitted_service_reads(){assert_eq!(project(1,2,0,0,None,true),Err(Error::MissingServiceByte));assert_eq!(project(1,1,0,0,Some(0),true),Err(Error::UnusedServiceByte));assert_eq!(project(1,2,0,0,Some(0),false),Err(Error::Unstable));}
}
