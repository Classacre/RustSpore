//! Bounded 692250 writer framing for a synthetic food/plant descriptor pair.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub frame:[u8;36],pub final_position:u32,pub end_calls:u32}
/// Caller admits exactly two stable synthetic descriptors for food +1C and
/// plant +20, a seekable memory stream, successful full writes and normally
/// returning nonreentrant callbacks. Source's outer writer serializes the header,
/// two key/length entries and field words, then restores descriptor bases. This
/// does not establish a complete live table, actual file or root ownership.
pub fn project(context:u32,food:u32,plant:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}let words=[context,28,2,0xabcd0001,4,0xabcd0011,4,food,plant];let mut frame=[0u8;36];for (i,v) in words.into_iter().enumerate(){frame[i*4..i*4+4].copy_from_slice(&v.to_be_bytes());}Ok(Outcome{frame,final_position:36,end_calls:3})
}
#[cfg(test)]mod tests{use super::*;#[test]fn framing(){let x=project(0x11223344,1,1001,true).unwrap();assert_eq!(&x.frame[0..4],&[0x11,0x22,0x33,0x44]);assert_eq!(&x.frame[28..32],&[0,0,0,1]);assert_eq!(project(0,0,0,false),Err(Error::Unadmitted));}}
