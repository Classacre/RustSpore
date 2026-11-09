//! Controlled finite BBA700 endpoint comparison. No claim about CRT sort,
//! merge history, native floating-point flags, or live registration.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {NonFiniteOrSubnormal}
/// Source returns -1 for first < second and +1 otherwise; it never returns 0.
/// Finite normal or zero inputs avoid unordered/special exception cases.
pub fn compare(first:u32,second:u32)->Result<i32,Error>{
 let a=f32::from_bits(first);let b=f32::from_bits(second);
 if !(a.is_normal()||a==0.)||!(b.is_normal()||b==0.){return Err(Error::NonFiniteOrSubnormal)}
 Ok(if b>a{-1}else{1})
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn finite_order_and_equal_zero_are_explicit(){let b=|x:f32|x.to_bits();assert_eq!(compare(b(-2.),b(1.)),Ok(-1));assert_eq!(compare(b(1.),b(-2.)),Ok(1));assert_eq!(compare(0,0x80000000),Ok(1));assert_eq!(compare(b(2.),b(2.)),Ok(1));assert_eq!(compare(1,0),Err(Error::NonFiniteOrSubnormal));assert_eq!(compare(f32::NAN.to_bits(),0),Err(Error::NonFiniteOrSubnormal));}
}
