//! Bounded 93A3F0 single-dword read with caller-supplied callback effects.
#[derive(Debug,PartialEq,Eq)]
pub struct Outcome { pub returned: bool, pub destination: u32, pub sync_calls: u32, pub status_calls: u32, pub read_calls: u32 }
#[derive(Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, MissingRead, UnexpectedRead, UnsupportedCount }
/// Models one four-byte destination with stable synthetic stream and callbacks
/// returning normally without reentrancy. `post_read` is the fresh destination
/// after the scripted byte callback, even for a partial read. A status failure
/// skips that callback and preserves `initial`. Mode zero swaps only after a
/// full four-byte return; other admitted mode is one. No live I/O or caller
/// behavior is established.
pub fn project(mode:u8, initial:u32, status_ok:bool, read:Option<(u32,u32)>, admitted:bool)->Result<Outcome,Error>{
 if !admitted || mode>1 {return Err(Error::Unadmitted)}
 let sync_calls=u32::from(mode==0);
 if !status_ok {if read.is_some(){return Err(Error::UnexpectedRead)}return Ok(Outcome{returned:false,destination:initial,sync_calls,status_calls:1,read_calls:0})}
 let (count,post_read)=read.ok_or(Error::MissingRead)?;
 if count>4{return Err(Error::UnsupportedCount)}
 let returned=count==4;
 Ok(Outcome{returned,destination:if returned && mode==0 {post_read.swap_bytes()} else {post_read},sync_calls,status_calls:1,read_calls:1})
}
#[cfg(test)] mod tests {use super::*;#[test] fn partial_and_full(){let x=project(0,0xaabbccdd,true,Some((2,0xaabb2211)),true).unwrap();assert!(!x.returned);assert_eq!(x.destination,0xaabb2211);assert_eq!(project(0,0,true,Some((4,0x44332211)),true).unwrap().destination,0x11223344);assert_eq!(project(1,0,true,Some((4,0x44332211)),true).unwrap().destination,0x44332211);assert!(project(0,0,false,Some((0,0)),true).is_err());}}
