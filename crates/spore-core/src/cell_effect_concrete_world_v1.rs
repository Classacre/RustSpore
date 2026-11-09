//! A712F0 early rejection/negative lookup joined to the 627DE0 fallback.
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub enum Error{Unadmitted,PositiveLookup}
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub struct Outcome{pub lookup_ids:[Option<u32>;2],pub slot_after:u32}
/// A nonnull stable world/slot and normally returning lookup are admitted.
/// Both scripted lookup callbacks must preserve world +20 and the slot: the
/// original second invocation reads world +20 afresh after the first lookup.
/// Lookup outcomes are supplied signed values. Positive lookup would allocate
/// and is excluded, as are live provider, ownership and placement.
pub fn project(id:u32,slot:u32,world_state:u32,lookup:[i32;2],admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if world_state==5{return Ok(Outcome{lookup_ids:[None,None],slot_after:slot})}
 if lookup[0]>=0||lookup[1]>=0{return Err(Error::PositiveLookup)}
 Ok(Outcome{lookup_ids:[Some(id),Some(0xe18d6423)],slot_after:slot})
}
#[cfg(test)]mod tests{use super::*;#[test]fn gate_and_miss(){assert_eq!(project(7,8,5,[0,0],true).unwrap().lookup_ids,[None,None]);assert_eq!(project(7,8,0,[-1,-2],true).unwrap().lookup_ids,[Some(7),Some(0xe18d6423)]);assert_eq!(project(7,8,0,[-1,0],true),Err(Error::PositiveLookup));}}
