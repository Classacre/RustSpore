//! Controlled B1DF30 Cell-to-Creature generic-path handoff.
//! Descriptor mutations are supplied scripts; no allocator, string helper,
//! service callback, free or live mode switch is performed here.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unstable,UnsupportedModes,InvalidBuffer,InsufficientCapacity}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Projection{
 pub buffer_token:u64,
 pub initial_literal:u32,
 pub suffix_literal:u32,
 /// A service lookup is pending after the caller-admitted helper script.
 pub pending_service:bool,
 /// The source cleanup branch is pending after the service returns normally.
 pub pending_cleanup:bool,
}
/// The fixed compared helper script writes the two literals and terminator
/// into a supplied buffer and sets the descriptor capacity end to buffer+capacity.
/// A stable, nonaliased buffer, helper success, descriptor integrity and normal
/// callback return are explicit fixture admissions. Token is not a pointer.
/// This restricted positive capacity avoids wrap/alias and is an extra safety
/// rule, not an assertion that the original allocator uses this layout.
pub fn cell_to_creature(old:u32,new:u32,buffer_token:u64,capacity:u32,script_stable:bool)->Result<Projection,Error>{
 if !script_stable{return Err(Error::Unstable)}
 if old!=0x1654c00||new!=0x1654c01{return Err(Error::UnsupportedModes)}
 if buffer_token==0{return Err(Error::InvalidBuffer)}
 if !(25..=i32::MAX as u32).contains(&capacity){return Err(Error::InsufficientCapacity)}
 Ok(Projection{buffer_token,initial_literal:0x145e428,suffix_literal:0x145e3ec,pending_service:true,pending_cleanup:true})
}
#[cfg(test)]mod tests{use super::*;#[test]fn admitted_script_and_rejections(){let p=cell_to_creature(0x1654c00,0x1654c01,1,32,true).unwrap();assert_eq!((p.initial_literal,p.suffix_literal,p.pending_service,p.pending_cleanup),(0x145e428,0x145e3ec,true,true));assert_eq!(cell_to_creature(0,0,1,32,true),Err(Error::UnsupportedModes));assert_eq!(cell_to_creature(0x1654c00,0x1654c01,0,32,true),Err(Error::InvalidBuffer));assert_eq!(cell_to_creature(0x1654c00,0x1654c01,1,24,true),Err(Error::InsufficientCapacity));assert_eq!(cell_to_creature(0x1654c00,0x1654c01,1,32,false),Err(Error::Unstable));}}
