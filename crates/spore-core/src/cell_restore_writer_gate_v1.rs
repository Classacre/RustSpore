//! Bounded 6A0060 writer gate with a supplied key and scripted callbacks.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingAfter,MissingEnd}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub returned:bool,pub words:Vec<u32>,pub getters:u32,pub after:u32,pub end:u32}
/// Caller supplies a stable nonzero object identity and post-virtual key when
/// present; synthetic stream writes and downstream callbacks return normally,
/// without reentrancy or mutation affecting this bounded path. Write return
/// values are ignored by source. No byte encoding, live identity or ownership
/// is inferred; key 0xABCDEF00 requires the separately admitted source table.
pub fn project(enabled:bool,object:Option<(u32,u32)>,after:Option<bool>,end:Option<bool>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !enabled{return Ok(Outcome{returned:false,words:vec![],getters:0,after:0,end:0})}
 let (words,after_count)=if let Some((ptr,key))=object{if ptr==0{return Err(Error::Unadmitted)} let words=vec![ptr,key];if !after.ok_or(Error::MissingAfter)?{return Ok(Outcome{returned:false,words,getters:2,after:1,end:0})}(words,1)}else{(vec![0],0)};
 Ok(Outcome{returned:end.ok_or(Error::MissingEnd)?,words,getters:2,after:after_count,end:1})
}
#[cfg(test)]mod tests{use super::*;#[test]fn null_and_failed_after(){assert_eq!(project(true,None,None,Some(true),true).unwrap().words,vec![0]);let x=project(true,Some((7,0xabcdef00)),Some(false),None,true).unwrap();assert!(!x.returned);assert_eq!(x.words,vec![7,0xabcdef00]);assert_eq!(x.end,0);}}
/// Original mode-zero one-word byte order, conditional on a successful stream
/// status and a normally returning callback. This predicts the callback payload
/// even if that callback reports failure; it does not establish persistence.
pub fn mode_zero_bytes(word:u32)->[u8;4]{word.to_be_bytes()}
