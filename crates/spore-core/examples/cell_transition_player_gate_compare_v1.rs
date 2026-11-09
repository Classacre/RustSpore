//! Compare bounded model with retained, independently reviewed native output.
use spore_core::cell_transition_player_gate_v1::{gate,PlayerFlags};
fn word(b:&[u8], o:usize)->u32 { u32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn main() {
 let path=std::env::args().nth(1).expect("native stdout");
 let b=std::fs::read(path).expect("read native stdout"); assert_eq!(b.len(),128);
 let cases=[None,Some(PlayerFlags{byte_112:0,byte_113:0}),Some(PlayerFlags{byte_112:1,byte_113:0}),Some(PlayerFlags{byte_112:0,byte_113:1})];
 for (i,case) in cases.into_iter().enumerate() {
  let out=gate(0x12345678,case,true).unwrap();
  let got=[out.call_before as u32,out.lookup as u32,out.pending_after_call as u32,out.root_5158];
  let want:Vec<_>=(0..4).map(|j|word(&b,i*32+j*4)).collect();
  assert_eq!(got.as_slice(),want,"case {i}"); let fp=word(&b,i*32+16);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(word(&b,i*32+20)&!0x3f,0x1f80);
 }
 println!("matched 4 retained controlled player-gate cases; no service or transition executed");
}
