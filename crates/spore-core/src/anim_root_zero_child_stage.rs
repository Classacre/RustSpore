//! Independent full-entry root staging for an explicitly supplied zero-child body.
//! Bounded original-host comparison; no generated-body or actor admission.
use crate::{native_sse::{self,Policy,SseError},native_x87::{Context,Extended,X87Error}};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Error{Length,Domain(&'static str),NonFiniteInput(usize),NonFiniteArithmetic,X87(X87Error),Sse(SseError)}
impl From<X87Error> for Error{fn from(e:X87Error)->Self{Self::X87(e)}}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
fn finite(x:u32)->bool{x&0x7f800000!=0x7f800000}
fn get(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn put(b:&mut[u8],o:usize,v:u32){b[o..o+4].copy_from_slice(&v.to_le_bytes())}
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn op(a:u32,b:u32,kind:u8)->Result<u32,Error>{
 let result:u32;
 unsafe{match kind{
 0=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","addss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 1=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","subss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 2=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","mulss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 _=>std::arch::asm!("movd {x},{a:e}","movd {y},{b:e}","divss {x},{y}","movd {r:e},{x}",x=out(xmm_reg)_,y=out(xmm_reg)_,a=in(reg)a,b=in(reg)b,r=lateout(reg)result,options(nostack,preserves_flags)),
 }}if finite(result){Ok(result)}else{Err(Error::NonFiniteArithmetic)}
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn op(_:u32,_:u32,_:u8)->Result<u32,Error>{Err(Error::Sse(SseError::UnsupportedArchitecture))}
fn add(a:u32,b:u32)->Result<u32,Error>{op(a,b,0)}fn sub(a:u32,b:u32)->Result<u32,Error>{op(a,b,1)}
fn mul(a:u32,b:u32)->Result<u32,Error>{op(a,b,2)}fn div(a:u32,b:u32)->Result<u32,Error>{op(a,b,3)}
const ONE:u32=0x3f800000;
fn f(v:u32)->f32{f32::from_bits(v)}
fn quat(b:&[u8],o:usize)->[u32;4]{[get(b,o),get(b,o+4),get(b,o+8),get(b,o+12)]}
fn writeq(b:&mut[u8],o:usize,q:[u32;4]){for(i,v)in q.into_iter().enumerate(){put(b,o+i*4,v)}}
fn norm(q:[u32;4],c:Context)->Result<[u32;4],Error>{
 let x=Extended::from_f32(f(q[0]))?;let y=Extended::from_f32(f(q[1]))?;let z=Extended::from_f32(f(q[2]))?;let w=Extended::from_f32(f(q[3]))?;
 let w2=c.mul(w,w)?;let x2=c.mul(x,x)?;let y2=c.mul(y,y)?;let z2=c.mul(z,z)?;
 let sum=c.add(c.add(c.add(x2,w2)?,y2)?,z2)?;let length=c.store_f32(c.sqrt(sum)?)?.to_bits();
 if f(length)==0.{return Ok(q)}
 let reciprocal=div(ONE,length)?;
 Ok([mul(reciprocal,q[0])?,mul(reciprocal,q[1])?,mul(reciprocal,q[2])?,mul(q[3],reciprocal)?])
}
/// 1256-byte declared context -> canonical root/scratch728-byte projection.
/// Root pointer0 is canonical zero in the projection only; host uses an owned definition pointer.
pub fn evaluate(input:&[u8])->Result<[u8;728],Error>{
 if input.len()!=1256{return Err(Error::Length)}
 let cw=get(input,0);if cw&0xfffff0ff!=0x7f||cw&0x300==0x100{return Err(Error::Domain("CW"))}
 if get(input,4)!=0||get(input,8)!=1||get(input,1252)!=0{return Err(Error::Domain("zero-child one-root topology"))}
 if get(input,20)>1||get(input,28)!=0||(32..48).step_by(4).any(|i|get(input,i)!=0){return Err(Error::Domain("control/pointers"))}
 for at in [12,16,24].into_iter().chain((48..88).step_by(4)).chain((92..192).step_by(4)).chain((1000..1028).step_by(4)){
  if !finite(get(input,at)){return Err(Error::NonFiniteInput(at))}
 }
 if get(input,12)&0x80000000!=0{return Err(Error::Domain("nonnegative dt"))}
 let c=Context::from_control_word(cw as u16)?;
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let mut root=[0u8;700];root.copy_from_slice(&input[32..732]);let def=&input[732..1256];let mut default=[0u32;3];
  for i in 0..3{default[i]=mul(get(def,0x10c+i*4),get(input,24))?}
  put(&mut root,0x58,add(get(input,32+0x58),get(input,32+0x7c))?);
  for(a,b)in[(0x5c,0x80),(0x60,0x84),(0x64,0x88)]{let v=add(get(&root,b),get(&root,a))?;put(&mut root,a,v)}
  let primary=quat(&root,0x68);let alternate=quat(&root,0x8c);
  let dot=add(add(add(mul(primary[0],alternate[0])?,mul(primary[1],alternate[1])?)?,mul(primary[2],alternate[2])?)?,mul(primary[3],alternate[3])?)?;
  let mut combined=[0;4];for i in 0..4{combined[i]=if f(dot)>=0.{add(primary[i],alternate[i])?}else{sub(primary[i],alternate[i])?}}
  writeq(&mut root,0x68,combined);let weight=add(get(&root,0x9c),get(&root,0x78))?;put(&mut root,0x78,weight);
  let pw=get(&root,0x64);
  if f(pw)<=0.{for i in 0..3{put(&mut root,0x190+i*4,default[i])}}
  else{
   let inverse=div(ONE,pw)?;for i in 0..3{let at=0x58+i*4;let v=mul(inverse,get(&root,at))?;put(&mut root,at,v)}
   let alpha=if f(pw)>1.{ONE}else{pw};let complement=sub(ONE,alpha)?;
   for i in 0..3{let value=add(mul(get(&root,0x58+i*4),alpha)?,mul(default[i],complement)?)?;put(&mut root,0x190+i*4,value)}put(&mut root,0x64,ONE);
  }
  let rest=quat(def,0x118);
  let staged=if f(weight)<=0.{rest}else{
   let normal=norm(combined,c)?;writeq(&mut root,0x68,normal);
   let result=if f(weight)>=1.{normal}else{
    let alpha=if f(weight)>1.{ONE}else{weight};let complement=sub(ONE,alpha)?;let mut a=[0;4];let mut b=[0;4];
    for i in 0..4{a[i]=if i==0{mul(alpha,normal[i])?}else{mul(normal[i],alpha)?};b[i]=mul(rest[i],complement)?}
    let dot=add(add(add(mul(a[3],b[3])?,mul(a[2],b[2])?)?,mul(a[1],b[1])?)?,mul(a[0],b[0])?)?;
    let mut mixed=[0;4];for i in 0..4{mixed[i]=if f(dot)>=0.{add(a[i],b[i])?}else{sub(a[i],b[i])?}}norm(mixed,c)?
   };put(&mut root,0x78,ONE);result
  };writeq(&mut root,0x19c,staged);root[0x38]=1;
  let [x,y,z,w]=staged;let [rx,ry,rz,rw]=rest;
  let orientation=[sub(add(sub(mul(x,rw)?,mul(rx,w)?)?,mul(ry,z)?)?,mul(rz,y)?)?,
   add(sub(sub(mul(y,rw)?,mul(z,rx)?)?,mul(ry,w)?)?,mul(rz,x)?)?,
   sub(sub(add(mul(y,rx)?,mul(z,rw)?)?,mul(ry,x)?)?,mul(rz,w)?)?,
   add(add(add(mul(y,ry)?,mul(rz,z)?)?,mul(rx,x)?)?,mul(w,rw)?)?];
  writeq(&mut root,0x200,staged);
  for at in[0x1b0,0x1bc,0x1c8,0x1d4]{for i in 0..3{let v=get(&root,0x190+i*4);put(&mut root,at+i*4,v)}}put(&mut root,0x210,0);put(&mut root,0x214,0);
  // Full9E8930 -> full9E7C80 observes zero children and leaves root unchanged.
  let dt=get(input,12);let smoothing=get(input,16);let complement=sub(ONE,smoothing)?;
  if get(input,20)==0{
   for i in 0..3{let velocity=if f(dt)<=f(0x358637bd){0}else{mul(sub(get(&root,0x190+i*4),get(&root,0x10+i*4))?,div(ONE,dt)?)?};put(&mut root,0x2c+i*4,velocity)}
   for i in 0..7{let v=get(&root,0x190+i*4);put(&mut root,0x10+i*4,v)}root[0x38]=1;
  }
  for i in 0..3{let at=0x3c+i*4;let v=add(mul(get(&root,at),smoothing)?,mul(get(&root,0x10+i*4),complement)?)?;put(&mut root,at,v)}
  let mut output=[0;728];output[..700].copy_from_slice(&root);writeq(&mut output,700,orientation);for i in 0..3{put(&mut output,716+i*4,default[i])}Ok(output)
 })
}
