//! Checked bounded 50B5EA..50B83A shelf-selection composition.
//! Caller supplies order, bounds and sqrt(area); initial shelf is empty.
//! Modeled groups must fit strictly in an empty row, 1..=12 groups/shelves,
//! finite normal-or-zero values and intermediate arithmetic, MXCSR 1F80.
//! Owned output and prevalidation exclude native allocation, lifecycle and
//! partial-failure effects, actual preceding groups, packing and live identity.
use crate::{creature_skin_shelf_append_v1::{self as shelf,Shelf},native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { InvalidCount, NonFiniteOrSubnormal, DoesNotFitEmpty, Capacity, Component, UnsupportedArchitecture }
impl From<SseError> for Error {fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Output {pub remaining:Vec<Option<u32>>,pub shelves:Vec<Shelf>}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
fn new_shelf()->Shelf{Shelf{width:0,height:0,groups:Vec::with_capacity(12)}}
/// Groups are in explicitly supplied order; each rectangle is [loU,hiU,loV,hiV].
pub fn select(groups:&[[u32;4]],sqrt_area:u32,policy:Policy)->Result<Output,Error>{
 if groups.is_empty()||groups.len()>12{return Err(Error::InvalidCount)}
 let root=admitted(f32::from_bits(sqrt_area))?;
 native_sse::scoped(policy,||{
  let threshold=admitted(root*f32::from_bits(0x3f8ccccd))?;let mut widths=Vec::with_capacity(groups.len());
  for g in groups{
   let mut v=[0f32;4];for i in 0..4{v[i]=admitted(f32::from_bits(g[i]))?;}
   let w=admitted(v[1]-v[0])?;admitted(v[3]-v[2])?;
   if w<=0.0||w>=threshold{return Err(Error::DoesNotFitEmpty)}widths.push(w);
  }
  let mut remaining:Vec<Option<u32>>=(0..groups.len() as u32).map(Some).collect();let mut shelves=vec![new_shelf()];let mut cursor=0;
  while cursor<groups.len(){
   while cursor<groups.len()&&remaining[cursor].is_none(){cursor+=1}
   if cursor==groups.len(){break}
   let current=remaining[cursor].ok_or(Error::Component)? as usize;let last=shelves.len()-1;
   let candidate=admitted(f32::from_bits(shelves[last].width)+widths[current])?;
   if threshold<=candidate {
    for i in cursor+1..groups.len(){if let Some(value)=remaining[i]{let k=value as usize;let sum=admitted(f32::from_bits(shelves[last].width)+widths[k])?;
      if sum<threshold {shelf::append(&mut shelves[last],value,[groups[k][2],groups[k][3]],widths[k].to_bits(),policy).map_err(|_|Error::Component)?;remaining[i]=None;}
    }}
    if shelves.len()>=12{return Err(Error::Capacity)}shelves.push(new_shelf());
   }else{shelf::append(&mut shelves[last],current as u32,[groups[current][2],groups[current][3]],widths[current].to_bits(),policy).map_err(|_|Error::Component)?;cursor+=1;}
  }
  Ok(Output{remaining,shelves})
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test] fn exact_fit_opens_row_and_later_candidate_fills_prior(){
  let b=|x:f32|x.to_bits();let groups=[[0,b(0.4),0,b(0.3)],[0,b(0.7),0,b(0.2)],[0,b(0.3),0,b(0.1)]];
  let o=select(&groups,b(1.),Policy::Mxcsr1f80).unwrap();assert_eq!(o.shelves.iter().map(|s|s.groups.clone()).collect::<Vec<_>>(),vec![vec![0,2],vec![1]]);assert_eq!(o.remaining,vec![Some(0),Some(1),None]);
 }
 #[test] fn rejects_nonprogress_nonfinite_and_count(){
  let p=Policy::Mxcsr1f80;assert_eq!(select(&[],1f32.to_bits(),p),Err(Error::InvalidCount));
  assert_eq!(select(&[[0,2f32.to_bits(),0,0]],1f32.to_bits(),p),Err(Error::DoesNotFitEmpty));
  assert_eq!(select(&[[0,0x7f800000,0,0]],1f32.to_bits(),p),Err(Error::NonFiniteOrSubnormal));
 }
}

