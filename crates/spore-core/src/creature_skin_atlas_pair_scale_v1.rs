//! Checked 50B83B..50BE09 shelf pairing and 50BE3A..50BE87 scale.
//! Supplied groups/shelves are bounded, positive finite normal dimensions,
//! finite normal-or-zero inputs/intermediates, MXCSR 1F80, 1..=12 groups/rows
//! and nonempty rows. Source allocator/lifecycle, prior groups, actual placement,
//! texture and live identity are not inferred. Returns owned, prevalidated data.
use crate::native_sse::{self,Policy,SseError};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{InvalidCount,InvalidIndex,NonFiniteOrSubnormal,NonPositive,Capacity,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Row{pub groups:Vec<u32>,pub width:u32,pub height:u32,pub shift:u32}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Output{pub width:u32,pub height:u32,pub scale:u32,pub rows:Vec<Row>}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
fn dimensions(g:[u32;4])->Result<(f32,f32),Error>{let lo=admitted(f32::from_bits(g[0]))?;let hi=admitted(f32::from_bits(g[1]))?;let lv=admitted(f32::from_bits(g[2]))?;let hv=admitted(f32::from_bits(g[3]))?;let w=admitted(hi-lo)?;let h=admitted(hv-lv)?;if w<=0.0||h<=0.0{return Err(Error::NonPositive)}Ok((w,h))}
/// Pair the supplied shelves, reversing each even shelf, then compute scale.
pub fn pair(groups:&[[u32;4]],rows:&[Row],policy:Policy)->Result<Output,Error>{
 if groups.is_empty()||groups.len()>12||rows.is_empty()||rows.len()>12{return Err(Error::InvalidCount)}
 for row in rows {if row.groups.is_empty()||row.groups.len()>12{return Err(Error::InvalidCount)}if row.groups.iter().any(|&i|i as usize>=groups.len()){return Err(Error::InvalidIndex)}}
 native_sse::scoped(policy,||{
  for &g in groups{dimensions(g)?;}for row in rows{let w=admitted(f32::from_bits(row.width))?;let h=admitted(f32::from_bits(row.height))?;admitted(f32::from_bits(row.shift))?;if w<=0.0||h<=0.0{return Err(Error::NonPositive)}}
  let mut out=rows.to_vec();let mut maxw=0f32;let mut totalh=0f32;
  for pair_index in 0..out.len()/2{
   let (left,right)=out.split_at_mut(pair_index*2+1);let a=&mut left[pair_index*2];let c=&mut right[0];a.groups.reverse();
   let ah=f32::from_bits(a.height);let ch=f32::from_bits(c.height);let sumh=admitted(ah+ch)?;let mut shift=f32::from_bits(sumh.to_bits()^0x80000000);let(mut i,mut j)=(0,0);let(mut useda,mut usedc)=(0f32,0f32);let mut steps=0;
   while i<a.groups.len()&&j<c.groups.len(){
    steps+=1;if steps>24{return Err(Error::Capacity)}
    let (wa,ha)=dimensions(groups[a.groups[i] as usize])?;let(wc,hc)=dimensions(groups[c.groups[j] as usize])?;
    let overlap=admitted(admitted(sumh-hc)?+shift)?;if overlap<ha{shift=admitted(admitted(ha-overlap)?+shift)?;}
    let ra=admitted(wa-useda)?;let rc=admitted(wc-usedc)?;
    if rc<=ra{if ra<=rc{i+=1;j+=1;useda=0.;usedc=0.;}else{j+=1;usedc=0.;useda=admitted(useda+rc)?;}}
    else{i+=1;useda=0.;usedc=admitted(usedc+ra)?;}
   }
   c.shift=shift.to_bits();let aw=f32::from_bits(a.width);let cw=f32::from_bits(c.width);let pairw=if cw<=aw&&aw!=cw{aw}else{cw};if maxw<=pairw{maxw=pairw;}
   let combined=admitted(sumh+shift)?;let pairh=if ah<=combined{combined}else{ah};totalh=admitted(totalh+pairh)?;
  }
  if out.len()&1!=0{let last=out.last().ok_or(Error::InvalidCount)?;let w=f32::from_bits(last.width);if w>maxw{maxw=w}totalh=admitted(totalh+f32::from_bits(last.height))?;}
  let largest=if totalh<=maxw{maxw}else{totalh};if largest<=0.0{return Err(Error::NonPositive)}let scale=admitted(1f32/largest)?;
  Ok(Output{width:maxw.to_bits(),height:totalh.to_bits(),scale:scale.to_bits(),rows:out})
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test]fn equal_shelves_reverse_and_scale(){let b=|x:f32|x.to_bits();let groups=[[0,b(1.),0,b(0.5)];4];let rows=[Row{groups:vec![0,1],width:b(2.),height:b(0.5),shift:0},Row{groups:vec![2,3],width:b(2.),height:b(0.5),shift:0}];let o=pair(&groups,&rows,Policy::Mxcsr1f80).unwrap();assert_eq!(o.rows[0].groups,[1,0]);assert_eq!((o.rows[1].shift,o.width,o.height,o.scale),(0,b(2.),b(1.),b(0.5)));}
 #[test]fn rejects_invalid_or_nonpositive(){let p=Policy::Mxcsr1f80;let row=Row{groups:vec![0],width:1f32.to_bits(),height:1f32.to_bits(),shift:0};assert_eq!(pair(&[],&[row.clone()],p),Err(Error::InvalidCount));assert_eq!(pair(&[[0,0,0,0]],&[row],p),Err(Error::NonPositive));}
}
