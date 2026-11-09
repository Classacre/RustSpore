//! Compare a synthetic two-row E63510 handoff with retained original rows.
use spore_core::cell_effect_curve_v1::{evaluate,Curve};
use spore_core::native_x87::Context;
fn word(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){
 let b=std::fs::read(std::env::args().nth(1).expect("retained source output")).unwrap();assert_eq!(b.len(),384);
 let queries=[0,0x3fcccccd,0x461c4000,0x46c35000];
 for mode in 0..8 {let cw=if mode<4 {0x27f} else {0x37f};let ctx=Context::from_control_word(cw).unwrap();
  for (i,(id,level,curve)) in [(0x5a314ebd,3,Curve::Near),(0x5a314eb9,11,Curve::Far)].into_iter().enumerate(){
   let o=mode*24+i*12;assert_eq!(word(&b,o),id);assert_eq!(word(&b,o+4),evaluate(curve,f32::from_bits(queries[mode%4]),ctx).unwrap().to_bits());assert_eq!(word(&b,o+8),level);
  }
  let fp=word(&b,192+mode*8);assert_eq!(fp&0xffff,u32::from(cw));assert_eq!((fp>>16)&0x3840,0);assert_eq!(word(&b,196+mode*8)&!0x3f,0x1f80);
 }
 assert!(b[256..].iter().all(|&x|x==0));println!("matched 16 original row handoffs; synthetic array and scalar, live effect and placement excluded");
}
