//! Bounded post-crossing 53FE60..54006D geometry and edge-cache commit.
use crate::native_sse::{self,Policy};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{InvalidBuffers,InvalidIndex,NonFinite,UnsupportedArchitecture}
impl From<native_sse::SseError> for Error {fn from(_:native_sse::SseError)->Self{Self::UnsupportedArchitecture}}
/// `records` uses native six-word corner rows, `positions` and `counts` are
/// preallocated, nonaliasing geometry arrays, and `len` is their used length.
/// Caller supplies the chosen corner, first sampled corner, computed finite
/// crossing, ordered edge and valid lookup result; lookup/status (including full-table abort), crossing,
/// allocations, queue topology, failures and runtime identity are not modeled.
/// Invalid inputs are rejected before writes, unlike native failure behavior.
pub fn commit(records:&mut [[u32;6]],selected:usize,first_corner:u32,positions:&mut [[f32;3]],counts:&mut [[u32;2]],len:&mut usize,edge_slots:&mut [[u32;3]],lookup_result:u32,edge:[u32;2],point:[f32;3],policy:Policy)->Result<u32,Error>{
 if lookup_result>=0x8000||selected>i32::MAX as usize||selected>=records.len()||edge_slots.len()!=0x8000||*len>positions.len()||*len>counts.len(){return Err(Error::InvalidBuffers)}
 if point.iter().any(|x|!x.is_finite()){return Err(Error::NonFinite)}
 let prior=records[selected][1];let new=prior==u32::MAX;
 let index=if new {if *len>=positions.len()||*len>=counts.len()||*len>i32::MAX as usize{return Err(Error::InvalidBuffers)}*len}else{let i=prior as usize;if i>=*len{return Err(Error::InvalidIndex)}i};
 let mut result=point;
 if !new {result=native_sse::scoped(policy,||{let mut v=[0.;3];for i in 0..3{v[i]=positions[index][i]+point[i];}Ok::<_,Error>(v)})?;if result.iter().any(|v|!v.is_finite()){return Err(Error::NonFinite)}}
 if new {records[selected][1]=index as u32;records[selected][2]=first_corner;positions[index]=result;counts[index]=[selected as u32,1];*len+=1;} else {positions[index]=result;counts[index][1]=counts[index][1].wrapping_add(1);}
 edge_slots[(lookup_result&0x7fff) as usize]=[edge[0],edge[1],index as u32];Ok(index as u32)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn append_then_accumulate(){let mut r=[[0,u32::MAX,77,0,0,0];1];let(mut p,mut c,mut len)=([[0.;3];2],[[0;2];2],0);let mut e=vec![[0;3];0x8000];assert_eq!(commit(&mut r,0,9,&mut p,&mut c,&mut len,&mut e,4,[1,2],[1.,2.,3.],Policy::Mxcsr1f80),Ok(0));assert_eq!((r[0][1],r[0][2],p[0],c[0],e[4],len),(0,9,[1.,2.,3.],[0,1],[1,2,0],1));assert_eq!(commit(&mut r,0,8,&mut p,&mut c,&mut len,&mut e,5,[3,4],[0.5,-1.,1.],Policy::Mxcsr1f80),Ok(0));assert_eq!((r[0][2],p[0],c[0],e[5],len),(9,[1.5,1.,4.],[0,2],[3,4,0],1));}
 #[test]fn bounded_capacity_before_writes(){let mut r=[[0,u32::MAX,77,0,0,0]];let(mut p,mut c,mut len)=([],[],0);let mut e=vec![[0;3];0x8000];assert_eq!(commit(&mut r,0,3,&mut p,&mut c,&mut len,&mut e,0,[1,2],[1.;3],Policy::Mxcsr1f80),Err(Error::InvalidBuffers));assert_eq!(r[0][1],u32::MAX);}
}
