//! Explicit installed layout projection, not live Cell state or provider choice.
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{spui,spui_std_drawable::StdDrawableSlots,spui_window_draw_state_v1::WindowSnapshot};
use std::{error::Error,io::Read,path::Path};
fn main()->Result<(),Box<dyn Error>> {
 let a:Vec<_>=std::env::args().skip(1).collect();
 if a.len()!=3||a[2]!="--admit-successful-base-constructor-property-application-no-mutation-and-stable-package" {return Err("usage: <explicit-package> <resolved-slot-mask-hex> --admit-successful-base-constructor-property-application-no-mutation-and-stable-package".into())}
 let mask=u8::from_str_radix(&a[1],16)?;
 let mut header=[0u8;dbpf::HEADER_SIZE];std::fs::File::open(&a[0])?.read_exact(&mut header)?;
 let header=dbpf::DbpfHeader::parse(&header)?;
 if header.index_size>64*1024*1024||header.index_entry_count>2_000_000{return Err("package index exceeds bound".into())}
 let package=NativePackageIndex::open(Path::new(&a[0]))?;
 let key=NativeResourceKey {instance_id:0x26b068d2,type_id:0x250fe9a2,group_id:0x40464200};
 let entry=package.index().find_unique(key)?.ok_or("explicit layout missing")?;
 if entry.stored_size>32*1024*1024||entry.decompressed_size>32*1024*1024{return Err("layout too large".into())}
 let data=package.read(entry)?;if data.len()!=entry.decompressed_size as usize{return Err("layout extent mismatch".into())}
 let hash=format!("{:x}",Sha256::digest(&data));
 if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda","6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&hash.as_str()){return Err("unapproved layout hash".into())}
 let file=spui::parse(&data)?;let body=spui::parse_elements(&file)?;
 let get=|idx|->Result<&spui::SpuiElement,Box<dyn Error>>{let mut it=body.elements.iter().filter(|e|e.index==idx);let e=it.next().ok_or("missing object")?;if it.next().is_some(){return Err("duplicate object".into())}Ok(e)};
 println!("CONTROLLED_WINDOW_FIXTURE layout_sha256={hash} mask={mask:02X} state=constructor_zero runtime_visibility=false active_provider=false");
 for id in 0x051b54c0..=0x051b54c4 {
  let mut matches=body.elements.iter().filter(|e|e.property(0xeec1b001).and_then(|p|p.u32_value())==Some(id));
  let w=matches.next().ok_or("window missing")?;
  if matches.next().is_some()||w.proxy_id!=0x4ec1b8d8{return Err("ambiguous window or unsupported class".into())}
  let flags=w.property(0xeec1b000).and_then(|p|p.u32_value()).ok_or("authored flags missing")?;
  let d=get(w.property(0xeec1b007).and_then(|p|p.reference()).ok_or("drawable missing")?)?;
  let slots=StdDrawableSlots::from_element(d)?;
  let mut state=WindowSnapshot::constructor_fields();state.authored_2c=flags;
  let selected=slots.selected_reference(state.state_ac,std::array::from_fn(|i|mask&(1<<i)!=0));
  println!("segment={id:08X} window={} drawable={} flags={flags:08X} eligible={} selected_reference={selected:?}",w.index,d.index,state.tree_eligible());
 }
 Ok(())
}
