//! Bounded 91FBA0 descriptor lookup and creation dispatch.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingLookup,MissingMethod,UnsupportedControl}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{Lookup{key:u32},Method{descriptor:u32,key:u32,arg2:u32}}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub actions:Vec<Action>,pub result:Option<u32>}
/// If `explicit_descriptor` is absent, `lookup_result` is a fresh result from
/// the scripted factory+14 callback; a null result ends the route. Otherwise
/// the explicit descriptor bypasses lookup. `method_result` is fresh from that
/// descriptor's +8 callback and can differ from the descriptor. Valid objects,
/// stable lifetimes and callbacks returning are admitted; no registry contents,
/// object creation or live descriptor identity is inferred.
pub fn project(key:u32,arg2:u32,explicit_descriptor:Option<u32>,lookup_result:Option<Option<u32>>,method_result:Option<Option<u32>>,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted||explicit_descriptor==Some(0)||lookup_result==Some(Some(0))||method_result==Some(Some(0)){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut actions=Vec::new();let desc=match explicit_descriptor{Some(v)=>Some(v),None=>{actions.push(Action::Lookup{key});lookup_result.ok_or(Error::MissingLookup)?}};
 let Some(id)=desc else{return Ok(Outcome{actions,result:None})};actions.push(Action::Method{descriptor:id,key,arg2});let result=method_result.ok_or(Error::MissingMethod)?;Ok(Outcome{actions,result})
}
#[cfg(test)]mod tests{use super::*;#[test]fn lookup_and_explicit_descriptor(){let x=project(1,2,None,Some(Some(3)),Some(Some(4)),0x27f,0x1f80,true).unwrap();assert_eq!(x.actions,vec![Action::Lookup{key:1},Action::Method{descriptor:3,key:1,arg2:2}]);assert_eq!(x.result,Some(4));let y=project(1,2,Some(3),None,Some(None),0x37f,0x1f80,true).unwrap();assert_eq!(y.actions,vec![Action::Method{descriptor:3,key:1,arg2:2}]);assert_eq!(y.result,None);}}
