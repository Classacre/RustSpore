//! Controlled two-field 692250 writer / 693710 reader composition.
use crate::cell_food_full_writer_v1;
#[derive(Debug, PartialEq, Eq)] pub enum Error { Unadmitted, Frame }
#[derive(Debug, PartialEq, Eq)] pub struct Outcome { pub frame: [u8; 36], pub food: u32, pub plant: u32 }
/// Admits exactly the two stable sorted synthetic descriptors, the supplied
/// context and normally returning inert callbacks with full successful reads,
/// writes and seeks. Identity, file persistence, arbitrary counts/errors and
/// live root ownership are not established. Native destination fields and
/// descriptor restoration are checked by the host guards, not its output rows.
pub fn project(context:u32,food:u32,plant:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted { return Err(Error::Unadmitted); }
 let frame=cell_food_full_writer_v1::project(context,food,plant,true).map_err(|_|Error::Unadmitted)?.frame;
 let word=|i:usize|u32::from_be_bytes(frame[i*4..i*4+4].try_into().unwrap());
 if [word(0),word(1),word(2),word(3),word(4),word(5),word(6)] != [context,28,2,0xabcd0001,4,0xabcd0011,4] {return Err(Error::Frame)}
 Ok(Outcome{frame,food:word(7),plant:word(8)})
}
#[cfg(test)] mod tests{use super::*;#[test]fn controlled_roundtrip(){let r=project(0x11223344,17,1001,true).unwrap();assert_eq!((r.food,r.plant),(17,1001));assert_eq!(project(0,0,0,false),Err(Error::Unadmitted));}}
