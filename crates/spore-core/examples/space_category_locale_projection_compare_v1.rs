//! Explicit en-us snapshot and supplied category prefixes; no live locale/UI claim.
use sha2::{Digest,Sha256};
use spore_core::{content_manifest::LocaleOutputState,locale_table,space_category_locale_projection_v1::{project,Outcome}};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut args=std::env::args_os().skip(1);
 let source=std::fs::read(args.next().ok_or("pass 520-byte source snapshot")?)?;
 let table_bytes=std::fs::read(args.next().ok_or("pass explicit locale table snapshot")?)?;
 if args.next().is_some()||source.len()!=520||table_bytes.len()!=3311{return Err("unexpected input length".into());}
 if format!("{:x}",Sha256::digest(&source))!="a44c39b9e511a14bce60dc8e8f7bbd99d7dc7609cbdd20bdf9be41005d8b06d1" ||
    format!("{:x}",Sha256::digest(&table_bytes))!="1455cc9b792b25807017551696dc247a9e3d1576cfb9ae2aeeadc0c22b6da24e" {return Err("input digest differs".into());}
 let table=locale_table::parse(&table_bytes)?;
 let state=LocaleOutputState{cached_flag:false,cached_pointer_present:false,classifier_nonzero:false,global_nonzero:false};
 // First prefix is from the full source snapshot; remaining five are explicitly
 // supplied from the separately pinned ordered graph comparison.
 let cases: [(u32, &str); 6]=[(0x20012,"All"),(0x2001c,"Planets"),(0x20008,"Creatures"),(0x20002,"Buildings"),(0x20019,"Vehicles"),(0x2000e,"Flora")];
 for (i,(id,want)) in cases.into_iter().enumerate(){
  let mut prefix=[0u8;8];prefix[..4].copy_from_slice(&0x21671745u32.to_le_bytes());prefix[4..].copy_from_slice(&id.to_le_bytes());
  if i==0 && source[..8]!=prefix{return Err("source prefix differs".into());}
  let expected:Vec<u16>=want.encode_utf16().collect();
  if project(&prefix,0x21671745,&table,Some(state))?!=Outcome::Raw(&expected){return Err(format!("category {i} differs").into());}
 }
 println!("matched six explicit en-us raw locale comparisons under supplied raw-return state; no live locale or UI claim");Ok(())
}
