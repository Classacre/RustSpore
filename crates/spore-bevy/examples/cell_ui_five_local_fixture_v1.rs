//! Explicit five-segment local geometry, not runtime screen placement.
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{native_sse::{self,Policy},spui,spui_layout::clamp_inverted_edges,
    spui_proportional_layout::ProportionalLayout};
use std::{error::Error,io::Read,path::Path};
fn main()->Result<(),Box<dyn Error>> {
 let a:Vec<_>=std::env::args().skip(1).collect();
 if a.len()!=4||a[3]!="--admit-stable-package-one-dispatch-no-other-mutation" {return Err("usage: <explicit-layout.package> <parent-width-f32-hex> <parent-height-f32-hex> --admit-stable-package-one-dispatch-no-other-mutation".into())}
 let parse=|s:&str|->Result<f32,Box<dyn Error>> {if s.len()!=8{return Err("span requires eight hex digits".into())}let v=f32::from_bits(u32::from_str_radix(s,16)?);if !v.is_finite()||v<=0.||v>4096.{return Err("span outside (0,4096]".into())}Ok(v)};
 let (width,height)=(parse(&a[1])?,parse(&a[2])?);
 let mut header=[0;dbpf::HEADER_SIZE];std::fs::File::open(&a[0])?.read_exact(&mut header)?;
 let h=dbpf::DbpfHeader::parse(&header)?;if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("package index exceeds bound".into())}
 let p=NativePackageIndex::open(Path::new(&a[0]))?;
 let key=NativeResourceKey {instance_id:0x26b068d2,type_id:0x250fe9a2,group_id:0x40464200};
 let entry=p.index().find_unique(key)?.ok_or("layout missing")?;
 if entry.stored_size>32*1024*1024||entry.decompressed_size>32*1024*1024{return Err("layout too large".into())}
 let data=p.read(entry)?;if data.len()!=entry.decompressed_size as usize{return Err("layout extent mismatch".into())}
 let hash=format!("{:x}",Sha256::digest(&data));
 if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda","6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&hash.as_str()){return Err("unapproved layout hash".into())}
 let file=spui::parse(&data)?;let body=spui::parse_elements(&file)?;
 let get=|idx|->Result<&spui::SpuiElement,Box<dyn Error>> {let mut it=body.elements.iter().filter(|e|e.index==idx);let e=it.next().ok_or("object missing")?;if it.next().is_some(){return Err("duplicate index".into())}Ok(e)};
 println!("FIVE_LOCAL_FIXTURE layout_sha256={hash} parent_width={:08X} parent_height={:08X} controlled=true screen_placement=false",width.to_bits(),height.to_bits());
 for id in 0x051b54c0..=0x051b54c4 {
  let mut it=body.elements.iter().filter(|e|e.property(0xeec1b001).and_then(|p|p.u32_value())==Some(id));
  let w=it.next().ok_or("segment missing")?;if it.next().is_some()||w.proxy_id!=0x4ec1b8d8{return Err("ambiguous segment or class".into())}
  let listener=get(w.property(0xeec1b00c).and_then(|p|p.reference()).ok_or("single listener missing")?)?;
  let layout=ProportionalLayout::from_element(listener)?;
  let authored=w.property(0xeec1b005).and_then(|p|p.rectangle()).ok_or("authored rectangle missing")?;
  let rect=native_sse::scoped(Policy::Mxcsr1f80,||->Result<_,Box<dyn Error>> {Ok(clamp_inverted_edges(layout.apply([0.,0.,width,height],authored)?)?.map(f32::floor))})?;
  println!("segment={id:08X} window={} listener={} local_bits={:08X},{:08X},{:08X},{:08X}",w.index,listener.index,rect[0].to_bits(),rect[1].to_bits(),rect[2].to_bits(),rect[3].to_bits());
 }
 Ok(())
}
