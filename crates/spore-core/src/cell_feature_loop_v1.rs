//! Checked synthetic E6ADE0 feature-loop skeleton. No geometry, source cache,
//! callbacks, contact acceptance or live actor ownership is inferred.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unstable,NegativeCount,CountExceedsRows,CountExceedsLimit}
/// Signed counts and 72-byte row class words are caller-admitted snapshots;
/// rejecting negative counts is an extra fixture restriction (native treats
/// nonpositive counts as empty). Each count is capped at 64 as a fixture limit.
/// The result retains original row indices and source outer/inner order.
pub fn pairs(counts:[i32;2],classes:[&[u32];2],stable:bool)->Result<Vec<(u32,u32)>,Error>{
 if !stable{return Err(Error::Unstable)}
 let mut n=[0usize;2];for i in 0..2{if counts[i]<0{return Err(Error::NegativeCount)}n[i]=counts[i] as usize;if n[i]>64{return Err(Error::CountExceedsLimit)}if n[i]>classes[i].len(){return Err(Error::CountExceedsRows)}}
 let mut out=Vec::new();for (i,&a) in classes[0][..n[0]].iter().enumerate(){if a==1{continue}for (j,&b) in classes[1][..n[1]].iter().enumerate(){if b!=1{out.push((i as u32,j as u32))}}}Ok(out)
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn retains_original_indices_and_nested_order(){assert_eq!(pairs([3,3],[&[0,1,2],&[1,0,2]],true).unwrap(),[(0,1),(0,2),(2,1),(2,2)]);}
 #[test]fn rejects_unstable_negative_short_and_oversize(){assert_eq!(pairs([0,0],[&[],&[]],false),Err(Error::Unstable));assert_eq!(pairs([-1,0],[&[],&[]],true),Err(Error::NegativeCount));assert_eq!(pairs([1,0],[&[],&[]],true),Err(Error::CountExceedsRows));assert_eq!(pairs([65,0],[&[0;65],&[]],true),Err(Error::CountExceedsLimit));}
}
