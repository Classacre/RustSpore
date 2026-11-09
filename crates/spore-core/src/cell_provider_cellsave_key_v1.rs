//! Bounded E4F490 provider enumeration and E5C920 class key callbacks.
pub const CELL_SAVE_KEY:u32=0xabcdef00;
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,UnsupportedControl}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub count:u32,pub buffer_after:Option<u32>,pub class_key:u32}
/// A supplied non-null buffer represents one valid writable dword; no buffer is
/// dereferenced when absent or capacity is zero. Callback objects and lifetime
/// are caller-admitted. Registry membership and live save ownership are unknown.
pub fn project(buffer:Option<u32>,capacity:u32,cw:u16,mx:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(cw,0x027f|0x037f)||mx!=0x1f80{return Err(Error::UnsupportedControl)}
 Ok(Outcome{count:1,buffer_after:buffer.map(|x|if capacity!=0{CELL_SAVE_KEY}else{x}),class_key:CELL_SAVE_KEY})
}
#[cfg(test)]mod tests{use super::*;#[test]fn writes_only_when_pointer_and_capacity_present(){assert_eq!(project(None,4,0x27f,0x1f80,true).unwrap().buffer_after,None);assert_eq!(project(Some(7),0,0x27f,0x1f80,true).unwrap().buffer_after,Some(7));assert_eq!(project(Some(7),2,0x37f,0x1f80,true).unwrap().buffer_after,Some(CELL_SAVE_KEY));assert_eq!(project(None,0,0x27f,0x1f80,false),Err(Error::Unadmitted));}}
