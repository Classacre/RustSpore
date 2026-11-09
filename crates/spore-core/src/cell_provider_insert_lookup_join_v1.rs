//! Successful fixed-capacity 91FFE0 insertion followed by 920510 selection.
use crate::{cell_provider_map_priority_join_v1,cell_provider_priority_scan_v1::Node};
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,Lookup(cell_provider_map_priority_join_v1::Error)}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub nodes:Vec<Node>,pub selected:Option<u32>}
/// One stable bucket with a distinct terminal sentinel; allocator succeeds with
/// a fresh disjoint node, growth is absent, and callbacks/lock return normally.
/// Existing links and lifetimes are supplied. No live registration is inferred.
pub fn project(key:u32,existing:&[Node],new:Node,cw:u16,mx:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let mut nodes=existing.to_vec();let at=nodes.iter().position(|n|n.key==new.key).map_or(0,|i|i+1);nodes.insert(at,new);
 let selected=cell_provider_map_priority_join_v1::project(key,&nodes,cw,mx,true).map_err(Error::Lookup)?;
 Ok(Outcome{nodes,selected})
}
#[cfg(test)]mod tests{use super::*;#[test]fn head_middle_tail_and_priority(){let n=|k,v,p|Node{key:k,value:v,priority:p};let x=project(7,&[n(8,1,1),n(7,2,3)],n(7,3,3),0x27f,0x1f80,true).unwrap();assert_eq!(x.nodes,[n(8,1,1),n(7,2,3),n(7,3,3)]);assert_eq!(x.selected,Some(2));let y=project(7,&[n(8,1,1)],n(7,3,-2),0x37f,0x1f80,true).unwrap();assert_eq!(y.nodes,[n(7,3,-2),n(8,1,1)]);assert_eq!(y.selected,Some(3));assert_eq!(project(7,&[],n(7,3,1),0x27f,0x1f80,false),Err(Error::Unadmitted));}}
