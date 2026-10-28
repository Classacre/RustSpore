//! Controlled offscreen DAE8CE17 diagnostic. Fixture inputs are not retail state.
//! Usage: cell_kind0_native_fixture_v1 <SporeApp.exe> <PatchData.package> <Spore_Graphics.package> <new.png> <new-target-artifact-directory>
//! Requires trusted, stable installed packages: preflight and reads reopen paths,
//! so replacement between checks is not prevented.
#[cfg(all(windows, feature = "native_shader_probe"))]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 1;
#[cfg(all(windows, target_env = "msvc", feature = "native_shader_probe"))]
#[used]
#[link_section = ".drectve"]
static NVIDIA_EXPORT: [u8; b" /EXPORT:NvOptimusEnablement ".len()] =
    *b" /EXPORT:NvOptimusEnablement ";
#[cfg(all(windows, feature = "native_shader_probe"))]
#[path = "../src/native_d3d9.rs"] mod native_d3d9;
#[cfg(all(windows, feature = "native_shader_probe"))]
#[path = "../src/native_d3dx.rs"] mod native_d3dx;

#[cfg(all(windows, feature = "native_shader_probe"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use bevy::math::{Mat4, Vec3};
    use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
    use image::ImageEncoder;
    use spore_core::{material_links, raster, rw4, shader_binding::UniformDescriptor, shader_codegen::{self, SourceTemplates}, shader_fragments, shader_pack, shader_pixel_codegen, shader_selection::{self, SelectionContext}};
    use std::{collections::{BTreeMap,BTreeSet}, io::{Read,Write}, path::Path, ptr};
    use windows::Win32::Graphics::Direct3D9::*;
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len()!=5 { return Err("usage: cell_kind0_native_fixture_v1 <SporeApp.exe> <PatchData.package> <Spore_Graphics.package> <new.png> <new-target-artifact-directory>".into()); }
    let exe=Path::new(&args[0]); let shader_path=Path::new(&args[1]); let asset_path=Path::new(&args[2]); let output=Path::new(&args[3]);
    let artifacts=Path::new(&args[4]);
    if output.exists() { return Err("output exists".into()); }
    if artifacts.exists() { return Err("artifact directory exists".into()); }
    let target_root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").canonicalize()?;
    let artifact_parent=artifacts.parent().ok_or("artifact directory requires parent")?.canonicalize()?;
    if !artifact_parent.starts_with(target_root) {return Err("artifact directory must be under target".into());}
    for (p,n) in [(shader_path,"PatchData.package"),(asset_path,"Spore_Graphics.package")] { if !p.file_name().and_then(|v|v.to_str()).is_some_and(|v|v.eq_ignore_ascii_case(n)) { return Err("explicit candidate package names required".into()); } }
    let source_exe=std::fs::File::open(exe)?;
    if source_exe.metadata()?.len()!=20_447_744 {return Err("unexpected executable size".into());}
    let mut image=Vec::with_capacity(20_447_744); source_exe.take(20_447_745).read_to_end(&mut image)?;
    let templates=SourceTemplates::from_original_executable(&image)?;
    let open_package=|path:&Path| -> Result<NativePackageIndex,Box<dyn std::error::Error>> {
        let mut file=std::fs::File::open(path)?; let mut bytes=[0;dbpf::HEADER_SIZE]; file.read_exact(&mut bytes)?;
        let header=dbpf::DbpfHeader::parse(&bytes)?;
        if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000 {return Err("package index exceeds preflight bounds".into());}
        Ok(NativePackageIndex::open(path)?)
    };
    let shaders=open_package(shader_path)?; let assets=open_package(asset_path)?;
    let read=|p:&NativePackageIndex,k:NativeResourceKey| -> Result<Vec<u8>,Box<dyn std::error::Error>> {
        let e=p.index().find_unique(k)?.ok_or("missing unique source key")?;
        if e.stored_size.max(e.decompressed_size)>32*1024*1024 { return Err("resource too large".into()); }
        Ok(p.read(e)?)
    };
    let raw=read(&assets,NativeResourceKey { instance_id:0xdae8ce17,type_id:0x2f4e681b,group_id:0 })?;
    let pack_bytes=read(&shaders,NativeResourceKey::from_scoped_instance(0x0469a3f7,0x4021200400000003))?;
    let fragment_bytes=read(&shaders,NativeResourceKey::from_scoped_instance(0x0469a3f7,0x4021200200000003))?;
    // Whole-payload FNV is a quick identity guard, not a cryptographic digest.
    let fnv=|b:&[u8]|b.iter().fold(0xcbf29ce484222325u64,|h,v|(h^u64::from(*v)).wrapping_mul(0x100000001b3));
    if fnv(&raw)!=0x345952411eab57d9 || fnv(&fragment_bytes)!=0x6bff0da5f226840b || fnv(&pack_bytes)!=0xde4eb5b87214d9e7 {return Err("candidate payload identity mismatch".into());}
    let rw=rw4::parse(&raw)?; let pack=shader_pack::parse(&pack_bytes)?; let library=shader_fragments::parse(&fragment_bytes)?;
    if raw.len()!=95536 || rw.sections.len()!=20 { return Err("unexpected candidate resource extent".into()); }
    let m=rw.mesh(&raw,13).ok_or("missing mesh")?; let vb=rw.vertex_buffer(&raw,4).ok_or("missing vertex buffer")?; let ib=rw.index_buffer(&raw,12).ok_or("missing index buffer")?;
    if m.primitive_type!=4 || m.index_buffer!=12 || m.vertex_buffers!=[4] || m.triangle_count!=896 || m.first_index!=0 || m.primitive_count!=2688 || m.first_vertex!=0 || m.vertex_count!=454 || vb.base_vertex_index!=0 || vb.vertex_count!=454 || vb.vertex_size!=36 || vb.data!=3 || vb.description!=7 || ib.start_index!=0 || ib.primitive_count!=2688 || ib.format!=101 || ib.primitive_type!=4 || ib.data!=11 { return Err("candidate mesh layout mismatch".into()); }
    let vertices=rw.object(&raw,3).ok_or("missing vertices")?; let indices=rw.object(&raw,11).ok_or("missing indices")?;
    if vertices.len()!=454*36 || indices.len()!=2688*2 || indices.chunks_exact(2).any(|p|u16::from_le_bytes([p[0],p[1]])>=454) { return Err("invalid source geometry extents".into()); }
    if vertices.chunks_exact(36).any(|v| v[28..32].iter().any(|&i| i>24 || i%3!=0) || v[32..36].iter().map(|&b|u32::from(b)).sum::<u32>()!=255) {return Err("unexpected palette indices or weights".into());}
    let desc=rw.vertex_description(&raw,7).ok_or("missing declaration")?;
    let expected=[(0,0,2,0,0,0,0),(0,12,5,0,3,0,2),(0,16,5,0,6,0,19),(0,20,1,0,5,0,6),(0,28,5,0,2,0,14),(0,32,8,0,1,0,15)];
    if desc.vertex_size!=36 || desc.elements.len()!=expected.len() { return Err("unexpected declaration size".into()); }
    for (e,&(stream,offset,typ,method,usage,ix,sem)) in desc.elements.iter().zip(&expected) {
        if (e.stream,e.offset,e.element_type,e.method,e.usage,e.usage_index,e.rw_decl)!=(stream,offset,typ,method,usage,ix,sem) { return Err("unexpected source declaration".into()); }
    }
    let link=rw.object(&raw,15).ok_or("missing mesh state link")?;
    if link != [13u32.to_le_bytes(),1u32.to_le_bytes(),14u32.to_le_bytes()].concat() { return Err("unexpected mesh state link".into()); }
    let state=material_links::compiled_state(&rw,&raw,14)?;
    if state.shader_id!=0x80000004 || state.soft_dirty!=0x10801c || state.hard_delta!=0 { return Err("unexpected compiled state".into()); }
    let bindings=state.bindings()?;
    if bindings.textures.len()!=2 || bindings.textures[0].stage!=0 || bindings.textures[0].raster_reference!=17 || bindings.textures[1].stage!=1 || bindings.textures[1].raster_reference!=19 { return Err("unexpected stage associations".into()); }
    let region=bindings.prefix.iter().find(|p|p.flag==8).ok_or("missing source data")?;
    let d=&region.bytes; if d.len()!=76 { return Err("unexpected data prefix length".into()); }
    let u16le=|at:usize| u16::from_le_bytes([d[at],d[at+1]]);
    let mut data=BTreeMap::new(); let mut at=0usize;
    loop {
        let h=d.get(at..at+8).ok_or("truncated source data")?; let id=u16::from_le_bytes([h[0],h[1]]); let a=usize::from(u16le(at+2)); let b=usize::from(u16le(at+4)); at+=8;
        if id==0 { break; }
        let n=if b==0 {4} else {a+b}; let bytes=d.get(at..at+n).ok_or("truncated source data")?; if data.insert(id,bytes.to_vec()).is_some() {return Err("duplicate data ID".into());} at+=n;
    }
    if at!=d.len() || data.keys().copied().collect::<Vec<_>>()!=[3,4,0x210] {return Err("unexpected source IDs".into());}
    data.insert(0x24e,vec![0,0]);
    let samplers=BTreeSet::from([0u16,1]); let ctx=SelectionContext {declaration_features:0x0008c045,shader_data:&data,available_samplers:&samplers};
    let bs:Vec<_>=pack.builders.iter().filter(|b|b.header.shader_id()==0x80000004 && b.header.name=="generic_skinning").collect(); if bs.len()!=1 {return Err("ambiguous shader builder".into());} let builder=bs[0];
    if builder.header.words[4]!=0xfffe0300 || builder.header.words[5]!=0xffff0300 {return Err("unexpected shader profiles".into());}
    let variants:Vec<_>=builder.variants.iter().filter(|v|v.render_type==0).collect(); if variants.len()!=1{return Err("ambiguous render type".into());} let v=variants[0];
    let vs=shader_selection::select(&v.vertex,&ctx)?; let ps=shader_selection::select(&v.pixel,&ctx)?;
    if vs.fragment_ids!=[0x17,0x19,0x15,3,0x10,0x31,1] || ps.fragment_ids!=[5,9,0x3b,0x2a] {return Err("fixture selection changed".into());}
    let vsrc=shader_codegen::generate_vertex(&library,&vs.fragment_ids,&templates)?; let psrc=shader_pixel_codegen::generate_pixel(&library,&ps.fragment_ids,&templates)?;
    let flags=if builder.header.words[6]&2!=0 {0x32} else {0x12}; let compiler=native_d3dx::Compiler::system()?;
    let mut module_path=[0u16;32768];
    let n=unsafe{GetModuleFileNameW(Some(compiler.module_handle()),&mut module_path)} as usize;
    if n==0 || n>=module_path.len() {return Err("unable to identify loaded compiler module".into());}
    let compiler_path=String::from_utf16(&module_path[..n])?;
    println!("KIND0_COMPILER module={compiler_path:?} flags={flags:02x} entrypoint={:?}",templates.entrypoint()?);
    let vcode=compiler.compile(&vsrc.hlsl,templates.entrypoint()?,"vs_3_0",flags)?;
    if !vcode.diagnostics.is_empty() {println!("KIND0_COMPILER_DIAGNOSTICS stage=vs message={:?}",vcode.diagnostics);}
    let pcode=compiler.compile(&psrc.hlsl,templates.entrypoint()?,"ps_3_0",flags)?;
    if !pcode.diagnostics.is_empty() {println!("KIND0_COMPILER_DIAGNOSTICS stage=ps message={:?}",pcode.diagnostics);}
    let vt=native_d3d9::tokens(&vcode.bytecode,0xfffe0300)?; let pt=native_d3d9::tokens(&pcode.bytecode,0xffff0300)?;
    std::fs::create_dir(artifacts)?;
    for (name,bytes) in [("vertex.hlsl",vsrc.hlsl.as_bytes()),("pixel.hlsl",psrc.hlsl.as_bytes()),("vertex.vs",vcode.bytecode.as_slice()),("pixel.ps",pcode.bytecode.as_slice())] {
        let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(artifacts.join(name))?; file.write_all(bytes)?;
    }
    println!("KIND0_ARTIFACTS directory={} vs_hlsl_fnv={:016x} ps_hlsl_fnv={:016x} vs_bytecode_fnv={:016x} ps_bytecode_fnv={:016x}",artifacts.display(),fnv(vsrc.hlsl.as_bytes()),fnv(psrc.hlsl.as_bytes()),fnv(&vcode.bytecode),fnv(&pcode.bytecode));
    let mut lo=Vec3::splat(f32::INFINITY); let mut hi=Vec3::splat(f32::NEG_INFINITY);
    for row in vertices.chunks_exact(36) { let f=|i|f32::from_le_bytes(row[i..i+4].try_into().unwrap()); let p=Vec3::new(f(0),f(4),f(8)); if !p.is_finite(){return Err("nonfinite position".into());} lo=lo.min(p); hi=hi.max(p); }
    let centre=(lo+hi)*0.5; let radius=(hi-lo).length()*0.5; if !radius.is_finite() || radius<=0.0 {return Err("invalid fixture bounds".into());}
    let model=Mat4::from_scale(Vec3::splat(1.0/radius))*Mat4::from_translation(-centre);
    let view=Mat4::look_at_lh(Vec3::new(0.,0.,-3.),Vec3::ZERO,Vec3::Y); let clip=Mat4::perspective_lh(0.75,1.,0.1,10.)*view*model;
    println!("KIND0_ASSUMPTIONS selector_24e=0 shader_data_3_4_210=stored all_other_shader_data=absent samplers=0,1 skinweights_3=[3,0,1,0] material_color=[1,1,1,1] tint_20c=[1,1,1,1] palette=64_identity_matrices camera_eye=[0,0,-3] camera_target=[0,0,0] camera_up=[0,1,0] fov_rad=0.75 aspect=1 near=0.1 far=10 centre={centre:?} radius={radius} model_cols={:?} clip_cols={:?} cull=none srgb=off unsupplied_device_states=d3d_defaults retail_state=false",model.to_cols_array(),clip.to_cols_array());
    let rows=|m:Mat4,n:usize| -> Vec<[f32;4]> {(0..n).map(|i|m.row(i).to_array()).collect()};
    let palette:Vec<[f32;4]>=(0..192).map(|i|match i%3 {0=>[1.,0.,0.,0.],1=>[0.,1.,0.,0.],_=>[0.,0.,1.,0.]}).collect();
    let mut declaration=Vec::new(); for e in &desc.elements {declaration.push(D3DVERTEXELEMENT9 {Stream:e.stream as u16,Offset:e.offset as u16,Type:e.element_type as u8,Method:e.method as u8,Usage:e.usage as u8,UsageIndex:e.usage_index as u8});} declaration.push(D3DVERTEXELEMENT9 {Stream:255,Type:D3DDECLTYPE_UNUSED.0 as u8,..Default::default()});
    let target=unsafe{native_d3d9::OffscreenDevice::new(768,768)?}; let dev=&target.device; let mut textures=Vec::new();
    for (stage,section) in [(0u32,17u32),(1,19)] {
        let r=rw.raster(&raw,section).ok_or("missing raster")?; if r.format!=raster::FORMAT_DXT5 || (r.width,r.height,r.mipmap_levels)!=(128,128,8) {return Err("unexpected raster".into());} let bytes=rw.object(&raw,r.data).ok_or("missing raster data")?;
        let mut framed=Vec::new(); let mut off=0usize; let mut base=0u32;
        for level in 0..r.mipmap_levels {let w=(r.width>>level).max(1) as usize; let h=(r.height>>level).max(1) as usize; let n=w.div_ceil(4)*h.div_ceil(4)*16; if level==0 {base=n as u32;} else {framed.extend_from_slice(&(n as u32).to_le_bytes());} let end=off.checked_add(n).ok_or("mip overflow")?; framed.extend_from_slice(bytes.get(off..end).ok_or("short mip")?);off=end;}
        if off!=bytes.len(){return Err("unconsumed mip bytes".into());}
        let tex=unsafe{target.upload_raster(&raster::Raster {width:r.width,height:r.height,mipmaps:r.mipmap_levels,format:r.format,data_size:base,data:framed})?}; unsafe{dev.SetTexture(stage,&tex)?;} textures.push(tex);
    }
    unsafe {
        for g in bindings.groups.as_ref().ok_or("missing state group")? {if g.id!=0 {return Err("unsupported render state group".into());} for p in &g.values {dev.SetRenderState(D3DRENDERSTATETYPE(p.state as i32),p.value)?;}}
        for t in &bindings.textures {for p in &t.list_a {dev.SetTextureStageState(t.stage,D3DTEXTURESTAGESTATETYPE(p.state as i32),p.value)?;} for p in &t.list_b {dev.SetSamplerState(t.stage,D3DSAMPLERSTATETYPE(p.state as i32),p.value)?;}}
        // Explicit fixture choices for states not identified by the saved state.
        dev.SetRenderState(D3DRS_CULLMODE,D3DCULL_NONE.0 as u32)?; dev.SetRenderState(D3DRS_SRGBWRITEENABLE,0)?; dev.SetSamplerState(0,D3DSAMP_SRGBTEXTURE,0)?; dev.SetSamplerState(1,D3DSAMP_SRGBTEXTURE,0)?;
        dev.Clear(0,ptr::null(),(D3DCLEAR_TARGET | D3DCLEAR_ZBUFFER) as u32,0xff18202e,1.,0)?; dev.BeginScene()?;
        dev.SetVertexDeclaration(&dev.CreateVertexDeclaration(declaration.as_ptr())?)?; dev.SetVertexShader(&dev.CreateVertexShader(vt.as_ptr())?)?; dev.SetPixelShader(&dev.CreatePixelShader(pt.as_ptr())?)?;
        for (stage,uniforms,registers) in [(true,&vsrc.uniforms,&vsrc.registers),(false,&psrc.uniforms,&psrc.registers)] {
            if uniforms.len()!=registers.len() {return Err("uniform/register length mismatch".into());}
            for (u,&start) in uniforms.iter().zip(registers) {let descriptor=UniformDescriptor {raw:u.metadata};let count=usize::from(descriptor.register_count());
                let values=match (stage,descriptor.callback_id(),count) {(true,0x22,1)=>vec![[1.;4]],(true,3,1)=>vec![[3.,0.,1.,0.]],(true,4,192)=>palette.clone(),(true,8,3)=>rows(model,3),(true,6,4)=>rows(clip,4),(false,0x20c,1)=>vec![[1.;4]],_=>return Err(format!("unsupported uniform {} {} {}",stage,descriptor.callback_id(),count).into())};
                if stage {dev.SetVertexShaderConstantF(start,values.as_ptr().cast(),count as u32)?;} else {dev.SetPixelShaderConstantF(start,values.as_ptr().cast(),count as u32)?;}
                println!("FIXTURE_UNIFORM stage={} id={:04x} start={} rows={}",if stage {"vs"} else {"ps"},descriptor.callback_id(),start,count);
            }
        }
        dev.DrawIndexedPrimitiveUP(D3DPT_TRIANGLELIST,0,454,896,indices.as_ptr().cast(),D3DFMT_INDEX16,vertices.as_ptr().cast(),36)?; dev.EndScene()?;
    }
    let rgba=unsafe{target.read_rgba()?}; let changed=rgba.chunks_exact(4).filter(|p|*p!=[0x18,0x20,0x2e,0xff]).count();
    if changed==0 {return Err("no changed pixels".into());} let file=std::fs::OpenOptions::new().write(true).create_new(true).open(output)?; image::codecs::png::PngEncoder::new(file).write_image(&rgba,768,768,image::ExtendedColorType::Rgba8)?;
    println!("KIND0_FIXTURE exe={} shader_package={} asset_package={} source=DAE8CE17 payload_fnv={:016x} fragment_fnv={:016x} pack_fnv={:016x} vs={:02x?} ps={:02x?} data_24e=0 unknown_data=absent stages=17,19 palette=identity_64_matrices palette_rows_0_to_26=identity_fixture material_tint=white skin_weights=3 camera=bounds_fit cull=none srgb=off changed={} retail_fidelity=false output={}",exe.display(),shader_path.display(),asset_path.display(),fnv(&raw),fnv(&fragment_bytes),fnv(&pack_bytes),vs.fragment_ids,ps.fragment_ids,changed,output.display());
    Ok(())
}
#[cfg(not(all(windows, feature = "native_shader_probe")))]
fn main() -> Result<(), Box<dyn std::error::Error>> { Err("requires Windows and native_shader_probe".into()) }
