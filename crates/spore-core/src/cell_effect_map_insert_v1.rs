//! Bounded no-rehash 594990 insertion into supplied synthetic effect maps.
use crate::cell_effect_map_lookup_v1::{Node,Table};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Error{Unadmitted,EmptyBuckets,InvalidChain,Capacity}
/// Caller admits a stable disjoint map, normally returning source-equivalent
/// no-rehash helper, and successful allocation up to `node_capacity`. This
/// models the node chain, not allocator callbacks, loaded EFFDIR or ownership.
/// Duplicate keys leave the existing value and chain unchanged.
pub fn insert(table:&mut Table,key:[u32;2],value:i32,node_capacity:usize,admitted:bool)->Result<bool,Error>{
 if !admitted{return Err(Error::Unadmitted)}if table.buckets.is_empty(){return Err(Error::EmptyBuckets)}
 let bucket=(key[0] as u64 % table.buckets.len() as u64) as usize;let mut at=table.buckets[bucket];let mut steps=0;
 while let Some(i)=at {if steps>=table.nodes.len(){return Err(Error::InvalidChain)}let node=table.nodes.get(i).ok_or(Error::InvalidChain)?;steps+=1;if node.key==key{return Ok(false)}at=node.next;}
 if table.nodes.len()>=node_capacity{return Err(Error::Capacity)}let next=table.buckets[bucket];let index=table.nodes.len();table.nodes.push(Node{key,value,next});table.buckets[bucket]=Some(index);Ok(true)
}
#[cfg(test)]mod tests{use super::*;#[test]fn duplicate_and_capacity(){let mut t=Table{buckets:vec![None],nodes:vec![]};assert_eq!(insert(&mut t,[1,0],4,1,true),Ok(true));assert_eq!(insert(&mut t,[1,0],7,1,true),Ok(false));assert_eq!(t.nodes[0].value,4);assert_eq!(insert(&mut t,[2,0],9,1,true),Err(Error::Capacity));}}
