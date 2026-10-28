//! Compare 64 synthetic-stage/source-body mode-2 cases with original output.
//! Input and native artifacts must be independently hash-verified before use.
//! This executable is a standalone example; it does not export the candidate.
#[path = "../src/anim_cid_child_mode2_v1.rs"]
mod candidate;
use std::{error::Error, fs::{self,OpenOptions}, io::{self,Write}, path::Path};
#[cfg(not(target_arch="x86_64"))]
compile_error!("This comparison requires x86_64 SSE2 and x87 state controls");
fn word(bytes:&[u8], offset:usize)->u32 { u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap()) }
fn words<const N:usize>(bytes:&[u8], offset:usize)->[u32;N] { std::array::from_fn(|i|word(bytes,offset+4*i)) }
fn append_words(dest:&mut Vec<u8>, words:&[u32]) { for v in words { dest.extend_from_slice(&v.to_le_bytes()); } }
fn region(offset:usize)->&'static str { match offset { 0..=11=>"counters",12..=23=>"position",24..=39=>"quaternion",40..=43=>"aggregate flags",44..=47=>"child flags",48..=51=>"child +210",52..=53=>"x87 control word",54..=55=>"x87 status word",56..=59=>"MXCSR",_=>"invalid" } }
fn compare(input:&[u8],native:&[u8])->Result<Vec<u8>,Box<dyn Error>> {
 if input.len()!=64*72 || native.len()!=64*60 { return Err("expected exactly 64 input and native records".into()); }
 let mut all=Vec::with_capacity(64*60);
 for (case,packet) in input.chunks_exact(72).enumerate() {
  let count=word(packet,0); if !(1..=255).contains(&count) {return Err(format!("case {case}: child count out of range").into());}
  let x=candidate::Input { parent_definition_position:words(packet,4), definition_quaternion:words(packet,16), child_definition_position:words(packet,32), stage_quaternion:words(packet,44), stage_position:words(packet,60) };
  let cw=0x027fu16;let csr=0x1f80u32;
  // Original host sets x87 CW 027F and MXCSR 1F80 for every invocation.
  unsafe {std::arch::asm!("fninit","fldcw [{cw}]","ldmxcsr [{csr}]",cw=in(reg)&cw,csr=in(reg)&csr,options(nostack));}
  let result=candidate::evaluate_explicit_synthetic_mode2(&x).map_err(|e|io::Error::new(io::ErrorKind::Other,format!("case {case}: {e:?}")))?;
  // Capture status immediately after candidate arithmetic, before comparison.
  let (mut actual_cw,mut sw,mut actual_csr)=(0u16,0u16,0u32);
  unsafe {std::arch::asm!("fnstcw [{cw}]","fnstsw [{sw}]","stmxcsr [{csr}]",cw=in(reg)&mut actual_cw,sw=in(reg)&mut sw,csr=in(reg)&mut actual_csr,options(nostack,preserves_flags));}
  let mut got=Vec::with_capacity(60);append_words(&mut got,&result.counters);append_words(&mut got,&result.position);append_words(&mut got,&result.quaternion);got.extend_from_slice(&[4,0,255,255]);got.extend_from_slice(&result.node_flags);got.extend_from_slice(&result.node_210.to_le_bytes());got.extend_from_slice(&actual_cw.to_le_bytes());got.extend_from_slice(&sw.to_le_bytes());got.extend_from_slice(&actual_csr.to_le_bytes());
  if got.len()!=60 {return Err("internal record length".into());} let expected=&native[case*60..(case+1)*60];
  if let Some(offset)=got.iter().zip(expected).position(|(a,b)|a!=b) {
   return Err(format!("first mismatch: case {case}, record byte {offset} ({}), native={:02x}, rust={:02x}; native record={} rust record={}",region(offset),expected[offset],got[offset],hex(expected),hex(&got)).into());
  }
  all.extend_from_slice(&got);
 }
 Ok(all)
}
fn hex(data:&[u8])->String { data.iter().map(|x|format!("{x:02x}")).collect() }
fn main()->Result<(),Box<dyn Error>> {
 let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=3 {return Err("usage: INPUT.bin NATIVE-OUTPUT.bin NEW-RUST-OUTPUT.bin".into());}
 let input=fs::read(Path::new(&args[0]))?;let native=fs::read(Path::new(&args[1]))?;let rows=compare(&input,&native)?;
 let mut output=OpenOptions::new().write(true).create_new(true).open(Path::new(&args[2]))?;output.write_all(&rows)?;output.flush()?;
 println!("CID_CHILD_MODE2_COMPARE cases=64 record_bytes=60 source_body_contexts=true synthetic_stage=true complete_bytes_equal=true observed_motion=false"); Ok(())
}
