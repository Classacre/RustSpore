//! Bounded E72CD0 pre-allocation gate; all observations supplied by caller.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct PlayerFlags { pub byte_112:u8, pub byte_113:u8 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Outcome { pub root_5169:u8, pub pending_effect_and_allocation:bool }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Unadmitted;
/// Caller admits source entry and successful lookup return with stable objects.
/// Player, mode, and initial root byte are fresh snapshots after that lookup;
/// pending effects and allocation are not performed or tied to a live record.
pub fn gate(player:Option<PlayerFlags>,mode_after_lookup:u32,root_5169_after_lookup:u8,admitted:bool)->Result<Outcome,Unadmitted>{
 if !admitted{return Err(Unadmitted)}
 let allowed=player.is_some_and(|p|p.byte_112!=1 && p.byte_113!=1);
 Ok(Outcome{root_5169:if allowed{1}else{root_5169_after_lookup},pending_effect_and_allocation:allowed && mode_after_lookup!=3})
}
#[cfg(test)]mod tests{use super::*;#[test]fn exact_byte_checks_and_fresh_mode(){let p=Some(PlayerFlags{byte_112:2,byte_113:2});assert!(gate(p,2,90,true).unwrap().pending_effect_and_allocation);assert!(!gate(p,3,90,true).unwrap().pending_effect_and_allocation);let x=gate(Some(PlayerFlags{byte_112:1,byte_113:0}),2,90,true).unwrap();assert_eq!(x.root_5169,90);assert!(!x.pending_effect_and_allocation);assert_eq!(gate(None,2,90,false),Err(Unadmitted));}}
