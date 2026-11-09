//! Explicit source UI fixture admission before any 3D scene begins.
//! The fixture is caller-supplied diagnostic geometry, not observed game state.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{spui, spui_draw::UiClipPath, spui_layout::ImageRegion,
    spui_std_drawable::StdDrawableSlots};
use std::{error::Error, io::Read, path::{Path, PathBuf}};

const MAX_RESOURCE: u32 = 32 * 1024 * 1024;
const MAX_FIXTURE: u64 = 1024 * 1024;

/// Package readers reopen paths. Installed inputs must remain trusted and stable
/// throughout admission and decoding; this is not atomic file admission.
fn admit_package_index(path:&Path)->Result<(),Box<dyn Error>> {
    let mut file=std::fs::File::open(path)?;
    let mut raw=[0u8;dbpf::HEADER_SIZE]; file.read_exact(&mut raw)?;
    admit_header(&raw)
}
fn admit_header(raw:&[u8])->Result<(),Box<dyn Error>> {
    let header=dbpf::DbpfHeader::parse(&raw)?;
    if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000 { return Err("UI package index exceeds admission bound".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_index_header_admission() {
        let mut raw=[0u8;dbpf::HEADER_SIZE]; raw[..4].copy_from_slice(b"DBPF");
        raw[0x24..0x28].copy_from_slice(&2_000_000u32.to_le_bytes());
        raw[0x2c..0x30].copy_from_slice(&(64u32*1024*1024).to_le_bytes());
        assert!(admit_header(&raw).is_ok());
        raw[0x24..0x28].copy_from_slice(&2_000_001u32.to_le_bytes());
        assert!(admit_header(&raw).is_err());
        raw[0x24..0x28].copy_from_slice(&0u32.to_le_bytes());
        raw[0x2c..0x30].copy_from_slice(&(64u32*1024*1024+1).to_le_bytes());
        assert!(admit_header(&raw).is_err());
    }
}

pub struct UiRequest {
    pub layout: PathBuf, pub atlas: PathBuf, pub fixture: PathBuf,
    pub clip: UiClipPath, pub half_pixel: bool,
}
pub struct AdmittedUi {
    pub vertex: Vec<u32>, pub pixel: Vec<u32>, pub rgba: Vec<u8>,
    pub rectangles: Vec<([f32;4],[f32;4])>, pub clip: UiClipPath, pub half_pixel: bool,
}
fn resource(p:&NativePackageIndex,k:NativeResourceKey)->Result<Vec<u8>,Box<dyn Error>> {
    let e=p.index().find_unique(k)?.ok_or("explicit UI resource absent")?;
    if e.stored_size>MAX_RESOURCE || e.decompressed_size>MAX_RESOURCE { return Err("UI resource exceeds admission bound".into()); }
    let bytes=p.read(e)?;
    if bytes.len()>MAX_RESOURCE as usize || bytes.len()!=e.decompressed_size as usize { return Err("UI decoded resource extent mismatch".into()); }
    Ok(bytes)
}
fn tokens(bytes:&[u8], profile:u32)->Result<Vec<u32>,Box<dyn Error>> {
    if bytes.len()<8 || bytes.len()%4!=0 { return Err("invalid source UI shader extent".into()); }
    let w:Vec<_>=bytes.chunks_exact(4).map(|x|u32::from_le_bytes(x.try_into().unwrap())).collect();
    if w[0]!=profile || w.last()!=Some(&0xffff) { return Err("invalid source UI shader profile".into()); } Ok(w)
}
fn rect(v:&serde_json::Value)->Result<[f32;4],Box<dyn Error>> {
    let a=v.as_array().ok_or("UI rectangle array")?;
    if a.len()!=4 { return Err("UI rectangle length".into()); }
    let mut r=[0f32;4]; for i in 0..4 { r[i]=a[i].as_f64().ok_or("UI rectangle number")? as f32; }
    if r.iter().any(|x|!x.is_finite()) { return Err("nonfinite UI rectangle".into()); } Ok(r)
}
pub fn admit(install:&Path, request:&UiRequest)->Result<AdmittedUi,Box<dyn Error>> {
    admit_package_index(&request.layout)?; admit_package_index(&request.atlas)?;
    let shader_path=install.join("Data/PatchData.package"); admit_package_index(&shader_path)?;
    let layout=NativePackageIndex::open(&request.layout)?; let atlas=NativePackageIndex::open(&request.atlas)?;
    let lk=NativeResourceKey {instance_id:0x26b068d2,type_id:0x250fe9a2,group_id:0x40464200};
    let ak=NativeResourceKey {instance_id:0xb8b50133,type_id:0x2f7d0004,group_id:0x31a44893};
    let lb=resource(&layout,lk)?; let ab=resource(&atlas,ak)?;
    let lh=format!("{:x}",Sha256::digest(&lb)); let ah=format!("{:x}",Sha256::digest(&ab));
    if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda","6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&lh.as_str()) ||
       !["44221369585ea3732a378390726a205d73f9e86781b84a06f6ada8d66319e153","a5de353327759af85b9844b3ab626bb5fd25df90d78f54bd462f314e4d9b5514"].contains(&ah.as_str()) { return Err("unaudited UI source payload".into()); }
    let file_handle=std::fs::File::open(&request.fixture)?;
    if file_handle.metadata()?.len()>MAX_FIXTURE { return Err("UI fixture too large".into()); }
    let mut bytes=Vec::new(); file_handle.take(MAX_FIXTURE+1).read_to_end(&mut bytes)?;
    if bytes.len() as u64>MAX_FIXTURE { return Err("UI fixture too large".into()); }
    let fixture_hash=format!("{:x}",Sha256::digest(&bytes)); let report:serde_json::Value=serde_json::from_slice(&bytes)?;
    if report["layout_sha256"].as_str()!=Some(lh.as_str()) || report["atlas_sha256"].as_str()!=Some(ah.as_str()) { return Err("UI fixture source digest mismatch".into()); }
    let file=spui::parse(&lb)?; let body=spui::parse_elements(&file)?;
    let index=|name:&str|->Result<u16,Box<dyn Error>> { Ok(u16::try_from(report[name].as_u64().ok_or("UI object index")?)?) };
    let get=|n:u16|->Result<&spui::SpuiElement,Box<dyn Error>> { let v:Vec<_>=body.elements.iter().filter(|e|e.index==n).collect(); if v.len()!=1 { return Err("ambiguous UI object".into()); } Ok(v[0]) };
    let window=get(index("window_index")?)?; let drawable=get(index("drawable_index")?)?; let image=get(index("image_index")?)?;
    let id=u32::from_str_radix(report["segment_id"].as_str().ok_or("UI segment ID")?,16)?;
    if !(0x051b54c0..=0x051b54c4).contains(&id) || window.proxy_id!=0x4ec1b8d8 || window.property(0xeec1b001).and_then(|p|p.u32_value())!=Some(id) { return Err("unsupported UI segment".into()); }
    let p=window.property(0xeec1b007).ok_or("missing drawable reference")?;
    if p.reference()!=Some(drawable.index) { return Err("UI drawable reference mismatch".into()); }
    if drawable.property(0xef3c000a).and_then(|p|p.u32_value())!=Some(2) { return Err("UI drawable mode mismatch".into()); }
    let slots=StdDrawableSlots::from_element(drawable)?;
    let state=u32::from_str_radix(report["state"].as_str().ok_or("UI state")?,16)?;
    let mask=u8::from_str_radix(report["resolved_mask"].as_str().ok_or("UI slot mask")?,16)?;
    if slots.selected_reference(state,std::array::from_fn(|i|mask & (1<<i)!=0))!=Some(image.index) { return Err("UI selected image mismatch".into()); }
    let region=ImageRegion::from_element(image)?;
    if region.flags.0!=0 { return Err("unsupported UI image flags".into()); }
    let resources:Vec<_>=file.image_resources.iter().chain(&file.atlas_resources).collect();
    let key=resources.get(usize::from(region.resource_reference)).ok_or("UI atlas ordinal")?;
    if key.instance!=ak.instance_id || key.type_id!=ak.type_id || key.group!=ak.group_id { return Err("UI atlas key mismatch".into()); }
    let calls=report["calls"].as_array().ok_or("UI calls")?;
    if calls.is_empty() || calls.len()>9 || report["call_count"].as_u64()!=Some(calls.len() as u64) { return Err("UI call count".into()); }
    let mut rectangles=Vec::new(); for c in calls {
        let d=rect(&c["destination"])?; let u=region.map_uv(rect(&c["relative_uv"])?)?;
        if d[0]>d[2] || d[1]>d[3] || d.iter().any(|x|*x<0. || *x>768.) || u.iter().any(|x|*x<0. || *x>1.) { return Err("UI rectangle outside admitted bounds".into()); }
        rectangles.push((d,u));
    }
    let decoded=image::load_from_memory_with_format(&ab,image::ImageFormat::Png)?.to_rgba8();
    if decoded.dimensions()!=(512,512) { return Err("unexpected UI atlas extent".into()); }
    let package=dbpf::PackageIndex::open(&shader_path)?;
    let entry=package.find_instance(0x0469a3f7,0x4021200400000003).ok_or("UI shader pack missing")?;
    if entry.stored_size>MAX_RESOURCE || entry.decompressed_size>MAX_RESOURCE { return Err("UI shader pack exceeds admission bound".into()); }
    let pb=package.read(entry)?;
    if pb.len()>MAX_RESOURCE as usize || pb.len()!=entry.decompressed_size as usize { return Err("UI shader pack extent mismatch".into()); }
    if format!("{:x}",Sha256::digest(&pb))!="ab5cc3cf86340ec1f7489cda7f801e8bc2d969e6e8a3babd8b1f34734a506409" { return Err("unverified UI shader pack".into()); }
    let pack=spore_core::shader_pack::parse(&pb)?;
    let shader=pack.standard.iter().find(|s|s.header.shader_id()==request.clip.shader_id() && s.header.name==request.clip.shader_name()).ok_or("UI shader missing")?;
    let variant=shader.variants.iter().find(|v|v.render_type==0).ok_or("UI shader render type")?;
    if variant.vertex_uniforms.len()!=1 || !variant.pixel_uniforms.is_empty() { return Err("UI uniform layout".into()); }
    let uniform=spore_core::shader_binding::UniformDescriptor { raw:variant.vertex_uniforms[0] };
    if uniform.callback_id()!=0x234 || uniform.dirty_data_id()!=0x234 || uniform.register_count()!=96 || uniform.fourth_word()!=0 { return Err("UI uniform descriptor".into()); }
    let vertex=tokens(&variant.vertex_bytecode,0xfffe0300)?; let pixel=tokens(&variant.pixel_bytecode,0xffff0300)?;
    println!("SOURCE_UI_FIXTURE layout={lh} atlas={ah} fixture={fixture_hash} window={} drawable={} image={} state={state:08X} mask={mask:02X} calls={} runtime_state=false",window.index,drawable.index,image.index,rectangles.len());
    Ok(AdmittedUi { vertex,pixel,rgba:decoded.into_raw(),rectangles,clip:request.clip,half_pixel:request.half_pixel })
}
