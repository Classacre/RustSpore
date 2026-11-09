//! Bounded E61350 old-save release and provider handoff.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedControl }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Action { MarkGlobal, ClearSlot, Release(u32), ProviderWithSlotAndZero }
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Outcome { pub actions:Vec<Action>, pub global_flag:u8, pub post_provider_slot:Option<u32> }
/// `old` is the slot value at source entry; `post_provider_slot` is a fresh
/// supplied snapshot after the provider returns. Caller admits stable disjoint
/// root/objects, a release callback that preserves slot/global flag, a provider
/// that preserves the flag and returns normally, and valid thiscall lifetimes.
/// This does not model destruction, provider behavior or live save identity.
pub fn project(old:Option<u32>,post_provider_slot:Option<u32>,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted || old==Some(0) || post_provider_slot==Some(0){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut actions=vec![Action::MarkGlobal];
 if let Some(id)=old {actions.push(Action::ClearSlot);actions.push(Action::Release(id));}
 actions.push(Action::ProviderWithSlotAndZero);
 Ok(Outcome{actions,global_flag:1,post_provider_slot})
}
#[cfg(test)]mod tests{use super::*;#[test]fn preserves_order_and_fresh_provider_result(){let x=project(Some(1),Some(2),0x27f,0x1f80,true).unwrap();assert_eq!(x.actions,vec![Action::MarkGlobal,Action::ClearSlot,Action::Release(1),Action::ProviderWithSlotAndZero]);assert_eq!(x.post_provider_slot,Some(2));assert_eq!(project(None,None,0x37f,0x1f80,true).unwrap().actions,vec![Action::MarkGlobal,Action::ProviderWithSlotAndZero]);assert_eq!(project(Some(0),None,0x27f,0x1f80,true),Err(Error::Unadmitted));}}
