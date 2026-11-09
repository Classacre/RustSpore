//! Checked E833B0 data4 lookup and E60B0D camera-vector continuation.
use crate::{cell_runtime_data::{CellRuntimeData,CellRuntimeDataError,DATA4_ID},native_x87::Context};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, UnsupportedControl, UnsupportedFloat, Reader(CellRuntimeDataError) }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Outcome { pub found:bool, pub value:u32, pub vector:[u32;3], pub continuation_pending:bool }

/// Inputs are stable, disjoint aggregate bytes and explicit current global and
/// stack vectors. The data4 reader uses its first matching row and writes +0
/// when missing. CW 027F/MXCSR 1F80 and finite normal/zero SSE values are
/// required. Provider ownership, live animation state and the later E60C66
/// continuation are not established or executed.
pub fn project(aggregate:&[u8],old:[u32;3],global:[u32;3],cw:u16,mxcsr:u32,admitted:bool)->Result<Outcome,Error> {
 if !admitted{return Err(Error::Unadmitted)}
 if cw!=0x027f||mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let context=Context::from_control_word(cw).map_err(|e|Error::Reader(CellRuntimeDataError::X87(e)))?;
 let sample=CellRuntimeData::from_aggregate_a0(aggregate).map_err(Error::Reader)?.lookup(DATA4_ID,context).map_err(Error::Reader)?;
 fn valid(x:f32)->bool{x.is_finite()&&(x==0.0||x.is_normal())}
 fn op(x:f32)->Result<f32,Error>{if valid(x){Ok(x)}else{Err(Error::UnsupportedFloat)}}
 let o=old.map(f32::from_bits);let g=global.map(f32::from_bits);let value=sample.value;
 if !valid(value)||!o.iter().chain(g.iter()).all(|&x|valid(x)){return Err(Error::UnsupportedFloat)}
 let limit=f32::from_bits(0x3f733333);let q=if value>limit{limit}else{value};
 let dx=op(g[0]-o[0])?;let dy=op(g[1]-o[1])?;let dz=op(g[2]-o[2])?;
 let x=op(q*dx)?;let y=op(q*dy)?;let z=op(q*dz)?;
 Ok(Outcome{found:sample.found,value:value.to_bits(),vector:[op(o[0]+x)?.to_bits(),op(y+o[1])?.to_bits(),op(z+o[2])?.to_bits()],continuation_pending:true})
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn missing_and_bounds(){let old=[1f32.to_bits(),2f32.to_bits(),3f32.to_bits()];let global=[5f32.to_bits(),8f32.to_bits(),11f32.to_bits()];
  assert_eq!(project(&[0;4],old,global,0x027f,0x1f80,true).unwrap().vector,old);
  assert_eq!(project(&[0;4],old,global,0x027f,0x1f80,false),Err(Error::Unadmitted));
  assert_eq!(project(&[0;4],old,global,0x037f,0x1f80,true),Err(Error::UnsupportedControl));
  assert_eq!(project(&[0;4],[1,0,0],global,0x027f,0x1f80,true),Err(Error::UnsupportedFloat));
  assert!(matches!(project(&[1,0,0,0],old,global,0x027f,0x1f80,true),Err(Error::Reader(_))));
 }
}
