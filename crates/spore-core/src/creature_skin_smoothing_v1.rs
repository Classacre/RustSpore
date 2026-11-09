//! Unverified 64-slot persistent-scratch model of 52B270 smoothing.
//! Actual iteration count, correction, topology identity and palette are admissions.
use crate::{creature_skin_paired_topology_v1::Topology,creature_skin_sort64_v1::{self,Entry},native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Length, NonFinite, InvalidTopology, TooManyEntries, UnsupportedArchitecture }
impl From<SseError> for Error { fn from(_:SseError)->Self{Self::UnsupportedArchitecture} }
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn add(mut a:f32,b:f32)->f32{unsafe{std::arch::asm!("addss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nostack,preserves_flags));}a}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn sub(mut a:f32,b:f32)->f32{unsafe{std::arch::asm!("subss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nostack,preserves_flags));}a}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn mul(mut a:f32,b:f32)->f32{unsafe{std::arch::asm!("mulss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nostack,preserves_flags));}a}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn div(mut a:f32,b:f32)->f32{unsafe{std::arch::asm!("divss {a}, {b}",a=inout(xmm_reg)a,b=in(xmm_reg)b,options(nostack,preserves_flags));}a}
fn read(r:&[u8;20],i:usize)->f32{f32::from_le_bytes(r[4+i*4..8+i*4].try_into().unwrap())}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn merge(entries:&mut [Entry;64],count:&mut usize,r:&[u8;20],factor:f32)->Result<(),Error>{for j in 0..4{let w=read(r,j);if !w.is_finite(){return Err(Error::NonFinite)}if w<=0.0{break}let p=mul(w,factor);if !p.is_finite(){return Err(Error::NonFinite)}let found=(0..*count).find(|&i|entries[i].key as u8==r[j]);if let Some(i)=found{entries[i].weight=add(p,entries[i].weight);if !entries[i].weight.is_finite(){return Err(Error::NonFinite)}}else{if *count==64{return Err(Error::TooManyEntries)}entries[*count].key=(entries[*count].key&0xffffff00)|u32::from(r[j]);entries[*count].weight=p;*count+=1}}Ok(())}
/// Executes at most eight explicitly requested passes; destination bytes are
/// supplied because source leaves unused index bytes unchanged. Finite inputs,
/// masked SSE and x87, successful storage, and no intermediate observation are admitted.
/// Arbitrary finite parameters are explicitly supplied; their runtime domain is not established.
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
pub fn smooth(records:&[[u8;20]], destination:&[[u8;20]], normals:&[[f32;3]], indices:&[u32], topology:&Topology, iterations:u8, parameter:f32, policy:Policy)->Result<Vec<[u8;20]>,Error>{
 let n=records.len();if n==0||n!=destination.len()||n!=normals.len()||n!=topology.starts.len()||indices.len()!=topology.links.len()||indices.len()%3!=0||iterations>8{return Err(Error::Length)}
 if !parameter.is_finite()||normals.iter().flatten().any(|x|!x.is_finite()){return Err(Error::NonFinite)}
 native_sse::scoped(policy,||{let mut src=records.to_vec();let mut dst=destination.to_vec();let mut entries=[Entry{key:0,weight:0.};64];for _ in 0..iterations{for vertex in 0..n{let start=topology.starts[vertex] as usize;let mut corner=start;let mut visits=0usize;let mut count=0usize;loop{if corner>=indices.len()||visits>=indices.len()||indices[corner] as usize!=vertex{return Err(Error::InvalidTopology)}visits+=1;let phase=corner%3;let neighbor_corner=match phase{0=>corner+1,1=>corner+1,_=>corner-2};let link_corner=match phase{0=>corner+2,_=>corner-1};if neighbor_corner>=indices.len()||link_corner>=indices.len(){return Err(Error::InvalidTopology)}let neighbor=indices[neighbor_corner] as usize;if neighbor>=n{return Err(Error::InvalidTopology)}let a=normals[neighbor];let b=normals[vertex];let dot=add(add(mul(a[0],b[0]),mul(a[1],b[1])),mul(a[2],b[2]));if !dot.is_finite(){return Err(Error::NonFinite)}if dot>f32::from_bits(0x3f5db22d){merge(&mut entries,&mut count,&src[neighbor],mul(dot,dot))?}let next=topology.links[link_corner] as usize;if next==link_corner||next==start{break}corner=next;}
 let mut sum=0f32;for e in &entries[..count]{sum=add(sum,e.weight)}if !sum.is_finite(){return Err(Error::NonFinite)}if sum>0.0{let f=div(parameter,sum);for e in &mut entries[..count]{e.weight=mul(f,e.weight);if !e.weight.is_finite(){return Err(Error::NonFinite)}}}merge(&mut entries,&mut count,&src[vertex],sub(1.0,parameter))?;
 // Source sorts all 64 persistent slots, not only the active prefix.
 creature_skin_sort64_v1::sort(&mut entries).map_err(|_|Error::TooManyEntries)?;
 let mut total=0f32;for i in 0..4{let w=if i<count{dst[vertex][i]=entries[i].key as u8;entries[i].weight}else{0.0};dst[vertex][4+i*4..8+i*4].copy_from_slice(&w.to_le_bytes());total=add(total,w)}if !total.is_finite(){return Err(Error::NonFinite)}if total>0.0{for i in 0..4{let w=div(read(&dst[vertex],i),total);dst[vertex][4+i*4..8+i*4].copy_from_slice(&w.to_le_bytes())}}
 }std::mem::swap(&mut src,&mut dst)}Ok(src)})
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
pub fn smooth(_records:&[[u8;20]], _destination:&[[u8;20]], _normals:&[[f32;3]], _indices:&[u32], _topology:&Topology, _iterations:u8, _parameter:f32, _policy:Policy)->Result<Vec<[u8;20]>,Error>{ Err(Error::UnsupportedArchitecture) }
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))] mod tests{use super::*;use crate::creature_skin_paired_topology_v1::build;
 #[test]fn zero_iterations_preserve_source_and_one_pass_is_finite(){let idx=[0,2,1,0,1,3,1,2,3,2,0,3];let top=build(&idx,4).unwrap();let mut rec=[[0u8;20];4];for (i,r) in rec.iter_mut().enumerate(){r[0]=i as u8;r[4..8].copy_from_slice(&1f32.to_le_bytes())}let dst=[[0x7fu8;20];4];assert_eq!(smooth(&rec,&dst,&[[0.,0.,1.];4],&idx,&top,0,0.,Policy::Mxcsr1f80).unwrap(),rec);let got=smooth(&rec,&dst,&[[0.,0.,1.];4],&idx,&top,1,0.,Policy::Mxcsr1f80).unwrap();for r in got{for i in 0..4{assert!(read(&r,i).is_finite())}}}
 #[test]fn rejects_bad_links_and_nonfinite(){let idx=[0,2,1,0,1,3,1,2,3,2,0,3];let mut top=build(&idx,4).unwrap();top.starts[0]=99;assert_eq!(smooth(&[[0;20];4],&[[0;20];4],&[[0.,0.,1.];4],&idx,&top,1,0.5,Policy::Mxcsr1f80),Err(Error::InvalidTopology));assert_eq!(smooth(&[[0;20];4],&[[0;20];4],&[[0.,0.,1.];4],&idx,&top,1,f32::NAN,Policy::Mxcsr1f80),Err(Error::NonFinite));}}
