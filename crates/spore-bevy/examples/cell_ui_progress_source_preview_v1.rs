//! Controlled single-segment source-atlas diagnostic. Not a retail HUD/frame.
//! Assumes one ProportionalLayout dispatch, clamp then floor, identity parent
//! matrix plus explicit translation, caller-supplied state and resolved-slot
//! mask, a single mode-2 pass, white tint, and software straight-alpha blending.
//! Does not implement source effect animation, runtime visibility or clipping.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{spui::{self, SpuiElement, SpuiValue}, spui_layout::{clamp_inverted_edges, ImageRegion},
    spui_proportional_layout::ProportionalLayout, spui_std_drawable::StdDrawableSlots,
    spui_std_drawable_mode2::mode2_calls};
use std::{collections::HashMap, error::Error, io::{BufWriter, Write}, path::Path};

const LAYOUT: NativeResourceKey = NativeResourceKey { instance_id: 0x26b0_68d2, type_id: 0x250f_e9a2, group_id: 0x4046_4200 };
const ATLAS: NativeResourceKey = NativeResourceKey { instance_id: 0xb8b5_0133, type_id: 0x2f7d_0004, group_id: 0x31a4_4893 };
fn resource(p: &NativePackageIndex, key: NativeResourceKey) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(p.read(p.index().find_unique(key)?.ok_or("explicit resource absent")?)?)
}
fn refs(e: &SpuiElement, id: u32) -> Result<Vec<u16>, Box<dyn Error>> {
    let p = e.property(id).ok_or("missing reference property")?;
    if p.type_code != 19 { return Err("bad reference type".into()); }
    let bytes = p.scalar_bytes().ok_or("bad reference data")?;
    if bytes.len() != usize::from(p.count)*2 { return Err("bad reference count".into()); }
    Ok(bytes.chunks_exact(2).map(|x| u16::from_le_bytes([x[0],x[1]])).collect())
}
fn scalar(e: &SpuiElement, id: u32) -> Result<u32, Box<dyn Error>> {
    e.property(id).and_then(|p| p.u32_value()).ok_or_else(|| format!("bad property {id:08X}").into())
}
fn structure_edges(e: &SpuiElement) -> Result<[f32;4], Box<dyn Error>> {
    let p = e.property(0xef3c_000c).ok_or("missing split structure")?;
    let values = match &p.value { SpuiValue::Structures { type_id: 0x32, values } if values.len()==1 => &values[0], _=>return Err("unsupported split structure".into()) };
    let mut result = [0.;4];
    for i in 0..4 {
        let p=values.iter().find(|p|p.id == (i+1) as u32).ok_or("missing split edge")?;
        let b=p.scalar_bytes().filter(|b|b.len()==4).ok_or("bad split edge")?;
        if p.type_code!=11 || p.count!=1 { return Err("bad split edge shape".into()); }
        result[i]=f32::from_le_bytes(b.try_into()?);
    }
    Ok([result[0], result[1], 1.-result[2], 1.-result[3]])
}
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len()!=13 { return Err("usage: <layout.package> <atlas.package> <segment-id-hex> <parent-width-f32-hex> <parent-height-f32-hex> <origin-x-f32-hex> <origin-y-f32-hex> <state-hex> <resolved-mask-hex> <canvas-width> <canvas-height> <new.png> <new.json>".into()); }
    let parse_f=|s:&str| -> Result<f32,Box<dyn Error>> { if s.len()!=8 { return Err("float inputs require 8 hex digits".into()); } let x=f32::from_bits(u32::from_str_radix(s,16)?); if !x.is_finite() { return Err("nonfinite input".into()); } Ok(x) };
    let id=u32::from_str_radix(&a[2],16)?;
    if !(0x051b_54c0..=0x051b_54c4).contains(&id) { return Err("unsupported segment ID".into()); }
    let (sx,sy,ox,oy)=(parse_f(&a[3])?,parse_f(&a[4])?,parse_f(&a[5])?,parse_f(&a[6])?);
    if sx<=0. || sy<=0. { return Err("parent spans must be positive".into()); }
    let state=u32::from_str_radix(&a[7],16)?; let mask=u8::from_str_radix(&a[8],16)?;
    let (w,h):(u32,u32)=(a[9].parse()?,a[10].parse()?);
    if !(1..=4096).contains(&w) || !(1..=4096).contains(&h) { return Err("canvas bounds".into()); }
    let target=Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("target").canonicalize()?;
    for name in [&a[11],&a[12]] {
        let path=Path::new(name); let parent=path.parent().ok_or("output parent missing")?.canonicalize()?;
        if !parent.starts_with(&target) || path.exists() || path.file_name().is_none() { return Err("outputs must be new files under target".into()); }
    }
    if a[11]==a[12] { return Err("output paths must differ".into()); }
    let lp=NativePackageIndex::open(Path::new(&a[0]))?; let ap=NativePackageIndex::open(Path::new(&a[1]))?;
    let layout_data=resource(&lp,LAYOUT)?; let atlas_data=resource(&ap,ATLAS)?;
    let layout_hash=format!("{:x}",Sha256::digest(&layout_data));
    let atlas_hash=format!("{:x}",Sha256::digest(&atlas_data));
    if !["5c93c578393784282700b0cdacc1d370870851d34fc5a76aa62bad1c9809edda", "6f85179ae30bda8f6aa9a7522523ca7c9bd8a33a3f44c31db50a70cfb7d567dc"].contains(&layout_hash.as_str()) ||
       !["44221369585ea3732a378390726a205d73f9e86781b84a06f6ada8d66319e153", "a5de353327759af85b9844b3ab626bb5fd25df90d78f54bd462f314e4d9b5514"].contains(&atlas_hash.as_str()) { return Err("unrecognized source payload digest".into()); }
    let file=spui::parse(&layout_data)?; let body=spui::parse_elements(&file)?;
    let objects:HashMap<_,_>=body.elements.iter().map(|e|(e.index,e)).collect();
    if objects.len()!=body.elements.len() { return Err("duplicate object index".into()); }
    let found:Vec<_>=body.elements.iter().filter(|e|e.property(0xeec1_b001).and_then(|p|p.u32_value())==Some(id)).collect();
    if found.len()!=1 || found[0].proxy_id!=0x4ec1_b8d8 { return Err("ambiguous or unsupported segment".into()); }
    let window=found[0]; let get=|i:u16| objects.get(&i).copied().ok_or("reference missing");
    let proc_refs=refs(window,0xeec1_b00c)?; let draw_refs=refs(window,0xeec1_b007)?;
    if proc_refs.len()!=1 || draw_refs.len()!=1 { return Err("unsupported layout/drawable graph".into()); }
    let layout=ProportionalLayout::from_element(get(proc_refs[0])?)?;
    let drawable=get(draw_refs[0])?; let slots=StdDrawableSlots::from_element(drawable)?;
    if scalar(drawable,0xef3c_000a)?!=2 { return Err("drawable mode not 2".into()); }
    let scales=drawable.property(0xef3c_000b).ok_or("missing scales")?;
    let bytes=scales.scalar_bytes().filter(|b|b.len()==8).ok_or("bad scale data")?;
    if scales.type_code!=17 || scales.count!=1 { return Err("bad scale shape".into()); }
    let scale=[f32::from_le_bytes(bytes[..4].try_into()?),f32::from_le_bytes(bytes[4..].try_into()?)];
    if scale.iter().any(|x| !x.is_finite()) { return Err("nonfinite source scale".into()); }
    let selected=slots.selected_reference(state,std::array::from_fn(|i|mask & (1<<i)!=0)).ok_or("state resolves to no image")?;
    let image=get(selected)?; let region=ImageRegion::from_element(image)?;
    if region.flags.0!=0 || region.intrinsic.iter().any(|x| *x<=0) { return Err("unsupported image flags/dimensions".into()); }
    let resources:Vec<_>=file.image_resources.iter().chain(&file.atlas_resources).collect();
    let key=resources.get(usize::from(region.resource_reference)).ok_or("image resource ordinal out of bounds")?;
    if key.instance!=ATLAS.instance_id || key.type_id!=ATLAS.type_id || key.group!=ATLAS.group_id { return Err("image does not reference explicit atlas key".into()); }
    let atlas=image::load_from_memory_with_format(&atlas_data,image::ImageFormat::Png)?.to_rgba8();
    if atlas.dimensions()!=(512,512) { return Err("unexpected atlas dimensions".into()); }
    let authored=window.property(0xeec1_b005).and_then(|p|p.rectangle()).ok_or("missing authored rectangle")?;
    let local=clamp_inverted_edges(layout.apply([0.,0.,sx,sy],authored)?)?.map(f32::floor);
    let dest=[local[0]+ox,local[1]+oy,local[2]+ox,local[3]+oy];
    if dest.iter().any(|x|!x.is_finite()) || dest[0]>=dest[2] || dest[1]>=dest[3] { return Err("empty/invalid explicit destination".into()); }
    let calls=mode2_calls(dest,region.intrinsic,[0.,0.,1.,1.],structure_edges(drawable)?,scale)?;
    let mut canvas=image::RgbaImage::new(w,h); let mut touched=0usize;
    for call in &calls {
        let r=call.destination; let uv=region.map_uv(call.relative_uv)?;
        if r.iter().chain(&uv).any(|x|!x.is_finite()) || uv.iter().any(|x|*x<0. || *x>1.) || uv[0]>uv[2] || uv[1]>uv[3] { return Err("unsupported mapped rectangle or UV".into()); }
        if r[0]>=r[2] || r[1]>=r[3] { continue; }
        for y in 0..h { for x in 0..w {
            let (px,py)=(x as f32+0.5,y as f32+0.5);
            if px<r[0] || px>=r[2] || py<r[1] || py>=r[3] { continue; }
            let u=uv[0]+((px-r[0])/(r[2]-r[0]))*(uv[2]-uv[0]);
            let v=uv[1]+((py-r[1])/(r[3]-r[1]))*(uv[3]-uv[1]);
            let fx=(u*512.-0.5).clamp(0.,511.); let fy=(v*512.-0.5).clamp(0.,511.);
            let (x0,y0)=(fx.floor() as u32,fy.floor() as u32); let (x1,y1)=((x0+1).min(511),(y0+1).min(511));
            let (dx,dy)=(fx-x0 as f32,fy-y0 as f32); let mut rgba=[0u8;4];
            for c in 0..4 { let top=atlas.get_pixel(x0,y0)[c] as f32*(1.-dx)+atlas.get_pixel(x1,y0)[c] as f32*dx; let bottom=atlas.get_pixel(x0,y1)[c] as f32*(1.-dx)+atlas.get_pixel(x1,y1)[c] as f32*dx; rgba[c]=(top*(1.-dy)+bottom*dy).round().clamp(0.,255.) as u8; }
            // Pieces meet without overlap for this admitted fixture; reject an
            // overlap instead of inventing a source blend/composition policy.
            if canvas.get_pixel(x,y)[3]!=0 && rgba[3]!=0 { return Err("overlapping pieces require unresolved compositing".into()); }
            if rgba[3]!=0 { touched+=1; } canvas.put_pixel(x,y,image::Rgba(rgba));
        }}
    }
    if touched==0 { return Err("no nontransparent source pixels in canvas".into()); }
    let mut png=BufWriter::new(std::fs::File::create_new(&a[11])?);
    canvas.write_to(&mut png,image::ImageFormat::Png)?; png.flush()?;
    drop(png);
    let png_hash=format!("{:x}",Sha256::digest(std::fs::read(&a[11])?));
    let call_records=calls.iter().map(|c| format!("{{\"destination\":{:?},\"relative_uv\":{:?}}}",c.destination,c.relative_uv)).collect::<Vec<_>>().join(",");
    let mut report=BufWriter::new(std::fs::File::create_new(&a[12])?);
    writeln!(report,"{{\n  \"scope\": \"controlled source segment; caller parent, state and slot resolution; one proportional dispatch, clamp/floor, identity matrix plus translation, one mode2 pass, white tint, software bilinear, no overlapping samples; no runtime frame or active provider\",\n  \"layout_sha256\": \"{layout_hash}\",\n  \"atlas_sha256\": \"{atlas_hash}\",\n  \"png_sha256\": \"{png_hash}\",\n  \"segment_id\": \"{id:08X}\",\n  \"window_index\": {},\n  \"drawable_index\": {},\n  \"image_index\": {},\n  \"state\": \"{state:08X}\",\n  \"resolved_mask\": \"{mask:02X}\",\n  \"parent_span_bits\": [\"{:08X}\", \"{:08X}\"],\n  \"origin_bits\": [\"{:08X}\", \"{:08X}\"],\n  \"destination\": {:?},\n  \"call_count\": {},\n  \"calls\": [{call_records}],\n  \"nontransparent_samples\": {}\n}}", window.index,drawable.index,image.index,sx.to_bits(),sy.to_bits(),ox.to_bits(),oy.to_bits(),dest,calls.len(),touched)?; report.flush()?;
    println!("Single-segment source-atlas diagnostic: {} calls, {touched} nontransparent samples. No retail HUD or runtime state claim.",calls.len());
    Ok(())
}
