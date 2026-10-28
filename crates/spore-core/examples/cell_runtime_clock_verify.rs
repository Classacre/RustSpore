//! Independent original clock comparison; root promotes after native admission.
use spore_core::{cell_runtime_clock::{evaluate_first_duration,ClockContext,ClockError,ClockInput,ClockTime},
    native_sse::Policy,native_x87::Context};
use std::{error::Error,fs::{File,OpenOptions},io::{BufReader,BufWriter,Read,Write}};
fn word(raw:&[u8],at:usize)->u32 {u32::from_le_bytes(raw[at..at+4].try_into().unwrap())}
fn wide(raw:&[u8],at:usize)->u64 {u64::from_le_bytes(raw[at..at+8].try_into().unwrap())}
fn half(raw:&[u8],at:usize)->u16 {u16::from_le_bytes(raw[at..at+2].try_into().unwrap())}
fn csr()->u32 {let mut x=0;unsafe {std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut x,options(nostack,preserves_flags));}x}
fn set_csr(x:u32) {unsafe {std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&x,options(nostack,preserves_flags));}}
fn x87()->(u16,u16) {let(mut c,mut s)=(0,0);unsafe {std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore {fn drop(&mut self){set_csr(self.0);}}
fn context(cw:u16)->ClockContext {ClockContext {x87:Context::from_control_word(cw).unwrap(),sse:Policy::Mxcsr1f80}}
fn policy_gate()->usize {
    let base=ClockInput {time:ClockTime::Absolute {time_bits:0x3f000000},duration_bits:1f64.to_bits(),resource_frames_bits:0x43050000,looping:false};
    let cases=[
        (ClockInput {looping:true,..base},ClockError::UnsupportedLoop),
        (ClockInput {time:ClockTime::Absolute {time_bits:0x7fc12345},..base},ClockError::NonFiniteInput),
        (ClockInput {duration_bits:0,..base},ClockError::NonPositiveDuration),
        (ClockInput {duration_bits:(-1f64).to_bits(),..base},ClockError::NonPositiveDuration),
        (ClockInput {resource_frames_bits:0,..base},ClockError::UnsupportedFrameCount),
        (ClockInput {time:ClockTime::Absolute {time_bits:0xbf000000},..base},ClockError::NegativeTime),
        (ClockInput {time:ClockTime::Absolute {time_bits:0x3f800000},..base},ClockError::DurationCrossing),
        (ClockInput {time:ClockTime::Advance {previous_time_bits:0x7ff8000000000000,delta_bits:0},..base},ClockError::NonFiniteInput),
        (ClockInput {time:ClockTime::Advance {previous_time_bits:1e300f64.to_bits(),delta_bits:0},..base},ClockError::NonFiniteResult),
    ];
    let mut checks=0;
    for incoming in [0x1f80,0xdf81,0xbf84] {
        set_csr(incoming);
        for(input,expected)in cases {let before=x87();assert_eq!(evaluate_first_duration(input,context(0x027f)),Err(expected));
            assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);checks+=1;}
    }checks
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=3 {return Err("usage: INPUT.bin NATIVE.bin RUST-PROJECTION.bin".into());}
    let _restore=Restore(csr());let policies=policy_gate();let mut input=BufReader::new(File::open(&args[0])?);
    let mut native=BufReader::new(File::open(&args[1])?);let mut output=BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&args[2])?);let mut cases=0;
    loop {let mut req=[0;64];if input.read(&mut req[..1])?==0 {break;}input.read_exact(&mut req[1..])?;
        let mut reply=[0;28];native.read_exact(&mut reply)?;let cw=half(&req,4);
        if word(&req,0)>1 || cw&0xf0ff!=0x7f || cw&0x300==0x100 || half(&req,6)!=0 || word(&req,32)!=0 || req[36..]!=[0;28] {
            return Err(format!("invalid request case{cases}").into());}
        if half(&reply,20)!=cw || half(&reply,22)&0x3800!=0 || word(&reply,24)&!0x3f!=0x1f80 {return Err(format!("invalid native metadata case{cases}").into());}
        let data=ClockInput {time:if word(&req,0)==0 {ClockTime::Absolute {time_bits:word(&req,8)}}else{
            ClockTime::Advance {previous_time_bits:wide(&req,12),delta_bits:word(&req,8)}},duration_bits:wide(&req,20),resource_frames_bits:word(&req,28),looping:false};
        let mut computed=None;
        for incoming in [0x1f80,0xdf81,0xbf84] {set_csr(incoming);let before=x87();let actual=evaluate_first_duration(data,context(cw))?.to_le_bytes();
            assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);
            if actual!=reply[..20] {let byte=actual.iter().zip(reply).position(|(a,b)|*a!=b).unwrap();return Err(format!("clock case{cases} differs modeled byte{byte}: Rust{:02X}/native{:02X}",actual[byte],reply[byte]).into());}
            if let Some(old)=computed {assert_eq!(actual,old);}computed=Some(actual);
        }output.write_all(&computed.unwrap())?;cases+=1;
    }
    if cases==0 || native.read(&mut[0])?!=0 {return Err("empty input or trailing native bytes".into());}output.flush()?;
    println!("CELL_RUNTIME_CLOCK_VERIFY cases={cases} caller_states=3 positive_policy_errors={policies} exact_bits=true first_duration_nonloop_only=true");Ok(())
}
