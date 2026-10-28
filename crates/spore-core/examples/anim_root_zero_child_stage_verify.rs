//! Full-entry original zero-child staging comparison. No actor admission.
use spore_core::anim_root_zero_child_stage as stage;
use std::{error::Error,fs::OpenOptions,io::{Read,Write}};
fn u32at(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn csr()->u32{let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
fn set(v:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set(self.0)}}
fn main()->Result<(),Box<dyn Error>>{
 let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=3{return Err("INPUT1256 NATIVE736 OUTPUT728".into())}
 let _restore=Restore(csr());let mut inputs=std::fs::File::open(&args[0])?;let mut native=std::fs::File::open(&args[1])?;let mut output=OpenOptions::new().create_new(true).write(true).open(&args[2])?;let mut count=0;
 loop{let mut raw=[0;1256];if inputs.read(&mut raw[..1])?==0{break}inputs.read_exact(&mut raw[1..])?;let mut reply=[0;736];native.read_exact(&mut reply)?;
  assert_eq!(u16::from_le_bytes(reply[728..730].try_into()?) as u32,u32at(&raw,0));assert_eq!(u16::from_le_bytes(reply[730..732].try_into()?)&0x3800,0);assert_eq!(u32at(&reply,732)&!0x3f,0x1f80);
  let mut actual=None;
  for host in[0x1f80,0xdf81,0xbf84]{set(host);let before=x87();let value=stage::evaluate(&raw).map_err(|e|format!("case{count}: {e:?}"))?;assert_eq!(csr(),host);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);
   if value!=reply[..728]{let i=value.iter().zip(reply).position(|(a,b)|*a!=b).unwrap();return Err(format!("case{count} byte{i} Rust{:02X} native{:02X}",value[i],reply[i]).into())}
   if let Some(old)=actual{assert_eq!(old,value)}actual=Some(value);
   let mut child=raw;child[1252..1256].copy_from_slice(&1u32.to_le_bytes());assert_eq!(stage::evaluate(&child),Err(stage::Error::Domain("zero-child one-root topology")));assert_eq!(csr(),host);
   let mut pose=raw;pose[8..12].copy_from_slice(&2u32.to_le_bytes());assert_eq!(stage::evaluate(&pose),Err(stage::Error::Domain("zero-child one-root topology")));assert_eq!(csr(),host);
   assert_eq!(stage::evaluate(&raw[..1255]),Err(stage::Error::Length));assert_eq!(csr(),host);
  }output.write_all(&actual.unwrap())?;count+=1;
 }assert!(count>0);assert_eq!(native.read(&mut[0])?,0);output.flush()?;println!("ROOT_ZERO_CHILD_STAGE cases={count} model_bytes=728 host_CSR_states=3 caller_context_only=true");Ok(())
}
