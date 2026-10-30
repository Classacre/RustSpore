//! Synthetic binary32 matrix projection adapter, not original execution.
use spore_core::{cell_start_quaternion_matrix_v1::{matrix_words, Error as MatrixError}, native_sse::Policy};
use std::{error::Error, fs::OpenOptions, io::{Read,Write}};

fn main() -> Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().collect();
    if args.len()!=3 { return Err("usage: quaternion-matrix input new-output".into()); }
    let input=std::fs::File::open(&args[1])?;
    if input.metadata()?.len()>1024*1024 { return Err("input too large".into()); }
    let mut bytes=Vec::new(); input.take(1024*1024+1).read_to_end(&mut bytes)?;
    if bytes.len()<12 || bytes.len()>1024*1024 || &bytes[..8]!=b"QMATR001" { return Err("invalid header".into()); }
    let count=u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    if count==0 || count>20_000 || bytes.len()!=12+count*16 { return Err("invalid extent".into()); }
    let mut output=b"QMATOUT1".to_vec(); output.extend_from_slice(&(count as u32).to_le_bytes());
    for row in bytes[12..].chunks_exact(16) {
        let mut q=[0u32;4];
        for (i,chunk) in row.chunks_exact(4).enumerate() { q[i]=u32::from_le_bytes(chunk.try_into()?); }
        let (status,result)=match matrix_words(q,Policy::Mxcsr1f80) {
            Ok(v)=>(0u32,v),
            Err(MatrixError::NonFiniteInput)=>(1,[0;9]),
            Err(MatrixError::NonFiniteIntermediate)=>(2,[0;9]),
            Err(e)=>return Err(format!("unsupported model error: {e:?}").into()),
        };
        output.extend_from_slice(&status.to_le_bytes());
        for value in result { output.extend_from_slice(&value.to_le_bytes()); }
    }
    let mut out=OpenOptions::new().write(true).create_new(true).open(&args[2])?;
    out.write_all(&output)?;
    println!("cases={count} model_only=true original_execution=false");
    Ok(())
}
