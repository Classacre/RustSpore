//! Compare original root coordinate conversion and zero-state accumulators.
use spore_core::anim_root_pose;
use std::{env,fs,path::PathBuf,io::Write};
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
fn csr()->u32 { let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v }
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
unsafe fn set_csr(v:u32){std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){unsafe{set_csr(self.0)}}}
fn word(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn mutate(raw:&[u8],at:usize,v:u32)->Vec<u8>{let mut x=raw.to_vec();x[at..at+4].copy_from_slice(&v.to_le_bytes());x}
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=env::args_os().skip(1).collect();
    if args.len()!=3{return Err("usage: INPUT_BIN NATIVE_176B_REPLIES OUTPUT_156B_PROJECTIONS".into())}
    let input=fs::read(PathBuf::from(&args[0]))?;
    let native=fs::read(PathBuf::from(&args[1]))?;
    if input.len()%144!=0 || native.len()!=input.len()/144*176{return Err("framing".into())}
    let _restore=Restore(csr());
    let mut output=Vec::with_capacity(input.len()/144*156);
    for (row,request) in input.chunks_exact(144).enumerate(){
        let mut accepted=None;
        for host in [0x1f80u32,0x3f80,0x9fc0]{
            unsafe{set_csr(host)};
            let value=anim_root_pose::evaluate(request);
            if csr()!=host{return Err(format!("case{row}: MXCSR restoration").into())}
            let value=value.map_err(|e|format!("case{row}: {e:?}"))?;
            if let Some(previous)=accepted{if previous!=value{return Err("host MXCSR changed output".into())}}
            accepted=Some(value);
        }
        let actual=accepted.unwrap();
        let reply=&native[row*176..(row+1)*176];
        if u16::from_le_bytes(reply[156..158].try_into()?) as u32!=word(request,0)
            || u16::from_le_bytes(reply[158..160].try_into()?)&0x3800!=0
            || word(reply,160)&!0x3f!=0x1f80 || reply[164..]!=[0u8;12]{return Err(format!("case{row}: native metadata").into())}
        if actual!=native[row*176..row*176+156]{
            let word=actual.chunks_exact(4).zip(native[row*176..row*176+156].chunks_exact(4))
                .position(|(a,b)|a!=b).unwrap();
            return Err(format!("case{row} word{word} mismatch").into())
        }
        output.extend(actual);
    }
    let first=input.get(..144).ok_or("empty corpus")?;
    for host in [0x1f80u32,0x3f80,0x9fc0]{
        unsafe{set_csr(host)};
        use anim_root_pose::Error;
        let invalid=[(0,0x17f,Error::Domain("CW")),(4,0x11,Error::Domain("POS flags")),
            (8,1,Error::Domain("channel/mirror")),(12,2,Error::Domain("channel/mirror")),
            (40,1,Error::Domain("mask")),(60,2,Error::Domain("mask")),
            (16,0,Error::Domain("positive blend")),(16,0xbf800000,Error::Domain("positive blend")),
            (108,0x7fc00000,Error::NonFiniteInput(27)),(124,0x7fc00000,Error::NonFiniteInput(31))];
        for (at,v,wanted) in invalid{
            if anim_root_pose::evaluate(&mutate(first,at,v))!=Err(wanted){return Err(format!("typed protocol at {at}").into())}
            if csr()!=host{return Err("error MXCSR restoration".into())}
        }
        if anim_root_pose::evaluate(&first[..143])!=Err(Error::InputLength){return Err("length error".into())}
        for value in [0,0xbf800000]{
            let mut raw=mutate(first,120,value);raw[140..144].copy_from_slice(&value.to_le_bytes());
            let result=anim_root_pose::evaluate(&raw).map_err(|e|format!("weight protocol {e:?}"))?;
            if result[44..116]!=[0u8;72] || word(&result,152)!=0 || word(&result,28)!=word(first,16){return Err("weight protocol".into())}
        }
        if csr()!=host{return Err("protocol MXCSR restoration".into())}
    }
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(PathBuf::from(&args[2]))?;
    file.write_all(&output)?;
    println!("{} source/reference-context projections agree",input.len()/144);
    Ok(())
}
