//! Checked E6DB40 caller-shaped handoff; no live actor or manager identity.
use crate::{cell_actor_geometry_projection_v1::Output,cell_registration_prefix_v1 as bounds};
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub struct Input{pub slot:u32,pub manager_token:u32,pub geometry:Option<Output>,pub stable_disjoint:bool}
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub enum Error{UnstableOrAliased,MissingGeometry,Bounds(bounds::Error)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)] pub enum Handoff{Skipped,PendingUpdate{manager_token:u32,slot:u32,min:[u32;2],max:[u32;2]}}
/// The caller admits geometry freshness, disjointness and a stable synthetic
/// manager/slot association. Slot FFFFFFFF skips before any other admission;
/// non-sentinel slots are opaque, not proven native registration handles.
/// Only the finite SSE/MXCSR1F80 bounds slice is modeled; BBBD10 is pending.
pub fn evaluate(input:Input)->Result<Handoff,Error>{
 if input.slot==u32::MAX{return Ok(Handoff::Skipped)}
 if !input.stable_disjoint{return Err(Error::UnstableOrAliased)}
 let geometry=input.geometry.ok_or(Error::MissingGeometry)?;
 let result=bounds::evaluate(bounds::Input{original_manager:0,current_manager:0,argument10:0,definition_b4:0,actor_handle:0,geometry,post_e86960_stable_disjoint:true}).map_err(Error::Bounds)?;
 match result{bounds::ResultV1::Register{min,max,..}=>Ok(Handoff::PendingUpdate{manager_token:input.manager_token,slot:input.slot,min,max}),bounds::ResultV1::Skipped=>unreachable!("equal admitted managers")}
}
#[cfg(test)]mod tests{use super::*;fn geom()->Output{Output{center:[1.5f32.to_bits(),2.5f32.to_bits(),0],scalar:0.5f32.to_bits()}}
 #[test]fn sentinel_skips_before_geometry_or_stability(){assert_eq!(evaluate(Input{slot:u32::MAX,manager_token:3,geometry:None,stable_disjoint:false}),Ok(Handoff::Skipped));}
 #[test]fn handoff_preserves_explicit_token_and_bounds(){let input=Input{slot:9,manager_token:17,geometry:Some(geom()),stable_disjoint:true};assert_eq!(evaluate(input),Ok(Handoff::PendingUpdate{manager_token:17,slot:9,min:[1f32.to_bits(),2f32.to_bits()],max:[2f32.to_bits(),3f32.to_bits()]}));assert_eq!(evaluate(Input{stable_disjoint:false,..input}),Err(Error::UnstableOrAliased));}
}
