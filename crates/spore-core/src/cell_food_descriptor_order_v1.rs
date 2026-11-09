//! Bounded 6928E0 descriptor enumeration and selected food/plant writers.
use crate::cell_food_writer_dispatch_v1 as writer;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Descriptor{pub identity:u32,pub key:u32,pub offset:u32,pub base:u32,pub has_read:bool,pub has_write:bool}
#[derive(Debug,PartialEq,Eq)]pub enum Error{Unadmitted,MissingTerminator,Address(writer::Error)}
#[derive(Debug,PartialEq,Eq)]pub struct Entry{pub key:u32,pub descriptor:u32,pub saved_base:u32}
/// Enumerates a finite supplied descriptor prefix until either callback is null
/// or source's 128-entry limit is reached. Descriptor storage, callback identity,
/// nonaliasing and lifetimes are admitted; this does not establish the live table.
pub fn enumerate(rows:&[Descriptor],admitted:bool)->Result<Vec<Entry>,Error>{
 if !admitted{return Err(Error::Unadmitted)}let mut result=Vec::new();for row in rows{if !row.has_read||!row.has_write{return Ok(result)}if result.len()==128{return Ok(result)}result.push(Entry{key:row.key,descriptor:row.identity,saved_base:row.base});}Err(Error::MissingTerminator)
}
/// Selected writers use the source-built order and supplied fresh field values.
/// The outer source writer loop, actual callbacks/files and live root ownership
/// are not established. Stable synthetic fields and successful writes are admitted.
pub fn write_selected(rows:&[Descriptor],manager_base:u32,fields:&[u32],admitted:bool)->Result<Vec<u8>,Error>{
 if !admitted{return Err(Error::Unadmitted)}if rows.len()!=fields.len(){return Err(Error::MissingTerminator)}let mut bytes=Vec::new();for (row,field) in rows.iter().zip(fields){let result=writer::project(row.base,manager_base,row.offset,*field,true).map_err(Error::Address)?;bytes.extend(result.encoded);}Ok(bytes)
}
#[cfg(test)]mod tests{use super::*;#[test]fn bounded_prefix(){let a=Descriptor{identity:1,key:2,offset:0x1c,base:0,has_read:true,has_write:true};let end=Descriptor{has_read:false,..a};assert_eq!(enumerate(&[a,end],true).unwrap().len(),1);assert_eq!(write_selected(&[a],0x1000,&[1000],true).unwrap(),[0,0,3,0xe8]);assert_eq!(enumerate(&[a],true),Err(Error::MissingTerminator));}}

