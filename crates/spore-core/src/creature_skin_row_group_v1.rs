//! Conditional 4CC63B..4CC81B source-row group initialization.
//! Frozen row/capability identity, successful construction and storage are caller
//! admissions. The sqrt callback supplies the binary32 spill of the selected
//! CRT/x87 routine; compatibility is not asserted for arbitrary inputs. No native
//! allocation, append, lifecycle, atlas placement or live identity is modeled.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { InvalidCapabilities, NonFiniteOrSubnormal, Sqrt, UnsupportedArchitecture }
impl From<SseError> for Error { fn from(_:SseError)->Self { Self::UnsupportedArchitecture } }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Group { pub id:u32, pub bounds:[u32;4], pub flags:u32 }
/// Supplies the CRT sqrt's binary32 spill for a finite nonnegative input.
pub trait SqrtSpill { fn sqrt_spill(&mut self,input:f32)->Option<f32>; }
fn admitted(v:f32)->Result<f32,Error> { if v.is_finite()&&!v.is_subnormal(){Ok(v)}else{Err(Error::NonFiniteOrSubnormal)} }
fn word(row:&[u8;140],at:usize)->u32 { u32::from_le_bytes(row[at..at+4].try_into().unwrap()) }
/// Returns None for the source row flag gate, or a checked new group. The id
/// must be the successful append ordinal supplied by the caller.
pub fn project<S:SqrtSpill>(row:&[u8;140],caps:&[u32],id:u32,policy:Policy,sqrt:&mut S)->Result<Option<Group>,Error> {
 if u16::from_le_bytes([row[8],row[9]])&1!=0 {return Ok(None)}
 let start=u16::from_le_bytes([row[12],row[13]]) as usize;let end=start.checked_add(row[16] as usize).ok_or(Error::InvalidCapabilities)?;
 let subset=caps.get(start..end).ok_or(Error::InvalidCapabilities)?;
 let lo=[word(row,20),word(row,24),word(row,28)].map(f32::from_bits);
 let hi=[word(row,32),word(row,36),word(row,40)].map(f32::from_bits);
 let factor=f32::from_bits(word(row,44));for x in lo.into_iter().chain(hi).chain([factor]) {admitted(x)?;}
 native_sse::scoped(policy,||{
  let x=admitted(hi[0]-lo[0])?;let y=admitted(hi[1]-lo[1])?;let z=admitted(hi[2]-lo[2])?;
  let zz=admitted(z*z)?;let yy=admitted(y*y)?;let xx=admitted(x*x)?;let xy=admitted(xx+yy)?;let sum=admitted(xy+zz)?;
  let root=admitted(sqrt.sqrt_spill(sum).ok_or(Error::Sqrt)?)?;if root<0.0{return Err(Error::Sqrt)}
  let mut side=admitted(root*factor)?;if subset.contains(&0x74756f6d){side=admitted(side*f32::from_bits(0x400ccccd))?;}
  side=admitted(side*f32::from_bits(0x3ecccccd))?;side=admitted(side+f32::from_bits(0x3da3d70a))?;
  Ok(Some(Group{id,bounds:[0,side.to_bits(),0,side.to_bits()],flags:0}))
 })
}
#[cfg(test)]mod tests {
 use super::*;struct Math;impl SqrtSpill for Math {fn sqrt_spill(&mut self,x:f32)->Option<f32>{Some(x.sqrt())}}
 #[test]fn gate_zero_and_mouth(){let mut row=[0u8;140];let p=Policy::Mxcsr1f80;row[8]=1;assert_eq!(project(&row,&[],0,p,&mut Math),Ok(None));row[8]=0;row[44..48].copy_from_slice(&1f32.to_le_bytes());let a=project(&row,&[],7,p,&mut Math).unwrap().unwrap();assert_eq!(a.bounds,[0,0x3da3d70a,0,0x3da3d70a]);row[32..36].copy_from_slice(&1f32.to_le_bytes());row[16]=1;let a=project(&row,&[0x74756f6d],0,p,&mut Math).unwrap().unwrap();assert_eq!(a.bounds[1],(0.4f32*2.2+0.08).to_bits());}
 #[test]fn rejects_invalid_ranges_and_values(){let mut r=[0;140];r[16]=1;assert_eq!(project(&r,&[],0,Policy::Mxcsr1f80,&mut Math),Err(Error::InvalidCapabilities));r[16]=0;r[20..24].copy_from_slice(&f32::NAN.to_le_bytes());assert_eq!(project(&r,&[],0,Policy::Mxcsr1f80,&mut Math),Err(Error::NonFiniteOrSubnormal));}
}
