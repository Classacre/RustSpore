//! Compare one fixed full-PROP source snapshot and native host row.
//! Caller admits stable files; native stream/allocation checks are host guards.
use sha2::{Digest,Sha256};
use spore_core::space_category_accessor_loop_v1::project;
fn be32(b:&[u8],at:usize)->u32 {u32::from_be_bytes(b[at..at+4].try_into().unwrap())}
fn main()->Result<(),Box<dyn std::error::Error>> {
 let mut args=std::env::args_os().skip(1);
 let root=std::fs::read(args.next().ok_or("pass full PROP path")?)?;
 let native=std::fs::read(args.next().ok_or("pass native stdout path")?)?;
 if args.next().is_some() || root.len()!=616 || native.len()!=44 {return Err("expected 616-byte PROP and 44-byte native row".into());}
 let digest=format!("{:x}",Sha256::digest(&root));
 if digest!="306be594fb294905b6d48342fbf71c1b34956f183b392814abdcb92a89ec864a" {return Err("full PROP digest differs".into());}
 let header=[(0,2),(4,0x7435a2d0),(8,0x0022001c),(12,1),(16,0x208),(540,0x7435a2d3),(544,0x0020009c),(548,5),(552,12)];
 if header.iter().any(|&(at,value)|be32(&root,at)!=value) {return Err("unexpected two-property shape".into());}
 // For these two sorted entries, the retained original lower-bound lookup finds D3 second.
 let ids=[be32(&root,4),be32(&root,540)];
 let index=ids.binary_search(&0x7435a2d3).map_err(|_|"missing property")?;
 if index!=1 {return Err("unexpected property index".into());}
 let mut records=[[0u32;3];5];
 for (i,record) in records.iter_mut().enumerate(){for (j,word) in record.iter_mut().enumerate(){let at=556+i*12+j*4;*word=u32::from_le_bytes(root[at..at+4].try_into()?);}}
 let outcome=project(true,0x20,0x9c,5,&records)?;
 let mut row=[0u32;11];for (i,word) in row.iter_mut().enumerate(){*word=u32::from_le_bytes(native[i*4..i*4+4].try_into()?);}
 if row[..2]!=[1,5] || row[2..7]!=outcome.visited_first_words || row[7..9]!=[5,1]
  || row[9]&0xffff!=0x027f || (row[9]>>16)&0x3840!=0 || row[10]&!0x3f!=0x1f80 {return Err("native row differs".into());}
 println!("matched full source-snapshot lookup/loop row; no live service claim");Ok(())
}
