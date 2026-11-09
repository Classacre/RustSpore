//! Bounded E80D75 retain/store/release projection; synthetic callback effects.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, MissingFresh, UnsupportedControl }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Action { FreshLookup, Retain(u32), Store(Option<u32>), Release(u32), MarkPending }
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Outcome { pub root_5190:Option<u32>, pub root_51da:u8, pub actions:Vec<Action> }
/// `old` is the source root slot snapshot: after the fresh helper returns on the fresh branch, or after reading the pending global on the pending branch. Zero is None.
/// The caller admits stable disjoint root/globals/objects, completing callbacks
/// that preserve root+5190 and all synthetic objects. FreshLookup is scripted;
/// no allocation, refcount, pointer validity or live ownership is established.
pub fn project(old:Option<u32>,pending:Option<u32>,fresh:Option<Option<u32>>,flag:u8,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted||old==Some(0)||pending==Some(0)||fresh==Some(Some(0)){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut actions=Vec::new();let pending_branch=pending.is_some();
 let next=if pending_branch{pending}else{actions.push(Action::FreshLookup);fresh.ok_or(Error::MissingFresh)?};
 if next!=old {if let Some(id)=next{actions.push(Action::Retain(id));}actions.push(Action::Store(next));if let Some(id)=old{actions.push(Action::Release(id));}}
 let flag=if pending_branch{actions.push(Action::MarkPending);1}else{flag};
 Ok(Outcome{root_5190:next,root_51da:flag,actions})
}
#[cfg(test)]mod tests{use super::*;#[test]fn ordering_and_same_pointer(){let x=project(Some(1),Some(2),None,0x5a,0x27f,0x1f80,true).unwrap();assert_eq!(x.actions,vec![Action::Retain(2),Action::Store(Some(2)),Action::Release(1),Action::MarkPending]);let y=project(Some(1),None,Some(Some(1)),0x5a,0x27f,0x1f80,true).unwrap();assert_eq!(y.actions,vec![Action::FreshLookup]);assert_eq!(y.root_51da,0x5a);assert_eq!(project(None,None,None,0,0x27f,0x1f80,true),Err(Error::MissingFresh));}}
