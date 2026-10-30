//! Synthetic explicit-cache geometry model adapter, not original execution.
use spore_core::{cell_actor_geometry_projection_v1::{project,ActorInput,AdmittedCacheSnapshot,Error as GeometryError},
                 native_sse::Policy};
use std::{error::Error,fs::OpenOptions,io::{Read,Write}};

fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().collect();
    if args.len()!=3 {return Err("usage: geometry-adapter input new-output".into());}
    let input=std::fs::File::open(&args[1])?;
    if input.metadata()?.len()>1024*1024 {return Err("input too large".into());}
    let mut bytes=Vec::new();input.take(1024*1024+1).read_to_end(&mut bytes)?;
    if bytes.len()<12 || bytes.len()>1024*1024 || &bytes[..8]!=b"GEOMIN01" {return Err("invalid header".into());}
    let count=u32::from_le_bytes(bytes[8..12].try_into()?) as usize;
    if count==0 || count>10_000 || bytes.len()!=12+count*84 {return Err("invalid extent".into());}
    let mut output=b"GEOMOUT1".to_vec();output.extend_from_slice(&(count as u32).to_le_bytes());
    for row in bytes[12..].chunks_exact(84) {
        let mut f=[0u32;21];for (i,c) in row.chunks_exact(4).enumerate(){f[i]=u32::from_le_bytes(c.try_into()?);}
        let flags=u8::try_from(f[5]).map_err(|_|"flags outside byte")?;
        let actor=ActorInput {handle_fc:f[0],position:[f[1],f[2],f[3]],scalar:f[4],flags_48:flags,
                              matrix:f[6..15].try_into()?};
        let cache=match f[15] {
            0=>None,
            1=>Some(AdmittedCacheSnapshot {key_fc:f[16],center:[f[17],f[18],f[19]],scalar:f[20]}),
            _=>return Err("invalid cache admission".into()),
        };
        let (status,values)=match project(actor,cache,Policy::Mxcsr1f80) {
            Ok(v)=>(0u32,[v.center[0],v.center[1],v.center[2],v.scalar]),
            Err(GeometryError::MissingCache)=>(1,[0;4]),
            Err(GeometryError::WrongCacheKey)=>(2,[0;4]),
            Err(GeometryError::NonFiniteInput)=>(3,[0;4]),
            Err(GeometryError::NonFiniteIntermediate)=>(4,[0;4]),
            Err(e)=>return Err(format!("unsupported model error: {e:?}").into()),
        };
        output.extend_from_slice(&status.to_le_bytes());
        for v in values {output.extend_from_slice(&v.to_le_bytes());}
    }
    let mut out=OpenOptions::new().write(true).create_new(true).open(&args[2])?;out.write_all(&output)?;
    println!("cases={count} model_only=true original_execution=false");Ok(())
}
