//! Compare the checked projection against retained guarded native rows only.
use spore_core::cell_mode_manager_prefix_v1::{project,PREVIOUS_MODE};
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let p=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(p).unwrap();assert_eq!(b.len(),216);
 let cases=[(0x01654c01,0x01654c01,0x01654c02,0x44),(0x01654c01,0x01654c00,0x01654c03,0x7f),(PREVIOUS_MODE,0x01654c01,0x01654c01,0x22),(PREVIOUS_MODE,0x01654c00,0x01654c01,0xa5),(0x01654c00,0x01654c01,0x01654c03,0),(0x12345678,0x01654c01,0x01654c03,0xff)];
 for (i,(req,old,previous,state)) in cases.into_iter().enumerate(){let effective=if req==PREVIOUS_MODE{previous}else{req};let changed=effective!=old;let p=project(req,old,previous,0x5a,changed.then_some(state),true).unwrap();let (a,z,restore)=p.changed.map(|x|(x.old,x.requested,x.restored_argument)).unwrap_or((0,0,0));
 // Notification and call count are observations of controlled inert stubs.
 let got=[p.return_al as u32,p.manager_28 as u32,a,z,changed as u32,if changed{9}else{0},restore];for j in 0..7{assert_eq!(got[j],w(&b,i*36+j*4),"case {i} word {j}");}let fp=w(&b,i*36+28);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,i*36+32)&!0x3f,0x1f80);
 }println!("matched six retained manager rows; callbacks and completion pending");}
