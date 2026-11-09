//! Controlled seed-selector comparison; input is caller-supplied, not live state.
use spore_core::creature_skin_group_selector_v1::select;
use std::{env,fs,io::{self,Write}};
fn main(){
 let args:Vec<_>=env::args().collect();assert_eq!(args.len(),2);
 let bytes=fs::read(&args[1]).unwrap();assert_eq!(bytes.len(),12*344);
 let mut out=Vec::new();
 for row in bytes.chunks_exact(344){
  let words:Vec<u32>=row.chunks_exact(4).map(|w|u32::from_le_bytes(w.try_into().unwrap())).collect();
  let got=select(&words[2..],words[1],words[0]).unwrap().unwrap_or(84);out.extend(got.to_le_bytes());
 }
 io::stdout().write_all(&out).unwrap();
}
