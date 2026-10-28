//! Pending native comparison for full original bbox helper and bounded publisher.
use spore_core::{anim_selector_z_bounds as bounds,native_x87};
use std::{error::Error,fs::{File,OpenOptions},io::{Read,Write}};
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn csr()->u32{let mut v=0;unsafe{std::arch::asm!("stmxcsr [{p}]",p=in(reg)&mut v,options(nostack,preserves_flags));}v}
fn set(v:u32){unsafe{std::arch::asm!("ldmxcsr [{p}]",p=in(reg)&v,options(nostack,preserves_flags));}}
struct Restore(u32);impl Drop for Restore{fn drop(&mut self){set(self.0)}}
fn main()->Result<(),Box<dyn Error>>{
 let a:Vec<_>=std::env::args_os().skip(1).collect();if a.len()!=3{return Err("INPUT NATIVE52 NEW_OUTPUT44".into())}
 let _restore=Restore(csr());let mut input=File::open(&a[0])?;let mut native=File::open(&a[1])?;let mut output=OpenOptions::new().write(true).create_new(true).open(&a[2])?;let(mut groups,mut rows)=(0,0);
 loop{let mut h=[0;32];if input.read(&mut h[..1])?==0{break}input.read_exact(&mut h[1..])?;assert_eq!(word(&h,0),0x315a4242);let n=word(&h,4)as usize;let q=word(&h,8)as usize;assert!((1..=255).contains(&n)&&(1..=12).contains(&q));assert_eq!(h[16..],[0;16]);
 let mut blocks=Vec::new();for _ in 0..n{let mut b=[0;324];input.read_exact(&mut b)?;let f=|j|word(&b,168+j*4);blocks.push(bounds::BoundsBlock{center:std::array::from_fn(|j|f(7+j)),orientation:std::array::from_fn(|j|f(10+j)),half_extents:std::array::from_fn(|j|f(18+j))});}
 for _ in 0..q{let mut cb=[0;4];input.read_exact(&mut cb)?;let cw=word(&cb,0);assert_eq!(cw&!0xf00,0x7f);let context=native_x87::Context::from_control_word(cw as u16)?;let mut reply=[0;52];native.read_exact(&mut reply)?;assert_eq!(word(&reply,0),word(&h,12));assert_eq!(word(&reply,4),cw);assert_eq!(u16::from_le_bytes(reply[44..46].try_into()?)as u32,cw);assert_eq!(u16::from_le_bytes(reply[46..48].try_into()?)&0x3800,0);assert_eq!(word(&reply,48)&!0x3f,0x1f80);
 let mut projection=[0;44];projection[..4].copy_from_slice(&h[12..16]);projection[4..8].copy_from_slice(&cb);
 for incoming in[0x1f80,0xdf81,0xbf84]{set(incoming);let v=bounds::publish(&blocks,context).map_err(|e|format!("body{} cw{cw:x}: {e:?}",word(&h,12)))?;assert_eq!(csr(),incoming);projection[8..].copy_from_slice(&v.projection());assert_eq!(projection,reply[..44],"body{} cw{cw:x}",word(&h,12));
 assert!(matches!(bounds::publish(&[],context),Err(bounds::Error::Count(0))));let mut bad=blocks.clone();bad[0].center[0]=0x7fc00000;assert!(matches!(bounds::publish(&bad,context),Err(bounds::Error::Geometry(0))));assert_eq!(csr(),incoming);
 }output.write_all(&projection)?;rows+=1;
 }groups+=1;
 }assert_eq!(native.read(&mut[0])?,0);assert!(groups>0);output.flush()?;println!("Z_BOUNDS_VERIFY bodies={groups} rows={rows} CSR_states=3 actor_association=false");Ok(())
}
