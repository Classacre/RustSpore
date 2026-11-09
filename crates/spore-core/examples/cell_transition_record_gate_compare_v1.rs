//! Compare E72CD0 pre-allocation gate with controlled retained native output.
use spore_core::cell_transition_record_gate_v1::{gate,PlayerFlags};
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),256);
 for i in 0..8{let player=if i==0{None}else{Some(PlayerFlags{byte_112:if i==1{1}else if i==5{2}else{0},byte_113:if i==2{1}else if i==5{2}else{0}})};let mode=if i==3||i==7{3}else{2};let x=gate(player,mode,0x5a,true).unwrap();let got=[1,x.pending_effect_and_allocation as u32,x.root_5169 as u32,mode];let want:Vec<_>=(0..4).map(|j|w(&b,i*32+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let fp=w(&b,i*32+16);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*32+20)&!0x3f,0x1f80);}
 println!("matched 8 retained pre-allocation gate cases; no actor or record observed");}
