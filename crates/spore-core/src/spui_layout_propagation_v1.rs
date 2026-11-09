//! Conditional 960B26..960BAF UI span gate and single-child dispatch decision.
//!
//! The caller must execute/admit 95A0F0, 9589D0 and 9581F0 in source order
//! before supplying the refreshed rectangle. A `Relayout` result stops before
//! 958930 and before any child-list read. After that external call the caller
//! must refresh the list/head and feed one live child at a time; after a child
//! callback it must refresh the next node and flags. This does not own pointers,
//! execute callbacks, model list mutation or establish source runtime flags.
use crate::native_sse::{self,Policy,SseError};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Gate { Stable, Relayout { height_compared: bool } }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { NonFinite, Sse(SseError) }
impl From<SseError> for Error {fn from(e:SseError)->Self{Self::Sse(e)}}

/// Snapshots are [left, top, right, bottom] in source order. This checked
/// model rejects nonfinite reached operands/results; native unordered branches
/// to relayout. The old rectangle was already snapshotted before callbacks;
/// a width difference stops before the new-height read/comparison.
/// Scoped MXCSR masks exceptions, preserves subnormals, and restores caller
/// state; native sticky flags and callback side effects are not reproduced.
pub fn compare_spans(old:[f32;4],refreshed:[f32;4],policy:Policy)->Result<Gate,Error>{
 native_sse::scoped(policy,||{
  if !old[0].is_finite()||!old[2].is_finite()||!refreshed[0].is_finite()||!refreshed[2].is_finite(){return Err(Error::NonFinite)}
  let old_width=std::hint::black_box(old[2])-std::hint::black_box(old[0]);
  let new_width=std::hint::black_box(refreshed[2])-std::hint::black_box(refreshed[0]);
  if !old_width.is_finite()||!new_width.is_finite(){return Err(Error::NonFinite)}
  if old_width!=new_width{return Ok(Gate::Relayout{height_compared:false})}
  if !old[1].is_finite()||!old[3].is_finite()||!refreshed[1].is_finite()||!refreshed[3].is_finite(){return Err(Error::NonFinite)}
  let old_height=std::hint::black_box(old[3])-std::hint::black_box(old[1]);
  let new_height=std::hint::black_box(refreshed[3])-std::hint::black_box(refreshed[1]);
  if !old_height.is_finite()||!new_height.is_finite(){return Err(Error::NonFinite)}
  if old_height!=new_height{Ok(Gate::Relayout{height_compared:true})}else{Ok(Gate::Stable)}
 })
}

/// One freshly read child object. For each reached list entry, source dispatches
/// its interface +94 if object +2C bit 0x800 or byte +31 bit 0x02 is set.
/// Caller is responsible for valid pointers, list order, root/head comparison,
/// post-958930 and post-callback mutation, and callback side effects.
pub fn child_dispatch(word_2c:u32,byte_31:u8)->bool{word_2c&0x800!=0||byte_31&2!=0}

#[cfg(test)] mod tests {
 use super::*;
 fn cmp(a:[f32;4],b:[f32;4])->Result<Gate,Error>{compare_spans(a,b,Policy::Mxcsr1f80)}
 #[test] fn translation_does_not_trigger_but_span_changes_do(){
  let a=[1.,2.,11.,22.];assert_eq!(cmp(a,[4.,5.,14.,25.]),Ok(Gate::Stable));
  assert_eq!(cmp(a,[1.,2.,12.,22.]),Ok(Gate::Relayout{height_compared:false}));
  assert_eq!(cmp(a,[1.,2.,11.,23.]),Ok(Gate::Relayout{height_compared:true}));
 }
 #[test] fn unreached_height_and_invalid_reached_arithmetic(){
  let a=[0.,f32::NAN,1.,0.];assert_eq!(cmp(a,[0.,0.,2.,0.]),Ok(Gate::Relayout{height_compared:false}));
  assert_eq!(cmp(a,[0.,0.,1.,0.]),Err(Error::NonFinite));
  assert_eq!(cmp([-f32::MAX,0.,f32::MAX,1.],[0.,0.,1.,1.]),Err(Error::NonFinite));
 }
 #[test] fn signed_zero_and_child_mask(){
  assert_eq!(cmp([-0.,0.,0.,1.],[0.,0.,0.,1.]),Ok(Gate::Stable));
  assert_eq!(cmp([0.,0.,f32::from_bits(1),1.],[0.,0.,0.,1.]),Ok(Gate::Relayout{height_compared:false}));
  assert!(!child_dispatch(0x400,1));assert!(child_dispatch(0x800,0));assert!(child_dispatch(0,2));assert!(child_dispatch(0x800,2));
 }
}
