//! Bounded E61A00 CellSaveGame base-object allocation callback.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Unadmitted,UnsupportedControl}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Outcome{pub storage:[u8;0xec],pub returned_object:bool}
/// Synthetic allocator returns the supplied distinct storage on success, or
/// null on failure; successful allocation size and lifetimes are admitted.
/// Only writes made by E61A00 are projected. E61390's subsequent fresh-save
/// initialization, actual allocation, registration and live identity are not.
pub fn project(mut storage:[u8;0xec],success:bool,cw:u16,mx:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 if !matches!(cw,0x027f|0x037f)||mx!=0x1f80{return Err(Error::UnsupportedControl)}
 if success {for (o,v) in [(0,0x01486048u32),(4,0x0140e924),(8,0),(0x10,0),(0x14,0),(0x18,0)]{storage[o..o+4].copy_from_slice(&v.to_le_bytes());}}
 Ok(Outcome{storage,returned_object:success})
}
#[cfg(test)]mod tests{use super::*;#[test]fn success_and_null_keep_unwritten_bytes(){let b=[0xa5;0xec];let x=project(b,true,0x27f,0x1f80,true).unwrap();assert_eq!(&x.storage[..4],&0x01486048u32.to_le_bytes());assert_eq!(x.storage[0xc],0xa5);assert_eq!(x.storage[0x1c],0xa5);let x=project(b,false,0x37f,0x1f80,true).unwrap();assert_eq!(x.storage,b);assert!(!x.returned_object);assert_eq!(project(b,true,0x27f,0x1f80,false),Err(Error::Unadmitted));}}
