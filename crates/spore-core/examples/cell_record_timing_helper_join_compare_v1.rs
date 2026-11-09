//! Compare explicitly joined models with retained contiguous native fixture.
use spore_core::cell_record_timing_helper_join_v1::{join,Outcome};
use spore_core::cell_record_update_helper_v1::Actor;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),280);
 for i in 0..7{let actor=if i==0{None}else{Some(Actor{field_248:if i>=5{0x12345678}else{0},field_fc:u32::from(i==5)})};let flag=u8::from(i==2);let present=i!=1;let aux=if i==5{Some(0)}else{None};let cb=if matches!(i,3|4|5|6){Some(u8::from(i==4))}else{None};let x=join(0x3f800000,0x3e800000,0x1f80,actor,flag,present,aux,cb,true).unwrap();
 let Outcome::Returned{remaining_before_helper,flag_after_helper,callback_called,pending_event_path,pending_other_path}=x else{panic!("unexpected alternate")};
 // Retained callback and downstream stubs do not alter record+1C after subtraction.
 let got=[1,u32::from(i==5),callback_called as u32,pending_other_path as u32,flag_after_helper as u32,remaining_before_helper,pending_event_path as u32];let want:Vec<_>=(0..7).map(|j|w(&b,i*40+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let fp=w(&b,i*40+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*40+36)&!0x3f,0x1f80);}
 println!("matched 7 retained timing/helper joined rows; live scheduler unobserved");}
