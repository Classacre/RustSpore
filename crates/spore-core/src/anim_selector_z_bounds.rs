//! Exact-order finite oriented-box extent producer/reference for9B6920 and
//! the bounded9C48DB..9C4992 publication slice. Unexported pending native gate.
use crate::{anim_body_finalizer_fields::FinalizedBodyFields,native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Error{Count(usize),Geometry(usize),X87(X87Error),Sse(SseError)}
impl From<X87Error>for Error{fn from(x:X87Error)->Self{Self::X87(x)}}impl From<SseError>for Error{fn from(x:SseError)->Self{Self::Sse(x)}}
#[derive(Debug,Clone,Copy)]pub struct BoundsBlock{pub center:[u32;3],pub orientation:[u32;4],pub half_extents:[u32;3]}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct DefinitionBounds{fields:[u32;6],retained_yz_sum:[u8;10]}
impl DefinitionBounds{pub fn fields(&self)->[u32;6]{self.fields}pub fn min_z(&self)->u32{self.fields[2]}pub fn extent_z(&self)->u32{self.fields[5]}pub fn projection(&self)->[u8;36]{let mut b=[0;36];for(i,x)in self.fields.iter().enumerate(){b[i*4..i*4+4].copy_from_slice(&x.to_le_bytes())}b[24..34].copy_from_slice(&self.retained_yz_sum);b}}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn op(a:u32,b:u32,k:u8)->u32{let result:u32;unsafe{match k{
0=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","addss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
1=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","subss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
_=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","mulss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),}}result}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]fn op(_:u32,_:u32,_:u8)->u32{unreachable!()}
fn add(a:u32,b:u32)->u32{op(a,b,0)}fn sub(a:u32,b:u32)->u32{op(a,b,1)}fn mul(a:u32,b:u32)->u32{op(a,b,2)}fn neg(a:u32)->u32{a^0x80000000}fn f(a:u32)->f32{f32::from_bits(a)}
/// No quaternion normalization is introduced. Absolute source orientation and
/// generated block extents are consumed exactly as supplied by the finalizer.
pub fn publish(blocks:&[BoundsBlock],context:Context)->Result<DefinitionBounds,Error>{
 if blocks.is_empty()||blocks.len()>255{return Err(Error::Count(blocks.len()))}
 for(i,b)in blocks.iter().enumerate(){if b.center.iter().any(|x|x&0x7fffffff>0x49800000)||b.orientation.iter().any(|x|x&0x7fffffff>0x40000000)||b.half_extents.iter().any(|x|x&0x80000000!=0||x&0x7fffffff==0||x&0x7fffffff>0x49800000){return Err(Error::Geometry(i))}}
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let mut min=[0x7f7fffff;3];let mut max=[0xff7fffff;3];
  // Source unroll uses four vectors, paired with opposite corners. Reduction
  // order differs between pairs; retain it rather than using a matrix library.
  let orders=[[[0,1,2],[1,0,2],[0,2,1]],[[1,2,0],[1,2,0],[2,1,0]],[[2,1,0],[2,1,0],[2,1,0]],[[0,1,2],[1,0,2],[0,1,2]]];
  for b in blocks{let[x,y,z,w]=b.orientation;let nx2=neg(mul(x,x));let ny2=neg(mul(y,y));let nz2=neg(mul(z,z));
   let coefficients=[[add(nz2,ny2),sub(mul(y,x),mul(z,w)),add(mul(z,x),mul(y,w))],
    [add(mul(y,x),mul(z,w)),add(nz2,nx2),sub(mul(z,y),mul(x,w))],
    [sub(mul(z,x),mul(y,w)),add(mul(z,y),mul(x,w)),add(ny2,nx2)]];
   let[hx,hy,hz]=b.half_extents;let vectors=[[hx,hy,hz],[neg(hx),hy,hz],[neg(hx),hy,neg(hz)],[hx,hy,neg(hz)]];
   for(pair,v)in vectors.into_iter().enumerate(){let mut rotated=[0;3];for axis in 0..3{let terms=std::array::from_fn::<_,3,_>(|j|mul(coefficients[axis][j],v[j]));let o=orders[pair][axis];rotated[axis]=add(mul(add(add(terms[o[0]],terms[o[1]]),terms[o[2]]),0x40000000),v[axis]);}
    for opposite in[true,false]{for axis in 0..3{let value=if opposite{sub(b.center[axis],rotated[axis])}else{add(rotated[axis],b.center[axis])};if f(value)<=f(min[axis]){min[axis]=value}if f(max[axis])<=f(value){max[axis]=value}}}
   }
  }
  let delta=[sub(max[0],min[0]),sub(max[1],min[1]),sub(max[2],min[2])];
  let y=Extended::from_f32(f(delta[1]))?;let z=Extended::from_f32(f(delta[2]))?;
  let retained=context.add(context.mul(y,y)?,context.mul(z,z)?)?;
  Ok(DefinitionBounds{fields:[min[0],min[1],min[2],delta[0],delta[1],delta[2]],retained_yz_sum:retained.to_le_bytes()})
 })
}
pub fn publish_finalized(body:&FinalizedBodyFields,context:Context)->Result<DefinitionBounds,Error>{let b=body.blocks.iter().map(|b|BoundsBlock{center:b.center.map(f32::to_bits),orientation:b.absolute_orientation.map(f32::to_bits),half_extents:b.half_extents.map(f32::to_bits)}).collect::<Vec<_>>();publish(&b,context)}
