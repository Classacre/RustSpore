//! Compare checked pending ending gate with caller-supplied retained native output.
use spore_core::cell_ending_gate_v1::{gate,Action};
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),27*28);
 for i in 0..27{let progress=999+(i/9)as i32;let first=((i%9)/3)as u32;let second=(i%3)as u32;let x=gate(progress,Some(first),Some(second),true).unwrap();let got=[x.resolve_globals as u32,x.resolve_globals as u32,x.resolve_globals as u32,x.fresh_second_read as u32,(x.action==Action::PendingEndingCall)as u32];let want:Vec<_>=(0..5).map(|j|word(&b,i*28+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let f=word(&b,i*28+20);assert_eq!(f&0xffff,0x27f);assert_eq!((f>>16)&0x3840,0);assert_eq!(word(&b,i*28+24)&!0x3f,0x1f80);}
 println!("matched 27 retained pending ending-gate rows; no ending executed");}
