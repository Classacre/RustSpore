//! Controlled source+decoded PCM assembly compared to a supplied PCM16 WAV.
#[path="eal31b_insertion_verify_v1.rs"] mod wave;
use std::{env,fs,io::Read};
fn bounded(p:&std::ffi::OsStr,max:u64)->Result<Vec<u8>,Box<dyn std::error::Error>> {
    let mut v=Vec::new();fs::File::open(p)?.take(max+1).read_to_end(&mut v)?;
    if v.len() as u64>max {return Err("input exceeds limit".into())}Ok(v)
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let a:Vec<_>=env::args_os().skip(1).collect();if a.len()!=6{return Err("expected source.snr raw-s16le.pcm comparison.wav raw-start-frames max-absolute-difference max-samples-over-one".into())}
    let source=bounded(&a[0],32*1024*1024)?;let raw=bounded(&a[1],128*1024*1024)?;let wav=bounded(&a[2],128*1024*1024)?;
    if raw.len()%2!=0{return Err("odd raw PCM bytes".into())}
    let samples:Vec<i16>=raw.chunks_exact(2).map(|c|i16::from_le_bytes([c[0],c[1]])).collect();
    let start=a[3].to_str().ok_or("invalid offset")?.parse()?;
    let allowed_max:u32=a[4].to_str().ok_or("invalid max difference")?.parse()?;
    let allowed_over_one:usize=a[5].to_str().ok_or("invalid mismatch count")?.parse()?;
    let result=spore_core::eal31b::assemble_first_insertion(&source,&samples,start)?;
    let expected=wave::wav_pcm16(&wav,result.channels,result.sample_rate)?;
    if expected.len()!=result.pcm.len()*2{return Err("WAV length differs".into())}
    let mut max=0u32;let mut exact=0usize;let mut within=0usize;
    for (i,&v) in result.pcm.iter().enumerate() {let w=i16::from_le_bytes([expected[i*2],expected[i*2+1]]);let d=(i32::from(v)-i32::from(w)).unsigned_abs();max=max.max(d);exact+=usize::from(d==0);within+=usize::from(d<=1);}
    println!("frames={} channels={} raw_start={} samples={} max_abs={} exact={} within_one={}; comparison only, decoder and alignment supplied",result.pcm.len()/result.channels,result.channels,start,result.pcm.len(),max,exact,within);
    if max>allowed_max || result.pcm.len()-within>allowed_over_one {return Err("comparison exceeds explicitly supplied thresholds".into())}
    Ok(())
}
