//! Bounded 92055C registry candidate scan over caller-supplied stable links.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Node { pub key: u32, pub value: u32, pub priority: i32 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, UnsupportedControl }
/// `nodes` is the ordered, finite, non-null chain before its sentinel. The
/// caller admits valid stable links, the sentinel, and lock/lifetime state.
/// A mismatched key stops the scan; signed priority ties keep the first node.
/// No registry contents, callbacks, unlock or live ownership are implied.
pub fn project(key:u32,nodes:&[Node],x87:u16,mxcsr:u32,admitted:bool)->Result<Option<u32>,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f)||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let mut best:Option<Node>=None;
 for &node in nodes { if node.key!=key {break} if best.map_or(true,|v|node.priority>v.priority){best=Some(node)} }
 Ok(best.map(|v|v.value))
}
#[cfg(test)] mod tests {use super::*; #[test] fn signed_ties_and_mismatch(){let n=[Node{key:7,value:1,priority:-1},Node{key:7,value:2,priority:1},Node{key:8,value:3,priority:99},Node{key:7,value:4,priority:100}];assert_eq!(project(7,&n,0x27f,0x1f80,true),Ok(Some(2)));let tied=[Node{key:7,value:1,priority:i32::MIN},Node{key:7,value:2,priority:i32::MIN}];assert_eq!(project(7,&tied,0x37f,0x1f80,true),Ok(Some(1)));assert_eq!(project(8,&n,0x27f,0x1f80,true),Ok(None));assert_eq!(project(7,&n,0x27f,0x1f80,false),Err(Error::Unadmitted));}}
