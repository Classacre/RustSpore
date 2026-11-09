//! Controlled Creature source frame, not a complete or native-positioned HUD.
use bevy::prelude::*;
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{spui,spui_layout::{ImageRegion,SimpleLayout,simple_window_layout},spui_std_drawable::StdDrawableSlots,spui_std_drawable_mode2::mode2_calls};
use std::{io::Read,path::Path};

#[derive(Resource)]
pub struct Frame { pub rgba:Vec<u8>,pub quads:Vec<Quad> }
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Quad {pub dest:[f32;4],pub uv:[f32;4]}
fn read_resource(path:&Path,key:NativeResourceKey)->Result<Vec<u8>,String>{
 let mut h=[0;dbpf::HEADER_SIZE];std::fs::File::open(path).map_err(|e|e.to_string())?.read_exact(&mut h).map_err(|e|e.to_string())?;
 let header=dbpf::DbpfHeader::parse(&h).map_err(|e|e.to_string())?;
 if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000{return Err("UI package index exceeds bound".into())}
 let p=NativePackageIndex::open(path).map_err(|e|e.to_string())?;
 let e=p.index().find_unique(key).map_err(|e|e.to_string())?.ok_or("source UI key missing")?;
 if e.stored_size>32*1024*1024||e.decompressed_size>32*1024*1024{return Err("UI payload exceeds bound".into())}
 let b=p.read(e).map_err(|e|e.to_string())?;if b.len()!=e.decompressed_size as usize{return Err("UI payload extent".into())}Ok(b)
}
fn rect(v:&serde_json::Value,k:&str)->Result<[f32;4],String>{
 let a=v[k].as_array().ok_or_else(||format!("{k} must be four numbers"))?;if a.len()!=4{return Err(format!("{k} must be four numbers"))}
 let mut x=[0.;4];for i in 0..4{x[i]=a[i].as_f64().ok_or("invalid rectangle")? as f32;}if x.iter().any(|x|!x.is_finite())||x[0]>=x[2]||x[1]>=x[3]{return Err("invalid rectangle extent".into())}Ok(x)
}
fn contains(a:[f32;4],b:[f32;4])->bool{a[0]<=b[0]&&a[1]<=b[1]&&a[2]>=b[2]&&a[3]>=b[3]}
fn has_child(e:&spui::SpuiElement,child:u16)->bool {
 e.property(0xeec1b00b).filter(|p|p.type_code==19).and_then(|p|p.scalar_bytes()).is_some_and(|bytes|bytes.chunks_exact(2).any(|x|u16::from_le_bytes([x[0],x[1]])==child))
}
fn local_fixture_mode(v:&serde_json::Value)->Result<bool,String>{
 let supplied=v.get("parent_966").is_some();let authored=v.get("admit_authored_966").is_some();let local=supplied||authored;let origin=v.get("screen_origin_966").is_some();let admission=v.get("admit_identity_translation").is_some();let direct=v.get("destination").is_some();
 if local {if supplied==authored||direct||!origin||v["admit_identity_translation"].as_bool()!=Some(true)||(authored&&v["admit_authored_966"].as_bool()!=Some(true)){return Err("local layout requires exactly one admitted parent mode, origin, identity translation and no destination".into())}}
 else if !direct||origin||admission{return Err("destination layout must not contain local-only fields".into())}
 Ok(local)
}
fn local_screen_frame(parent:[f32;4],origin:[f32;2],authored:[f32;4],proc:SimpleLayout)->Result<[f32;4],String>{
 let local=simple_window_layout(parent,authored,&[proc]).map_err(|e|e.to_string())?;
 let result=[local[0]+origin[0],local[1]+origin[1],local[2]+origin[0],local[3]+origin[1]];
 if result.iter().any(|x|!x.is_finite())||result[0]>=result[2]||result[1]>=result[3]{return Err("invalid resolved frame rectangle".into())}Ok(result)
}
pub fn load(layout:&Path,atlas:&Path,fixture:&Path)->Result<Frame,String>{
 let l=read_resource(layout,NativeResourceKey{instance_id:0x5ed10964,type_id:0x250fe9a2,group_id:0x40464200})?;
 let a=read_resource(atlas,NativeResourceKey{instance_id:0xfe081daa,type_id:0x2f7d0004,group_id:0x31a44893})?;
 let lh=format!("{:x}",Sha256::digest(&l));let ah=format!("{:x}",Sha256::digest(&a));
 if !["16b7c334373f58c9bf7933ce88dc7ceb18a7fd4b9a73bde23377ceb256459f2c","8f902bc04b4e4dae4a10f0a76c9e07c3019e2bcbb451bc088afac186afee9724"].contains(&lh.as_str()) || ah!="e39080bbd54cf228a138b1d3d566926b80601f9e19cb199adac8f4641f48f3c1"{return Err("unapproved source UI payload hash".into())}
 let f=std::fs::File::open(fixture).map_err(|e|e.to_string())?;let mut bytes=Vec::new();f.take(65537).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()>65536{return Err("fixture exceeds limit".into())}
 let v:serde_json::Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
 if v["layout_sha256"].as_str()!=Some(lh.as_str())||v["atlas_sha256"].as_str()!=Some(ah.as_str()){return Err("fixture digest mismatch".into())}
 let viewport=rect(&v,"viewport")?;let clip=rect(&v,"clip")?;
 if viewport!=[0.,0.,1280.,800.]||!contains(viewport,clip){return Err("clip must be within 1280x800 viewport".into())}
 let state=u32::try_from(v["state"].as_u64().ok_or("state missing")?).map_err(|_|"state out of range")?;
 let mask=u8::try_from(v["resolved_mask"].as_u64().ok_or("resolved_mask missing")?).map_err(|_|"mask out of range")?;
 let file=spui::parse(&l).map_err(|e|e.to_string())?;let body=spui::parse_elements(&file).map_err(|e|e.to_string())?;
 let get=|n|body.elements.iter().find(|e|e.index==n).ok_or("source UI element missing");
 let window=get(963)?;let drawable=get(886)?;let image=get(885)?;
 let use_local=local_fixture_mode(&v)?;
 let dest=if use_local {
  if !has_child(get(969)?,968)||!has_child(get(968)?,966)||!has_child(get(966)?,963){return Err("source parent chain mismatch".into())}
  let parent=if v.get("admit_authored_966").is_some() {
   let parent_window=get(966)?;
   if parent_window.property(0xeec1b00c).is_some(){return Err("unexpected 966 procedure property".into())}
   let p=parent_window.property(0xeec1b005).and_then(|p|p.rectangle()).ok_or("966 authored rectangle missing")?;
   if p!=[-7.,-242.,315.,-55.]{return Err("unexpected 966 authored rectangle".into())}
   p
  } else {rect(&v,"parent_966")?};
  let origin=v["screen_origin_966"].as_array().ok_or("screen_origin_966 missing")?;
  if origin.len()!=2{return Err("screen origin must have two components".into())}
  let mut xy=[0f32;2];for i in 0..2{xy[i]=origin[i].as_f64().ok_or("invalid screen origin")? as f32;}
  if xy.iter().any(|x|!x.is_finite()){return Err("invalid screen origin".into())}
  let authored=window.property(0xeec1b005).and_then(|p|p.rectangle()).ok_or("frame authored rectangle missing")?;
  let refs=window.property(0xeec1b00c).and_then(|p|p.reference()).ok_or("frame layout reference missing")?;
  if refs!=887{return Err("unexpected source layout reference".into())}
  let proc=SimpleLayout::from_element(get(refs)?).map_err(|e|e.to_string())?;
  if proc.flags!=6{return Err("unexpected source layout flags".into())}
  local_screen_frame(parent,xy,authored,proc)?
 } else {rect(&v,"destination")?};
 if !contains(clip,dest){return Err("clip must contain destination".into())}
 if window.proxy_id!=0x4ec1b8d8||window.property(0xeec1b001).and_then(|p|p.u32_value())!=Some(0x053f4ad8)||window.property(0xeec1b007).and_then(|p|p.reference())!=Some(886)||drawable.property(0xef3c000a).and_then(|p|p.u32_value())!=Some(2){return Err("source UI graph mismatch".into())}
 let slots=StdDrawableSlots::from_element(drawable).map_err(|e|e.to_string())?;
 if slots.selected_reference(state,std::array::from_fn(|i|mask&(1<<i)!=0))!=Some(885){return Err("fixture state does not select source image".into())}
 let region=ImageRegion::from_element(image).map_err(|e|e.to_string())?;
 if region.flags.0!=0||region.resource_reference!=90{return Err("unsupported source image".into())}
 let resources:Vec<_>=file.image_resources.iter().chain(&file.atlas_resources).collect();let key=resources.get(90).ok_or("atlas ordinal")?;
 if (key.instance,key.type_id,key.group)!=(0xfe081daa,0x2f7d0004,0x31a44893){return Err("atlas graph mismatch".into())}
 // Split words are bound by the checked SPUI payload SHA; state and placement are explicit fixture values.
 let split=[0x3ee66666,0x3e800000,0x3f0ccccd,0x3f400000].map(f32::from_bits);
 let calls=mode2_calls(dest,region.intrinsic,[0.,0.,1.,1.],split,[1.,1.]).map_err(|e|e.to_string())?;
 let mut quads=Vec::new();for c in calls {let uv=region.map_uv(c.relative_uv).map_err(|e|e.to_string())?;if !contains(clip,c.destination)||c.destination[0]>c.destination[2]||c.destination[1]>c.destination[3]||uv.iter().any(|x|*x<0.||*x>1.)||uv[0]>uv[2]||uv[1]>uv[3]{return Err("unsupported source quad".into())}if c.destination[0]<c.destination[2]&&c.destination[1]<c.destination[3]{quads.push(Quad{dest:c.destination,uv});}}
 if quads.is_empty(){return Err("empty source frame".into())}
 let decoded=image::load_from_memory_with_format(&a,image::ImageFormat::Png).map_err(|e|e.to_string())?.to_rgba8();if decoded.dimensions()!=(512,1024){return Err("atlas dimensions mismatch".into())}
 println!("CREATURE_SOURCE_FRAME local_layout={} authored_966_admitted={} parent_and_transform_caller_admitted={} controlled=true native_placement_unverified=true",use_local,v.get("admit_authored_966").is_some(),use_local);
 println!("CREATURE_SOURCE_FRAME controlled=true layout_sha256={lh} atlas_sha256={ah} state={state} mask={mask} destination={dest:?} clip={clip:?} quads={} stable_packages_required=true native_hud=false",quads.len());
 Ok(Frame{rgba:decoded.into_raw(),quads})
}
#[cfg(test)] mod tests{use super::*;
#[test] #[ignore = "requires installed assets and explicit fixture"] fn installed_source_graph_and_quad_emission(){
 let root=std::env::var_os("RUSTSPORE_INSTALL").expect("RUSTSPORE_INSTALL");let root=Path::new(&root);
 let frame=load(&root.join("Data/PatchData.package"),&root.join("Data/Spore_Graphics.package"),Path::new(&std::env::var_os("RUSTSPORE_FRAME_FIXTURE").expect("RUSTSPORE_FRAME_FIXTURE"))).unwrap();
 assert_eq!(frame.rgba.len(),512*1024*4);assert_eq!(frame.quads.len(),9);assert_eq!(frame.quads[0].dest[0..2],[960.,60.]);
 assert!(frame.quads.iter().all(|q|contains([960.,60.,1210.,220.],q.dest)));
 let local=load(&root.join("Data/PatchData.package"),&root.join("Data/Spore_Graphics.package"),Path::new(&std::env::var_os("RUSTSPORE_FRAME_LOCAL_FIXTURE").expect("RUSTSPORE_FRAME_LOCAL_FIXTURE"))).unwrap();
 assert_eq!(local.quads.len(),9);assert_eq!(local.quads[0].dest[0..2],[98.,213.]);
 assert!(local.quads.iter().all(|q|contains([98.,213.,340.,363.],q.dest)));
 let authored=load(&root.join("Data/PatchData.package"),&root.join("Data/Spore_Graphics.package"),Path::new(&std::env::var_os("RUSTSPORE_FRAME_AUTHORED_FIXTURE").expect("RUSTSPORE_FRAME_AUTHORED_FIXTURE"))).unwrap();
 assert_eq!(authored.quads,local.quads);
}
#[test]fn mixed_or_incomplete_layout_modes_are_rejected(){
 assert!(!local_fixture_mode(&serde_json::json!({"destination":[0,0,1,1]})).unwrap());
 assert!(local_fixture_mode(&serde_json::json!({"destination":[0,0,1,1],"screen_origin_966":[0,0]})).is_err());
 assert!(local_fixture_mode(&serde_json::json!({"destination":[0,0,1,1],"admit_identity_translation":true})).is_err());
 assert!(local_fixture_mode(&serde_json::json!({"parent_966":[0,0,1,1],"screen_origin_966":[0,0]})).is_err());
 assert!(local_fixture_mode(&serde_json::json!({"parent_966":[0,0,1,1],"screen_origin_966":[0,0],"admit_identity_translation":true,"destination":[0,0,1,1]})).is_err());
 assert!(local_fixture_mode(&serde_json::json!({"parent_966":[0,0,1,1],"screen_origin_966":[0,0],"admit_identity_translation":true})).unwrap());
 assert!(local_fixture_mode(&serde_json::json!({"admit_authored_966":true,"screen_origin_966":[0,0],"admit_identity_translation":true})).unwrap());
 assert!(local_fixture_mode(&serde_json::json!({"admit_authored_966":false,"screen_origin_966":[0,0],"admit_identity_translation":true})).is_err());
 assert!(local_fixture_mode(&serde_json::json!({"admit_authored_966":true,"parent_966":[0,0,1,1],"screen_origin_966":[0,0],"admit_identity_translation":true})).is_err());
}
#[test]fn supplied_parent_drives_source_local_layout(){
 let proc=SimpleLayout{flags:6};
 assert_eq!(local_screen_frame([0.,0.,322.,187.],[100.,200.],[-2.,-174.,240.,-24.],proc).unwrap(),[98.,213.,340.,363.]);
 assert!(local_screen_frame([0.,0.,1.,1.],[f32::MAX,f32::MAX],[f32::MAX,0.,f32::MAX,1.],proc).is_err());
}
#[test]fn clip_containment_and_rect_admission(){assert!(contains([0.,0.,1280.,800.],[1.,2.,10.,20.]));assert!(!contains([0.,0.,10.,10.],[1.,2.,11.,9.]));assert!(rect(&serde_json::json!({"x":[1,2,1,4]}),"x").is_err());}}
