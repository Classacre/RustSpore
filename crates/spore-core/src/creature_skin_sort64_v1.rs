//! Finite 64-slot subset of source 52C930 sort; heap fallback is rejected.
//! Comparison on fixed supplied scratch is distinct from live smoothing state.
#[derive(Clone, Copy, Debug, PartialEq)] pub struct Entry { pub key:u32, pub weight:f32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Error { NonFinite, HeapFallback, InvalidPartition, UnsupportedArchitecture }
impl From<crate::native_sse::SseError> for Error { fn from(_:crate::native_sse::SseError)->Self { Self::UnsupportedArchitecture } }
fn median(a:&[Entry;64],i:usize,j:usize,k:usize)->usize{let (x,y,z)=(a[i].weight,a[j].weight,a[k].weight);if x>y{if y>z{j}else if x>z{k}else{i}}else if x>z{i}else if y>z{k}else{j}}
fn intro(a:&mut[Entry;64],lo:usize,mut hi:usize,mut depth:usize)->Result<(),Error>{while hi-lo>28&&depth>0{let pivot=a[median(a,lo,lo+(hi-lo)/2,hi-1)].weight;let(mut i,mut j)=(lo,hi);loop{while i<hi&&a[i].weight>pivot{i+=1}if i>=hi{return Err(Error::InvalidPartition)}j-=1;while pivot>a[j].weight{if j==lo{return Err(Error::InvalidPartition)}j-=1}if i>=j{break}a.swap(i,j);i+=1;}if i<=lo||i>=hi{return Err(Error::InvalidPartition)}depth-=1;intro(a,i,hi,depth)?;hi=i;}if depth==0{return Err(Error::HeapFallback)}Ok(())}
/// Sorts all 64 entries in original descending comparison and tie order.
/// Nonfinite values and the source heap fallback are outside this bounded model.
fn sort_inner(a:&mut[Entry;64])->Result<(),Error>{if a.iter().any(|x|!x.weight.is_finite()){return Err(Error::NonFinite)}intro(a,0,64,12)?;for i in 1..28{let x=a[i];let mut j=i;while j>0&&x.weight>a[j-1].weight{a[j]=a[j-1];j-=1}a[j]=x}for i in 28..64{let x=a[i];let mut j=i;while j>0&&x.weight>a[j-1].weight{a[j]=a[j-1];j-=1}if j==0{return Err(Error::InvalidPartition)}a[j]=x}Ok(())}
/// Run with round-to-nearest, masked exceptions, DAZ/FTZ disabled (MXCSR 1F80).
#[cfg(any(target_arch="x86",target_arch="x86_64"))]
pub fn sort(a:&mut[Entry;64])->Result<(),Error>{crate::native_sse::scoped(crate::native_sse::Policy::Mxcsr1f80,||sort_inner(a))}
#[cfg(not(any(target_arch="x86",target_arch="x86_64")))]
pub fn sort(_a:&mut[Entry;64])->Result<(),Error>{Err(Error::UnsupportedArchitecture)}
#[cfg(test)]mod tests{use super::*;#[test]fn equal_weights_follow_partition_not_stable_order(){let mut a=std::array::from_fn(|i|Entry{key:i as u32,weight:1.});sort(&mut a).unwrap();assert_eq!(a[0].key,32);assert_eq!(a[15].key,47)}#[test]fn rejects_nonfinite(){let mut a=[Entry{key:0,weight:0.};64];a[9].weight=f32::NAN;assert_eq!(sort(&mut a),Err(Error::NonFinite));}}
