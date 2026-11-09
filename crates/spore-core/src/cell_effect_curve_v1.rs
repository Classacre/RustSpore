//! Bounded E50080 model for the two authenticated effect-map rows.
//! The scalar and x87 context are supplied; no active provider, pool or state is inferred.
use crate::native_x87::{Context,Extended,X87Error};
use crate::native_sse::Policy;
use crate::cell_start_table_v1::{self,Row};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Curve { Near, Far }
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Error { Input, X87(X87Error), Start(cell_start_table_v1::Error) }
impl From<X87Error> for Error { fn from(e:X87Error)->Self {Self::X87(e)} }
impl From<cell_start_table_v1::Error> for Error { fn from(e:cell_start_table_v1::Error)->Self {Self::Start(e)} }
fn ratio(x:f32,start:f32,end:f32,ctx:Context)->Result<Extended,Error> {
 let x=Extended::from_f32(x)?;let start=Extended::from_f32(start)?;let end=Extended::from_f32(end)?;
 let numerator=ctx.sub(x,start)?;let denominator=ctx.add(ctx.sub(end,start)?,Extended::from_f32(f32::from_bits(0x37800000))?)?;
 Ok(ctx.div(numerator,denominator)?)
}
/// Finite nonnegative caller scalar; returns the float32 committed by the
/// controlled host. Native x87 sticky flags and live resource choice are excluded.
pub fn evaluate(curve:Curve,x:f32,ctx:Context)->Result<f32,Error>{
 if !x.is_finite() || x<0.0 {return Err(Error::Input)}
 let one=Extended::from_i32(1);
 let value=match curve {
  Curve::Near=>{
   let tmp=ctx.store_f32(ratio(x,1.5,f32::from_bits(0x3ff33333),ctx)?)?;
   if !tmp.is_finite(){return Err(Error::Input)}
   ctx.sub(one,Extended::from_f32(tmp.clamp(0.0,1.0))?)?
  },
  Curve::Far=>{
   if x<5700.0 || x>=22000.0 {Extended::from_i32(0)}
   else if x<14950.0 {ratio(x,5700.0,14950.0,ctx)?}
   else if x<20000.0 {one}
   else {ctx.sub(one,ratio(x,20000.0,22000.0,ctx)?)?}
  }
 };
 Ok(ctx.store_f32(value)?)
}
/// Compose the separately bounded start-table projection with the two curves.
/// Caller admits the same stable +514C, selected installed rows, nonaliasing,
/// and valid resource lifetimes. No pool or runtime identity is selected.
pub fn from_start_table(progress:i32,rows:&[Row],special_bits:u32,numerator_bits:u32,policy:Policy,ctx:Context)->Result<(f32,f32,f32),Error>{
 if !(0..=1000).contains(&progress){return Err(Error::Input)}
 let start=cell_start_table_v1::evaluate(progress,rows,special_bits,numerator_bits,policy)?;
 let x=f32::from_bits(start.global_514c_bits);
 Ok((x,evaluate(Curve::Near,x,ctx)?,evaluate(Curve::Far,x,ctx)?))
}
#[cfg(test)] mod tests{
 use super::*;
 #[test] fn source_endpoints_and_rejections(){let c=Context::from_control_word(0x037f).unwrap();
  for (x,a,b) in [(0.5,1.,0.),(1.5,1.,0.),(5.,0.,0.),(5700.,0.,0.),(15000.,0.,1.),(22000.,0.,0.)]{assert_eq!(evaluate(Curve::Near,x,c),Ok(a));assert_eq!(evaluate(Curve::Far,x,c),Ok(b));}
  assert_eq!(evaluate(Curve::Near,f32::NAN,c),Err(Error::Input));assert_eq!(evaluate(Curve::Far,-1.,c),Err(Error::Input));
 }
}
