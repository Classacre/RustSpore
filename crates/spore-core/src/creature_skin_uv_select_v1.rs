//! Bounded bit-preserving 50A7B0 coordinate selection for a supplied group.
//! Input position words and group are explicit; no actual UV group, cache,
//! projection allocation or live creature identity is inferred.
/// Select two coordinate words and optionally flip the sign bit of the first.
pub fn select(position_bits: [u32;3], group: u32) -> [u32;2] {
    let axis=group >> 1;
    let u=position_bits[((axis+1)%3) as usize] ^ if group & 1 != 0 {0x80000000} else {0};
    let v=position_bits[((axis+2)%3) as usize];
    [u,v]
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn axes_sign_and_raw_payloads() {
  let p=[0x00000000,0x80000000,0x7fc12345];
  assert_eq!(select(p,0),[0x80000000,0x7fc12345]);
  assert_eq!(select(p,1),[0,0x7fc12345]);
  assert_eq!(select(p,5),[0x80000000,0x80000000]);
  assert_eq!(select(p,0xffffffff),[0xffc12345,0]);
 }
}
