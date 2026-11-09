//! Isolated sorted endpoint crossing helpers with the source pair-effect flag
//! explicitly zero. This does not model pair changes or live collection state.
use crate::cell_fresh_sweep_registration_v1::{Endpoint,Slot};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Side{Minimum,Maximum}
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Direction{Left,Right}
#[derive(Debug,PartialEq,Eq)] pub enum Error{InvalidAxis,TooManyEndpoints,InvalidSlot,InvalidEndpoint,InvalidRank,DuplicateLink,InvalidFloat,Tie,OtherEndpointsUnsorted,WrongDirection}
#[derive(Debug,PartialEq,Eq)] pub struct Reordered{pub slots:Vec<Slot>,pub endpoints:Vec<Endpoint>}
/// Inputs are disjoint owned snapshots, all endpoints finite normal/zero and
/// distinct; no tombstones, pair effects or pending links are admitted. All
/// ranks must name their exact positions; all records except the chosen one
/// must already be sorted. The chosen endpoint's value is already updated.
/// These extra preflights are not source checks; failures return no mutations.
pub fn reorder(slots:&[Slot],axis:usize,endpoints:&[Endpoint],selected:usize,side:Side,direction:Direction)->Result<Reordered,Error>{
 if axis>1{return Err(Error::InvalidAxis)}if endpoints.len()>4096{return Err(Error::TooManyEndpoints)}if selected>=slots.len()||selected>=0x8000{return Err(Error::InvalidSlot)}
 let link=axis+if side==Side::Minimum{0}else{2};let rank=usize::from(slots[selected].endpoint_links[link]);if rank>=endpoints.len(){return Err(Error::InvalidRank)}
 let mut seen=std::collections::HashSet::new();let mut values=std::collections::HashSet::new();let mut previous=None;
 for (i,e) in endpoints.iter().enumerate(){let x=f32::from_bits(e.value_bits);if !(x.is_normal()||x==0.){return Err(Error::InvalidFloat)}let key=if x==0.{0}else{e.value_bits};if !values.insert(key){return Err(Error::Tie)}
  if e.opening>1||e.slot>=0x8000||usize::from(e.slot)>=slots.len(){return Err(Error::InvalidEndpoint)}let l=axis+if e.opening==0{0}else{2};if !seen.insert((e.slot,l)){return Err(Error::DuplicateLink)}if usize::from(slots[e.slot as usize].endpoint_links[l])!=i{return Err(Error::InvalidRank)}
  if i!=rank {if previous.is_some_and(|p:f32|p>=x){return Err(Error::OtherEndpointsUnsorted)}previous=Some(x)}
 }
 let chosen=endpoints[rank];if usize::from(chosen.slot)!=selected||chosen.opening!=if side==Side::Minimum{0}else{1}{return Err(Error::InvalidRank)}
 let mut result=Reordered{slots:slots.to_vec(),endpoints:endpoints.to_vec()};let mut pos=rank;
 loop{let next=match direction{Direction::Left if pos>0=>pos-1,Direction::Right if pos+1<endpoints.len()=>pos+1,_=>break};let x=f32::from_bits(result.endpoints[pos].value_bits);let y=f32::from_bits(result.endpoints[next].value_bits);if match direction{Direction::Left=>y<=x,Direction::Right=>x<=y}{break}
  let crossed=result.endpoints[next];let crossed_link=axis+if crossed.opening==0{0}else{2};result.endpoints.swap(pos,next);result.slots[selected].endpoint_links[link]=next as u16;result.slots[crossed.slot as usize].endpoint_links[crossed_link]=pos as u16;pos=next;
 }
 if result.endpoints.windows(2).any(|p|f32::from_bits(p[0].value_bits)>=f32::from_bits(p[1].value_bits)){return Err(Error::WrongDirection)}Ok(result)
}
#[cfg(test)] mod tests{
 use super::*;
 fn e(flag:u16,slot:u16,v:f32)->Endpoint{Endpoint{opening:flag,slot,value_bits:v.to_bits()}}
 #[test] fn moves_left_and_rewrites_both_ranks(){let slots=[Slot{actor:1,tag:0,endpoint_links:[2,7,3,8]},Slot{actor:2,tag:0,endpoint_links:[0,9,1,10]}];let input=[e(0,1,1.),e(1,1,2.),e(0,0,0.),e(1,0,4.)];let out=reorder(&slots,0,&input,0,Side::Minimum,Direction::Left).unwrap();assert_eq!(out.slots[0].endpoint_links,[0,7,3,8]);assert_eq!(out.slots[1].endpoint_links,[1,9,2,10]);assert_eq!(input[2].value_bits,0f32.to_bits());}
 #[test] fn moves_right_and_rejects_wrong_direction(){let slots=[Slot{actor:1,tag:0,endpoint_links:[0,7,1,8]},Slot{actor:2,tag:0,endpoint_links:[2,9,3,10]}];let input=[e(0,0,0.),e(1,0,5.),e(0,1,2.),e(1,1,4.)];let out=reorder(&slots,0,&input,0,Side::Maximum,Direction::Right).unwrap();assert_eq!(out.slots[0].endpoint_links,[0,7,3,8]);assert_eq!(out.slots[1].endpoint_links,[1,9,2,10]);assert_eq!(reorder(&slots,0,&input,0,Side::Maximum,Direction::Left),Err(Error::WrongDirection));}
}
