//! Finite leading class-5 node prefix, not the complete grid or creature.
use crate::{creature_generated_skin_edges_v1::{ROW_SIZE, MAX_ROWS}, creature_generated_skin_kernel_v1::Lane, native_sse::{self,Policy,SseError}};

#[derive(Clone, Copy, Debug)]
pub struct FrozenRowsAdmission;
#[derive(Debug, PartialEq, Eq)]
pub enum Error { NoFlaggedRow, TooManyRows, NonFinite, AllocationFailed, UnsupportedArchitecture }
impl From<SseError> for Error { fn from(_: SseError) -> Self { Self::UnsupportedArchitecture } }
#[derive(Debug, PartialEq)]
pub struct Prefix { pub lanes: Vec<Lane>, pub lower: [f32;3], pub upper: [f32;3] }
fn word(row: &[u8; ROW_SIZE], offset: usize) -> f32 { f32::from_le_bytes(row[offset..offset+4].try_into().unwrap()) }
/// Requires stable prepared row bytes, successful original builder and storage,
/// and masked x87 exceptions with adequate stack. Only the leading class-5
/// nodes are represented; segment and class-3 lanes are deliberately absent.
pub fn build(rows: &[[u8; ROW_SIZE]], policy: Policy, _: FrozenRowsAdmission) -> Result<Prefix, Error> {
 if rows.len()>MAX_ROWS { return Err(Error::TooManyRows); }
 if !rows.iter().any(|r| u16::from_le_bytes([r[8],r[9]]) & 1 != 0) { return Err(Error::NoFlaggedRow); }
 native_sse::scoped(policy, || {
  let mut out=Prefix { lanes: Vec::new(), lower:[f32::MAX;3], upper:[-f32::MAX;3] };
  out.lanes.try_reserve_exact(rows.iter().take_while(|r|r[11]==5).count()).map_err(|_| Error::AllocationFailed)?;
  for row in rows.iter().take_while(|r|r[11]==5) {
   let size=word(row,0x2c);
   let base=[word(row,0x54),word(row,0x58),word(row,0x5c)];
   let axis=[word(row,0x48),word(row,0x4c),word(row,0x50)];
   if std::iter::once(size).chain(base).chain(axis).any(|x|!x.is_finite()) { return Err(Error::NonFinite); }
   let factor=f32::from_bits(0xbe23c528)*size;
   let mut center=[0.;3];
   for i in 0..3 { let scaled=axis[i]*factor; center[i]=base[i]+scaled; if !scaled.is_finite() || !center[i].is_finite() { return Err(Error::NonFinite); } }
   let first=size*f32::from_bits(0x3ec18cd8);
   let second=first*f32::from_bits(0x3f333333);
   let radius=if f32::from_bits(0x3d23d70a)>second {f32::from_bits(0x3d23d70a)} else {second};
   let cutoff=radius*radius; let scale=1.0/cutoff;
   if [factor,first,second,radius,cutoff,scale].into_iter().any(|x|!x.is_finite()) { return Err(Error::NonFinite); }
   for i in 0..3 { let lo=center[i]-radius; let hi=center[i]+radius;
    if !lo.is_finite() || !hi.is_finite() {return Err(Error::NonFinite);}
    if lo<out.lower[i] {out.lower[i]=lo;} if hi>out.upper[i] {out.upper[i]=hi;}
   }
   out.lanes.push(Lane { center, cutoff, scale, coefficient:1., key:(i16::from_le_bytes([row[0],row[1]]) as i32) as u32 });
  }
  Ok(out)
 })
}
#[cfg(test)]
mod tests {
 use super::*;
 fn row(key:i16,size:f32,axis:[f32;3],base:[f32;3])->[u8;ROW_SIZE] {
  let mut r=[0;ROW_SIZE];r[..2].copy_from_slice(&key.to_le_bytes());r[8]=1;r[11]=5;r[0x2c..0x30].copy_from_slice(&size.to_le_bytes());
  for (off,v) in [0x48,0x4c,0x50].into_iter().zip(axis) {r[off..off+4].copy_from_slice(&v.to_le_bytes());}
  for (off,v) in [0x54,0x58,0x5c].into_iter().zip(base) {r[off..off+4].copy_from_slice(&v.to_le_bytes());} r
 }
 #[test] fn prefix_gate_order_signed_key_and_bounds() {
  let a=row(-2,0.,[0.;3],[1.,2.,3.]);let b=row(7,0.,[0.;3],[-1.,0.,1.]);let mut end=a;end[11]=3;
  let p=build(&[a,b,end,a],Policy::Mxcsr1f80,FrozenRowsAdmission).unwrap();
  assert_eq!(p.lanes.iter().map(|l|l.key).collect::<Vec<_>>(),[0xfffffffe,7]);
  let r=f32::from_bits(0x3d23d70a); assert_eq!(p.lower,[-1.-r,-r,1.-r]);assert_eq!(p.upper,[1.+r,2.+r,3.+r]);
 }
 #[test] fn rejects_unflagged_and_nonfinite() {
  let mut r=row(1,1.,[1.;3],[0.;3]);r[8]=0;assert_eq!(build(&[r],Policy::Mxcsr1f80,FrozenRowsAdmission),Err(Error::NoFlaggedRow));
  r[8]=1;r[0x2c..0x30].copy_from_slice(&f32::NAN.to_le_bytes()); assert_eq!(build(&[r],Policy::Mxcsr1f80,FrozenRowsAdmission),Err(Error::NonFinite));
 }
}
