//! Controlled A68330 -> 920510 lookup and priority composition for one bucket.
use crate::cell_provider_priority_scan_v1::{self,Node};
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,Scan(cell_provider_priority_scan_v1::Error)}
/// `bucket` is a stable, finite chain in lookup order. The caller supplies a
/// nonzero single bucket and a distinct non-null terminal sentinel, valid lock
/// and unlock callbacks and stable node lifetimes. Lookup skips collisions and
/// the scan stops at the first differing key. Live registration is not inferred.
pub fn project(key:u32,bucket:&[Node],cw:u16,mx:u32,admitted:bool)->Result<Option<u32>,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let start=bucket.iter().position(|n|n.key==key).unwrap_or(bucket.len());
 cell_provider_priority_scan_v1::project(key,&bucket[start..],cw,mx,true).map_err(Error::Scan)
}
#[cfg(test)]mod tests{use super::*;#[test]fn lookup_skips_collision_and_keeps_first_tie(){let b=[Node{key:8,value:3,priority:9},Node{key:7,value:1,priority:-1},Node{key:7,value:2,priority:-1},Node{key:8,value:4,priority:5},Node{key:7,value:5,priority:99}];assert_eq!(project(7,&b,0x27f,0x1f80,true),Ok(Some(1)));assert_eq!(project(9,&b,0x27f,0x1f80,true),Ok(None));assert_eq!(project(7,&b,0x27f,0x1f80,false),Err(Error::Unadmitted));}}
