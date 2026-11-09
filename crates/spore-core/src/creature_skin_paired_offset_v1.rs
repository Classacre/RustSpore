//! Conditional composition of checked atlas shelf pairing and row advancement.
//! Supplied groups/shelves, empty origin, source-order row boundaries and a
//! successful owned calculation are admissions. Native allocator/lifecycle,
//! omitted earlier groups, actual atlas placement/texture and live identity are
//! not inferred. Numeric domain follows pair/offset normal-or-zero restrictions.
use crate::{creature_skin_atlas_offset_v1 as offset,creature_skin_atlas_pair_scale_v1::{self as pair,Row},native_sse::{self,Policy,SseError}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Pair(pair::Error),Offset(offset::Error),NonFiniteOrSubnormal,UnsupportedArchitecture}
impl From<SseError> for Error{fn from(_:SseError)->Self{Self::UnsupportedArchitecture}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Placement{pub group:u32,pub row:u32,pub width:u32,pub height:u32,pub offset:[u32;2],pub row_x_after:u32,pub row_y_after:u32}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Output{pub pairing:pair::Output,pub placements:Vec<Placement>}
fn admitted(x:f32)->Result<f32,Error>{if x.is_finite()&&!x.is_subnormal(){Ok(x)}else{Err(Error::NonFiniteOrSubnormal)}}
/// Pair supplied rows and advance row coordinates in source operation order.
pub fn compose(groups:&[[u32;4]],rows:&[Row],policy:Policy)->Result<Output,Error>{
 let pairing=pair::pair(groups,rows,policy).map_err(Error::Pair)?;
 native_sse::scoped(policy,||{
  let mut placements=Vec::new();let mut row_y=0f32;
  for (row_index,row) in pairing.rows.iter().enumerate(){let mut row_x=0f32;
   for (i,&group) in row.groups.iter().enumerate(){let v=offset::project(groups[group as usize],row.height,row.shift,row_x.to_bits(),row_y.to_bits(),row_index as u32,policy).map_err(Error::Offset)?;
    row_x=admitted(row_x+f32::from_bits(v.width))?;
    if i+1==row.groups.len(){let step=admitted(f32::from_bits(row.height)+f32::from_bits(row.shift))?;row_y=admitted(step+row_y)?;}
    placements.push(Placement{group,row:row_index as u32,width:v.width,height:v.height,offset:v.offset,row_x_after:row_x.to_bits(),row_y_after:row_y.to_bits()});
   }
  }
  Ok(Output{pairing,placements})
 })
}
#[cfg(all(test,any(target_arch="x86",target_arch="x86_64")))]
mod tests{
 use super::*;
 #[test]fn paired_row_order_and_coordinate_advances(){let b=|x:f32|x.to_bits();let groups=[[0,b(1.),0,b(0.5)];4];let rows=[Row{groups:vec![0,1],width:b(2.),height:b(0.5),shift:0},Row{groups:vec![2,3],width:b(2.),height:b(0.5),shift:0}];let o=compose(&groups,&rows,Policy::Mxcsr1f80).unwrap();assert_eq!(o.placements.iter().map(|p|p.group).collect::<Vec<_>>(),vec![1,0,2,3]);assert_eq!((o.placements[0].row_x_after,o.placements[0].row_y_after),(b(1.),0));assert_eq!((o.placements[1].row_x_after,o.placements[1].row_y_after),(b(2.),b(0.5)));assert_eq!((o.placements[3].row_x_after,o.placements[3].row_y_after),(b(2.),b(1.)));}
 #[test]fn invalid_input_propagates(){assert!(matches!(compose(&[],&[],Policy::Mxcsr1f80),Err(Error::Pair(pair::Error::InvalidCount))));}
}
