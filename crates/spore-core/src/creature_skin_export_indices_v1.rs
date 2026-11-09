//! Bounded 46E821..46E83A index adjustment with successful append admitted.
//! The caller supplies each group's vertex count and the initial global base;
//! four preceding groups, allocation/lifecycle and live identity are excluded.
#[derive(Debug,PartialEq,Eq)]pub enum Error{MismatchedGroupCounts,InvalidLocalIndex}
/// Original ADD wraps modulo 2^32. This does not establish a valid live buffer.
pub fn adjust(local:u32,base:u32)->u32{local.wrapping_add(base)}
/// Concatenate supplied local index streams using each group's vertex count.
/// Reject indices that would not address the caller-supplied group vertices.
pub fn project(groups:&[&[u32]],vertex_counts:&[u32],initial_base:u32)->Result<Vec<u32>,Error>{
 if groups.len()!=vertex_counts.len(){return Err(Error::MismatchedGroupCounts)}
 let mut out=Vec::new();let mut base=initial_base;
 for (indices,&count) in groups.iter().zip(vertex_counts){
  if indices.iter().any(|&x|x>=count){return Err(Error::InvalidLocalIndex)}
  out.extend(indices.iter().map(|&x|adjust(x,base)));base=base.wrapping_add(count);
 }
 Ok(out)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn preserves_group_order_and_supplied_base(){assert_eq!(project(&[&[0,2,1],&[1,0]],&[3,2],7),Ok(vec![7,9,8,11,10]));}
 #[test]fn wraps_and_rejects_invalid_groups(){assert_eq!(adjust(u32::MAX,1),0);assert_eq!(project(&[&[0,1],&[0]],&[2,1],u32::MAX),Ok(vec![u32::MAX,0,1]));assert_eq!(project(&[&[2]],&[2],0),Err(Error::InvalidLocalIndex));assert_eq!(project(&[&[]],&[],0),Err(Error::MismatchedGroupCounts));}
}
