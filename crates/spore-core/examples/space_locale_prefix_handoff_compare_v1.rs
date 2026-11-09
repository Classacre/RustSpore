//! Compare three guarded-page native rows; no live locale or UI claim.
use sha2::{Digest,Sha256};
use spore_core::space_locale_prefix_handoff_v1::project;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let mut args=std::env::args_os().skip(1);
 let data=std::fs::read(args.next().ok_or("pass 520-byte snapshot")?)?;
 let native=std::fs::read(args.next().ok_or("pass retained native stdout")?)?;
 if args.next().is_some()||data.len()!=520||native.len()!=84{return Err("unexpected snapshot or output length".into());}
 if format!("{:x}",Sha256::digest(&data))!="a44c39b9e511a14bce60dc8e8f7bbd99d7dc7609cbdd20bdf9be41005d8b06d1"{return Err("snapshot digest differs".into());}
 for i in 0..3 {
  let mut row=[0u32;7];for(j,w)in row.iter_mut().enumerate(){*w=u32::from_le_bytes(native[i*28+j*4..i*28+j*4+4].try_into()?);}
  let result=project(i!=2,if i==1 {0x21}else{0x22},0x9c,&data)?;
  let words=result.receiver_words.unwrap_or([0,0]);
  if row[..5]!=[1,u32::from(result.accepted),words[0],words[1],u32::from(result.accepted)]
    || row[5]&0xffff!=0x027f || (row[5]>>16)&0x3840!=0 || row[6]&!0x3f!=0x1f80 {return Err(format!("row {i} differs").into());}
 }
 println!("matched three controlled guarded-page prefix handoffs; no live locale claim");Ok(())
}
