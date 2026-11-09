//! Bounded 8D9040 stream path setter before its scripted assignment callback.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingTerminator}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:bool,pub field_264:u32,pub field_268:u32,pub assign_units:Option<usize>}
/// Stable object storage and a normally returning nonreentrant assign callback
/// are caller admissions. Active objects skip the path read and field writes.
/// Otherwise the first UTF-16 NUL selects the assigned range after both fields
/// are cleared. Actual allocation, string mutation and file access are excluded.
pub fn project(active:u32,field_264:u32,field_268:u32,path:&[u16],admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if active!=0{return Ok(Outcome{returned:false,field_264,field_268,assign_units:None})}
 let n=path.iter().position(|&x|x==0).ok_or(Error::MissingTerminator)?;
 Ok(Outcome{returned:true,field_264:0,field_268:0,assign_units:Some(n)})
}
#[cfg(test)]mod tests{use super::*;#[test]fn path_range_and_active_skip(){assert_eq!(project(0,1,2,&[0x58,0,0x59,0],true).unwrap().assign_units,Some(1));assert_eq!(project(7,1,2,&[],true).unwrap().field_264,1);assert_eq!(project(0,1,2,&[65],true),Err(Error::MissingTerminator));}}
