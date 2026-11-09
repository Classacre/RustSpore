//! Bounded E60C66 global-vector setup and original ramp projection.
use crate::{cell_swimming::{retained_ramp,CellSwimmingError},native_x87::Context};
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedControl, UnsupportedFloat, Helper(CellSwimmingError) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub flags:u32, pub vector:[u32;3], pub helper_value:u32, pub output:[u32;3], pub next_handoff_pending:bool }

/// Caller supplies the current stable flag/vector and stack value; no live
/// global or camera identity follows. CW 027F and MXCSR 1F80 are required.
/// Only reached finite binary32 normal/zero values and results are admitted;
/// the initializer branch does not inspect the supplied old vector.
pub fn project(flags:u32,old:[u32;3],input:u32,cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}if cw!=0x027f||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 fn valid(x:f32)->bool{x.is_finite()&&(x==0.0||x.is_normal())}
 let x=f32::from_bits(input);if !valid(x){return Err(Error::UnsupportedFloat)}
 let vector=if flags&2==0{[0,0xbf000000,0]}else{old};let v=vector.map(f32::from_bits);
 if !v.iter().all(|&x|valid(x)){return Err(Error::UnsupportedFloat)}
 let ctx=Context::from_control_word(cw).map_err(|e|Error::Helper(CellSwimmingError::X87(e)))?;
 let retained=retained_ramp(ctx,20.0,0.0,x,1.0,0.0).map_err(Error::Helper)?;
 let q=ctx.store_f32(retained).map_err(|e|Error::Helper(CellSwimmingError::X87(e)))?;
 if !valid(q){return Err(Error::UnsupportedFloat)}
 let result=[q*v[0],q*v[1],v[2]*q];if !result.iter().all(|&x|valid(x)){return Err(Error::UnsupportedFloat)}
 Ok(Outcome{flags:flags|2,vector,helper_value:q.to_bits(),output:result.map(f32::to_bits),next_handoff_pending:true})
}
#[cfg(test)]mod tests{use super::*;#[test]fn flag_and_admissions(){let old=[2f32.to_bits(),(-3f32).to_bits(),4f32.to_bits()];let x=10f32.to_bits();
 let init=project(0,[1,1,1],x,0x027f,0x1f80,true).unwrap();assert_eq!(init.vector,[0,0xbf000000,0]);assert_eq!(init.flags,2);
 assert_eq!(project(0x82,old,x,0x027f,0x1f80,true).unwrap().flags,0x82);
 assert_eq!(project(2,old,x,0x027f,0x1f80,false),Err(Error::Unadmitted));
 assert_eq!(project(2,old,x,0x037f,0x1f80,true),Err(Error::UnsupportedControl));
 assert_eq!(project(2,old,1,0x027f,0x1f80,true),Err(Error::UnsupportedFloat));
}}
