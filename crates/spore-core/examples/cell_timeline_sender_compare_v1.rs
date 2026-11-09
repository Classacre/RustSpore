//! Compare source suffix projection with trapped native sender rows.
use spore_core::cell_timeline_sender_v1::{sender,HelperResult,Route};
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),192);
 for i in 0..6 {let equal=i==0;let busy=u8::from(i==1);let flag=if i==2 {0}else{1};let h=if i>=3 {Some(HelperResult{accepted:i!=3,flag_after:if i==5 {127}else{flag}})} else {None};
  let x=sender(equal,busy,flag,h,true).unwrap();let route=match x.route{Route::EqualObjectCut=>1,Route::BusyCut=>2,Route::NoSend=>3,Route::PendingSend=>4};
  // In the retained host, the hide/notify stubs are admitted not to mutate this flag.
  let sequence=if x.pending_send{4}else if x.helper_called{1}else{0};let got=[route,x.helper_called as u32,x.pending_hide as u32,x.pending_send as u32,x.flag_at_cut as u32,sequence];let want:Vec<_>=(0..6).map(|j|w(&b,i*32+j*4)).collect();assert_eq!(got.as_slice(),want,"case {i}");let fp=w(&b,i*32+24);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*32+28)&!0x3f,0x1f80);
 }
 println!("matched 6 retained sender cases; message delivery remains pending");}
