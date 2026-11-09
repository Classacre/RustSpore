//! Bounded first-candidate -> early B7 producer -> E59B40 composition.
//! Synthetic actor tokens and supplied actor words are distinct identities;
//! this does not bind source actors or establish a final two-pass selector.
use crate::{cell_feature_first_candidate_v1::{PendingCandidate,LocalGeometry},cell_feeding_producer_prefix_v1::{self as producer,PrefixInput,PrefixResult},cell_feeding_contact_arbitration_e59b40_v1::{self as arbitration,ContactOutput,ArbitrationWrites}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Binding{pub actor_tokens:[u32;2],pub actor_words:[u32;2],pub current_player_word:u32,pub stable_disjoint:bool}
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unstable,ActorMismatch,IdenticalActors,OpaqueSmallLocals,Producer(producer::PrefixError)}
#[derive(Debug,PartialEq,Eq)]pub enum Step{FeatureNineExit,DefinitionASevenExit,UnsupportedProducer,Submitted{selection:ContactOutput,writes:ArbitrationWrites}}
/// Actors, point, normal, output and current-player data must be stable,
/// disjoint synthetic snapshots. Only the B7 return path is submitted; B6 and
/// all later producer/tail paths remain unsupported. A submitted selection is
/// intermediate: other features and the reverse pass can still change it.
pub fn apply(candidate:&PendingCandidate,binding:Binding,mut selection:ContactOutput)->Result<Step,Error>{
 if !binding.stable_disjoint{return Err(Error::Unstable)}if binding.actor_tokens!=candidate.actor_tokens{return Err(Error::ActorMismatch)}if binding.actor_tokens[0]==binding.actor_tokens[1]||binding.actor_words[0]==binding.actor_words[1]{return Err(Error::IdenticalActors)}
 if candidate.classes.contains(&9){return Ok(Step::FeatureNineExit)}
 if candidate.definition_b4[1]!=7 {return if candidate.definition_b4[0]==7{Ok(Step::DefinitionASevenExit)}else{Ok(Step::UnsupportedProducer)}}
 let (point,normal)=match candidate.geometry{LocalGeometry::OpaqueSmall=>return Err(Error::OpaqueSmallLocals),LocalGeometry::Nondegenerate{point,normal}=>(point,normal)};
 let input=PrefixInput{feature_a:candidate.classes[0] as i32,feature_b:candidate.classes[1] as i32,definition_a:candidate.definition_b4[0] as i32,definition_b:candidate.definition_b4[1] as i32,actor_ids:binding.actor_words,current_player_id:binding.current_player_word,feature_indices:candidate.indices,depth_bits:candidate.depth,point_words:point,normal_words:normal};
 match producer::evaluate_prefix(input).map_err(Error::Producer)?{PrefixResult::PendingSubmission(c)=>{let writes=arbitration::submit_contact_candidate(&mut selection,c,binding.current_player_word);Ok(Step::Submitted{selection,writes})},_=>unreachable!("checked B7 prefix")}
}
#[cfg(test)]mod tests{use super::*;fn candidate()->PendingCandidate{PendingCandidate{actor_tokens:[11,22],definition_b4:[2,7],second_definition_b0:4,relation:0,classes:[2,3],indices:[0,1],depth:0.5f32.to_bits(),geometry:LocalGeometry::Nondegenerate{point:[1,2,3],normal:[4,5,6]}}}fn selection()->ContactOutput{ContactOutput{actor_ids:[0;2],selector:0,feature_indices:[0;2],depth_bits:0,point_words:[0;3],normal_words:[0;3]}}fn binding()->Binding{Binding{actor_tokens:[11,22],actor_words:[101,202],current_player_word:101,stable_disjoint:true}}
 #[test]fn b7_produces_intermediate_arbitrated_selection(){let Step::Submitted{selection,writes}=apply(&candidate(),binding(),selection()).unwrap()else{panic!()};assert_eq!(selection.selector,19);assert_eq!(selection.actor_ids,[101,202]);assert!(writes.priority_or_depth);}
 #[test]fn guards_opaque_and_mismatched_identity(){let mut c=candidate();c.geometry=LocalGeometry::OpaqueSmall;assert_eq!(apply(&c,binding(),selection()),Err(Error::OpaqueSmallLocals));c.classes[0]=9;assert_eq!(apply(&c,binding(),selection()),Ok(Step::FeatureNineExit));let mut b=binding();b.actor_tokens[0]=55;assert_eq!(apply(&candidate(),b,selection()),Err(Error::ActorMismatch));}
}
