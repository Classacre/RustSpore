//! Controlled five-segment Bevy fixture; no native HUD selection or placement.
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use spore_core::{native_sse::{self,Policy},spui::{self,SpuiElement,SpuiValue},spui_layout::{clamp_inverted_edges,ImageRegion},
 spui_proportional_layout::ProportionalLayout,spui_std_drawable::StdDrawableSlots,spui_std_drawable_mode2::mode2_calls};
use std::{error::Error,io::Read,path::Path};
const LAYOUT:NativeResourceKey=NativeResourceKey{instance_id:0x26b068d2,type_id:0x250fe9a2,group_id:0x40464200};
const ATLAS:NativeResourceKey=NativeResourceKey{instance_id:0xb8b50133,type_id:0x2f7d0004,group_id:0x31a44893};
pub struct FiveUi {pub rgba:Vec<u8>,pub rectangles:Vec<([f32;4],[f32;4])>}
fn resource(path:&Path,key:NativeResourceKey)->Result<Vec<u8>,Box<dyn Error>>{
 let mut h=[0;dbpf::HEADER_SIZE];std::fs::File::open(path)?.read_exact(&mut h)?;let h=dbpf::DbpfHeader::parse(&h)?;
 if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("UI package index exceeds bound".into())}
 let p=NativePackageIndex::open(path)?;let e=p.index().find_unique(key)?.ok_or("source UI key missing")?;
 if e.stored_size>32*1024*1024||e.decompressed_size>32*1024*1024{return Err("source UI resource exceeds bound".into())}
 let b=p.read(e)?;if b.len()!=e.decompressed_size as usize{return Err("source UI extent mismatch".into())}Ok(b)
}
fn split(e:&SpuiElement)->Result<[f32;4],Box<dyn Error>>{
 let p=e.property(0xef3c000c).ok_or("split missing")?;
 let fields=match &p.value {SpuiValue::Structures{type_id:0x32,values} if values.len()==1=>&values[0],_=>return Err("split shape".into())};
 let mut v=[0.;4];for i in 0..4{let p=fields.iter().find(|p|p.id==i as u32+1).ok_or("split field")?;let b=p.scalar_bytes().filter(|x|x.len()==4).ok_or("split bytes")?;if p.type_code!=11||p.count!=1{return Err("split field shape".into())}v[i]=f32::from_le_bytes(b.try_into()?);}
 if v.iter().any(|x|!x.is_finite()){return Err("nonfinite split".into())}Ok([v[0],v[1],1.-v[2],1.-v[3]])
}
pub fn load(layout:&Path,atlas:&Path,span:[f32;2],origin:[f32;2],state:u32,mask:u8,cursor:Option<[f32;2]>)->Result<FiveUi,Box<dyn Error>>{
 if span.iter().any(|x|!x.is_finite()||*x<=0.||*x>4096.)||origin.iter().any(|x|!x.is_finite()){return Err("invalid explicit parent span or origin".into())}
 let lb=resource(layout,LAYOUT)?;let ab=resource(atlas,ATLAS)?;let lh=format!("{:x}",Sha256::digest(&lb));let ah=format!("{:x}",Sha256::digest(&ab));
 if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda","6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&lh.as_str())||!["44221369585ea3732a378390726a205d73f9e86781b84a06f6ada8d66319e153","a5de353327759af85b9844b3ab626bb5fd25df90d78f54bd462f314e4d9b5514"].contains(&ah.as_str()){return Err("unaudited UI source payload".into())}
 let f=spui::parse(&lb)?;let body=spui::parse_elements(&f)?;
 let get=|idx|->Result<&SpuiElement,Box<dyn Error>>{let mut it=body.elements.iter().filter(|e|e.index==idx);let e=it.next().ok_or("missing UI object")?;if it.next().is_some(){return Err("duplicate UI object".into())}Ok(e)};
 let mut rectangles=Vec::new();let mut null_segments=0usize;let mut zero_segments=0usize;let mut segment_rectangles=Vec::new();
 for id in 0x051b54c0..=0x051b54c4 {
  let mut it=body.elements.iter().filter(|e|e.property(0xeec1b001).and_then(|p|p.u32_value())==Some(id));let w=it.next().ok_or("segment missing")?;
  if it.next().is_some()||w.proxy_id!=0x4ec1b8d8{return Err("segment class or identity mismatch".into())}
  let listener=get(w.property(0xeec1b00c).and_then(|p|p.reference()).ok_or("layout reference missing")?)?;let proc=ProportionalLayout::from_element(listener)?;
  let draw=get(w.property(0xeec1b007).and_then(|p|p.reference()).ok_or("drawable reference missing")?)?;
  if draw.property(0xef3c000a).and_then(|p|p.u32_value())!=Some(2){return Err("drawable mode mismatch".into())}
  let scale_p=draw.property(0xef3c000b).ok_or("scale missing")?;let scale_b=scale_p.scalar_bytes().filter(|b|b.len()==8).ok_or("scale bytes")?;
  if scale_p.type_code!=17||scale_p.count!=1{return Err("scale shape".into())}let scale=[f32::from_le_bytes(scale_b[..4].try_into()?),f32::from_le_bytes(scale_b[4..].try_into()?)];
  if scale.iter().any(|v|!v.is_finite()){return Err("nonfinite scale".into())}
  let slots=StdDrawableSlots::from_element(draw)?;
  let authored=w.property(0xeec1b005).and_then(|p|p.rectangle()).ok_or("authored rectangle missing")?;
  let dest=native_sse::scoped(Policy::Mxcsr1f80,||->Result<_,Box<dyn Error>>{
   let local=clamp_inverted_edges(proc.apply([0.,0.,span[0],span[1]],authored)?)?.map(f32::floor);
   let dest=[local[0]+origin[0],local[1]+origin[1],local[2]+origin[0],local[3]+origin[1]];
   if dest.iter().any(|v|!v.is_finite())||dest[0]<0.||dest[1]<0.||dest[2]>1280.||dest[3]>800.||dest[0]>dest[2]||dest[1]>dest[3]{return Err("segment outside target or inverted".into())}
   Ok(dest)
  })?;
  segment_rectangles.push(dest);
  let selected=slots.selected_reference(state,std::array::from_fn(|i|mask&(1<<i)!=0));
  if selected.is_none(){null_segments+=1;continue}
  let image=get(selected.unwrap())?;
  let region=ImageRegion::from_element(image)?;if region.flags.0!=0||region.intrinsic.iter().any(|v|*v<=0){return Err("unsupported image region".into())}
  let refs:Vec<_>=f.image_resources.iter().chain(&f.atlas_resources).collect();let k=refs.get(usize::from(region.resource_reference)).ok_or("atlas ordinal")?;
  if (k.instance,k.type_id,k.group)!=(ATLAS.instance_id,ATLAS.type_id,ATLAS.group_id){return Err("atlas key mismatch".into())}
  if dest[0]==dest[2]||dest[1]==dest[3]{zero_segments+=1;continue}
  let quads=native_sse::scoped(Policy::Mxcsr1f80,||->Result<_,Box<dyn Error>>{
   mode2_calls(dest,region.intrinsic,[0.,0.,1.,1.],split(draw)?,scale).map_err(Into::into)
  })?;
  for q in quads {let uv=region.map_uv(q.relative_uv)?;let d=q.destination;
   if d.iter().any(|v|!v.is_finite())||d[0]<0.||d[1]<0.||d[2]>1280.||d[3]>800.||uv.iter().any(|v|!v.is_finite()||*v<0.||*v>1.)||d[0]>d[2]||d[1]>d[3]||uv[0]>uv[2]||uv[1]>uv[3]{return Err("unsupported UI quad".into())}
   if d[0]<d[2]&&d[1]<d[3]{rectangles.push((d,uv));}
  }
 }
 if let Some([progress,y])=cursor {
  if !y.is_finite(){return Err("invalid explicit cursor y".into())}
  let selection=spore_core::cell_ui_progress_selection_v1::select(progress,spore_core::native_x87::Context::from_control_word(0x037f)?).map_err(|e|format!("cursor selection: {e:?}"))?;
  let segment=segment_rectangles[selection.segment as usize];
  let x=spore_core::cell_ui_progress_selection_v1::cursor_x(segment[0],segment[2],selection.fraction,Policy::Mxcsr1f80).map_err(|e|format!("cursor arithmetic: {e:?}"))?;
  let mut parents=body.elements.iter().filter(|e|e.property(0xeec1b001).and_then(|p|p.u32_value())==Some(0x051b5168));
  let parent=parents.next().ok_or("cursor parent missing")?;if parents.next().is_some()||parent.proxy_id!=0x4ec1b8d8{return Err("cursor parent mismatch".into())}
  let child=get(parent.property(0xeec1b00b).and_then(|p|p.reference()).ok_or("cursor child missing")?)?;
  if child.proxy_id!=0x4ec1b8d8||child.property(0xeec1b005).and_then(|p|p.rectangle())!=Some([-8.,-8.,9.,37.]){return Err("cursor child geometry mismatch".into())}
  let draw=get(child.property(0xeec1b007).and_then(|p|p.reference()).ok_or("cursor drawable missing")?)?;
  if draw.proxy_id!=0xcf3c4eae{return Err("cursor drawable class mismatch".into())}
  let image=get(draw.property(0x4f3c4a26).and_then(|p|p.reference()).ok_or("cursor image missing")?)?;let region=ImageRegion::from_element(image)?;
  if region.flags.0!=0||region.intrinsic!=[17,45]{return Err("cursor image shape mismatch".into())}
  let refs:Vec<_>=f.image_resources.iter().chain(&f.atlas_resources).collect();let key=refs.get(usize::from(region.resource_reference)).ok_or("cursor atlas ordinal")?;
  if (key.instance,key.type_id,key.group)!=(ATLAS.instance_id,ATLAS.type_id,ATLAS.group_id){return Err("cursor atlas key mismatch".into())}
  let dest=[x-8.,y-8.,x+9.,y+37.];if dest.iter().any(|v|!v.is_finite())||dest[0]<0.||dest[1]<0.||dest[2]>1280.||dest[3]>800.{return Err("cursor outside target".into())}
  let uv=region.map_uv([0.,0.,1.,1.])?;if uv.iter().any(|v|!v.is_finite()||*v<0.||*v>1.)||uv[0]>=uv[2]||uv[1]>=uv[3]{return Err("unsupported cursor atlas bounds".into())}rectangles.push((dest,uv));
  println!("CELL_SOURCE_UI_CURSOR controlled_progress=true controlled_target_pixel_y=true admitted_segment_projection=true static_image=true native_animation=false assumed_visible=true native_visibility=false runtime_state=false x87=037F mxcsr=1F80 progress={progress:?} segment={} fraction={:?} anchor=[{x:?},{y:?}]",selection.segment,selection.fraction);
 }
 if rectangles.len()>46{return Err("unsupported UI quad count".into())}
 let decoded=image::load_from_memory_with_format(&ab,image::ImageFormat::Png)?.to_rgba8();if decoded.dimensions()!=(512,512){return Err("atlas extent".into())}
 println!("CELL_SOURCE_UI_FIVE controlled_placement=true controlled_state=true native_shader=false bevy_shader=true native_visibility=false runtime_state=false layout_sha256={lh} atlas_sha256={ah} span={span:?} origin={origin:?} state={state:08X} mask={mask:02X} quads={} null_segments={null_segments} zero_segments={zero_segments} null_fallback_unimplemented=true",rectangles.len());
 Ok(FiveUi{rgba:decoded.into_raw(),rectangles})
}

#[cfg(test)] mod tests {
 use super::*;
 #[test]
 #[ignore = "requires explicitly supplied stable installed packages"]
 fn installed_null_and_collapsed_segments_remain_valid_and_checked(){
  let root=std::env::var_os("RUSTSPORE_INSTALL").expect("RUSTSPORE_INSTALL");
  let path=Path::new(&root).join("Data/PatchData.package");
  let origin=[40.,40.];
  let normal=load(&path,&path,[1000.,60.],origin,0,1,None).unwrap();
  assert!(!normal.rectangles.is_empty());assert_eq!(normal.rgba.len(),512*512*4);
  let no_images=load(&path,&path,[1000.,60.],origin,0,0,None).unwrap();
  assert!(no_images.rectangles.is_empty());assert_eq!(no_images.rgba,normal.rgba);
  let collapsed=load(&path,&path,[20.,60.],origin,0,1,None).unwrap();
  assert!(collapsed.rectangles.is_empty());assert_eq!(collapsed.rgba,normal.rgba);
  assert!(load(&path,&path.with_file_name("missing-source-ui-test.package"),[1000.,60.],origin,0,0,None).is_err());
  assert!(load(&path,&path,[1000.,60.],[2000.,40.],0,0,None).is_err());
  let cursor=load(&path,&path,[1000.,60.],origin,0,1,Some([0.5,80.])).unwrap();assert_eq!(cursor.rectangles.len(),normal.rectangles.len()+1);
  let (destination,uv)=cursor.rectangles.last().unwrap();assert_eq!(*destination,[533.,72.,550.,117.]);assert_eq!(*uv,[0x3f720008,0x3e200000,0x3f7a8006,0x3e7a0019].map(f32::from_bits));
  assert!(load(&path,&path,[1000.,60.],origin,0,1,Some([1.1,80.])).is_err());
 }
}
