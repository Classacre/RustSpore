//! Controlled BBA9C0 rank/tag predicate. Inputs are explicitly supplied slot
//! words, not proof of completed native sort, registration or actor identity.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RankedSlot {pub tag:u32, pub links:[u16;4]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {InvalidExcludedAxis}
/// The source uses signed 16-bit endpoint positions; it skips one axis and
/// rejects equal nonzero tag words. Link provenance/lifetimes are admissions.
pub fn eligible(a:RankedSlot,b:RankedSlot,excluded_axis:usize)->Result<bool,Error>{
 if excluded_axis>=2{return Err(Error::InvalidExcludedAxis)}
 if a.tag!=0 && a.tag==b.tag{return Ok(false)}
 for axis in 0..2 {if axis==excluded_axis{continue}if (a.links[axis+2] as i16)<(b.links[axis] as i16)||(b.links[axis+2] as i16)<(a.links[axis] as i16){return Ok(false)}}
 Ok(true)
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn source_tag_and_signed_rank_branches(){let a=RankedSlot{tag:0,links:[0,1,2,4]};let mut b=RankedSlot{tag:0,links:[1,3,3,5]};assert_eq!(eligible(a,b,0),Ok(true));b.links[1]=5;b.links[3]=6;assert_eq!(eligible(a,b,0),Ok(false));b.tag=7;let mut c=a;c.tag=7;assert_eq!(eligible(c,b,0),Ok(false));b.tag=0;b.links=[0,0x8000,2,0x8001];assert_eq!(eligible(a,b,0),Ok(false));assert_eq!(eligible(a,b,2),Err(Error::InvalidExcludedAxis));}
}
