//! Bounded 91FBE0 factory acquisition, validation and release ordering.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingValidator,UnsupportedControl}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{Acquire{key:u32,arg3:u32,arg4:u32},Validate{candidate:u32,type_id:u32},ReleaseFirst(u32),ReleaseSecond(u32)}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub actions:Vec<Action>,pub result:Option<u32>}
/// Candidate is the fresh return of the scripted factory+24 callback. If it is
/// present, validator_result is the fresh return of candidate+0C and may differ
/// from candidate; zero validator result triggers source +0 then +4 callbacks.
/// Stable disjoint objects and normally returning callbacks are caller admissions;
/// registration, factory behavior, reference counts and live identity are not inferred.
pub fn project(key:u32,type_id:u32,arg3:u32,arg4:u32,candidate:Option<u32>,validator_result:Option<Option<u32>>,x87:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted||candidate==Some(0)||validator_result==Some(Some(0)){return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut actions=vec![Action::Acquire{key,arg3,arg4}];
 let Some(id)=candidate else{return Ok(Outcome{actions,result:None})};
 actions.push(Action::Validate{candidate:id,type_id});let result=validator_result.ok_or(Error::MissingValidator)?;
 if result.is_none(){actions.push(Action::ReleaseFirst(id));actions.push(Action::ReleaseSecond(id));}
 Ok(Outcome{actions,result})
}
#[cfg(test)]mod tests{use super::*;#[test]fn candidate_and_result_stay_distinct(){let x=project(1,2,0,0,Some(3),Some(Some(4)),0x27f,0x1f80,true).unwrap();assert_eq!(x.result,Some(4));assert_eq!(x.actions,vec![Action::Acquire{key:1,arg3:0,arg4:0},Action::Validate{candidate:3,type_id:2}]);let y=project(1,2,0,0,Some(3),Some(None),0x37f,0x1f80,true).unwrap();assert_eq!(y.actions[2..],[Action::ReleaseFirst(3),Action::ReleaseSecond(3)]);assert_eq!(project(1,2,0,0,Some(3),None,0x27f,0x1f80,true),Err(Error::MissingValidator));}}
