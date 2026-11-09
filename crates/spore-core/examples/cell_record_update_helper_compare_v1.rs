//! Compare bounded helper routes with retained source-instruction cases.
use spore_core::cell_record_update_helper_v1::{helper,Actor,Route};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),320);
 for i in 0..8{let actor=if i==0{None}else{Some(Actor{field_248:if i>=5{0x12345678}else{0},field_fc:if i==5||i==6{1}else{0}})};let flag=u8::from(i==2);let present=i!=1;let aux=if i==5||i==6{Some(u32::from(i==6))}else{None};let cb=if matches!(i,3|4|5|7){Some(u8::from(i==4))}else{None};let x=helper(actor,flag,present,aux,cb,true).unwrap();let (alt,called,after,al)=match x{Route::Returned{al,flag_after,callback_called}=>(0,callback_called as u32,flag_after as u32,al as u32),Route::AlternatePending=>(1,0,flag as u32,0x7e)};
 // 0x7E is the retained host's synthetic trap marker, not a source return.
 // Its synthetic AltLookup leaves the record flag unchanged before that trap.
 let got=[1,u32::from(i==5||i==6),called,alt,after,al];let want:Vec<_>=(0..6).map(|j|w(&b,i*40+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let fp=w(&b,i*40+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*40+36)&!0x3f,0x1f80);}
 println!("matched 8 retained helper routes; alternate and real callbacks remain unobserved");}
