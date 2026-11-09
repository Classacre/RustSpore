//! Bounded E7D8B0 identity/progress prefix; downstream effects excluded.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, MissingSaveFlag, MissingAppliedInputs }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Outcome { ActorMismatch, SaveFlagZero, Applied { player_id:u32, old_progress:u32, new_progress:u32 } }
/// Caller admits valid actor/root/save lifetimes, stable same root/save across
/// source reloads and no intervening mutation. Actors and save are distinct.
/// Result stops before E4ED00 and does not execute the callback or live writes.
pub fn apply(current_player:u32,second_actor:u32,save_flag:Option<u8>,first_actor:Option<u32>,progress:Option<u32>,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if second_actor!=current_player{return Ok(Outcome::ActorMismatch)}
 if save_flag.ok_or(Error::MissingSaveFlag)?==0{return Ok(Outcome::SaveFlagZero)}
 let (Some(player_id),Some(old_progress))=(first_actor,progress) else{return Err(Error::MissingAppliedInputs)};
 Ok(Outcome::Applied{player_id,old_progress,new_progress:old_progress.wrapping_add(150)})
}
#[cfg(test)]mod tests{use super::*;#[test]fn guards_nonzero_and_wrap(){assert_eq!(apply(1,2,None,None,None,true),Ok(Outcome::ActorMismatch));assert_eq!(apply(1,1,Some(0),None,None,true),Ok(Outcome::SaveFlagZero));assert_eq!(apply(1,1,Some(2),Some(3),Some(0xfffffff0),true),Ok(Outcome::Applied{player_id:3,old_progress:0xfffffff0,new_progress:0x86}));assert_eq!(apply(1,1,Some(1),Some(3),None,true),Err(Error::MissingAppliedInputs));assert_eq!(apply(1,1,None,None,None,false),Err(Error::Unadmitted));}}
