//! Source-root coordinate conversion and zero-state POS/ROT accumulation.
//!
//! Independent finite API for the exact 31/71/B1 POS,32 ROT,CHAN0/10 domain.
//! Explicit reference context only; no actor/body association or staged pose.
//! Bounded original-host comparison; full actor binding and pose staging are separate.

use crate::native_sse::{self, Policy, SseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InputLength,
    Domain(&'static str),
    NonFiniteInput(usize),
    NonFiniteArithmetic,
    Sse(SseError),
}
impl From<SseError> for Error { fn from(value: SseError) -> Self { Self::Sse(value) } }

fn finite(value: u32) -> bool { value & 0x7f80_0000 != 0x7f80_0000 }

#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn binary(a: u32,b: u32,operation: u8) -> Result<u32,Error> {
    let output: u32;
    // SAFETY: register-only scalar SSE under the enclosing scoped policy.
    // Separate MULSS/ADDSS/SUBSS retain operand order and binary32 commits.
    unsafe {
        match operation {
            0 => std::arch::asm!("movd {left}, {a:e}","movd {right}, {b:e}",
                "mulss {left}, {right}","movd {result:e}, {left}",
                left=out(xmm_reg) _,right=out(xmm_reg) _,a=in(reg) a,b=in(reg) b,
                result=lateout(reg) output,options(nostack,preserves_flags)),
            1 => std::arch::asm!("movd {left}, {a:e}","movd {right}, {b:e}",
                "addss {left}, {right}","movd {result:e}, {left}",
                left=out(xmm_reg) _,right=out(xmm_reg) _,a=in(reg) a,b=in(reg) b,
                result=lateout(reg) output,options(nostack,preserves_flags)),
            _ => std::arch::asm!("movd {left}, {a:e}","movd {right}, {b:e}",
                "subss {left}, {right}","movd {result:e}, {left}",
                left=out(xmm_reg) _,right=out(xmm_reg) _,a=in(reg) a,b=in(reg) b,
                result=lateout(reg) output,options(nostack,preserves_flags)),
        }
    }
    if finite(output) { Ok(output) } else { Err(Error::NonFiniteArithmetic) }
}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
fn binary(_:u32,_:u32,_:u8)->Result<u32,Error>{Err(SseError::UnsupportedArchitecture.into())}
fn mul(a:u32,b:u32)->Result<u32,Error>{binary(a,b,0)}
fn add(a:u32,b:u32)->Result<u32,Error>{binary(a,b,1)}
fn sub(a:u32,b:u32)->Result<u32,Error>{binary(a,b,2)}

/// Independent 99BD80 product, with measured scalar operation/operand order.
fn product(sample:[u32;4],rest:[u32;4])->Result<[u32;4],Error>{
    let [x,y,z,w]=sample;let [a,b,c,d]=rest;
    Ok([
        add(sub(add(mul(a,w)?,mul(x,d)?)?,mul(b,z)?)?,mul(c,y)?)?,
        sub(add(add(mul(z,a)?,mul(b,w)?)?,mul(y,d)?)?,mul(c,x)?)?,
        add(add(sub(mul(z,d)?,mul(y,a)?)?,mul(c,w)?)?,mul(b,x)?)?,
        sub(sub(sub(mul(w,d)?,mul(a,x)?)?,mul(y,b)?)?,mul(c,z)?)?,
    ])
}

/// Consume the native gate's144-byte explicit-state protocol and return156
/// projected bytes. Native CW/SW/MXCSR metadata is checked separately by host.
pub fn evaluate(raw:&[u8])->Result<[u8;156],Error>{
    if raw.len()!=144{return Err(Error::InputLength)}
    let mut words=[0u32;36];
    for (word,bytes) in words.iter_mut().zip(raw.chunks_exact(4)){
        *word=u32::from_le_bytes(bytes.try_into().unwrap());
    }
    let w=words;
    if w[0]&0xffff_f0ff!=0x7f || w[0]&0x300==0x100{return Err(Error::Domain("CW"))}
    if !matches!(w[1],0x31|0x71|0xb1){return Err(Error::Domain("POS flags"))}
    if !matches!(w[2],0|0x10) || w[3]>1{return Err(Error::Domain("channel/mirror"))}
    if w[10]&!8!=0 || w[15]&0xff00_0000!=0 || w[15]&6!=6{return Err(Error::Domain("mask"))}
    for index in (4..36).filter(|i|!matches!(*i,10|15)){
        if !finite(w[index]){return Err(Error::NonFiniteInput(index))}
    }
    if w[4]&0x8000_0000!=0 || w[4]&0x7fff_ffff==0{return Err(Error::Domain("positive blend"))}
    native_sse::scoped(Policy::Mxcsr1f80,||{
        let scale=w[5];let height=w[2]&0x10!=0;let mirror=w[3]!=0;
        let mut position=[w[27],w[28],w[29]];
        match w[1]&0xc0{
            0x40=>{
                for index in 0..3{
                    let factor=mul(w[6+index],scale)?;
                    if index!=2 || !height{
                        position[index]=if index==2 {mul(factor,position[index])?}
                            else {mul(position[index],factor)?};
                    }
                }
            },
            0x80=>{
                let factor=mul(if w[10]&8!=0{w[9]}else{w[11]},scale)?;
                // FLD32 and comparison with source double1e-6 have no rounded
                // arithmetic between them; every finite binary32 is exact here.
                if f64::from(f32::from_bits(factor))>f64::from_bits(0x3eb0_c6f7_a0b5_ed8d){
                    position[0]=mul(position[0],factor)?;
                    position[1]=mul(position[1],factor)?;
                    if !height{position[2]=mul(factor,position[2])?}
                }
            },_=>{},
        }
        if mirror{position[0]^=0x8000_0000}
        let base=[mul(w[12],scale)?,mul(w[13],scale)?,mul(w[14],scale)?];
        position[0]=add(base[0],position[0])?;
        position[1]=add(base[1],position[1])?;
        position[2]=if height{mul(sub(1f32.to_bits(),position[2])?,base[2])?}
            else{add(base[2],position[2])?};
        let mut sampled=[w[31],w[32],w[33],w[34]];
        if mirror{sampled[1]^=0x8000_0000;sampled[2]^=0x8000_0000;}
        let rotation=product(sampled,[w[16],w[17],w[18],w[19]])?;
        let mut pose=[0u32;22];let mut cache=[0u32;9];let mut bitmap=0u32;
        cache[0]=add(0,mul(w[30],position[0])?)?;
        cache[1]=add(mul(w[30],position[1])?,0)?;
        cache[2]=add(0,mul(w[30],position[2])?)?;
        cache[3]=add(w[30],0)?;
        let position_weight=mul(w[30],w[4])?;
        if f32::from_bits(position_weight)>0.0{
            pose[4]=add(mul(position[0],position_weight)?,0)?;
            pose[5]=add(mul(position[1],position_weight)?,0)?;
            pose[6]=add(0,mul(position[2],position_weight)?)?;
            pose[7]=add(0,position_weight)?;bitmap|=1;
        }
        // Initial quaternion sums are zero. Original hemisphere dot is zero,
        // so source99C8D0 takes its add branch; FLD32/FSTP32 copies finite bits.
        for index in 0..4{
            let weighted=mul(w[35],rotation[index])?;
            cache[4+index]=if index==3{add(0,weighted)?}else{add(weighted,0)?};
        }
        cache[8]=add(0,w[35])?;
        let rotation_weight=mul(w[35],w[4])?;
        if f32::from_bits(rotation_weight)>0.0{
            for index in 0..4{
                let weighted=mul(rotation[index],rotation_weight)?;
                pose[8+index]=if index==3{add(0,weighted)?}else{add(weighted,0)?};
            }
            pose[12]=add(0,rotation_weight)?;bitmap|=1;
        }
        pose[0]=add(0,w[4])?;
        let mut output=[0u8;156];
        let all:Vec<u32>=position.into_iter().chain(rotation).chain(pose).chain(cache).chain([bitmap]).collect();
        for (bytes,word) in output.chunks_exact_mut(4).zip(all){bytes.copy_from_slice(&word.to_le_bytes())}
        Ok(output)
    })
}
