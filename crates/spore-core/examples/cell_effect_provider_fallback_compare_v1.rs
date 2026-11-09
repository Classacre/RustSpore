//! Compare controlled provider fallback to retained source rows; no host execution.
use spore_core::cell_effect_provider_fallback_v1::{project,Callback};
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("retained source output")).unwrap();assert_eq!(b.len(),160);
 let scripts=[(1,0,0x11111111,0x22222222),(0,1,0x11111111,0x22222222),(2,0,0x11111111,0x22222222),(1,0,0,0x22222222)];
 for (i,(a,z,p,f)) in scripts.into_iter().enumerate(){let x=project(0x5a314ebd,0xa5a5a5a5,Callback{al:a,slot_after:p},Some(Callback{al:z,slot_after:f}),true).unwrap();let second=x.fallback;let o=i*40;
  assert_eq!([w(&b,o),w(&b,o+4),w(&b,o+8),w(&b,o+12),w(&b,o+16),w(&b,o+20),w(&b,o+24),w(&b,o+28)],[1+u32::from(second.is_some()),x.primary.id,second.map_or(0,|c|c.id),x.primary.slot_before,second.map_or(0,|c|c.slot_before),x.primary.slot_after,second.map_or(0,|c|c.slot_after),x.slot_after]);let fp=w(&b,o+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+36)&!0x3f,0x1f80);
 }println!("matched four original fallback handoffs; callbacks and ownership supplied");}
