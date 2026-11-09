//! Bounded E75C50 callback and E75BB0 fresh mode gate.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Outcome { pub pending_player_call: bool, pub callback_return_after_pending_call_returns: bool }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Unadmitted;
/// `mode_after_prefix` is a fresh root+5158 snapshot after the admitted prior
/// UI/service prefix returns. The resulting player call is pending; no actual
/// prefix, callback delivery, root identity or E72CD0 effect is inferred.
/// The callback return is conditional on the pending player call returning.
pub fn receiver(mode_after_prefix:u32,admitted:bool)->Result<Outcome,Unadmitted>{
 if !admitted{return Err(Unadmitted)}
 Ok(Outcome{pending_player_call:mode_after_prefix==5,callback_return_after_pending_call_returns:false})
}
#[cfg(test)]mod tests{use super::*;#[test]fn fresh_mode_only(){assert!(receiver(5,true).unwrap().pending_player_call);assert!(!receiver(4,true).unwrap().pending_player_call);assert!(!receiver(6,true).unwrap().callback_return_after_pending_call_returns);assert_eq!(receiver(5,false),Err(Unadmitted));}}
