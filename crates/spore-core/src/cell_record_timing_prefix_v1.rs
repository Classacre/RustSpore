//! Bounded E7E6F0 binary32 subtraction and scripted-helper branch.
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error { Unadmitted, UnsupportedFloat, UnsupportedControl }
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub struct Outcome { pub remaining_before_helper:u32, pub pending_event_path:bool, pub pending_other_path:bool }
/// Caller admits record lifetime, source entry, and the active MXCSR context.
/// Inputs and result must be finite binary32 normals or zeros; helper output is
/// explicitly supplied. The remaining value is before the helper, which may
/// mutate state. Neither pending path executes the real helper or scheduler.
pub fn step(old:u32,delta:u32,helper_al:u8,mxcsr:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 fn supported(v:u32)->bool{let e=v&0x7f800000;let f=v&0x007fffff;e!=0x7f800000 && (e!=0 || f==0)}
 if !supported(old)||!supported(delta){return Err(Error::UnsupportedFloat)}
 let remaining=(f32::from_bits(old)-f32::from_bits(delta)).to_bits();
 if !supported(remaining){return Err(Error::UnsupportedFloat)}
 Ok(Outcome{remaining_before_helper:remaining,pending_event_path:helper_al==0,pending_other_path:helper_al!=0})
}
#[cfg(test)]mod tests{use super::*;#[test]fn branch_and_restrictions(){let x=step(0x3f800000,0x3e800000,0,0x1f80,true).unwrap();assert_eq!(x.remaining_before_helper,0x3f400000);assert!(x.pending_event_path);assert!(step(0x3e800000,0x3f000000,2,0x1f80,true).unwrap().pending_other_path);assert_eq!(step(1,0,0,0x1f80,true),Err(Error::UnsupportedFloat));assert_eq!(step(0,0,0,0,true),Err(Error::UnsupportedControl));assert_eq!(step(0,0,0,0x1f80,false),Err(Error::Unadmitted));}}
