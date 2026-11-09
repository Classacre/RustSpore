//! Compare bounded progress/identity prefix to retained source-instruction rows.
use spore_core::cell_progress_callback_prefix_v1::{apply,Outcome};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),192);
 let initial=[20,20,20,850,0xfffffff0,0x7fffffff];for i in 0..6{let current=if i==0{0x33333333}else{0x22222222};let flag=if i==1{0}else if i==3{2}else{1};let x=apply(current,0x22222222,Some(flag),Some(0x11111111),Some(initial[i]),true).unwrap();let (route,id,new,old,fresh)=match x{Outcome::ActorMismatch=>(1,current,initial[i],0,0),Outcome::SaveFlagZero=>(2,current,initial[i],0,0),Outcome::Applied{player_id,old_progress,new_progress}=>(3,player_id,new_progress,old_progress,new_progress)};let got=[route,id,new,old,fresh];let want:Vec<_>=(0..5).map(|j|w(&b,i*32+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let fp=w(&b,i*32+20);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*32+24)&!0x3f,0x1f80);}
 println!("matched 6 retained progress callback prefixes; downstream effects unobserved");}
