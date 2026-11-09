//! Bounded 6A0E00→6A0B30→6B5100 array-prefix handoff.
//! Supplied descriptor and bytes are stable, callback/receiver return normally;
//! no locale service, caching, localization or displayed UI is modeled.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Outcome { pub accepted: bool, pub receiver_words: Option<[u32;2]> }

/// Only the pointer-backed descriptor form (`flags & 0x30 != 0`) is admitted.
/// On callback failure or non-0x22 type, the source does not read the data.
/// The scripted receiver return value is ignored by the original accessor.
pub fn project(callback_success:bool,type_word:u16,flags:u16,data:&[u8])->Result<Outcome,&'static str>{
 if !callback_success || type_word!=0x22 {return Ok(Outcome{accepted:false,receiver_words:None});}
 if flags&0x30==0 {return Err("inline descriptor representation not admitted");}
 let prefix=data.get(..8).ok_or("prefix shorter than 8 bytes")?;
 Ok(Outcome{accepted:true,receiver_words:Some([
  u32::from_le_bytes(prefix[..4].try_into().unwrap()),
  u32::from_le_bytes(prefix[4..8].try_into().unwrap())
 ])})
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn exact_prefix_and_short_circuits(){
  let b=[0x45,0x17,0x67,0x21,0x12,0,2,0];
  assert_eq!(project(true,0x22,0x9c,&b).unwrap().receiver_words,Some([0x21671745,0x20012]));
  assert!(!project(true,0x21,0x9c,&[]).unwrap().accepted);
  assert!(!project(false,0x22,0x9c,&[]).unwrap().accepted);
  assert!(project(true,0x22,0x9c,&b[..7]).is_err());
  assert!(project(true,0x22,0,&b).is_err());
 }
}
