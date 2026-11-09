//! Bounded 9315B0 open-result behavior with synthetic OS results.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingCreateResult,MissingErrorResult}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Fields{pub f214:u32,pub f218:u32,pub f21c:u32,pub f220:u32,pub f224:u32,pub f228:u32}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:bool,pub handle:u32,pub fields:Fields,pub create_calls:u32,pub error_calls:u32}
/// Handle/arguments and `fields_after_stubs` are supplied snapshots after any
/// synthetic OS callbacks return; callbacks preserve stream +4 and no other
/// intervening mutation is admitted. OS handle
/// and error are opaque scripted results. A real file, path, side effects and
/// ownership are not established. No actual OS operation is performed here.
pub fn project(handle:u32,access:u32,mode:u32,share:u32,flags:u32,fields_after_stubs:Fields,created:Option<u32>,last_error:Option<u32>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if handle!=u32::MAX||access==0{return Ok(Outcome{returned:handle!=u32::MAX,handle,fields:fields_after_stubs,create_calls:0,error_calls:0})}
 let h=created.ok_or(Error::MissingCreateResult)?;let mut f=fields_after_stubs;
 if h==u32::MAX{f.f224=last_error.ok_or(Error::MissingErrorResult)?;return Ok(Outcome{returned:false,handle:h,fields:f,create_calls:1,error_calls:1})}
 f.f214=access;f.f218=if mode==6{if access&2!=0{4}else{3}}else{mode};f.f21c=share;f.f220=flags;f.f224=0;f.f228=if access&1!=0&&access&2==0&&share&2==0{0xffff_fffe}else{u32::MAX};
 Ok(Outcome{returned:true,handle:h,fields:f,create_calls:1,error_calls:0})
}
#[cfg(test)]mod tests{use super::*;#[test]fn results(){let f=Fields{f214:1,f218:2,f21c:3,f220:4,f224:5,f228:6};let x=project(u32::MAX,1,6,1,0,f,Some(0),None,true).unwrap();assert!(x.returned);assert_eq!((x.handle,x.fields.f218,x.fields.f228),(0,3,0xffff_fffe));assert_eq!(project(u32::MAX,1,6,1,0,f,Some(u32::MAX),Some(87),true).unwrap().fields.f224,87);}}

