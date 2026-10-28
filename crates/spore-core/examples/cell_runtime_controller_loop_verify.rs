//! Fulloriginal controller-loop setter comparison; root owns promotion/run.
use spore_core::cell_runtime_controller_loop::{apply_property_loop,ControllerLoopSnapshot,ControllerSlot,PropertyLoopMode,ControllerLoopError};
use std::{error::Error,fs::{File,OpenOptions},io::{BufReader,BufWriter,Read,Write}};
fn w(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn h(b:&[u8],at:usize)->u16{u16::from_le_bytes(b[at..at+2].try_into().unwrap())}
fn csr()->u32{let mut x=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));}x}
fn set(x:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set(self.0);}}
fn state(q:&[u8])->ControllerLoopSnapshot{ControllerLoopSnapshot{primary_index:w(q,4)as u8,secondary_index:w(q,12)as u8,primary:ControllerSlot{epoch:w(q,8),linked_handle:w(q,20),has_aid:w(q,28)!=0,loop_request:w(q,36)as u8},secondary:ControllerSlot{epoch:w(q,16),linked_handle:0,has_aid:w(q,32)!=0,loop_request:w(q,40)as u8},aid_loop_bytes:[w(q,44)as u8,w(q,48)as u8]}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("usage: INPUT.bin NATIVE.bin RUST-PROJECTION.bin".into());}let _restore=Restore(csr());let mut input=BufReader::new(File::open(&a[0])?);let mut native=BufReader::new(File::open(&a[1])?);let mut out=BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&a[2])?);let mut count=0;
 loop{let mut q=[0;64];if input.read(&mut q[..1])?==0{break;}input.read_exact(&mut q[1..])?;let mut r=[0;36];native.read_exact(&mut r)?;
 assert!(w(&q,4)<16&&w(&q,12)<16&&w(&q,4)!=w(&q,12)&&w(&q,28)<=1&&w(&q,32)<=1);for at in[36,40,44,48]{assert!(w(&q,at)<=255);}assert_eq!(w(&q,52),u32::from(w(&q,24)==1));assert_eq!(&q[56..],&[0;8]);assert_eq!(h(&r,28),0x027f);assert_eq!(h(&r,30)&0x3800,0);assert_eq!(w(&r,32),0x1f80);
 let mut computed=None;for incoming in[0x1f80,0xdf81,0xbf84]{set(incoming);let before=x87();let actual=apply_property_loop(state(&q),w(&q,0),PropertyLoopMode::from_source_word(w(&q,24)))?.to_le_bytes();assert_eq!(csr(),incoming);assert_eq!(x87(),before);assert_eq!(&actual,&r[..28],"case{count}");if let Some(old)=computed{assert_eq!(actual,old);}computed=Some(actual);}out.write_all(&computed.unwrap())?;count+=1;
 }assert_eq!(count,536);assert_eq!(native.read(&mut[0])?,0);out.flush()?;
 let base=ControllerLoopSnapshot{primary_index:0,secondary_index:1,primary:ControllerSlot{epoch:1,linked_handle:0,has_aid:true,loop_request:0},secondary:ControllerSlot{epoch:2,linked_handle:0,has_aid:true,loop_request:1},aid_loop_bytes:[0,1]};let mut guards=0;
 for incoming in[0x1f80,0xdf81,0xbf84]{set(incoming);for(s,handle,error)in[(ControllerLoopSnapshot{primary_index:16,..base},0x101,ControllerLoopError::PrimaryIndexOutOfRange),(ControllerLoopSnapshot{secondary_index:16,..base},0x101,ControllerLoopError::SecondaryIndexOutOfRange),(ControllerLoopSnapshot{secondary_index:0,..base},0x101,ControllerLoopError::AliasedSlots),(base,0x100,ControllerLoopError::PrimaryHandleIndexMismatch),(ControllerLoopSnapshot{primary:ControllerSlot{linked_handle:0x2ff,..base.primary},..base},0x101,ControllerLoopError::LinkedHandleIndexMismatch)]{let before=x87();assert_eq!(apply_property_loop(s,handle,PropertyLoopMode::Repeat),Err(error));assert_eq!(csr(),incoming);assert_eq!(x87(),before);guards+=1;}}
 println!("CELL_RUNTIME_CONTROLLER_LOOP_VERIFY cases={count} caller_states=3 typed_errors={guards} exact_bits=true linked_secondary_preserved=true counter_event_state_unadmitted=true");Ok(())
}
