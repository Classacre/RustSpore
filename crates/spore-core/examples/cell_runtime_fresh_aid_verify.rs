//! Fulloriginal freshaid clock-fields/reset comparison; root owns promotion/run.
use spore_core::cell_runtime_fresh_aid::{initialize_fresh_aid,SourceGlobalSerial,EventResetBoundary,FreshAidError};
use std::{error::Error,fs::{File,OpenOptions},io::{BufReader,BufWriter,Read,Write}};
fn w(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn h(b:&[u8],at:usize)->u16{u16::from_le_bytes(b[at..at+2].try_into().unwrap())}
fn csr()->u32{let mut x=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));}x}
fn set(x:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set(self.0);}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("usage: INPUT.bin NATIVE.bin RUST-PROJECTION.bin".into());}let _restore=Restore(csr());let mut input=BufReader::new(File::open(&a[0])?);let mut native=BufReader::new(File::open(&a[1])?);let mut output=BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&a[2])?);let mut count=0;
 loop{let mut q=[0;64];if input.read(&mut q[..1])?==0{break;}input.read_exact(&mut q[1..])?;let mut r=[0;64];native.read_exact(&mut r)?;let cw=h(&q,4);assert!(cw&0xf0ff==0x7f&&cw&0x300!=0x100);assert_eq!(h(&q,6),0);assert_eq!(&q[20..],&[0;44]);assert_eq!(h(&r,56),cw);assert_eq!(h(&r,58)&0x3800,0);assert_eq!(w(&r,60),0x1f80);
 let events=match w(&q,8){0=>{assert_eq!(w(&q,12),0);assert_eq!(w(&q,16),0);EventResetBoundary::FreshOnly},1=>EventResetBoundary::BoundedPending{count:w(&q,12),capacity:w(&q,16)},_=>return Err("invalidmode".into())};let mut computed=None;
 for incoming in[0x1f80,0xdf81,0xbf84]{set(incoming);let before=x87();let actual=initialize_fresh_aid(SourceGlobalSerial::ExplicitSnapshot(w(&q,0)),events)?.to_le_bytes();assert_eq!(csr(),incoming);assert_eq!(x87(),before);assert_eq!(&actual,&r[..56],"case{count}");if let Some(old)=computed{assert_eq!(actual,old);}computed=Some(actual);}output.write_all(&computed.unwrap())?;count+=1;
 }assert_eq!(count,288);assert_eq!(native.read(&mut[0])?,0);output.flush()?;let mut guards=0;
 for incoming in[0x1f80,0xdf81,0xbf84]{set(incoming);for(s,e,error)in[(SourceGlobalSerial::Unestablished,EventResetBoundary::FreshOnly,FreshAidError::UnestablishedGlobalSerial),(SourceGlobalSerial::ExplicitSnapshot(0),EventResetBoundary::Unestablished,FreshAidError::UnestablishedEventBoundary),(SourceGlobalSerial::ExplicitSnapshot(0),EventResetBoundary::BoundedPending{count:0,capacity:17},FreshAidError::EventCapacityOutOfRange),(SourceGlobalSerial::ExplicitSnapshot(0),EventResetBoundary::BoundedPending{count:2,capacity:1},FreshAidError::EventCountExceedsCapacity)]{let before=x87();assert_eq!(initialize_fresh_aid(s,e),Err(error));assert_eq!(csr(),incoming);assert_eq!(x87(),before);guards+=1;}}
 println!("CELL_RUNTIME_FRESH_AID_VERIFY cases={count} caller_states=3 typed_errors={guards} exact_bits=true cycle_epoch_initialized_one=true no_clock_event_dispatch_admission=true");Ok(())
}
