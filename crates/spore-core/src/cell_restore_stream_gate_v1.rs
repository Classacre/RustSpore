//! Bounded flag-zero 6A0110 backend stream/resolve gate.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Action{GetInner,GetStream,ReadFirst,ReadSecond,Resolve{first:u32,second:u32},End}
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,UnsupportedControl,MissingFirst,MissingSecond,MissingResolver,MissingEnd}
#[derive(Debug,PartialEq,Eq)]pub struct Resolver{pub succeeded:bool,pub slot_after:u32}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:bool,pub slot:u32,pub actions:Vec<Action>}
/// State/stream words, post-resolver slot and callback returns are fresh caller
/// snapshots. This covers flag=0 only; valid stable objects, callbacks returning
/// normally without reentrancy and no intervening mutation are admissions.
/// The source ignores 93A3F0 return values; the fixture supplies words after
/// successful full reads. Mode 0 byte-swaps each dword on that path. Short or
/// failed reads, raw byte order and real stream side effects are not modeled.
/// It does not identify a serialized key, resolver implementation or live owner.
pub fn project(enabled:bool,initial_slot:u32,first:Option<u32>,second:Option<u32>,resolver:Option<Resolver>,end:Option<bool>,cw:u16,mx:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}if !matches!(cw,0x027f|0x037f)||mx!=0x1f80{return Err(Error::UnsupportedControl)}
 if !enabled{return Ok(Outcome{returned:false,slot:initial_slot,actions:vec![]})}
 let mut actions=vec![Action::GetInner,Action::GetStream,Action::ReadFirst];let first=first.ok_or(Error::MissingFirst)?;
 let slot=if first==0{0}else{let second=second.ok_or(Error::MissingSecond)?;actions.push(Action::ReadSecond);actions.push(Action::Resolve{first,second});let r=resolver.ok_or(Error::MissingResolver)?;if !r.succeeded{return Ok(Outcome{returned:false,slot:r.slot_after,actions})}r.slot_after};
 actions.push(Action::End);Ok(Outcome{returned:end.ok_or(Error::MissingEnd)?,slot,actions})
}
#[cfg(test)]mod tests{use super::*;#[test]fn conditional_reads_and_fresh_result(){let x=project(true,9,Some(1),Some(0xabcdef00),Some(Resolver{succeeded:true,slot_after:7}),Some(true),0x27f,0x1f80,true).unwrap();assert_eq!(x.slot,7);assert!(x.returned);assert!(x.actions.contains(&Action::Resolve{first:1,second:0xabcdef00}));let z=project(true,9,Some(0),None,None,Some(false),0x37f,0x1f80,true).unwrap();assert_eq!(z.slot,0);assert!(!z.actions.contains(&Action::ReadSecond));assert!(project(false,9,None,None,None,None,0x27f,0x1f80,true).unwrap().actions.is_empty());}}
