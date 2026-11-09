use spore_core::creature_skin_corner_lookup_v1::{corner_hash,lookup,SLOT_COUNT};
use std::io::Write;
fn main() {
 let mut out=Vec::new();
 for case in 0..5 {
  let key=0x00123456u32; let mut table=vec![u32::MAX;SLOT_COUNT];
  let first=(corner_hash(key)&0x7fff) as usize;
  match case {1=>table[first]=key,2|3=>{table[first]=0x01123456;if case==3 {let second=lookup(&table,key).unwrap() as usize;table[second]=key;}},4=>table.fill(0),_=>{}}
  out.extend_from_slice(&lookup(&table,key).unwrap().to_le_bytes());
 }
 std::io::stdout().write_all(&out).unwrap();
}
