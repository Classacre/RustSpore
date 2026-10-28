//! Pending all1644-policy selector composition. Genuine native-publisher fields
//! must be joined by body ID; this module does not calculate replacement bounds.
use crate::{anim_nonroot_selector::{self,FinalizedSelectorBlock,Selection},native_sse::{self,Policy,SseError}};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Error{Base(anim_nonroot_selector::Error),Sse(SseError),Projection,Selector([u32;4]),Midpoint}
impl From<anim_nonroot_selector::Error>for Error{fn from(e:anim_nonroot_selector::Error)->Self{Self::Base(e)}}
impl From<SseError>for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
/// Values read from one accepted original publisher44 projection, never inferred
/// from the Rust bbox reference. The external fixture must verify its native run
/// pins and exact body/source join; this parser alone cannot prove provenance.
#[derive(Debug,Clone,Copy)]pub struct DefinitionZ{body_id:u32,min:u32,extent:u32}
impl DefinitionZ{
 pub fn from_native_publisher_projection(bytes:&[u8],body_id:u32)->Result<Self,Error>{
  if bytes.len()!=44{return Err(Error::Projection)}let w=|i|u32::from_le_bytes(bytes[i..i+4].try_into().unwrap());let cw=w(4);
  if w(0)!=body_id||cw&!0xf00!=0x7f||cw&0x300==0x100||bytes[42..44]!=[0;2]{return Err(Error::Projection)}
  let fields=std::array::from_fn::<_,6,_>(|i|w(8+i*4));
  if fields.iter().any(|x|x&0x7fffffff>0x4e800000)||fields[3..].iter().any(|x|x&0x80000000!=0){return Err(Error::Projection)}
  if u16::from_le_bytes([bytes[40],bytes[41]])&0x7fff==0x7fff{return Err(Error::Projection)}
  Ok(Self{body_id,min:fields[2],extent:fields[5]})
 }
 /// Production composition: consume the independent Rust publisher result after
 /// its original native conformance gate, including publish_finalized output.
 pub fn from_published_bounds(bounds:&crate::anim_selector_z_bounds::DefinitionBounds,body_id:u32)->Result<Self,Error>{
  let f=bounds.fields();if f.iter().any(|x|x&0x7fffffff>0x4e800000)||f[3..].iter().any(|x|x&0x80000000!=0){return Err(Error::Projection)}
  Ok(Self{body_id,min:f[2],extent:f[5]})
 }
 pub fn body_id(self)->u32{self.body_id}
}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn midpoint(min:u32,extent:u32)->u32{let result:u32;unsafe{std::arch::asm!("movd {x},{extent:e}","movd {y},{half:e}","mulss {x},{y}","movd {y},{min:e}","addss {x},{y}","movd {out:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,extent=in(reg)extent,half=in(reg)0x3f000000u32,min=in(reg)min,out=lateout(reg)result,options(nostack,preserves_flags));}result}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn midpoint(_:u32,_:u32)->u32{unreachable!()}
/// Delegates1641 already accepted policies unchanged. The three newly observed
/// Z policies are exactly4C/1040 (two distinct capabilities for1040). Z testing
/// applies to the candidate's center BEFORE hierarchy1000 replaces it by a spine
/// ancestor; capacity truncation happens after the complete ordered reduction.
pub fn select(blocks:&[FinalizedSelectorBlock],selector:[u32;4],mirrored:bool,capacity:usize,bounds:DefinitionZ)->Result<Vec<Selection>,Error>{
 if selector[0]&0x800c0==0{return Ok(anim_nonroot_selector::select(blocks,selector,mirrored,capacity)?)}
 if selector[2]!=0||selector[3]!=0||!matches!(selector[0],0x4c|0x1040){return Err(Error::Selector(selector))}
 if capacity>255{return Err(Error::Base(anim_nonroot_selector::Error::Capacity(capacity)))}
 let hierarchy=selector[0]==0x1040;let plain=[if hierarchy{0}else{0xc},selector[1],0,0];
 // Full255 capacity also invokes accepted tree/mask/geometry validation first.
 let candidates=anim_nonroot_selector::select(blocks,plain,mirrored,255)?;
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let mid=midpoint(bounds.min,bounds.extent);if mid&0x7f800000==0x7f800000{return Err(Error::Midpoint)}let mid=f32::from_bits(mid);let mut result:Vec<Selection>=Vec::new();
  for candidate in candidates{let i=candidate.runtime_ordinal as usize;if blocks[i].center[2]<mid{continue}
   let mut ordinal=i;
   if hierarchy{if let Some(mut parent)=blocks[i].parent{while blocks[parent].nonzero_mask&8==0{parent=blocks[parent].parent.ok_or(Error::Base(anim_nonroot_selector::Error::Parent(i)))?}ordinal=parent}}
   if !result.iter().any(|x|x.runtime_ordinal==ordinal as u32){result.push(Selection{runtime_ordinal:ordinal as u32,source_palette_id:blocks[ordinal].source_palette_id})}
  }
  result.truncate(capacity);Ok(result)
 })
}
