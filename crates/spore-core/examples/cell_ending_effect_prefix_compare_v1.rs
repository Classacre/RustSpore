use spore_core::{cell_ending_gate_v1::Action,cell_ending_effect_prefix_v1::{self as gate,Actor,Pending}};
use std::{env,fs};
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let p=env::args().nth(1).expect("native stdout");let b=fs::read(p).unwrap();assert_eq!(b.len(),20*32);
 for i in 0..20{let e0=(i/10)as u32;let flag=((i%10)/5)as u8;let mode=i%5;let actor=Actor{token:1,byte_112:(mode==2)as u8,byte_113:(mode==3)as u8,byte_188:(mode==4)as u8};let lookup=if e0==0&&flag==0{None}else{Some(if mode==0{None}else{Some(actor)})};let x=gate::project(Action::PendingEndingCall,e0,flag,lookup,true).unwrap();
  let got=[1,1,1,x.lookup_player as u32,(x.pending==Pending::Effect) as u32,matches!(x.pending,Pending::EligiblePlayer(_)) as u32];for j in 0..6{assert_eq!(got[j],word(&b,i*32+j*4),"case {i} word {j}")}let status=word(&b,i*32+24);assert_eq!(status&0xffff,0x27f);assert_eq!((status>>16)&0x3840,0);assert_eq!(word(&b,i*32+28)&!0x3f,0x1f80);
 }println!("matched 20 retained source gate rows; pending branches only");}
