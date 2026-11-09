//! Compare bounded callback dispatch against twelve retained source rows.
use spore_core::cell_stream_callback_dispatch_v1::{project,Branch};
fn w(b:&[u8],o:usize)->u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main(){
 let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),384);
 for i in 0..12 {let c=i%6; let x=project(c<4,if c<4 {Some((c&1) as u8)} else {None},Some(if c<4 {(c>>1) as u8} else {(c-4) as u8}),true).unwrap();
 let (ret,trap)=match x.branch {Branch::ClosePending=>(0,1),Branch::FailurePending=>(0,2),Branch::Returned(al)=>(al as u32,0)};let r=&b[i*32..(i+1)*32];
 // Trap stubs return zero; real downstream return values are not modeled.
 assert_eq!((w(r,0),w(r,4),w(r,8),w(r,12)),(ret,trap,x.calls_6c,x.calls_64));let fp=w(r,16);assert_eq!(fp&0xffff,if i>=6{0x37f}else{0x27f});assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,20)&!0x3f,0x1f80);assert_eq!((w(r,24),w(r,28)),(0xc0dec0de,0xa5a5a5a5)); }
 println!("matched twelve callback-dispatch source rows; downstream operations trapped");
}
