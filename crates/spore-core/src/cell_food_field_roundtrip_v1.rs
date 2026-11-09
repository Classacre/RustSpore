//! Bounded food-progression field callback roundtrip on a synthetic stream.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub source:u32,pub encoded:[u8;4],pub destination:u32,pub write_calls:u32,pub read_calls:u32}
/// The caller admits distinct stable fields, a normally returning nonreentrant
/// stream chain, successful full four-byte write/read and unchanged bytes between
/// calls. The supplied dword is not a live save or actor state. Descriptor table
/// traversal, actual files, short/error reads and ownership remain unverified.
pub fn project(field:u32,admitted:bool)->Result<Outcome,Error>{if !admitted{return Err(Error::Unadmitted)}let encoded=field.to_be_bytes();Ok(Outcome{source:field,encoded,destination:u32::from_be_bytes(encoded),write_calls:1,read_calls:1})}
#[cfg(test)]mod tests{use super::*;#[test]fn words(){let x=project(1000,true).unwrap();assert_eq!(x.encoded,[0,0,3,0xe8]);assert_eq!(x.destination,1000);assert_eq!(project(0,false),Err(Error::Unadmitted));}}
