//! Controlled startup bounds through E5F450 and guarded E5F840 enumeration.
use crate::{cell_start_bounds_v1, native_sse::{self, Policy, SseError}};

#[derive(Debug, PartialEq)]
pub enum Error { Unadmitted, Bounds(cell_start_bounds_v1::Error), Sse(SseError), Range }
impl From<SseError> for Error { fn from(e:SseError)->Self { Self::Sse(e) } }

/// Exactly the fixture's source start vector, progress 0, level 3, +514C=1,
/// +68 of 0 or 15 and type 0 or 1. The helper's 0.125 result is admitted from
/// the separately retained source fixture. Stable synthetic resolver, globals,
/// and PC53-equivalent arithmetic for this fixed product are admitted. The
/// pre-loop range/capacity gate and zero-initialized fifth word are harness
/// controls; no pool allocation, active provider, or live placement is modeled.
pub fn project(value_68:u32, kind:u32, policy:Policy, admitted:bool)->Result<Vec<[u32;5]>,Error>{
 if !admitted || ![0,15f32.to_bits()].contains(&value_68) || kind>1 {return Err(Error::Unadmitted)}
 let bounds=cell_start_bounds_v1::project(value_68,policy,true).map_err(Error::Bounds)?;
 native_sse::scoped(policy,||{
  use std::hint::black_box as b;
  let v=bounds.map(f32::from_bits);
  let (xmin,ymin,xmax,ymax)=if kind==0 {
   let cx=b(b(v[3]+v[0])*0.5);let cy=b(b(v[4]+v[1])*0.5);
   let dx=b(v[3]-v[0]);let dy=b(v[4]-v[1]);
   (b(cx-dx),b(cy-dy),b(dx+cx),b(dy+cy))
  } else {(v[6],v[7],v[8],v[9])};
  let step=((f32::from_bits(0x3f59999a) as f64)*0.125f64*300.0f64) as f32;
  let inverse=b(1.0f32/step);
  let limits=[b(inverse*xmin).floor() as i32,b(inverse*ymin).floor() as i32,
              b(inverse*xmax).ceil() as i32,b(inverse*ymax).ceil() as i32];
  let [x0,y0,x1,y1]=limits;
  if x0<0 || y0<0 || x1>4 || y1>4 || x1<x0 || y1<y0 || (x1-x0)*(y1-y0)>16 {return Err(Error::Range)}
  let mut out=Vec::new();for x in x0..x1 {for y in y0..y1 {out.push([kind,3,x as u32,y as u32,0]);}}
  Ok(out)
 })
}
#[cfg(test)] mod tests {use super::*;#[test]fn bounded_cases(){for kind in 0..=1 {for v in [0,15f32.to_bits()] {let x=project(v,kind,Policy::Mxcsr1f80,true).unwrap();assert_eq!(x.len(),4);assert_eq!(x[0],[kind,3,2,2,0]);assert_eq!(x[3],[kind,3,3,3,0]);}}assert_eq!(project(1,0,Policy::Mxcsr1f80,true),Err(Error::Unadmitted));}}
