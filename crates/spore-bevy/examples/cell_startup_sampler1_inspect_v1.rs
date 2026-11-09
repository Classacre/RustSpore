//! Inspect a pinned sampler-1 payload against the retained CPU upload trace.
//! File stability is explicitly admitted; repeated reads are not atomic.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::raster;
use std::{error::Error, path::Path, io::Read};
const PAYLOAD_SHA: &str = "ceecebb4937eef972474e222dc2069238ce171f8f0a97ab89a8981328da96150";
const TRACE_SHA: &str = "2138da8f6f60324d22a25bcf10a837ac88acaab522f48ae2f288291fe54b4292";
fn hash(b: &[u8]) -> String { format!("{:x}", Sha256::digest(b)) }
fn read_payload(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
 let mut file=std::fs::File::open(path)?; let mut header=[0;dbpf::HEADER_SIZE]; file.read_exact(&mut header)?;
 let header=dbpf::DbpfHeader::parse(&header)?; if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000 {return Err("package index exceeds bound".into())}
 let index=NativePackageIndex::open(path)?;
 let key=NativeResourceKey {instance_id:0xe219e5d3,type_id:0x2f4e681c,group_id:0x40612901};
 let entry=index.index().find_unique(key)?.ok_or("exact sampler-1 key absent")?;
 if entry.stored_size>16*1024*1024 || entry.decompressed_size>16*1024*1024 {return Err("payload exceeds bound".into())}
 let bytes=index.read(entry)?; if hash(&bytes)!=PAYLOAD_SHA {return Err("sampler-1 payload hash mismatch".into())} Ok(bytes)
}
fn check_rgba(bgra: &[u8], rgba: &[u8]) -> Result<(), &'static str> {
 if bgra.len()!=rgba.len() || bgra.len()%4!=0 {return Err("pixel extent mismatch")}
 for (b,r) in bgra.chunks_exact(4).zip(rgba.chunks_exact(4)) {
  if [b[2],b[1],b[0],b[3]] != r {return Err("pixel mismatch")}
 } Ok(())
}
fn compare(payload: &[u8], trace: &[u8]) -> Result<(), Box<dyn Error>> {
 if hash(payload)!=PAYLOAD_SHA || hash(trace)!=TRACE_SHA {return Err("input hash mismatch".into())}
 let text=std::str::from_utf8(trace)?; let raster=raster::parse(payload)?;
 if raster.format!=raster::FORMAT_RAW_RGBA || raster.width!=256 || raster.height!=256 {return Err("unexpected sampler-1 raster".into())}
 let levels=raster.mip_levels()?; if levels.len()!=9 {return Err("unexpected mip count".into())}
 for (i,level) in levels.iter().enumerate() {
  let side=(256u32>>i).max(1); if level.width!=side || level.height!=side {return Err("unexpected mip dimensions".into())}
  let prefix=format!("SOURCE_INPUT_MIP mesh=0 sampler=1 level={i} width={side} height={side} sha256=");
  let mut rows=text.lines().filter_map(|line|line.strip_prefix(&prefix)); let row=rows.next().ok_or("trace mip absent")?;
  if rows.next().is_some() {return Err("duplicate trace mip".into())}
  let (digest,hex)=row.split_once(" data_hex=").ok_or("bad trace framing")?;
  if digest.len()!=64 || hex.len()!=level.data.len()*2 {return Err("trace extent".into())}
  let expected=level.data.iter().map(|x|format!("{x:02x}")).collect::<String>();
  if hex!=expected || digest!=hash(level.data) {return Err("trace mip payload mismatch".into())}
  let rgba=raster::decode_pixels(raster.format,side,side,level.data)?;check_rgba(level.data,&rgba)?;
  println!("sampler1 level={i} width={side} height={side} bgra_sha256={} rgba_sha256={}", hash(level.data),hash(&rgba));
 }
 Ok(())
}
fn main() -> Result<(),Box<dyn Error>> {
 let args:Vec<_>=std::env::args_os().skip(1).collect();
 if args.len()!=3 || args[0]!="--admit-stable-files" {return Err("usage: --admit-stable-files <Spore_Content.package> <retained-v4-trace>".into())}
 let p=Path::new(&args[1]);let trace_path=Path::new(&args[2]);let before=read_payload(p)?;if std::fs::metadata(trace_path)?.len()>16*1024*1024 {return Err("trace exceeds bound".into())}let trace=std::fs::read(trace_path)?;
 compare(&before,&trace)?;let after=read_payload(p)?;if std::fs::metadata(trace_path)?.len()>16*1024*1024 {return Err("trace exceeds bound".into())}let trace_after=std::fs::read(trace_path)?;
 if before!=after || trace!=trace_after {return Err("input changed during inspection".into())}
 println!("sampler1 matched all nine levels; exact key/payload and trace pinned; same-file stability caller-admitted; no live identity or GPU filtering claim");Ok(())
}
#[cfg(test)] mod tests {use super::*;#[test] fn channel_and_extent_checks(){assert!(check_rgba(&[3,2,1,4],&[1,2,3,4]).is_ok());assert!(check_rgba(&[3,2,1,4],&[3,2,1,4]).is_err());assert!(check_rgba(&[3,2,1,4],&[1,2,3]).is_err());assert!(check_rgba(&[1],&[1]).is_err());}}
