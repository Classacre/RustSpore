//! Full fresh activation and private loop/counter history comparison candidate.
use spore_core::{anim,cell_runtime_fresh_aid::{initialize_fresh_aid,SourceGlobalSerial,EventResetBoundary},
 cell_runtime_fresh_loop::{activate_fresh_loop,FreshLoopError,SourceGeneration,ZeroMaskBoundary,LoopStep},
 cell_runtime_controller_loop::PropertyLoopMode,cell_runtime_source_clock_root_data::SourceMetadataAdmission,
 cell_runtime_loop_clock::{LoopArithmeticContext,ModfDispatch},native_sse::Policy,native_x87::Context};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
fn word(b:&[u8],a:usize)->u32{u32::from_le_bytes(b[a..a+4].try_into().unwrap())}
fn half(b:&[u8],a:usize)->u16{u16::from_le_bytes(b[a..a+2].try_into().unwrap())}
fn csr()->u32{let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v}
fn set_csr(v:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set_csr(self.0);}}
fn context(cw:u16)->LoopArithmeticContext{LoopArithmeticContext{x87:Context::from_control_word(cw).unwrap(),sse:Policy::Mxcsr1f80,modf_dispatch:ModfDispatch::ExplicitFiniteSse2}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("usage: INPUT-DIR NATIVE864.bin RUST792.bin".into());}
 let root=Path::new(&a[0]);let input=fs::read(root.join("input.bin"))?;let native=fs::read(&a[1])?;let table=fs::read_to_string(root.join("cases.tsv"))?;
 assert_eq!(input.len()%256,0);let n=input.len()/256;assert_eq!(n,300);assert_eq!(native.len(),n*864);
 let mut output=OpenOptions::new().create_new(true).write(true).open(&a[2])?;let _restore=Restore(csr());let(mut cases,mut guards)=(0,0);
 for line in table.lines().skip(1){let fields:Vec<_>=line.split('\t').collect();assert_eq!(fields.len(),4);assert_eq!(fields[0].parse::<usize>()?,cases);
  let raw=&input[cases*256..(cases+1)*256];let reply=&native[cases*864..(cases+1)*864];let payload=fs::read(root.join(fields[1]))?;let animation=anim::parse(&payload)?;
  assert_eq!(word(&payload,8),25);assert_eq!(word(raw,24),animation.header_flags_280);assert_eq!(word(raw,28),animation.frame_duration.to_bits());assert_eq!(word(raw,32),animation.duration.to_bits());assert_eq!(word(raw,36),animation.bone_count);
  assert_eq!(half(raw,6),0);assert_eq!(word(raw,8),1);assert_eq!(word(raw,12),0);assert!(word(raw,16)<=16);assert_eq!(word(raw,40),8);assert_eq!(&raw[44..60],&[0;16]);for at in[60,64,68]{assert_eq!(word(raw,at),1);}assert_eq!(&raw[72..128],&[0;56]);
  let cw=half(raw,4);assert_eq!(cw,u16::from_str_radix(fields[3],16)?);let ctx=context(cw);let fresh=initialize_fresh_aid(SourceGlobalSerial::ExplicitSnapshot(word(raw,0)),EventResetBoundary::BoundedPending{count:0,capacity:word(raw,16)})?;
  for j in 0..9{let r=&reply[j*96..(j+1)*96];assert_eq!(half(r,88),cw);assert_eq!(half(r,90)&0x3800,0);assert_eq!(word(r,92)&!0x3f,0x1f80);assert_eq!(half(r,10),0);}
  let mut computed=None;
  for incoming in[0x1f80,0xdf81,0xbf84]{set_csr(incoming);let before=x87();
   let init=|admission,generation,boundary|activate_fresh_loop(fresh,&animation,admission,generation,PropertyLoopMode::Repeat,boundary,ctx);
   assert_eq!(init(SourceMetadataAdmission::Unestablished,SourceGeneration::ExplicitIsolatedSnapshot(0),ZeroMaskBoundary::UnboundFreshFourZeroMasks),Err(FreshLoopError::UnestablishedResourceMapping));guards+=1;
   assert_eq!(init(SourceMetadataAdmission::ExplicitValidatedV25Resource,SourceGeneration::Unestablished,ZeroMaskBoundary::UnboundFreshFourZeroMasks),Err(FreshLoopError::UnestablishedGeneration));guards+=1;
   assert_eq!(init(SourceMetadataAdmission::ExplicitValidatedV25Resource,SourceGeneration::ExplicitIsolatedSnapshot(0),ZeroMaskBoundary::Unestablished),Err(FreshLoopError::UnestablishedZeroMaskBoundary));guards+=1;
   let(mut state,initial)=init(SourceMetadataAdmission::ExplicitValidatedV25Resource,SourceGeneration::ExplicitIsolatedSnapshot(word(raw,20)),ZeroMaskBoundary::UnboundFreshFourZeroMasks)?;assert_eq!(state.channel_count(),word(raw,36));
   let mut bytes=Vec::with_capacity(792);bytes.extend(initial.to_le_bytes());
   let saved=state;for(step,error)in[(LoopStep::Absolute{time_bits:0xbf800000},FreshLoopError::NegativeTime),(LoopStep::Absolute{time_bits:0x7fc12345},FreshLoopError::NonFiniteTime),(LoopStep::Advance{delta_bits:0x7f800000},FreshLoopError::NonFiniteTime)]{assert_eq!(state.step(step,0,ctx),Err(error));assert_eq!(state,saved);guards+=1;}
   assert_eq!(state.step(LoopStep::Absolute{time_bits:0},word(raw,16)+1,ctx),Err(FreshLoopError::PendingEventsExceedCapacity));assert_eq!(state,saved);guards+=1;
   for j in 0..8{let at=128+j*16;assert_eq!(word(raw,at+12),0);let step=match word(raw,at){0=>LoopStep::Absolute{time_bits:word(raw,at+4)},1=>LoopStep::Advance{delta_bits:word(raw,at+4)},_=>return Err("unknown step".into())};bytes.extend(state.step(step,word(raw,at+8),ctx)?.to_le_bytes());}
   assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);
   for j in 0..9{if bytes[j*88..(j+1)*88]!=reply[j*96..j*96+88]{return Err(format!("fresh loop case {cases} snapshot {j} differs").into());}}
   if let Some(ref old)=computed{assert_eq!(old,&bytes);}computed=Some(bytes);
  }output.write_all(&computed.unwrap())?;cases+=1;
 }assert_eq!(cases,n);output.flush()?;println!("CELL_RUNTIME_FRESH_LOOP_VERIFY cases={cases} snapshots={} projection_bytes=792 incoming_states=3 positive_typed_guards={guards} isolated_zero_mask_boundary_only=true",cases*9);Ok(())
}
