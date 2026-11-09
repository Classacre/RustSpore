//! Bounded E673A0 fixture phases; no provider, native vector or callbacks.
//! Admitted snapshots are owned/disjoint/stable only where explicitly stated.
use spore_core::native_sse::{self,Policy,SseError};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct Input {pub ids:[u32;2],pub point_id:u64,pub point:[u32;3],pub distance:u32,pub current:u32}
#[derive(Debug,PartialEq,Eq)]
pub(super) enum Error {NonFinite,InvalidBounds,InvalidOrder,IdentityMismatch,Sse(SseError)}
impl From<SseError> for Error{fn from(e:SseError)->Self{Self::Sse(e)}}
#[derive(Debug,PartialEq,Eq)]
pub(super) enum BoundsResult {Outside,PendingProvider(Input)}
#[derive(Debug,PartialEq,Eq)]
pub(super) enum ProviderResult {PendingCleanup,PendingLookup {input:Input,key:u32}}
fn finite(v:u32)->bool{f32::from_bits(v).is_finite()}

/// Gate a caller-admitted point and bounds, stopping before holder/provider calls.
pub(super) fn bounds(input:Input,lower:[u32;3],upper:[u32;3])->Result<BoundsResult,Error>{
 if !finite(input.distance)||input.point.iter().chain(&lower).chain(&upper).any(|&v|!finite(v)){return Err(Error::NonFinite)}
 native_sse::scoped(Policy::Mxcsr1f80,||{
  for i in 0..3 {let lo=f32::from_bits(lower[i]);let hi=f32::from_bits(upper[i]);let p=f32::from_bits(input.point[i]);
   if lo>hi{return Err(Error::InvalidBounds)}if p<lo||p>hi{return Ok(BoundsResult::Outside)} }
  Ok(BoundsResult::PendingProvider(input))
 })
}
/// Caller admits provider/holder success and the fresh +6C word. No such call
/// is executed; a pending cleanup or pending lookup is returned.
pub(super) fn after_provider(input:Input,min_distance:u32)->Result<ProviderResult,Error>{
 if !finite(input.distance)||!finite(min_distance){return Err(Error::NonFinite)}
 native_sse::scoped(Policy::Mxcsr1f80,||{
  if f32::from_bits(min_distance)>f32::from_bits(input.distance){return Ok(ProviderResult::PendingCleanup)}
  Ok(ProviderResult::PendingLookup{key:packed_key(input.ids),input})
 })
}
pub(super) fn packed_key(ids:[u32;2])->u32{let [a,b]=if ids[0]<=ids[1]{ids}else{[ids[1],ids[0]]};((a&0xffff)<<16)|(b&0xffff)}

/// E616C0's signed lower bound over an explicitly supplied, strictly sorted
/// key slice. Rejection of duplicates and >65536 rows is a fixture policy.
pub(super) fn lookup(keys:&[u32],key:u32)->Result<Option<usize>,Error>{
 if keys.len()>65536 || keys.windows(2).any(|p|(p[0] as i32)>=(p[1] as i32)){return Err(Error::InvalidOrder)}
 let (mut lo,mut hi)=(0,keys.len());while lo<hi{let mid=lo+(hi-lo)/2;if (keys[mid] as i32)<(key as i32){lo=mid+1}else{hi=mid}}
 Ok((lo<keys.len()&&keys[lo]==key).then_some(lo))
}

/// Owned entry snapshot, not a source pointer or vector slot. Identity is an
/// externally controlled fixture epoch/slot label, distinct from the packed key.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct Entry {pub identity:u64,pub key:u32,pub ids:[u32;2],pub created:u32,pub updated:u32,pub point:[u32;3],pub impact:u8,pub handles:[u32;2]}
#[derive(Debug,PartialEq,Eq)]
pub(super) struct ExistingTicket {identity:u64,key:u32,point_id:u64}
/// Admit the first E65810 result and apply only its entry +10 write. The
/// pending second lookup may reallocate or invoke effects; no implicit resume.
pub(super) fn begin_existing(entry:&mut Entry,key:u32,current:u32,point_id:u64)->Result<ExistingTicket,Error>{
 if entry.key!=key{return Err(Error::IdentityMismatch)}entry.updated=current;
 Ok(ExistingTicket{identity:entry.identity,key,point_id})
}
/// Explicitly admit a second lookup to the same owned entry and the same point
/// identity. Fresh point words are read now; rejection preserves first write.
pub(super) fn finish_existing(ticket:ExistingTicket,entry:&mut Entry,point_id:u64,fresh_point:[u32;3])->Result<(),Error>{
 if entry.identity!=ticket.identity||entry.key!=ticket.key||point_id!=ticket.point_id{return Err(Error::IdentityMismatch)}
 entry.point=fresh_point;Ok(())
}
/// Admit a fresh, zero-owned slot and no intervening mutation. No allocation or
/// holder cleanup is executed. Result captures only named source-written fields.
pub(super) fn new_entry(identity:u64,input:Input,key:u32,impact_threshold:u32,point_id:u64,fresh_point:[u32;3])->Result<Entry,Error>{
 if key!=packed_key(input.ids)||point_id!=input.point_id{return Err(Error::IdentityMismatch)}
 if !finite(input.distance)||!finite(impact_threshold){return Err(Error::NonFinite)}
 native_sse::scoped(Policy::Mxcsr1f80,||{
  let ids=if input.ids[0]<=input.ids[1]{input.ids}else{[input.ids[1],input.ids[0]]};
  Ok(Entry{identity,key,ids,created:input.current,updated:input.current,point:fresh_point,
    impact:u8::from(f32::from_bits(input.distance)>f32::from_bits(impact_threshold)),handles:[0;2]})
 })
}

#[cfg(test)]
mod tests{
 use super::*;fn b(v:f32)->u32{v.to_bits()}
 fn input()->Input{Input{ids:[0x20002,0x10001],point_id:9,point:[b(0.),b(1.),b(2.)],distance:b(5.),current:42}}
 #[test]fn gates_and_signed_lower_bound_are_separate_admissions(){
  let i=input();let lo=[b(0.);3];let hi=[b(2.);3];assert_eq!(bounds(i,lo,hi).unwrap(),BoundsResult::PendingProvider(i));
  assert_eq!(after_provider(i,b(6.)).unwrap(),ProviderResult::PendingCleanup);
  let ProviderResult::PendingLookup{key,..}=after_provider(i,b(5.)).unwrap() else{panic!()};assert_eq!(key,0x00010002);
  assert_eq!(lookup(&[0x80000000,0x00010002,0x7fffffff],key).unwrap(),Some(1));
  assert_eq!(lookup(&[0x00010002,0x80000000],key),Err(Error::InvalidOrder));
  let mut out=i;out.point[0]=b(-1.);assert_eq!(bounds(out,lo,hi).unwrap(),BoundsResult::Outside);
  out.point[0]=f32::NAN.to_bits();assert_eq!(bounds(out,lo,hi),Err(Error::NonFinite));
 }
 #[test]fn existing_write_preserves_identity_and_admits_fresh_point_after_second_lookup(){
  let i=input();let key=packed_key(i.ids);let mut e=Entry{identity:7,key,ids:[0x90001,0xa0002],created:5,updated:6,point:[1,2,3],impact:1,handles:[8,9]};
  let ticket=begin_existing(&mut e,key,42,i.point_id).unwrap();assert_eq!(e.updated,42);assert_eq!(e.ids,[0x90001,0xa0002]);assert_eq!(e.created,5);assert_eq!(e.impact,1);
  finish_existing(ticket,&mut e,9,[4,5,6]).unwrap();assert_eq!(e.point,[4,5,6]);
  let ticket=begin_existing(&mut e,key,43,9).unwrap();let mut replacement=Entry{identity:8,..e};
  assert_eq!(finish_existing(ticket,&mut replacement,9,[7,8,9]),Err(Error::IdentityMismatch));assert_eq!(replacement.point,[4,5,6]);assert_eq!(e.updated,43);
  let ticket=begin_existing(&mut e,key,44,9).unwrap();
  assert_eq!(finish_existing(ticket,&mut e,10,[7,8,9]),Err(Error::IdentityMismatch));assert_eq!(e.updated,44);assert_eq!(e.point,[4,5,6]);
 }
 #[test]fn new_entry_keeps_offsets_and_strict_impact_comparison(){
  let i=input();let key=packed_key(i.ids);let equal=new_entry(1,i,key,b(5.),i.point_id,[4,5,6]).unwrap();assert_eq!(equal.ids,[0x10001,0x20002]);assert_eq!(equal.point,[4,5,6]);assert_eq!((equal.created,equal.updated,equal.impact,equal.handles),(42,42,0,[0,0]));
  assert_eq!(new_entry(2,i,key,b(4.),i.point_id,i.point).unwrap().impact,1);assert_eq!(new_entry(3,i,key,b(6.),i.point_id,i.point).unwrap().impact,0);
  assert_eq!(packed_key([0x90001,0xa0002]),key);assert_eq!(new_entry(4,i,key^1,b(5.),i.point_id,i.point),Err(Error::IdentityMismatch));
  assert_eq!(new_entry(5,i,key,b(5.),i.point_id+1,i.point),Err(Error::IdentityMismatch));
 }
}
