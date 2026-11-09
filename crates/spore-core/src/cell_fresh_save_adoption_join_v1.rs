//! Conditional E61390 -> E61470 mode-3 -> E80D75 composition.
use crate::{cell_fresh_save_constructor_v1 as constructor, cell_save_ownership_projection_v1 as ownership};
#[derive(Clone,Debug,PartialEq,Eq)] pub enum Error { Constructor(constructor::Error), Ownership(ownership::Error), InvalidIdentity }
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Outcome { pub object:[u8;0xec], pub adoption:ownership::Outcome }
/// Models successful construction followed by the source mode-3 branch and
/// fresh-pointer adoption. The caller supplies a distinct nonzero object identity,
/// its post-cleanup synthetic vtable, helper outputs and old slot *after* the
/// scripted helper returns. Allocator, helper effects, callbacks, identity and
/// lifetimes are admitted; no live state, actual refcount or mode choice follows.
pub fn project(initial:[u8;0xec],new_id:u32,old_after_helper:Option<u32>,synthetic_vtable:u32,helper_78:u32,helper_34:u32,scale:u32,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if new_id==0 || synthetic_vtable==0 || old_after_helper==Some(new_id){return Err(Error::InvalidIdentity)}
 let mut made=constructor::construct(initial,helper_78,helper_34,x87,mxcsr,admitted).map_err(Error::Constructor)?;
 made.object[0xc..0x10].copy_from_slice(&3u32.to_le_bytes());
 made.object[0x7c..0x80].copy_from_slice(&scale.to_le_bytes());
 made.object[..4].copy_from_slice(&synthetic_vtable.to_le_bytes());
 let adoption=ownership::project(old_after_helper,None,Some(Some(new_id)),made.root_51da,x87,mxcsr,admitted).map_err(Error::Ownership)?;
 Ok(Outcome{object:made.object,adoption})
}
#[cfg(test)]mod tests{use super::*;use crate::cell_save_ownership_projection_v1::Action;#[test]fn composed_fresh_and_replace(){let x=project([0xa5;0xec],2,Some(1),0x1234,7,8,9,0x27f,0x1f80,true).unwrap();assert_eq!(&x.object[0xc..0x10],&3u32.to_le_bytes());assert_eq!(x.adoption.actions,vec![Action::FreshLookup,Action::Retain(2),Action::Store(Some(2)),Action::Release(1)]);assert_eq!(x.adoption.root_51da,0);assert_eq!(project([0;0xec],2,Some(2),1,0,0,0,0x27f,0x1f80,true),Err(Error::InvalidIdentity));}}
