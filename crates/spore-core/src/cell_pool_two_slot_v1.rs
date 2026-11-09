//! Fixed two-slot fresh-allocation and cursor projection; no free-list reuse.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, UnsupportedPoolState, OutOfBounds }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Pool { pub slots:[[u8;0x7c];2], pub count:u32, pub high_water:u32, pub generation:u32 }
impl Pool {
 /// The caller admits the source pool/slot lifetime and nonaliasing.
 pub fn new(admitted:bool)->Result<Self,Error>{if !admitted{return Err(Error::Unadmitted)}Ok(Self{slots:[[0xa5;0x7c];2],count:0,high_water:0,generation:0})}
 /// Source fresh-slot path only. Released-slot reuse is deliberately unsupported.
 pub fn allocate(&mut self)->Result<Option<u32>,Error>{
  if self.count!=self.high_water || self.high_water>2 || self.generation>0xfffd{return Err(Error::UnsupportedPoolState)}
  if self.high_water==2{return Ok(None)}
  let index=self.high_water;self.high_water+=1;self.count+=1;self.generation+=1;let handle=(self.generation<<16)|index;
  let slot=&mut self.slots[index as usize];slot.fill(0);slot[..4].copy_from_slice(&handle.to_le_bytes());Ok(Some(handle))
 }
 /// Source B71D00 only projects the low index; it does not validate a handle.
 pub fn lookup_offset(&self,handle:u32)->Result<usize,Error>{let index=(handle&0xffff) as usize;if index>=2{return Err(Error::OutOfBounds)}Ok(index*0x7c)}
 /// Scan at and beyond a supplied cursor; source accepts a slot by low-index match.
 pub fn iterate_offset(&self,cursor:&mut u32)->Result<Option<usize>,Error>{
  if self.high_water>2{return Err(Error::UnsupportedPoolState)}
  if *cursor==u32::MAX{return Ok(None)}
  if *cursor>2{return Err(Error::OutOfBounds)}
  while *cursor<self.high_water {let i=*cursor as usize;let h=u32::from_le_bytes(self.slots[i][..4].try_into().unwrap());if h&0xffff==*cursor{*cursor+=1;return Ok(Some(i*0x7c))}*cursor+=1;}
  *cursor=u32::MAX;Ok(None)
 }
}
#[cfg(test)]mod tests{use super::*;#[test]fn fresh_full_lookup_and_skip(){let mut p=Pool::new(true).unwrap();let a=p.allocate().unwrap().unwrap();let b=p.allocate().unwrap().unwrap();assert_eq!((a,b),(0x10000,0x20001));assert_eq!(p.allocate().unwrap(),None);assert_eq!(p.lookup_offset(b),Ok(0x7c));let mut c=0;assert_eq!(p.iterate_offset(&mut c),Ok(Some(0)));p.slots[0][..4].copy_from_slice(&u32::MAX.to_le_bytes());c=0;assert_eq!(p.iterate_offset(&mut c),Ok(Some(0x7c)));assert_eq!(p.iterate_offset(&mut c),Ok(None));assert_eq!(Pool::new(false),Err(Error::Unadmitted));}}
