//! Candidate full700-byte native/body-reference verifier; not an exported gate.
use spore_core::native_x87;
use spore_core::anim_cid_node_init_v1 as model;
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn csr()->u32{let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v}
fn setcsr(v:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){setcsr(self.0);}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("usage: INPUT-DIR NATIVE-RAW.bin RUST-PROJECTION.bin".into());}
 let root=Path::new(&a[0]);let input=fs::read(root.join("input.bin"))?;let gold=fs::read(root.join("expected.bin"))?;let raw=fs::read(&a[1])?;let mut output=OpenOptions::new().write(true).create_new(true).open(&a[2])?;let _restore=Restore(csr());let(mut pos,mut native,mut expected,mut cases,mut count,mut guards)=(0,0,0,0,0,0);
 while pos<input.len(){let header=&input[pos..pos+32];let n=word(header,4)as usize;assert_eq!(word(header,0),0x31444f4e);assert_eq!(word(header,8),0x27f);assert_eq!(word(header,12),0x1f80);assert_eq!(word(header,20),0x3f800000);let blocks=&input[pos+32..pos+32+n*1128];let nodes:Vec<_>=blocks.chunks_exact(1128).map(|b|model::DefinitionNode{parent:word(b,0x1fc),first_child:word(b,0x204),child_count:word(b,0x208),center:std::array::from_fn(|k|word(b,0x10c+k*4)),quaternion:std::array::from_fn(|k|word(b,0x118+k*4))}).collect();let size=32+n*700;let reply=&raw[native..native+size+8];assert_eq!(u16::from_le_bytes(reply[size..size+2].try_into().unwrap()),0x27f);assert_eq!(u16::from_le_bytes(reply[size+2..size+4].try_into().unwrap())&0x3800,0);assert_eq!(word(reply,size+4)&!0x3f,0x1f80);
  let mut computed=None;
  for incoming in[0x1f80,0xdf81,0xbf84]{setcsr(incoming);let before=x87();assert_eq!(model::initialize(&nodes,model::Admission::Unestablished),Err(model::Error::Unestablished));guards+=1;assert_eq!(model::initialize(&nodes,model::Admission::SourceFactoryScaleOne{control_word:0x37f}),Err(model::Error::ControlWord));guards+=1;
   let rows=model::initialize(&nodes,model::Admission::SourceFactoryScaleOne{control_word:0x27f})?;let mut projection=header.to_vec();for row in rows{projection.extend(row);}assert_eq!(projection,&reply[..size],"native case {cases}");assert_eq!(projection,&gold[expected..expected+size],"independent source reference case {cases}");assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);if let Some(ref old)=computed{assert_eq!(old,&projection);}computed=Some(projection);
  }output.write_all(&computed.unwrap())?;pos+=32+n*1128;native+=size+8;expected+=size;cases+=1;count+=n;
 }assert_eq!(cases,1322);assert_eq!(count,46256);assert_eq!(native,raw.len());assert_eq!(expected,gold.len());output.flush()?;println!("CID_NODE_INIT_VERIFY cases={cases} nodes={count} typed_guards={guards} incoming_CSR_states=3 full_node_bytes=700");Ok(())
}
