//! Bounded synthetic A6D7B0/5948D0 table lookup; no live provider identity.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct Node{pub key:[u32;2],pub value:i32,pub next:Option<usize>}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Table{pub buckets:Vec<Option<usize>>,pub nodes:Vec<Node>}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Error{Unadmitted,EmptyBuckets,TooManyTables,InvalidChain,ValueOutOfRange}
/// Caller admits stable, disjoint table storage and source-equivalent sentinel
/// behavior. Each Vec entry represents an entire bucket chain, with None as
/// the terminal pointer; malformed chains fail closed. Restrict table index and
/// nonnegative values to unambiguous packed representations. Real loading,
/// map order, mounted provider and lifetime remain supplied/unverified.
pub fn project(tables:&[Option<Table>],key:[u32;2],admitted:bool)->Result<Option<u32>,Error>{
 if !admitted{return Err(Error::Unadmitted)}if tables.len()>128{return Err(Error::TooManyTables)}
 for (index,table) in tables.iter().enumerate(){let Some(t)=table else{continue};if t.buckets.is_empty(){return Err(Error::EmptyBuckets)}
  let bucket=(key[0] as u64 % t.buckets.len() as u64) as usize;let mut at=t.buckets[bucket];let mut steps=0usize;
  while let Some(i)=at{if steps>=t.nodes.len(){return Err(Error::InvalidChain)}let node=t.nodes.get(i).ok_or(Error::InvalidChain)?;steps+=1;
   if node.key==key {if node.value>=0 {let value=node.value as u32;if value>0x00ff_ffff{return Err(Error::ValueOutOfRange)}return Ok(Some(((index as u32)<<24)|value))}break}at=node.next;
  }
 }
 Ok(None)
}
#[cfg(test)]mod tests{use super::*;#[test]fn skip_negative_and_reject_cycle(){let neg=Table{buckets:vec![Some(0)],nodes:vec![Node{key:[1,0],value:-1,next:None}]};let hit=Table{buckets:vec![Some(0)],nodes:vec![Node{key:[1,0],value:9,next:None}]};assert_eq!(project(&[Some(neg),Some(hit)],[1,0],true),Ok(Some(0x01000009)));let cycle=Table{buckets:vec![Some(0)],nodes:vec![Node{key:[2,0],value:1,next:Some(0)}]};assert_eq!(project(&[Some(cycle)],[1,0],true),Err(Error::InvalidChain));}}
