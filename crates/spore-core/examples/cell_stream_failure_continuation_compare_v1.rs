//! Compare bounded failure continuation with twelve retained source rows.
use spore_core::cell_stream_failure_continuation_v1::{project,Branch};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),384);
 for i in 0..12 {let c=i%6;let (flag,field,al)=match c{0=>(0,0,None),1=>(1,0,None),2=>(1,2,Some(0)),3=>(1,2,Some(1)),4=>(0,2,None),_=>(1,2,Some(1))};let x=project(flag,field,al,true).unwrap();let trap=match x.branch{Branch::ClosePending=>1,Branch::SuccessPending=>2};let r=&b[i*32..(i+1)*32];
 // Retained callback scripts preserve fields after the source reads them; trap AL is synthetic.
 assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12)),(0,trap,x.calls_6c,x.calls_other));let fp=w(r,16);assert_eq!(fp&0xffff,if i>=6{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,20)&!0x3f,0x1f80);assert_eq!((w(r,24),w(r,28)),((field<<8)|u32::from(flag),0xa5a5a5a5));}
 println!("matched twelve failure-continuation rows; close and +90 trapped");}
