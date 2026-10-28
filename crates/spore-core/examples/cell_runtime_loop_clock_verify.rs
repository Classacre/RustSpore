//! Compare the explicit arithmetic boundary; this does not admit loop state.
use spore_core::{cell_runtime_loop_clock::{evaluate_loop_arithmetic,LoopArithmeticInput,LoopArithmeticContext,
    LoopStateBoundary,ModfDispatch,LoopArithmeticError},native_sse::Policy,native_x87::{Context,Extended}};
use std::{error::Error,fs::{File,OpenOptions},io::{Read,Write}};
fn word(raw:&[u8],at:usize)->u32{u32::from_le_bytes(raw[at..at+4].try_into().unwrap())}
fn wide(raw:&[u8],at:usize)->u64{u64::from_le_bytes(raw[at..at+8].try_into().unwrap())}
fn half(raw:&[u8],at:usize)->u16{u16::from_le_bytes(raw[at..at+2].try_into().unwrap())}
fn csr()->u32{let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v}
fn set_csr(v:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}}
fn x87()->(u16,u16){let(mut c,mut s)=(0,0);unsafe{std::arch::asm!("fnstcw [{c}]","fnstsw [{s}]",c=in(reg)&mut c,s=in(reg)&mut s,options(nostack,preserves_flags));}(c,s)}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set_csr(self.0);}}
fn context(cw:u16)->LoopArithmeticContext{LoopArithmeticContext{x87:Context::from_control_word(cw).unwrap(),sse:Policy::Mxcsr1f80,modf_dispatch:ModfDispatch::ExplicitFiniteSse2}}
fn policy_checks()->usize{
    let input=LoopArithmeticInput{time_bits:0x3fa00000,duration_bits:1f64.to_bits(),resource_frames_bits:0x43050000,state_boundary:LoopStateBoundary::CallerSuppliedArithmeticSnapshot};
    let variants=[(LoopArithmeticInput{state_boundary:LoopStateBoundary::Unestablished,..input},LoopArithmeticError::UnestablishedStateBoundary),
        (LoopArithmeticInput{time_bits:0xbe800000,..input},LoopArithmeticError::NegativeTime),
        (LoopArithmeticInput{duration_bits:0,..input},LoopArithmeticError::NonPositiveDuration),
        (LoopArithmeticInput{time_bits:0x7fc12345,..input},LoopArithmeticError::NonFiniteInput),
        (LoopArithmeticInput{resource_frames_bits:0,..input},LoopArithmeticError::UnsupportedFrameCount),
        (LoopArithmeticInput{time_bits:0x7f7fffff,duration_bits:1,..input},LoopArithmeticError::NonFiniteQuotient)];
    let mut count=0;
    for incoming in[0x1f80,0xdf81,0xbf84]{set_csr(incoming);
        for(value,error)in variants{let before=x87();assert_eq!(evaluate_loop_arithmetic(value,context(0x027f)),Err(error));
            assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);count+=1;}
        assert_eq!(evaluate_loop_arithmetic(input,LoopArithmeticContext{modf_dispatch:ModfDispatch::Unestablished,..context(0x027f)}),Err(LoopArithmeticError::UnestablishedCrtDispatch));count+=1;
    }count
}
fn main()->Result<(),Box<dyn Error>>{
    let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=3{return Err("usage: INPUT.bin NATIVE.bin RUST56.bin".into());}
    let _restore=Restore(csr());let policies=policy_checks();let mut inputs=File::open(&args[0])?;let mut native=File::open(&args[1])?;
    let mut output=OpenOptions::new().write(true).create_new(true).open(&args[2])?;let(mut cases,mut retained_differences)=(0,0);
    loop{let mut req=[0;64];if inputs.read(&mut req[..1])?==0{break;}inputs.read_exact(&mut req[1..])?;let mut reply=[0;64];native.read_exact(&mut reply)?;
        let cw=half(&req,4);assert_eq!(word(&req,0),0);assert_eq!(half(&req,6),0);assert_eq!(word(&req,32),1);assert_eq!(word(&req,36),1);assert_eq!(word(&req,40),1);assert_eq!(req[44..],[0;20]);
        assert_eq!(half(&reply,56),cw);assert_eq!(half(&reply,58)&0x3800,0);assert_eq!(word(&reply,60)&!0x3f,0x1f80);
        assert_eq!(half(&reply,18),0);assert_eq!(half(&reply,46),0);assert_eq!(word(&reply,20),word(&reply,24));
        let committed=Extended::from_f32(f32::from_bits(word(&reply,20)))?.to_le_bytes();retained_differences+=usize::from(committed!=reply[8..18]);
        let input=LoopArithmeticInput{time_bits:word(&req,8),duration_bits:wide(&req,20),resource_frames_bits:word(&req,28),state_boundary:LoopStateBoundary::CallerSuppliedArithmeticSnapshot};
        let mut computed=None;
        for incoming in[0x1f80,0xdf81,0xbf84]{set_csr(incoming);let before=x87();let actual=evaluate_loop_arithmetic(input,context(cw))?.to_le_bytes();
            assert_eq!(csr(),incoming);assert_eq!(x87().0,before.0);assert_eq!(x87().1&0x3800,before.1&0x3800);
            if actual!=reply[..56]{let byte=actual.iter().zip(reply).position(|(a,b)|*a!=b).unwrap();return Err(format!("loop case{cases} differs byte{byte}: Rust{:02X}/native{:02X}",actual[byte],reply[byte]).into());}
            if let Some(old)=computed{assert_eq!(old,actual);}computed=Some(actual);
        }output.write_all(&computed.unwrap())?;cases+=1;
    }
    assert!(cases>0&&retained_differences>0);assert_eq!(native.read(&mut[0])?,0);output.flush()?;
    println!("CELL_RUNTIME_LOOP_CLOCK_VERIFY cases={cases} modeled_bytes=56 incoming_states=3 positive_policy_errors={policies} retained_quotient_differs_from_commit={retained_differences} arithmetic_boundary_only=true");Ok(())
}
