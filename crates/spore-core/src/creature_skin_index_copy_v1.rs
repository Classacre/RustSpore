//! Bounded 541380 triangle-index copy into preallocated destination storage.
#[derive(Debug,PartialEq,Eq)]pub enum Error{InsufficientCapacity}
/// Copies complete 32-bit index words without reordering, returning the number
/// of destination words advanced. Native valid nonoverlapping memcpy pointers,
/// successful storage and whole-word source framing are admissions; the
/// capacity error is a checked model boundary, not native failure behavior.
pub fn copy_indices(source:&[u32],destination:&mut [u32])->Result<usize,Error>{
 if destination.len()<source.len(){return Err(Error::InsufficientCapacity)}
 destination[..source.len()].copy_from_slice(source);Ok(source.len())
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn copies_in_order_and_preserves_tail(){let mut dst=[9;5];assert_eq!(copy_indices(&[3,1,4],&mut dst),Ok(3));assert_eq!(dst,[3,1,4,9,9]);}
 #[test]fn rejects_short_buffer_before_mutation(){let mut dst=[7;1];assert_eq!(copy_indices(&[1,2],&mut dst),Err(Error::InsufficientCapacity));assert_eq!(dst,[7]);}
}
