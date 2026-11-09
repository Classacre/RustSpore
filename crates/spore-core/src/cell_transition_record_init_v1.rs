//! Bounded E72E03 record initialization; no allocator or scheduler is run.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Unadmitted, UnsupportedFloat, UnsupportedControl }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Outcome { pub record:[u8;0x7c], pub root_5194:u32, pub pending_notification:bool }
/// Caller admits a resolved, nonaliasing record/root and stable global inputs.
/// Only the checked binary32 normal/zero and x87/MXCSR controls are accepted.
/// The source call to E598B0 remains pending; no allocation or scheduling occurs.
pub fn initialize(mut record:[u8;0x7c], float_bits:u32, vec_a:[u32;3], vec_b:[u32;4], handle:u32, x87:u16, mxcsr:u32, admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(x87,0x027f|0x037f) || mxcsr!=0x1f80{return Err(Error::UnsupportedControl)}
 let e=float_bits & 0x7f800000;let fraction=float_bits & 0x007fffff;
 if e==0x7f800000 || (e==0 && fraction!=0){return Err(Error::UnsupportedFloat)}
 fn put(r:&mut [u8;0x7c],o:usize,v:u32){r[o..o+4].copy_from_slice(&v.to_le_bytes())}
 for (o,v) in [(4,0),(0x1c,float_bits),(0x20,float_bits),(0x24,0x1f),(0x28,0),(0x2c,0),(0x30,u32::MAX),(0x34,u32::MAX),(0x38,0),(0x3c,0),(0x40,0),(0x44,0),(0x70,0),(0x78,0)]{put(&mut record,o,v)}
 record[8]=0;record[0x74]=0;
 for (o,v) in vec_a.into_iter().enumerate(){put(&mut record,0x48+o*4,v);put(&mut record,0x54+o*4,v)}
 for (o,v) in vec_b.into_iter().enumerate(){put(&mut record,0x60+o*4,v)}
 Ok(Outcome{record,root_5194:handle,pending_notification:true})
}
#[cfg(test)] mod tests{use super::*;#[test]fn preserves_unwritten_and_rejects_unadmitted_inputs(){let x=initialize([0xa5;0x7c],0x80000000,[1,2,3],[4,5,6,7],9,0x27f,0x1f80,true).unwrap();assert_eq!(&x.record[0x1c..0x20],&0x80000000u32.to_le_bytes());assert_eq!(&x.record[8..12],&[0,0xa5,0xa5,0xa5]);assert_eq!(x.root_5194,9);assert!(x.pending_notification);assert_eq!(initialize([0;0x7c],1,[0;3],[0;4],0,0x27f,0x1f80,true),Err(Error::UnsupportedFloat));assert_eq!(initialize([0;0x7c],0,[0;3],[0;4],0,0x27f,0x1f80,false),Err(Error::Unadmitted));}}
