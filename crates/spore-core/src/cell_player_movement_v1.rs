//! Conditional E7C94D movement with supplied checked actor identity.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Actor{pub handle:u32,pub kind:u32,pub position:[u32;3],pub velocity:[u32;3],pub flags:u16,pub revision:u16}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Route{NoActor,PredicateFalse,OtherKind,Updated}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Outcome{pub route:Route,pub actor:Actor,pub first_match:bool,pub fresh_match:bool}
#[derive(Debug,PartialEq)]pub enum Error{Unadmitted,UnreadableSlot,UnsupportedFloat,Sse(SseError)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
/// The first checked lookup is a fixture identity gate, not the source iterator.
/// The caller supplies the iterator actor, normally returned predicate, fresh
/// root type/scalar and stable disjoint storage; no callback or actor mutation
/// occurs before the second lookup. Pool allocation, scheduling, live identity,
/// the alternate kind and downstream camera logic remain outside this boundary.
pub fn project(handle:u32,high_water:i32,actor:Actor,predicate:bool,root_kind:u32,scalar:u32,policy:Policy,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let index=(handle&0xffff) as usize;
 let matched=if handle==0 || index as i32>high_water {false} else {if index!=0{return Err(Error::UnreadableSlot)}actor.handle==handle};
 let mut out=Outcome{route:Route::NoActor,actor,first_match:matched,fresh_match:matched};
 if !matched{return Ok(out)}
 if !predicate{out.route=Route::PredicateFalse;return Ok(out)}
 if actor.kind!=root_kind{out.route=Route::OtherKind;return Ok(out)}
 fn ok(v:f32)->bool{v.is_finite()&&(v==0.0||v.is_normal())}
 let p=actor.position.map(f32::from_bits);let v=actor.velocity.map(f32::from_bits);let dt=f32::from_bits(scalar);
 if !p.iter().chain(v.iter()).all(|x|ok(*x))||!ok(dt){return Err(Error::UnsupportedFloat)}
 native_sse::scoped(policy,||{use std::hint::black_box as b;let x=b(b(v[0])*b(dt));let y=b(b(v[1])*b(dt));let z=b(b(v[2])*b(dt));let xyz=[b(x)+b(p[0]),b(p[1])+b(y),b(p[2])+b(z)];if ![x,y,z].iter().chain(xyz.iter()).all(|x|ok(*x)){return Err(Error::UnsupportedFloat)}out.actor.position=xyz.map(f32::to_bits);out.actor.flags|=4;out.actor.revision=out.actor.revision.wrapping_add(1);out.route=Route::Updated;Ok(out)})
}
#[cfg(test)]mod tests{use super::*;#[test]fn controlled(){let a=Actor{handle:0x10000,kind:11,position:[100f32.to_bits(),100f32.to_bits(),0],velocity:[1f32.to_bits(),2f32.to_bits(),0],flags:0x1230,revision:0xffff};let out=project(a.handle,1,a,true,11,0.5f32.to_bits(),Policy::Mxcsr1f80,true).unwrap();assert_eq!(out.actor.position,[100.5f32.to_bits(),101f32.to_bits(),0]);assert_eq!((out.actor.flags,out.actor.revision),(0x1234,0));assert_eq!(project(0x20000,1,a,true,11,0,Policy::Mxcsr1f80,true).unwrap().route,Route::NoActor);assert_eq!(project(a.handle,1,a,true,11,0,Policy::Mxcsr1f80,false),Err(Error::Unadmitted));}}
