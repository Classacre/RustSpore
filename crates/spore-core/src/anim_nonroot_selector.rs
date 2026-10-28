//! Capability/spatial primary mode0 selection on finalized generated body blocks.
//! Unexported pending original9B2150 gate. Z/selector-class/other hierarchy
//! policies reject rather than inventing missing body-definition fields.
use crate::{anim_body_capabilities::CapabilitySlots,anim_body_finalizer_fields::FinalizedBodyFields,native_sse::{self,Policy,SseError}};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Error{Count(usize),Ordinal(usize),Parent(usize),RootSpine,Geometry(usize),Selector([u32;4]),Capacity(usize),Sse(SseError)}
impl From<SseError>for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[derive(Debug,Clone)]pub struct FinalizedSelectorBlock{pub ordinal:u32,pub parent:Option<usize>,pub source_palette_id:u32,pub presence_mask:u32,pub nonzero_mask:u32,pub capabilities:CapabilitySlots,pub center:[f32;3],pub half_extents:[f32;3]}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct Selection{pub runtime_ordinal:u32,pub source_palette_id:u32}
fn capability(block:&FinalizedSelectorBlock,id:u32)->u8{for(i,x)in block.capabilities.ids.iter().enumerate(){if *x==0{break}if *x==id{return block.capabilities.numbers[i]}}0}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn binary(a:f32,b:f32,kind:u8)->f32{
 let result:u32;let(a,b)=(a.to_bits(),b.to_bits());unsafe{match kind{
 0=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","addss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 1=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","subss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 _=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","mulss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 }}f32::from_bits(result)
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn binary(_:f32,_:f32,_:u8)->f32{unreachable!()}
fn add(a:f32,b:f32)->f32{binary(a,b,0)}fn sub(a:f32,b:f32)->f32{binary(a,b,1)}fn mul(a:f32,b:f32)->f32{binary(a,b,2)}
fn floor100(x:f32)->i32{mul(x,100.).floor()as i32}
fn mirror(flags:u32)->u32{let x=match flags&0x2000c{4=>8,8=>4,0x20000=>0x20004,0x20004=>0x20000,v=>v};let rank=match flags&0xe000{0x2000=>0x4000,0x4000=>0x2000,v=>v};(flags&!0x2e00c)|x|rank}
fn masks(cap:&CapabilitySlots)->(u32,u32){let(mut presence,mut nonzero)=(0,0);for(i,id)in cap.ids.iter().enumerate(){if *id==0{break}let bit=match *id{0x746f6f72=>1,0x626d696c=>2,0x746f6f66=>4,0x6e697073=>8,0x74667370=>16,0x74756f6d=>32,0x70737267=>64,0x7274736e=>128,_=>0};presence|=bit;if cap.numbers[i]!=0{nonzero|=bit}}(presence,nonzero)}
/// Full mode0 policies observed in1641 nonroot channels; final runtime ordinals,
/// never original source-row or palette ordinals. Real nonzero-child trees allowed.
pub fn select(blocks:&[FinalizedSelectorBlock],selector:[u32;4],mirrored:bool,capacity:usize)->Result<Vec<Selection>,Error>{
 if blocks.is_empty()||blocks.len()>255{return Err(Error::Count(blocks.len()))}if capacity>255{return Err(Error::Capacity(capacity))}
 let flags=if mirrored{mirror(selector[0])}else{selector[0]};let x=flags&0x2000c;let y=flags&0x40030;let rank=flags&0xe000;let hierarchy=flags&0x801c00;
 if selector[2]!=0||selector[3]!=0||flags&!0x6f03c!=0||!matches!(x,0|4|8|12|0x20000|0x20004|0x20008)||!matches!(y,0|0x10|0x20|0x30|0x40000|0x40010|0x40020)||!matches!(rank,0|0x2000|0x4000|0x6000|0x8000)||!matches!(hierarchy,0|0x1000)||hierarchy!=0&&flags!=0x1000{return Err(Error::Selector(selector))}
 if blocks[0].parent.is_some()||blocks[0].nonzero_mask&8==0{return Err(Error::RootSpine)}
 for(i,b)in blocks.iter().enumerate(){
  if b.ordinal!=i as u32{return Err(Error::Ordinal(i))}if masks(&b.capabilities)!=(b.presence_mask,b.nonzero_mask){return Err(Error::Parent(i))}
  if b.center.iter().any(|x|x.to_bits()&0x7fffffff>0x49800000)||b.half_extents.iter().any(|x|x.to_bits()&0x80000000!=0||x.to_bits()&0x7fffffff==0||x.to_bits()&0x7fffffff>0x49800000){return Err(Error::Geometry(i))}
  if i!=0&&b.parent.is_none(){return Err(Error::Parent(i))}
  let mut at=i;let mut seen=0;while let Some(parent)=blocks[at].parent{if parent>=blocks.len()||parent==at||seen>=blocks.len(){return Err(Error::Parent(i))}at=parent;seen+=1;}if at!=0{return Err(Error::Parent(i))}
 }
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let mut min=[f32::MAX;2];let mut max=[-f32::MAX;2];
  for block in blocks{if capability(block,selector[1])!=0{for axis in 0..2{if block.center[axis]<min[axis]{min[axis]=block.center[axis]}if max[axis]<block.center[axis]{max[axis]=block.center[axis]}}}}
  let mut result:Vec<usize>=Vec::new();
  for(i,b)in blocks.iter().enumerate(){if capability(b,selector[1])==0{continue}
   let tolerance=mul(add(add(b.half_extents[2],b.half_extents[1]),b.half_extents[0]),f32::from_bits(0x3e19999a));let px=b.center[0];let py=b.center[1];
   let reject_x=match x{0=>false,4=>px< -tolerance,8=>tolerance<px,12=>px.abs()>tolerance,
    0x20000=>floor100(px)<floor100(sub(max[0],mul(sub(max[0],min[0]),f32::from_bits(0x3f066666)))),
    0x20004=>floor100(add(mul(sub(max[0],min[0]),f32::from_bits(0x3f066666)),min[0]))<floor100(px),
    _=>floor100(sub(px,add(mul(sub(max[0],min[0]),0.5),min[0]))).abs()>10};
   let reject_y=match y{0=>false,0x10=>py>0.,0x20=>py<0.,0x30=>py.abs()>tolerance,
    0x40000=>floor100(py)>floor100(add(mul(sub(max[1],min[1]),f32::from_bits(0x3f066666)),min[1])),
    0x40010=>floor100(py)<floor100(sub(max[1],mul(sub(max[1],min[1]),f32::from_bits(0x3f066666)))),
    _=>floor100(sub(py,add(mul(sub(max[1],min[1]),0.5),min[1]))).abs()>10};
   if reject_x||reject_y{continue}
   if result.is_empty(){result.push(i)}else if rank==0{result.push(i)}else{
    let first=blocks[result[0]].center;let replace=match rank{0x2000=>first[0]<px,0x4000=>px<first[0],0x6000=>py<first[1],_=>first[1]<py};if replace{let last=result.len()-1;result[last]=i}
   }
   if hierarchy==0x1000{if let Some(mut parent)=b.parent{
    result.pop();while blocks[parent].nonzero_mask&8==0{parent=blocks[parent].parent.ok_or(Error::Parent(i))?}
    if !result.contains(&parent){result.push(parent)}
   }}
  }
  result.truncate(capacity);Ok(result.into_iter().map(|i|Selection{runtime_ordinal:i as u32,source_palette_id:blocks[i].source_palette_id}).collect())
 })
}
/// Compose directly with the already conformed source finalizer boundary.
pub fn select_finalized(body:&FinalizedBodyFields,selector:[u32;4],mirrored:bool,capacity:usize)->Result<Vec<Selection>,Error>{
 let blocks=body.blocks.iter().map(|b|FinalizedSelectorBlock{ordinal:b.ordinal,parent:b.parent,source_palette_id:b.source_palette_id,presence_mask:b.presence_mask,nonzero_mask:b.nonzero_mask,capabilities:b.capabilities,center:b.center,half_extents:b.half_extents}).collect::<Vec<_>>();select(&blocks,selector,mirrored,capacity)
}
