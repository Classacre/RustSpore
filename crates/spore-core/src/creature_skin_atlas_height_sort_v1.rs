//! Checked mode-zero 50EE30/50FAD0/50FB90 small-range group-height sort.
//! Admits 2..=28 caller-supplied groups, finite normal-or-zero vertical bounds
//! and heights under MXCSR 1F80. Larger partition/heap paths, n=1 heap path,
//! native pointer ownership and actual group identity are not modeled.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { UnsupportedCount, NonFiniteOrSubnormal, UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
fn height(pair:[u32;2])->Result<f32,Error>{let lo=admitted(f32::from_bits(pair[0]))?;let hi=admitted(f32::from_bits(pair[1]))?;admitted(hi-lo)}
/// Return original input indices in descending height order with stable ties.
pub fn sort(bounds:&[[u32;2]],policy:Policy)->Result<Vec<u32>,Error>{
 if !(2..=28).contains(&bounds.len()){return Err(Error::UnsupportedCount)}
 native_sse::scoped(policy,||{
  for &p in bounds {height(p)?;}
  let mut order:Vec<u32>=(0..bounds.len() as u32).collect();
  for i in 1..order.len(){let chosen=order[i];let mut j=i;while j>0 && height(bounds[chosen as usize])?>height(bounds[order[j-1] as usize])? {order[j]=order[j-1];j-=1}order[j]=chosen;}
  Ok(order)
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn descending_stable_ties_and_zero(){
  let p=Policy::Mxcsr1f80;let v=[[0,1f32.to_bits()],[0,2f32.to_bits()],[0,1f32.to_bits()],[0,2f32.to_bits()]];assert_eq!(sort(&v,p),Ok(vec![1,3,0,2]));
  assert_eq!(sort(&[[0,0x80000000],[0x80000000,0]],p),Ok(vec![0,1]));
 }
 #[test] fn rejects_unsupported_and_nonfinite(){
  let p=Policy::Mxcsr1f80;assert_eq!(sort(&[[0,0]],p),Err(Error::UnsupportedCount));assert_eq!(sort(&[[0,0];29],p),Err(Error::UnsupportedCount));
  assert_eq!(sort(&[[0,1],[0,0]],p),Err(Error::NonFiniteOrSubnormal));assert_eq!(sort(&[[0,0x7f800000],[0,0]],p),Err(Error::NonFiniteOrSubnormal));
 }
}
