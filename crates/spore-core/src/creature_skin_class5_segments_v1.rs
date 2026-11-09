//! Leading class-5 builder-segment descriptors. These are inputs to 4FD3C0,
//! not the sampled lanes that function can append.
use crate::creature_generated_skin_edges_v1::{ROW_SIZE, MAX_ROWS};
#[derive(Clone, Copy, Debug)]
pub struct FrozenRowsAdmission;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment { pub from_node: usize, pub to_node: usize, pub spacing: f32, pub key: u32 }
#[derive(Debug, PartialEq, Eq)]
pub enum Error { NoFlaggedRow, TooManyRows, AllocationFailed }
/// Models 413D70's leading class-5 segment appends, assuming an initially
/// empty builder and successful native vector allocation. The node indices
/// refer only to that builder's direct node prefix, not a palette or mesh.
pub fn build(rows: &[[u8; ROW_SIZE]], _: FrozenRowsAdmission) -> Result<Vec<Segment>, Error> {
 if rows.len()>MAX_ROWS {return Err(Error::TooManyRows);}
 if !rows.iter().any(|r|u16::from_le_bytes([r[8],r[9]]) & 1 != 0) {return Err(Error::NoFlaggedRow);}
 let count=rows.iter().take_while(|r|r[11]==5).count();
 let mut segments=Vec::new(); segments.try_reserve_exact(count.saturating_sub(1)).map_err(|_| Error::AllocationFailed)?;
 for i in 1..count {
  let previous=i16::from_le_bytes([rows[i-1][2],rows[i-1][3]]) as i32 as u32;
  segments.push(Segment {from_node:i-1,to_node:i,spacing:f32::from_bits(0x3e99999a),key:previous|0x80000000});
 }
 Ok(segments)
}
#[cfg(test)]
mod tests {
 use super::*;
 fn row(class:u8,link:i16,flag:bool)->[u8;ROW_SIZE] { let mut r=[0;ROW_SIZE];r[11]=class;r[2..4].copy_from_slice(&link.to_le_bytes());r[8]=flag as u8;r }
 #[test] fn contiguous_nodes_and_previous_signed_key() {
  let r=[row(5,7,false),row(5,-2,false),row(5,9,false),row(3,2,true),row(5,1,false)];
  assert_eq!(build(&r,FrozenRowsAdmission).unwrap(),vec![Segment{from_node:0,to_node:1,spacing:f32::from_bits(0x3e99999a),key:0x80000007},Segment{from_node:1,to_node:2,spacing:f32::from_bits(0x3e99999a),key:0xfffffffe}]);
 }
 #[test] fn gate_and_short_prefix() {
  assert_eq!(build(&[row(5,0,false)],FrozenRowsAdmission),Err(Error::NoFlaggedRow));
  assert!(build(&[row(3,0,true)],FrozenRowsAdmission).unwrap().is_empty());
  assert!(build(&[row(5,0,true)],FrozenRowsAdmission).unwrap().is_empty());
 }
}
