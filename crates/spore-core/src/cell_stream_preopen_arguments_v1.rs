//! Bounded 8D8105 pre-open argument preparation, before virtual +4C.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub remapped_before_callback:u32,pub pending_arguments:[u32;4],pub preparatory_calls:u32}
/// EAX is the admitted preceding first-argument bit-1 result. Source remaps
/// mode 6 before the synthetic +3C call. `mode_after` and `field_14_after` are
/// fresh snapshots after that normally returning nonreentrant callback. Stable
/// stream/vtable/frame and callback effects are caller admissions; +4C remains
/// pending, and no real open, file or ownership follows.
pub fn project(eax_bit:u32,mode_before:u32,mode_after:u32,field_14_after:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let mapped=if mode_before==6{if eax_bit==0{3}else{4}}else{mode_before};
 Ok(Outcome{remapped_before_callback:mapped,pending_arguments:[field_14_after,mode_after,1,2],preparatory_calls:1})
}
#[cfg(test)]mod tests{use super::*;#[test]fn mapping_and_fresh(){assert_eq!(project(0,6,9,7,true).unwrap(),Outcome{remapped_before_callback:3,pending_arguments:[7,9,1,2],preparatory_calls:1});assert_eq!(project(2,6,4,0,true).unwrap().remapped_before_callback,4);assert_eq!(project(0,5,5,0,false),Err(Error::Unadmitted));}}
