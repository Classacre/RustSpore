//! Pure bounded projection of Window listener-mask reduction at 9614F0.
//! Caller supplies a fresh finite ordered mask snapshot after registration.
//! No pointer, list lifetime, event or owner callback is modeled.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Reduced { pub byte_31:u8, pub owner_callback_pending:bool }

/// Reduce a caller-supplied list of mask words. The pending callback is only
/// the conditional call at 96154E, not its effects or subsequent state.
pub fn reduce(initial_31:u8, word_2c:u32, owner_present:bool, masks:&[u32])->Reduced {
 let mut byte=initial_31 & !0x06;
 let mut bit0=false;
 for m in masks {bit0|=m&2!=0;if m&0x100!=0{byte|=2}if m&0x20!=0{byte|=4}}
 let changed=(byte&1!=0)!=bit0;
 byte=(byte&!1)|u8::from(bit0);
 Reduced {byte_31:byte,owner_callback_pending:changed&&word_2c&8==0&&owner_present}
}
#[cfg(test)] mod tests{
 use super::*;
 #[test] fn proportional_mask_sets_the_child_relayout_bit(){
  let r=reduce(0xf0,3,false,&[0x108]);assert_eq!(r.byte_31,0xf2);assert!(!r.owner_callback_pending);
  assert!(crate::spui_layout_propagation_v1::child_dispatch(3,r.byte_31));
 }
 #[test] fn recomputes_masks_and_exposes_conditional_callback(){
  let r=reduce(0xff,3,true,&[2]);assert_eq!(r.byte_31,0xf9);assert!(!r.owner_callback_pending);
  let r=reduce(0xf1,3,true,&[]);assert_eq!(r.byte_31,0xf0);assert!(r.owner_callback_pending);
  assert!(!reduce(0xf1,11,true,&[]).owner_callback_pending);
 }
}
