//! Controlled E4EE10 world-callback/setter and request-store composition.
use crate::cell_camera_request_store_v1 as store;
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error{Store(store::Error),MissingBacking,UnreadableBacking}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Backing{pub field_2d8:u32,pub field_2dc:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome{pub store:store::Outcome,pub attach_called:bool,pub backings:[Backing;2]}
/// The factory result, fresh request +17C after world+40, and backing contents
/// after that callback are supplied snapshots. The caller admits stable,
/// disjoint storage, normally returning world callbacks, and that request
/// +17C remains unchanged across the two original setters. The initial backing
/// snapshots apply on paths without a nonzero factory return. Actual world
/// selection, allocation, resource identity and live ownership are excluded.
pub fn project(triple:[u32;3],initial_request:u32,factory_result:Option<u32>,handle_after:u32,fc_after:u32,component_index:usize,structure_word:u32,backings:[Backing;2],backing_after_attach:Option<usize>,admitted:bool)->Result<Outcome,Error>{
 let s=store::project(triple,initial_request,factory_result,handle_after,fc_after,component_index,admitted).map_err(Error::Store)?;
 let attach=s.factory_called&&s.component_request!=0;let mut result=backings;
 if attach{let index=backing_after_attach.ok_or(Error::MissingBacking)?;let target=result.get_mut(index).ok_or(Error::UnreadableBacking)?;target.field_2d8=structure_word;target.field_2dc=1;}
 Ok(Outcome{store:s,attach_called:attach,backings:result})
}
#[cfg(test)]mod tests{use super::*;#[test]fn fresh_backing(){let initial=[Backing{field_2d8:1,field_2dc:2},Backing{field_2d8:3,field_2dc:4}];let x=project([1,0x3d97a8e4,9],0,Some(42),0x10000,1,0,7,initial,Some(1),true).unwrap();assert_eq!(x.backings[0],initial[0]);assert_eq!(x.backings[1],Backing{field_2d8:7,field_2dc:1});assert_eq!(x.store.provider_return,42);assert_eq!(project([1,0x3d97a8e4,9],0,Some(42),0x10000,1,0,7,initial,None,true),Err(Error::MissingBacking));}}
