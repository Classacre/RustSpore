//! Promote only after independent original-byte metadata admission.
use spore_core::{cell_runtime_clip_metadata::{prepare_metadata,ResourceMetadata,MetadataContext,MetadataError},native_sse::Policy,native_x87::Context};
use std::{error::Error,fs::{File,OpenOptions},io::{BufReader,BufWriter,Read,Write}};
fn word(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn half(b:&[u8],at:usize)->u16{u16::from_le_bytes(b[at..at+2].try_into().unwrap())}
fn csr()->u32{let mut x=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));}x}
fn set_csr(x:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set_csr(self.0);}}
fn context(cw:u16)->MetadataContext{MetadataContext{x87:Context::from_control_word(cw).unwrap(),sse:Policy::Mxcsr1f80}}
fn policies()->usize{
 let base=ResourceMetadata{flags:8,frames_bits:0x43050000,frame_interval_bits:0x3d088889};
 let tests=[(ResourceMetadata{frames_bits:0x7fc12345,..base},MetadataError::NonFiniteInput),(ResourceMetadata{frame_interval_bits:0x7f800000,..base},MetadataError::NonFiniteInput),(ResourceMetadata{frames_bits:0x3f800000,..base},MetadataError::UnsupportedFrameCount),(ResourceMetadata{frames_bits:0,..base},MetadataError::UnsupportedFrameCount),(ResourceMetadata{frame_interval_bits:0,..base},MetadataError::NonPositiveInterval),(ResourceMetadata{frame_interval_bits:0xbf800000,..base},MetadataError::NonPositiveInterval),(ResourceMetadata{frames_bits:0x7f7fffff,frame_interval_bits:0x7f7fffff,..base},MetadataError::NonFiniteDuration),(ResourceMetadata{frames_bits:0x3f800001,frame_interval_bits:1,..base},MetadataError::NonPositiveDuration)];
 let mut count=0;for incoming in[0x1f80,0xdf81,0xbf84]{set_csr(incoming);for(input,error)in tests{let before=x87();assert_eq!(prepare_metadata(input,context(0x027f)),Err(error));assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);count+=1;}}count
}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("usage: INPUT.bin NATIVE.bin RUST-PROJECTION.bin".into());}
 let _restore=Restore(csr());let errors=policies();let mut input=BufReader::new(File::open(&a[0])?);let mut native=BufReader::new(File::open(&a[1])?);let mut output=BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&a[2])?);let mut count=0;
 loop{let mut req=[0;32];if input.read(&mut req[..1])?==0{break;}input.read_exact(&mut req[1..])?;let mut reply=[0;24];native.read_exact(&mut reply)?;let cw=half(&req,0);
 if cw&0xf0ff!=0x7f||cw&0x300==0x100||half(&req,2)!=0||req[16..]!=[0;16]{return Err("invalid request policy".into());}
 if half(&reply,16)!=cw||half(&reply,18)&0x3800!=0||word(&reply,20)&!0x3f!=0x1f80{return Err("invalid native metadata".into());}
 let data=ResourceMetadata{flags:word(&req,4),frames_bits:word(&req,8),frame_interval_bits:word(&req,12)};let mut computed=None;
 for incoming in[0x1f80,0xdf81,0xbf84]{set_csr(incoming);let before=x87();let actual=prepare_metadata(data,context(cw))?.to_le_bytes();assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);if actual!=reply[..16]{return Err(format!("metadata case{count} differs").into());}if let Some(old)=computed{assert_eq!(actual,old);}computed=Some(actual);}
 output.write_all(&computed.unwrap())?;count+=1;
 }if count==0||native.read(&mut[0])?!=0{return Err("empty/trailing stream".into());}output.flush()?;println!("CELL_RUNTIME_CLIP_METADATA_VERIFY cases={count} caller_states=3 positive_policy_errors={errors} exact_bits=true constructor_default_loop_only=true");Ok(())
}
