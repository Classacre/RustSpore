//! Controlled E5CE46 request store and fresh E51760 projection.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, MissingFactoryResult, UnreadableComponent }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub factory_called:bool, pub component_request:u32, pub provider_return:u32, pub projected_index:Option<usize>, pub continuation_pending:bool }

/// `triple` is the original gate's supplied stack snapshot. `factory_result`
/// is the fresh scripted E4EE10 return when the gate is reached. The actor
/// handle/+FC are fresh after that call; `initial_request` is the component's
/// value on the no-factory path. The caller admits that the store's component
/// and the fresh pool slot are the same stable, disjoint object and that base/
/// stride arithmetic is valid and nonwrapping. Only the supplied component
/// index is readable. B71D00 does not validate generation; later request
/// methods, actual factory/allocation and live ownership remain pending.
pub fn project(triple:[u32;3],initial_request:u32,factory_result:Option<u32>,handle_after:u32,fc_after:u32,component_index:usize,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let called=triple[0]!=0&&triple[1]==0x3d97a8e4;
 let stored=if called{factory_result.ok_or(Error::MissingFactoryResult)?}else{initial_request};
 let index=if handle_after==0||fc_after==0{None}else{let i=(handle_after&0xffff)as usize;if i!=component_index{return Err(Error::UnreadableComponent)}Some(i)};
 Ok(Outcome{factory_called:called,component_request:stored,provider_return:if index.is_some(){stored}else{0},projected_index:index,continuation_pending:true})
}
#[cfg(test)]mod tests{use super::*;#[test]fn gate_and_fresh_lookup(){let good=[1,0x3d97a8e4,9];
 assert_eq!(project(good,9,None,0x10000,1,0,true),Err(Error::MissingFactoryResult));
 assert_eq!(project(good,9,Some(42),0x20000,1,0,true).unwrap().provider_return,42);
 assert_eq!(project(good,9,Some(42),0x10001,1,0,true),Err(Error::UnreadableComponent));
 assert_eq!(project(good,9,Some(42),0x10001,0,0,true).unwrap().provider_return,0);
 assert_eq!(project(good,9,Some(42),0x10000,1,0,false),Err(Error::Unadmitted));
}}
