//! Compare synthetic E80D75 branch projection against seven retained native rows.
use spore_core::cell_save_ownership_projection_v1::{project,Action};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let path=std::env::args().nth(1).expect("native stdout");let b=std::fs::read(path).unwrap();assert_eq!(b.len(),224);
let cases=[(Some(1),Some(1),None),(Some(1),Some(2),None),(None,Some(2),None),(Some(1),None,Some(Some(1))),(Some(1),None,Some(Some(2))),(Some(1),None,Some(None)),(None,None,Some(None))];
for (i,(old,pending,fresh)) in cases.into_iter().enumerate(){let r=&b[i*32..(i+1)*32];let x=project(old,pending,fresh,0x5a,0x27f,0x1f80,true).unwrap();let retain=x.actions.iter().any(|a|matches!(a,Action::Retain(_)));let release=x.actions.iter().any(|a|matches!(a,Action::Release(_)));let seq=match(retain,release){(false,false)=>0,(true,false)=>1,(false,true)=>2,(true,true)=>3};assert_eq!(w(r,0),x.root_5190.unwrap_or(0));assert_eq!(w(r,4),u32::from(x.root_51da));assert_eq!(w(r,8),seq);assert_eq!(w(r,12),u32::from(x.actions.contains(&Action::FreshLookup)));let fp=w(r,16);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(r,20)&!0x3f,0x1f80);assert_eq!(w(r,24),i as u32);assert_eq!(w(r,28),0xa5a5a5a5);}
println!("matched seven synthetic retain/store rows; callbacks and fresh helper scripted");}
