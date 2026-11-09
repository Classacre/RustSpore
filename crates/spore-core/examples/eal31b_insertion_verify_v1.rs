//! Compare one source PCM insertion against the beginning of a supplied PCM16 WAV.
//! This is a controlled literal-sample check, not full EAL31b decoding.
use std::{env,fs,io::Read};
pub(crate) fn wav_pcm16(wav:&[u8],channels:usize,sample_rate:u32)->Result<&[u8],Box<dyn std::error::Error>> {
    if wav.len()<12 || &wav[..4]!=b"RIFF" || &wav[8..12]!=b"WAVE" {return Err("unsupported comparison inputs".into())}
    let riff_len=u32::from_le_bytes(wav[4..8].try_into()?) as usize;
    if riff_len.checked_add(8)!=Some(wav.len()) {return Err("RIFF extent mismatch".into())}
    let mut pos=12usize;let(mut format,mut payload)=(None,None);
    while pos<wav.len() {
        if pos.checked_add(8).is_none_or(|n|n>wav.len()) {return Err("truncated WAV chunk header".into())}
        let size=u32::from_le_bytes(wav[pos+4..pos+8].try_into()?) as usize;
        let start=pos+8;let end=start.checked_add(size).ok_or("WAV size overflow")?;
        let bytes=wav.get(start..end).ok_or("truncated WAV chunk")?;
        match &wav[pos..pos+4] {
            b"fmt "=>{if format.replace(bytes).is_some(){return Err("duplicate fmt chunk".into())}},
            b"data"=>{if payload.replace(bytes).is_some(){return Err("duplicate data chunk".into())}},_=>{}
        }
        pos=end.checked_add(size&1).ok_or("WAV alignment overflow")?;
        if pos>wav.len(){return Err("missing WAV chunk padding".into())}
    }
    let fmt=format.ok_or("missing fmt")?;let data=payload.ok_or("missing data")?;
    if fmt.len()<16 || u16::from_le_bytes(fmt[0..2].try_into()?)!=1 ||
       u16::from_le_bytes(fmt[2..4].try_into()?) as usize!=channels ||
       u32::from_le_bytes(fmt[4..8].try_into()?)!=sample_rate ||
       u16::from_le_bytes(fmt[14..16].try_into()?)!=16 {return Err("unsupported WAV format".into())}
    let align=channels.checked_mul(2).filter(|&x|x!=0).ok_or("block alignment overflow")?;
    if u16::from_le_bytes(fmt[12..14].try_into()?) as usize!=align ||
       u32::from_le_bytes(fmt[8..12].try_into()?)!=sample_rate.checked_mul(align.try_into()?).ok_or("byte rate overflow")? ||
       data.len()%align!=0 {return Err("inconsistent PCM16 framing".into())}
    Ok(data)
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let a:Vec<_>=env::args_os().skip(1).collect();if a.len()!=2{return Err("expected source.snr comparison.wav".into())}
    let mut source=Vec::new();fs::File::open(&a[0])?.take(32*1024*1024+1).read_to_end(&mut source)?;
    let mut wav=Vec::new();fs::File::open(&a[1])?.take(128*1024*1024+1).read_to_end(&mut wav)?;
    if source.len()>32*1024*1024 || wav.len()>128*1024*1024{return Err("input too large".into())}
    let (r,insertions)=spore_core::eal31b::reconstruct_with_insertions(&source)?;
    if insertions.len()!=1 || insertions[0].pcm.is_empty(){return Err("unsupported comparison inputs".into())}
    let data=wav_pcm16(&wav,r.channels,r.sample_rate)?;
    let pcm=&insertions[0].pcm;let prefix=data.get(..pcm.len().checked_mul(2).ok_or("PCM overflow")?).ok_or("short WAV")?;
    for (i,v) in pcm.iter().enumerate(){if i16::from_le_bytes(prefix[i*2..i*2+2].try_into()?)!=*v{return Err(format!("PCM differs at sample {i}").into())}}
    println!("literal insertion matched: {} frames, {} channels, granule {}, declared offset {}; no full audio claim",pcm.len()/r.channels,r.channels,insertions[0].granule,insertions[0].offset_samples);
    Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    fn wav()->Vec<u8> {
        let mut b=b"RIFF\x28\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x44\xac\x00\x00\x10\xb1\x02\x00\x04\x00\x10\x00data\x04\x00\x00\x00".to_vec();
        b.extend_from_slice(&[0,0,0,0]);b
    }
    #[test] fn rejects_extent_alignment_and_duplicate_chunks() {
        let b=wav();assert_eq!(wav_pcm16(&b,2,44100).unwrap(),[0,0,0,0]);
        let mut bad=b.clone();bad[4]=0;assert!(wav_pcm16(&bad,2,44100).is_err());
        let mut bad=b.clone();bad[28]=0;assert!(wav_pcm16(&bad,2,44100).is_err());
        let mut bad=b.clone();bad[32]=2;assert!(wav_pcm16(&bad,2,44100).is_err());
        let mut bad=b.clone();bad[40]=3;assert!(wav_pcm16(&bad,2,44100).is_err());
        let mut bad=b.clone();bad.extend_from_slice(&b[12..36]);let len=(bad.len()-8) as u32;bad[4..8].copy_from_slice(&len.to_le_bytes());assert!(wav_pcm16(&bad,2,44100).is_err());
        let mut bad=b.clone();bad.extend_from_slice(b"JUNK\x01\0\0\0\x00");let len=(bad.len()-8) as u32;bad[4..8].copy_from_slice(&len.to_le_bytes());assert!(wav_pcm16(&bad,2,44100).is_err());
    }
}
