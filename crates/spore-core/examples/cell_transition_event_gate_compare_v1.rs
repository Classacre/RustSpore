//! Compare event boundary and separately retained player rows; composition is modeled.
use spore_core::cell_transition_event_gate_v1::{dispatch,Outcome,PlayerSnapshot,EVENT_TYPE,BUTTON_ID};
use spore_core::cell_transition_player_gate_v1::PlayerFlags;
fn w(b:&[u8],i:usize)->u32{u32::from_le_bytes(b[i..i+4].try_into().unwrap())}
fn main(){let mut a=std::env::args().skip(1);let event=std::fs::read(a.next().expect("event output")).unwrap();let player=std::fs::read(a.next().expect("player output")).unwrap();assert_eq!(event.len(),128);assert_eq!(player.len(),128);
 for j in 0..4 { let fp=w(&player,j*32+16); assert_eq!(fp&0xffff,0x27f); assert_eq!((fp>>16)&0x3840,0); assert_eq!(w(&player,j*32+20)&!0x3f,0x1f80); }
 let cases=[(0,BUTTON_ID),(EVENT_TYPE,BUTTON_ID-1),(EVENT_TYPE,BUTTON_ID+1),(EVENT_TYPE,BUTTON_ID)];
 for (i,(typ,id)) in cases.into_iter().enumerate(){
  for j in 0..4{let flags=match j{0=>None,1=>Some(PlayerFlags{byte_112:0,byte_113:0}),2=>Some(PlayerFlags{byte_112:1,byte_113:0}),_=>Some(PlayerFlags{byte_112:0,byte_113:1})};
   let out=dispatch(typ,id,Some(PlayerSnapshot{root_5158_after_helper:0x12345678,flags_after_lookup:flags}),true).unwrap();let hit=matches!(out,Outcome::TransitionButton(_));
   assert_eq!([hit as u32,hit as u32],[w(&event,i*32),w(&event,i*32+4)],"event {i}");
   if let Outcome::TransitionButton(p)=out{assert_eq!([p.call_before as u32,p.lookup as u32,p.pending_after_call as u32,p.root_5158],[w(&player,j*32),w(&player,j*32+4),w(&player,j*32+8),w(&player,j*32+12)],"player {j}");}
  }
  let fp=w(&event,i*32+8);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&event,i*32+12)&!0x3f,0x1f80);
 }
 println!("matched 4 native event rows and 4 separately retained player rows; 16 joins are modeled, not native dispatch");
}
