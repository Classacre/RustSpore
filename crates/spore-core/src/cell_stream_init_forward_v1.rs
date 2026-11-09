//! Bounded 69D940 → 8D7850 stream initializer projection.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:bool,pub stream_byte_0c:u8}
/// Caller admits a stable wrapper+18 stream pointer, unchanged source-compatible
/// vtables, disjoint writable storage and no concurrent mutation. The original
/// initializer sets byte +0C only when zero and returns AL=1; no real stream
/// initialization, file open, lifetime or live identity is inferred.
pub fn project(initial:u8,admitted:bool)->Result<Outcome,Error>{if !admitted{return Err(Error::Unadmitted)}Ok(Outcome{returned:true,stream_byte_0c:if initial==0{1}else{initial}})}
#[cfg(test)]mod tests{use super::*;#[test]fn zero_and_nonzero(){assert_eq!(project(0,true).unwrap().stream_byte_0c,1);assert_eq!(project(0xff,true).unwrap().stream_byte_0c,0xff);assert_eq!(project(0,false),Err(Error::Unadmitted));}}
