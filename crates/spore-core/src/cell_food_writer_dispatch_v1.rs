//! Bounded 6923B4 descriptor getter and food-field writer dispatch.
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,AddressOverflow}
#[derive(Debug,PartialEq,Eq)]pub struct Outcome{pub descriptor_base:u32,pub field_address:u32,pub encoded:[u8;4]}
/// The caller admits stable disjoint descriptor, manager, field and synthetic
/// stream, valid callback pointers and normally returning nonreentrant calls.
/// `field` is fresh at the computed address. Null descriptor base takes manager
/// base; nonnull base is preserved. Actual enumeration, files, live save/root
/// identity and ownership remain unverified.
pub fn project(descriptor_base:u32,manager_base:u32,offset:u32,field:u32,admitted:bool)->Result<Outcome,Error>{
 if !admitted{return Err(Error::Unadmitted)}
 let base=if descriptor_base==0{manager_base}else{descriptor_base};let field_address=base.checked_add(offset).ok_or(Error::AddressOverflow)?;
 Ok(Outcome{descriptor_base:base,field_address,encoded:field.to_be_bytes()})
}
#[cfg(test)]mod tests{use super::*;#[test]fn bases(){let a=project(0,0x1000,0x1c,1000,true).unwrap();assert_eq!((a.descriptor_base,a.field_address,a.encoded),(0x1000,0x101c,[0,0,3,0xe8]));assert_eq!(project(u32::MAX,0,1,0,true),Err(Error::AddressOverflow));}}
