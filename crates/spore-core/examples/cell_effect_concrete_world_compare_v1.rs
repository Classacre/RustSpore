//! Compare controlled negative lookups with retained original source rows.
//! The fixture lookup stub preserves world +20 and the slot on both calls.
use spore_core::cell_effect_concrete_world_v1::project;
fn w(b:&[u8],o:usize)->u32{u32::from_le_bytes(b[o..o+4].try_into().unwrap())}
fn main(){let b=std::fs::read(std::env::args().nth(1).expect("native rows")).unwrap();assert_eq!(b.len(),160);
 for i in 0..4 {let x=project(0x5a314ebd,0xa5a5a5a5,if i%2==0{0}else{5},[-1,-1],true).unwrap();let hit=x.lookup_ids[0].is_some();let o=i*40;
 assert_eq!([w(&b,o),w(&b,o+4),w(&b,o+8),w(&b,o+12),w(&b,o+16),w(&b,o+20),w(&b,o+24),w(&b,o+28)],[if hit{2}else{0},x.lookup_ids[0].unwrap_or(0),x.lookup_ids[1].unwrap_or(0),if hit{0xa5a5a5a5}else{0},if hit{0xa5a5a5a5}else{0},if hit{0xa5a5a5a5}else{0},if hit{0xa5a5a5a5}else{0},x.slot_after]);let fp=w(&b,o+32);assert_eq!(fp&0xffff,0x27f);assert_eq!((fp>>16)&0x3840,0);assert_eq!(w(&b,o+36)&!0x3f,0x1f80);}
 println!("matched four concrete-world negative-lookup handoffs; provider state synthetic");}
